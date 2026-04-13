use core::panic;
use std::collections::HashMap;

use super::cranelift::data_context::DataDescription;
use super::cranelift::module::{Linkage, Module};
use crate::{
    ir::{self, GlobalValueMap, SourceLocs, StringMap},
    runtime::string,
};
use cranelift_codegen::{
    Context,
    binemit::CodeOffset,
    ir::{
        AbiParam, Block, BlockArg, InstBuilder, JumpTableData, MemFlags, Signature, TrapCode,
        condcodes::{FloatCC, IntCC},
        types::{I8, I32, I64},
    },
    isa::CallConv,
    verify_function,
};
use cranelift_frontend::Variable;

pub(crate) struct TranslateSignature {
    pub id: String,
    pub signature: ir::Signature,
}

fn translate_signature(
    ctx: &super::JitContext,
    signature: &crate::ir::Signature,
    call_conv: CallConv,
) -> Signature {
    let mut params: Vec<AbiParam> = signature
        .parameters
        .iter()
        .map(|a| AbiParam::new(ctx.translate_type(a)))
        .collect();
    params.insert(0, AbiParam::new(ctx.context_type()));
    Signature {
        params,
        returns: signature
            .ret_types
            .iter()
            .map(|a| AbiParam::new(ctx.translate_type(a)))
            .collect(),
        call_conv,
    }
}

fn construct_panic_message(
    ctx: &mut super::JitContext,
    builder: &mut cranelift_frontend::FunctionBuilder,
    source_locs: &SourceLocs,
    source_loc: usize,
    str_map: &StringMap,
    reason: &str,
) -> cranelift_codegen::ir::Value {
    let source_loc = source_locs.locations[source_loc].clone();
    let panic_message = format!(
        "Panic at {}:{}:{}: {}",
        str_map.get(source_loc.file),
        source_loc.line,
        source_loc.col,
        reason
    );

    let data_id = ctx
        .module
        .declare_anonymous_data(false, false)
        .expect("Failed to create anonymous data");

    let mut data_desc = DataDescription::new();
    data_desc.clear();
    data_desc.define(string::convert_to_interal_string(&panic_message));
    ctx.module
        .define_data(data_id, &data_desc)
        .expect("Could not define data");

    let local_data_id = ctx.module.declare_data_in_func(data_id, builder.func);
    builder.ins().symbol_value(I64, local_data_id)
}

pub fn translate_function(
    ctx: &mut super::JitContext,
    func: &ir::Function,
    dest: &mut Context,
    signatures: &Vec<TranslateSignature>,
    str_map: &StringMap,
    source_locs: &SourceLocs,
    globals: &GlobalValueMap,
) {
    let mut translated = &mut dest.func;
    translated.signature =
        translate_signature(ctx, &func.signature, translated.signature.call_conv);
    //translated.name = cranelift_codegen::ir::UserFuncName::user(0, func.id);
    let call_conv = translated.signature.call_conv;

    let mut func_bld_ctx = cranelift_frontend::FunctionBuilderContext::new();
    let mut builder = cranelift_frontend::FunctionBuilder::new(&mut translated, &mut func_bld_ctx);

    let blocks: Vec<Block> = func.blocks.iter().map(|_| builder.create_block()).collect();

    let variables: Vec<Variable> = func
        .variables
        .iter()
        .map(|var| builder.declare_var(ctx.translate_type(&var.typ)))
        .collect();
    let mut data_desc = DataDescription::new();

    let mut declared_signatures: HashMap<String, cranelift_codegen::ir::FuncRef> = HashMap::new();

    builder.append_block_params_for_function_params(blocks[0]);
    builder.seal_block(blocks[0]);

    builder.switch_to_block(blocks[0]);
    for i in 0..func.signature.parameters.len() {
        builder.def_var(variables[i], builder.block_params(blocks[0])[1 + i]);
    }

    for (var, func_var) in variables.iter().zip(func.variables.iter()) {
        if matches!(func_var.typ, crate::ir::Type::Reference) {
            builder.declare_var_needs_stack_map(*var);
        }
    }

    let runtime_ctx = builder.block_params(blocks[0])[0];
    let mut stack = Vec::new();
    let frontend_config = ctx.isa().frontend_config();

    let mut translate_call = |ctx: &mut super::JitContext,
                              builder: &mut cranelift_frontend::FunctionBuilder,
                              stack: &mut Vec<cranelift_codegen::ir::Value>,
                              id: &str| {
        let sig = signatures
            .iter()
            .find(|s| s.id == *id)
            .expect(format!("Could not find signature for {}", id).as_str());
        if !declared_signatures.contains_key(id) {
            let signature = translate_signature(ctx, &sig.signature, call_conv);
            let func_id = ctx
                .module
                .declare_function(&sig.id, Linkage::Import, &signature)
                .expect("Failed to declare function");
            let func_ref = ctx.module.declare_func_in_func(func_id, builder.func);
            declared_signatures.insert(id.to_string(), func_ref);
        }

        let mut args = stack
            .iter()
            .rev()
            .take(sig.signature.parameters.len())
            .cloned()
            .collect::<Vec<_>>();
        args.reverse();
        stack.truncate(stack.len() - args.len());
        args.insert(0, runtime_ctx);

        let call = builder.ins().call(declared_signatures[id], &args);
        for r in builder.inst_results(call) {
            stack.push(*r);
        }
    };

    let panic_block = builder.create_block();

    for (i, block) in func.blocks.iter().enumerate() {
        builder.switch_to_block(blocks[i]);

        for (inst, source_loc) in block.iter() {
            match &inst {
                ir::Inst::Nop => {}
                ir::Inst::Dup(i) => {
                    let val = *stack.iter().rev().nth(*i).unwrap();
                    stack.push(val);
                }
                ir::Inst::AddInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().iadd(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::SubInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().isub(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::MulInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().imul(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::DivInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().sdiv(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::ModInt => todo!(),
                ir::Inst::EquInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::NeqInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::LtInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::GtInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::LeqInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::GeqInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::AddNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fadd(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::SubNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fsub(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::MulNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fmul(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::DivNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fdiv(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::EquNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::Equal, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::NeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::LtNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::GtNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::LeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::GeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::EquString | ir::Inst::NeqString => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();

                    let mut flags = MemFlags::new();
                    flags.set_notrap();
                    flags.set_readonly();

                    let lhs_len = builder.ins().load(I64, flags, lhs, 0);
                    let rhs_len = builder.ins().load(I64, flags, rhs, 0);
                    let pointer_equal = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                    let length_equal = builder.ins().icmp(IntCC::Equal, lhs_len, rhs_len);
                    let lhs_shorter = builder
                        .ins()
                        .icmp(IntCC::UnsignedLessThan, lhs_len, rhs_len);
                    let compare_length = builder.ins().select(lhs_shorter, lhs_len, rhs_len);
                    let lhs_data = builder.ins().iadd_imm(lhs, 8);
                    let rhs_data = builder.ins().iadd_imm(rhs, 8);
                    let cmp_res =
                        builder.call_memcmp(frontend_config, lhs_data, rhs_data, compare_length);
                    let bytes_equal = builder.ins().icmp_imm(IntCC::Equal, cmp_res, 0);
                    let same_content = builder.ins().band(length_equal, bytes_equal);
                    let mut result = builder.ins().bor(pointer_equal, same_content);
                    if matches!(inst, ir::Inst::NeqString) {
                        result = builder.ins().bnot(result);
                    }
                    stack.push(result);
                }
                ir::Inst::And => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().band(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::Or => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().bor(lhs, rhs);
                    stack.push(res);
                }
                ir::Inst::LoadConstInt(value) => {
                    let val = builder.ins().iconst(I64, *value);
                    stack.push(val);
                }
                ir::Inst::LoadConstByte(value) => {
                    let val = builder.ins().iconst(I8, *value as i64);
                    stack.push(val);
                }
                ir::Inst::LoadConstNumber(value) => {
                    let val = builder.ins().f64const(*value);
                    stack.push(val);
                }
                ir::Inst::LoadConstBool(value) => {
                    let val = builder.ins().iconst(I8, if *value { 1 } else { 0 });
                    stack.push(val);
                }
                ir::Inst::LoadConstString(value) => {
                    let data_id = ctx
                        .module
                        .declare_anonymous_data(false, false)
                        .expect("Failed to create anonymous data");
                    data_desc.clear();
                    data_desc.define(string::convert_to_interal_string(str_map.get(*value)));
                    ctx.module
                        .define_data(data_id, &data_desc)
                        .expect("Could not define data");
                    let local_data_id = ctx.module.declare_data_in_func(data_id, builder.func);
                    let addr = builder.ins().symbol_value(I64, local_data_id);
                    stack.push(addr);
                }
                ir::Inst::LoadGlobal(value) => {
                    let data_id = ctx
                        .module
                        .declare_anonymous_data(false, false)
                        .expect("Failed to create anonymous data");
                    data_desc.clear();
                    match globals.get(*value) {
                        ir::GlobalValue::VirtualTable(functions) => {
                            data_desc.define_zeroinit(functions.len() * 8);
                            for (i, func) in functions.iter().enumerate() {
                                let sig = signatures.iter().find(|s| s.id == *func).expect(
                                    format!("Could not find signature for {}", func).as_str(),
                                );
                                let signature = translate_signature(ctx, &sig.signature, call_conv);
                                let func_id = ctx
                                    .module
                                    .declare_function(&sig.id, Linkage::Import, &signature)
                                    .expect("Failed to declare function");
                                let func_ref =
                                    ctx.module.declare_func_in_data(func_id, &mut data_desc);
                                data_desc.write_function_addr((i * 8) as CodeOffset, func_ref);
                            }
                        }
                    }
                    ctx.module
                        .define_data(data_id, &data_desc)
                        .expect("Could not define data");
                    let local_data_id = ctx.module.declare_data_in_func(data_id, builder.func);
                    let addr = builder.ins().symbol_value(I64, local_data_id);
                    stack.push(addr);
                }
                ir::Inst::Truncate => {
                    let val = stack.pop().unwrap();
                    let res = builder.ins().fcvt_to_sint(I64, val);
                    stack.push(res);
                }
                ir::Inst::Promote => {
                    let val = stack.pop().unwrap();
                    let res = builder
                        .ins()
                        .fcvt_from_sint(cranelift_codegen::ir::types::F64, val);
                    stack.push(res);
                }
                ir::Inst::Load(var) => {
                    stack.push(builder.use_var(variables[*var]));
                }
                ir::Inst::Store(var) => {
                    let val = stack.pop().unwrap();
                    builder.def_var(variables[*var], val);
                }
                ir::Inst::Tee(var) => {
                    let val = stack.pop().unwrap();
                    builder.def_var(variables[*var], val);
                    stack.push(val);
                }
                ir::Inst::CondBr(c, a) => {
                    let cond = stack.pop().unwrap();
                    builder.ins().brif(cond, blocks[*c], &[], blocks[*a], &[]);
                }
                ir::Inst::Br(target) => {
                    builder.ins().jump(blocks[*target], &[]);
                }
                ir::Inst::BrTable(def, a) => {
                    let val = stack.pop().unwrap();
                    let val = builder.ins().ireduce(I32, val);
                    let table = a
                        .iter()
                        .map(|b| builder.func.dfg.block_call(blocks[*b], &[]))
                        .collect::<Vec<_>>();
                    let jump_table =
                        JumpTableData::new(builder.func.dfg.block_call(blocks[*def], &[]), &table);
                    let jt = builder.create_jump_table(jump_table);
                    builder.ins().br_table(val, jt);
                }
                ir::Inst::Ret => {
                    let ret_vals = stack
                        .iter()
                        .rev()
                        .take(func.signature.ret_types.len())
                        .cloned()
                        .collect::<Vec<_>>();
                    stack.truncate(stack.len() - ret_vals.len());
                    builder.ins().return_(&ret_vals);
                }
                ir::Inst::Call(id) => {
                    translate_call(ctx, &mut builder, &mut stack, &id);
                }
                ir::Inst::IndirectCall(signature) => {
                    let mut args = stack
                        .iter()
                        .rev()
                        .take(signature.parameters.len())
                        .cloned()
                        .collect::<Vec<_>>();
                    args.reverse();
                    stack.truncate(stack.len() - args.len());
                    args.insert(0, runtime_ctx);
                    let func = stack.pop().unwrap();
                    let signature = translate_signature(ctx, signature, call_conv);
                    let sig_ref = builder.import_signature(signature);
                    let call = builder.ins().call_indirect(sig_ref, func, &args);
                    for r in builder.inst_results(call) {
                        stack.push(*r);
                    }
                }
                ir::Inst::NewArray(size, typ) => {
                    let clir_typ = ctx.translate_type(&typ);
                    let val = builder.ins().iconst(I64, *size as i64);
                    stack.push(val);
                    let val = builder.ins().iconst(I64, clir_typ.bytes() as i64);
                    stack.push(val);
                    let scan_elements = matches!(typ, ir::Type::Reference);
                    stack.push(builder.ins().iconst(I8, if scan_elements { 1 } else { 0 }));
                    translate_call(ctx, &mut builder, &mut stack, "__create_array");
                }
                ir::Inst::LoadArray(typ) => {
                    let array = stack.pop().unwrap();
                    let index = stack.pop().unwrap();
                    // create a new block that everything after this load goes into
                    let continue_block = builder.create_block();
                    // load the array size, which is directly at the array pointer
                    let array_size = builder.ins().load(I64, MemFlags::new(), array, 0);
                    // check the index is allg
                    let index_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, array_size, 0);
                    let index_ok_size =
                        builder.ins().icmp(IntCC::SignedLessThan, index, array_size);
                    let index_ok = builder.ins().band(index_ok_size, index_ok_zero);
                    // jump to continue block if we are allg otherwise panic block
                    let panic_message = construct_panic_message(
                        ctx,
                        &mut builder,
                        source_locs,
                        source_loc.unwrap(),
                        str_map,
                        "Out of bounds.",
                    );
                    builder.ins().brif(
                        index_ok,
                        continue_block,
                        &[],
                        panic_block,
                        &vec![BlockArg::Value(panic_message)],
                    );

                    builder.switch_to_block(continue_block);
                    // load the value now we know the array index is ok
                    let clir_typ = ctx.translate_type(&typ);
                    let offset = builder.ins().imul_imm(index, clir_typ.bytes() as i64);
                    let array_begin = builder.ins().iadd_imm(array, 8);
                    let pointer = builder.ins().iadd(array_begin, offset);
                    let value =
                        builder
                            .ins()
                            .load(clir_typ, MemFlags::new(), pointer, 0);
                    stack.push(value);
                }
                ir::Inst::StoreArray(typ) => {
                    let array = stack.pop().unwrap();
                    let index = stack.pop().unwrap();
                    let value = stack.pop().unwrap();
                    // create a new block that everything after this load goes into
                    let continue_block = builder.create_block();
                    // load the array size, which is directly at the array pointer
                    let array_size = builder.ins().load(I64, MemFlags::new(), array, 0);
                    // check the index is allg
                    let index_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, array_size, 0);
                    let index_ok_size =
                        builder.ins().icmp(IntCC::SignedLessThan, index, array_size);
                    let index_ok = builder.ins().band(index_ok_size, index_ok_zero);
                    // jump to continue block if we are allg otherwise panic block
                    let panic_message = construct_panic_message(
                        ctx,
                        &mut builder,
                        source_locs,
                        source_loc.unwrap(),
                        str_map,
                        "Out of bounds.",
                    );
                    builder.ins().brif(
                        index_ok,
                        continue_block,
                        &[],
                        panic_block,
                        &vec![BlockArg::Value(panic_message)],
                    );

                    builder.switch_to_block(continue_block);
                    // store the value now we know the array index is ok
                    let clir_typ = ctx.translate_type(&typ);
                    let offset = builder.ins().imul_imm(index, clir_typ.bytes() as i64);
                    let array_begin = builder.ins().iadd_imm(array, 8);
                    let pointer = builder.ins().iadd(array_begin, offset);
                    builder
                        .ins()
                        .store(MemFlags::new().with_aligned(), value, pointer, 0);
                }
                ir::Inst::NewObject(size) => {
                    let val = builder.ins().iconst(I64, *size as i64);
                    stack.push(val);
                    translate_call(ctx, &mut builder, &mut stack, "__create_object");
                }
                ir::Inst::GetObject(i, typ) => {
                    let object = stack.pop().unwrap();
                    let offset = *i as i64 * 8;
                    let pointer = builder.ins().iadd_imm(object, offset);
                    let value =
                        builder
                            .ins()
                            .load(ctx.translate_type(&typ), MemFlags::new(), pointer, 0);
                    stack.push(value);
                }
                ir::Inst::SetObject(i, _) => {
                    let object = stack.pop().unwrap();
                    let value = stack.pop().unwrap();
                    let offset = *i as i64 * 8;
                    let pointer = builder.ins().iadd_imm(object, offset);
                    builder
                        .ins()
                        .store(MemFlags::new().with_aligned(), value, pointer, 0);
                }
                ir::Inst::CheckYield => {
                    translate_call(ctx, &mut builder, &mut stack, "__check_yield");
                }
                ir::Inst::Assert => {
                    let condition = stack.pop().unwrap();
                    let continue_block = builder.create_block();
                    let panic_message = construct_panic_message(
                        ctx,
                        &mut builder,
                        source_locs,
                        source_loc.unwrap(),
                        str_map,
                        "Assertion Failed.",
                    );
                    builder.ins().brif(
                        condition,
                        continue_block,
                        &[],
                        panic_block,
                        &vec![BlockArg::Value(panic_message)],
                    );
                    builder.switch_to_block(continue_block);
                }
            }
        }
    }

    // This is the panic block
    builder.switch_to_block(panic_block);
    let message = builder.append_block_param(panic_block, ctx.translate_type(&ir::Type::Reference));
    stack.push(message);
    translate_call(ctx, &mut builder, &mut stack, "__panic");
    builder
        .ins()
        .trap(TrapCode::user(1).expect("Could not get trap code"));

    builder.seal_all_blocks();
    builder.finalize();

    //println!("{}", func.id);
    //println!("{}", translated.display());
    let res = verify_function(&translated, ctx.isa());
    if let Err(errors) = res {
        panic!("{}", errors);
    }
}
