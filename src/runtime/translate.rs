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
        .flat_map(|a| ctx.translate_type(a))
        .map(|a| AbiParam::new(a))
        .collect();
    params.insert(0, AbiParam::new(ctx.context_type()));
    Signature {
        params,
        returns: signature
            .ret_types
            .iter()
            .flat_map(|a| ctx.translate_type(a))
            .map(|a| AbiParam::new(a))
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

struct ValueStack {
    stack: Vec<(cranelift_codegen::ir::Value, Option<cranelift_codegen::ir::Value>)>,
}

impl ValueStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn s_push(&mut self, value: cranelift_codegen::ir::Value) {
        self.push(value, None);
    }

    pub fn s_pop(&mut self) -> cranelift_codegen::ir::Value {
        let (value, second) = self.pop();
        if second.is_some() {
            panic!("Expected a single value on the stack, but got a fat pointer");
        }
        value
    }

    pub fn push(&mut self, value: cranelift_codegen::ir::Value, fat_ptr: Option<cranelift_codegen::ir::Value>) {
        self.stack.push((value, fat_ptr));
    }

    pub fn pop(&mut self) -> (cranelift_codegen::ir::Value, Option<cranelift_codegen::ir::Value>) {
        self.stack.pop().expect("Stack underflow")
    }

    pub fn duplicate(&mut self, i: usize) {
        let val = *self.stack.iter().rev().nth(i).unwrap();
        self.stack.push(val);
    }

    pub fn take_and_flatten(&mut self, n: usize) -> Vec<cranelift_codegen::ir::Value> {
        let len = self.stack.len();
        if n > len {
            panic!("Stack underflow");
        }
        let mut taken = Vec::new();
        for (v, fat_ptr) in self.stack.iter().rev().take(n) {
            if let Some(fat_ptr) = fat_ptr {
                taken.push(*fat_ptr);
            }
            taken.push(*v);
        }
        self.stack.truncate(len - n);
        taken
    }
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

    let variables: Vec<(Variable, Option<Variable>)> = func
        .variables
        .iter()
        .map(|var| {
            let abi_type = ctx.translate_type(&var.typ);
            let var = builder.declare_var(abi_type.root);
            let fat_ptr_var = abi_type.fat_ptr_typ.map(|t| builder.declare_var(t));
            (var, fat_ptr_var)
        })
        .collect();
    let mut data_desc = DataDescription::new();

    let mut declared_signatures: HashMap<String, cranelift_codegen::ir::FuncRef> = HashMap::new();

    builder.append_block_params_for_function_params(blocks[0]);
    builder.seal_block(blocks[0]);

    builder.switch_to_block(blocks[0]);
    for i in 0..func.signature.parameters.len() {
        builder.def_var(variables[i].0, builder.block_params(blocks[0])[1 + i]);
    }

    for (var, func_var) in variables.iter().zip(func.variables.iter()) {
        if matches!(func_var.typ, crate::ir::Type::Reference | crate::ir::Type::Array) {
            builder.declare_var_needs_stack_map(var.0);
        }
    }

    let runtime_ctx = builder.block_params(blocks[0])[0];
    let mut stack = ValueStack::new();
    let frontend_config = ctx.isa().frontend_config();

    let mut translate_call = |ctx: &mut super::JitContext,
                              builder: &mut cranelift_frontend::FunctionBuilder,
                              stack: &mut ValueStack,
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

        let mut args = stack.take_and_flatten(sig.signature.parameters.len());
        args.reverse();
        args.insert(0, runtime_ctx);

        let call = builder.ins().call(declared_signatures[id], &args);
        let mut results = builder.inst_results(call).iter();
        for typ in sig.signature.ret_types.iter() {
            if matches!(typ, crate::ir::Type::Array) {
                stack.push(*results.next().unwrap(), Some(*results.next().unwrap()));
            } else {
                stack.s_push(*results.next().unwrap());
            }
        }
    };

    let panic_block = builder.create_block();

    for (i, block) in func.blocks.iter().enumerate() {
        builder.switch_to_block(blocks[i]);

        for (inst, source_loc) in block.iter() {
            match &inst {
                ir::Inst::Nop => {}
                ir::Inst::Dup(i) => {
                    stack.duplicate(*i);
                }
                ir::Inst::AddInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().iadd(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::SubInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().isub(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::MulInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().imul(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::DivInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().sdiv(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::ModInt => todo!(),
                ir::Inst::EquInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::NeqInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::LtInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::GtInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::LeqInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::GeqInt => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder
                        .ins()
                        .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::AddNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fadd(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::SubNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fsub(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::MulNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fmul(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::DivNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fdiv(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::EquNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::Equal, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::NeqNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::LtNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::GtNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::LeqNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::GeqNumber => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::EquString | ir::Inst::NeqString => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();

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
                    stack.s_push(result);
                }
                ir::Inst::And => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().band(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::Or => {
                    let rhs = stack.s_pop();
                    let lhs = stack.s_pop();
                    let res = builder.ins().bor(lhs, rhs);
                    stack.s_push(res);
                }
                ir::Inst::LoadConstInt(value) => {
                    let val = builder.ins().iconst(I64, *value);
                    stack.s_push(val);
                }
                ir::Inst::LoadConstByte(value) => {
                    let val = builder.ins().iconst(I8, *value as i64);
                    stack.s_push(val);
                }
                ir::Inst::LoadConstNumber(value) => {
                    let val = builder.ins().f64const(*value);
                    stack.s_push(val);
                }
                ir::Inst::LoadConstBool(value) => {
                    let val = builder.ins().iconst(I8, if *value { 1 } else { 0 });
                    stack.s_push(val);
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
                    stack.s_push(addr);
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
                    stack.s_push(addr);
                }
                ir::Inst::Truncate => {
                    let val = stack.s_pop();
                    let res = builder.ins().fcvt_to_sint(I64, val);
                    stack.s_push(res);
                }
                ir::Inst::Promote => {
                    let val = stack.s_pop();
                    let res = builder
                        .ins()
                        .fcvt_from_sint(cranelift_codegen::ir::types::F64, val);
                    stack.s_push(res);
                }
                ir::Inst::Load(var) => {
                    let var = &variables[*var];
                    if let Some(fat_ptr_var) = var.1 {
                        let fat_ptr = builder.use_var(fat_ptr_var);
                        stack.push(builder.use_var(var.0), Some(fat_ptr));
                    } else {
                        stack.s_push(builder.use_var(var.0));
                    }
                }
                ir::Inst::Store(var) => {
                    let var = &variables[*var];
                    let val = stack.pop();
                    assert!((var.1.is_none() && val.1.is_none()) || (var.1.is_some() && val.1.is_some()), "Mistake in value size");
                    builder.def_var(var.0, val.0);
                    if let Some(fat_ptr_var) = var.1 {
                        builder.def_var(fat_ptr_var, val.1.expect("Expected a fat pointer"))
                    }
                }
                ir::Inst::Tee(var) => {
                    let var = &variables[*var];
                    let val = stack.pop();
                    assert!((var.1.is_none() && val.1.is_none()) || (var.1.is_some() && val.1.is_some()), "Mistake in value size");
                    if let Some(fat_ptr_var) = var.1 {
                        builder.def_var(var.0, val.0);
                        builder.def_var(fat_ptr_var, val.1.expect("Expected a fat pointer"));
                        stack.push(val.0, val.1);
                    } else {
                        builder.def_var(var.0, val.0);
                        stack.s_push(val.0);
                    }
                }
                ir::Inst::CondBr(c, a) => {
                    let cond = stack.s_pop();
                    builder.ins().brif(cond, blocks[*c], &[], blocks[*a], &[]);
                }
                ir::Inst::Br(target) => {
                    builder.ins().jump(blocks[*target], &[]);
                }
                ir::Inst::BrTable(def, a) => {
                    let val = stack.s_pop();
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
                    let ret_vals = stack.take_and_flatten(func.signature.ret_types.len());
                    builder.ins().return_(&ret_vals);
                }
                ir::Inst::Call(id) => {
                    translate_call(ctx, &mut builder, &mut stack, &id);
                }
                ir::Inst::IndirectCall(signature) => {
                    let mut args = stack.take_and_flatten(signature.parameters.len());
                    args.reverse();
                    args.insert(0, runtime_ctx);
                    let func = stack.s_pop();
                    let sig = translate_signature(ctx, signature, call_conv);
                    let sig_ref = builder.import_signature(sig);
                    let call = builder.ins().call_indirect(sig_ref, func, &args);
                    let mut results = builder.inst_results(call).iter();
                    for typ in signature.ret_types.iter() {
                        if matches!(typ, crate::ir::Type::Array) {
                            stack.push(*results.next().unwrap(), Some(*results.next().unwrap()));
                        } else {
                            stack.s_push(*results.next().unwrap());
                        }
                    }
                }
                ir::Inst::NewArray(size, typ) => {
                    // allocate
                    let clir_typ = ctx.translate_type(&typ);
                    let val = builder.ins().iconst(I64, *size as i64);
                    stack.s_push(val);
                    let val = builder.ins().iconst(I64, clir_typ.bytes() as i64);
                    stack.s_push(val);
                    let scan_elements = matches!(typ, ir::Type::Reference);
                    stack.s_push(builder.ins().iconst(I8, if scan_elements { 1 } else { 0 }));
                    translate_call(ctx, &mut builder, &mut stack, "__create_array");
                    
                    let array_ptr = stack.s_pop();
                    let size = builder.ins().iconst(I64, *size as i64);
                    stack.push(array_ptr, Some(size));
                }
                ir::Inst::LoadArray(typ) => {
                    let (array_ptr, array_size) = stack.pop();
                    let array_size = array_size.expect("Expected a fat pointer for the array size");

                    let index = stack.s_pop();
                    // create a new block that everything after this load goes into
                    let continue_block = builder.create_block();
                    // load the array size, which is directly at the array pointer
                    // let array_size = builder.ins().load(I64, MemFlags::new(), array, 0);
                    // check the index is allg
                    let index_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, index, 0);
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
                    let abi_typ = ctx.translate_type(&typ);
                    let offset = builder.ins().imul_imm(index, abi_typ.bytes() as i64);
                    //let array_begin = builder.ins().iadd_imm(array, 8);
                    let pointer = builder.ins().iadd(array_ptr, offset);
                    let value =
                        builder
                            .ins()
                            .load(abi_typ.root, MemFlags::new(), pointer, 0);
                    stack.s_push(value);

                    if let Some(fat_ptr_typ) = abi_typ.fat_ptr_typ {
                        let pointer = builder.ins().iadd_imm(pointer, abi_typ.root.bytes() as i64);
                        let fat_ptr = builder
                            .ins()
                            .load(fat_ptr_typ, MemFlags::new(), pointer, 0);
                        stack.s_push(fat_ptr);
                    }
                }
                ir::Inst::StoreArray(typ) => {
                    let (array_ptr, array_size) = stack.pop();
                    let array_size = array_size.expect("Expected a fat pointer for the array size");

                    let index = stack.s_pop();
                    let value = stack.s_pop();
                    // create a new block that everything after this load goes into
                    let continue_block = builder.create_block();
                    // check the index is allg
                    let index_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, index, 0);
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
                    let abi_typ = ctx.translate_type(&typ);
                    let offset = builder.ins().imul_imm(index, abi_typ.bytes() as i64);
                    //let array_begin = builder.ins().iadd_imm(array, 8);
                    let pointer = builder.ins().iadd(array_ptr, offset);
                    builder
                        .ins()
                        .store(MemFlags::new().with_aligned(), value, pointer, 0);
                    if abi_typ.fat_ptr_typ.is_some() {
                        let fat_ptr = stack.s_pop();
                        let pointer = builder.ins().iadd_imm(pointer, abi_typ.root.bytes() as i64);
                        builder
                            .ins()
                            .store(MemFlags::new().with_aligned(), fat_ptr, pointer, 0);
                    }
                }
                ir::Inst::CreateSlice(typ) => {
                    let (array_ptr, array_size) = stack.pop();
                    let array_size = array_size.expect("Expected a fat pointer for the array size");
                    let end = stack.s_pop();
                    let start = stack.s_pop();
                    
                    // create a new block that everything after this load goes into
                    let continue_block = builder.create_block();
                    // check the start is allg
                    let start_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, start, 0);
                    let start_ok_size =
                        builder.ins().icmp(IntCC::SignedLessThan, start, array_size);
                    let start_ok = builder.ins().band(start_ok_size, start_ok_zero);
                    // check the end is allg
                    let end_ok_zero =
                        builder
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, end, 0);
                    let end_ok_size =
                        builder.ins().icmp(IntCC::SignedLessThanOrEqual, end, array_size);
                    let end_ok = builder.ins().band(end_ok_size, end_ok_zero);
                    // check both
                    let bounds_ok = builder.ins().band(start_ok, end_ok);

                    // check the start is less than or equal to the end
                    let start_end_ok =
                        builder.ins().icmp(IntCC::SignedLessThanOrEqual, start, end);
                    
                    let ok = builder.ins().band(bounds_ok, start_end_ok);                    

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
                        ok,
                        continue_block,
                        &[],
                        panic_block,
                        &vec![BlockArg::Value(panic_message)],
                    );

                    builder.switch_to_block(continue_block);
                    // array_ptr + (start * element_size), end - start
                    let abi_typ = ctx.translate_type(&typ);
                    let offset = builder.ins().imul_imm(start, abi_typ.bytes() as i64);
                    let pointer = builder.ins().iadd(array_ptr, offset);
                    let len = builder.ins().isub(end, start);
                    stack.push(pointer, Some(len));
                }
                ir::Inst::ArrayLen => {
                    let (_, slice_len) = stack.pop();
                    let slice_len = slice_len.expect("Expected a fat pointer for the slice length");
                    stack.s_push(slice_len);
                }   
                ir::Inst::NewObject(size) => {
                    let val = builder.ins().iconst(I64, *size as i64);
                    stack.s_push(val);
                    translate_call(ctx, &mut builder, &mut stack, "__create_object");
                }
                ir::Inst::GetObject(i, typ) => {
                    let object = stack.s_pop();
                    let offset = *i as i64 * 8;
                    let pointer = builder.ins().iadd_imm(object, offset);
                    let abi_type = ctx.translate_type(&typ);
                    let value =
                        builder
                            .ins()
                            .load(abi_type.root, MemFlags::new(), pointer, 0);
                    stack.s_push(value);
                    if let Some(fat_ptr_typ) = abi_type.fat_ptr_typ {let value =
                        builder
                            .ins()
                            .load(fat_ptr_typ, MemFlags::new(), pointer, 0);
                        stack.s_push(value);
                    }
                }
                ir::Inst::SetObject(i, _) => {
                    let object = stack.s_pop();
                    let value = stack.s_pop();
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
                    let condition = stack.s_pop();
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
    let message = builder.append_block_param(panic_block, ctx.translate_type(&ir::Type::Reference).root);
    stack.s_push(message);
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
