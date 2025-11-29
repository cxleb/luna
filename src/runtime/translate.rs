use std::collections::HashMap;

use crate::{ir::{self, StringMap}, runtime::string};
use cranelift_codegen::{Context, ir::{AbiParam, Block, InstBuilder, MemFlags, Signature, condcodes::{FloatCC, IntCC}, types::{I8, I64}}, isa::CallConv, verify_function};
use cranelift_frontend::Variable;
use cranelift_module::{DataDescription, Linkage, Module};

pub(crate) struct TranslateSignature {
    pub id: String,
    pub signature: ir::Signature,
}

fn translate_signature(ctx: &super::JitContext, signature: &crate::ir::Signature, call_conv: CallConv) -> Signature {
    let mut params: Vec<AbiParam> = signature.parameters.iter().map(|a| AbiParam::new(ctx.translate_type(a))).collect();
    params.insert(0, AbiParam::new(ctx.context_type()));
    Signature { 
        params, 
        returns: signature.ret_types.iter().map(|a| AbiParam::new(ctx.translate_type(a))).collect(), 
        call_conv
    }
}

pub fn translate_function(ctx: &mut super::JitContext, func: &ir::Function, dest: &mut Context, signatures: &Vec<TranslateSignature>, str_map: &StringMap) {
    let mut translated = &mut dest.func;
    translated.signature = translate_signature(ctx, &func.signature, translated.signature.call_conv);
    //translated.name = cranelift_codegen::ir::UserFuncName::user(0, func.id);
    let call_conv = translated.signature.call_conv;
    
    let mut func_bld_ctx = cranelift_frontend::FunctionBuilderContext::new();
    let mut builder = cranelift_frontend::FunctionBuilder::new(&mut translated, &mut func_bld_ctx);
    
    let blocks: Vec<Block> = func.blocks.iter()
        .map(|_| builder.create_block())
        .collect();

    let variables: Vec<Variable> = func.variables.iter()
        .map(|var| {
            builder.declare_var(ctx.translate_type(&var.typ))
        })
        .collect();
    let mut data_desc = DataDescription::new();
                    
    let mut declared_signatures: HashMap<String, cranelift_codegen::ir::FuncRef> = HashMap::new();

    builder.append_block_params_for_function_params(blocks[0]);
    builder.seal_block(blocks[0]);
    
    builder.switch_to_block(blocks[0]); 
    for i in 0..func.signature.parameters.len() {
        builder.def_var(variables[i], builder.block_params(blocks[0])[1+i]);
    }

    let runtime_ctx = builder.block_params(blocks[0])[0];
    let mut stack = Vec::new();

    let mut translate_call = |
        ctx: &mut super::JitContext,
        builder: &mut cranelift_frontend::FunctionBuilder, 
        stack: &mut Vec<cranelift_codegen::ir::Value>, 
        id: &str| {
        let sig = signatures.iter().find(|s| s.id == *id).unwrap();
        if !declared_signatures.contains_key(id) {
            let signature = translate_signature(ctx, &sig.signature, call_conv);
            let func_id = ctx.module.declare_function(&sig.id, Linkage::Import, &signature).expect("Failed to declare function");
            let func_ref = ctx.module.declare_func_in_func(func_id, builder.func);
            declared_signatures.insert(id.to_string(), func_ref);
        }

        let mut args = stack.iter().rev().take(sig.signature.parameters.len()).cloned().collect::<Vec<_>>();
        args.reverse();
        stack.truncate(stack.len() - args.len());
        args.insert(0, runtime_ctx);

        let call = builder.ins().call(declared_signatures[id], &args);
        for r in builder.inst_results(call) {
            stack.push(*r);
        }
    };

    for (i, block) in func.blocks.iter().enumerate() {
        builder.switch_to_block(blocks[i]);
        
        for inst in block.ins.iter() {
            match inst {
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
                ir::Inst::NeqInt =>{
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
                },
                ir::Inst::GtInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::LeqInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::GeqInt => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                    stack.push(res);
                },
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
                },
                ir::Inst::NeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::LtNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::GtNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::LeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::GeqNumber => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    let res = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                    stack.push(res);
                },
                ir::Inst::LoadConstInt(value) => {
                    let val = builder.ins().iconst(I64, *value);
                    stack.push(val);
                }
                ir::Inst::LoadConstNumber(value) => {
                    let val = builder.ins().f64const(*value);
                    stack.push(val);
                }
                ir::Inst::LoadConstBool(value) => {
                    let val = builder.ins().iconst(I8, if *value { 1 } else { 0 } );
                    stack.push(val);
                }
                ir::Inst::LoadConstString(value) => {
                    let data_id = ctx.module.declare_anonymous_data(false, false).expect("Failed to create anonymous data");
                    data_desc.clear();
                    data_desc.define(string::convert_to_interal_string(str_map.get(*value))); 
                    ctx.module.define_data(data_id, &data_desc).expect("Could not define data");
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
                    let res = builder.ins().fcvt_from_sint(cranelift_codegen::ir::types::F64, val);
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
                &ir::Inst::Br(target) => {
                    builder.ins().jump(blocks[target], &[]);
                }
                ir::Inst::Ret => {
                    let ret_vals = stack.iter().rev().take(func.signature.ret_types.len()).cloned().collect::<Vec<_>>();
                    stack.truncate(stack.len() - ret_vals.len());
                    builder.ins().return_(&ret_vals);
                }
                ir::Inst::Call(id) => {
                    translate_call(ctx, &mut builder, &mut stack, &id);
                }
                ir::Inst::IndirectCall => todo!(),
                ir::Inst::NewArray(size) => {
                    let val = builder.ins().iconst(I64, *size as i64);
                    stack.push(val);
                    translate_call(ctx, &mut builder, &mut stack, "__create_array");
                }
                ir::Inst::LoadArray(typ) => {
                    let array = stack.pop().unwrap();
                    let index = stack.pop().unwrap();
                    let offset = builder.ins().imul_imm(index, 8);
                    let pointer = builder.ins().iadd(array, offset);
                    let value = builder.ins().load(ctx.translate_type(&typ), MemFlags::new(), pointer, 0);
                    stack.push(value);
                    //translate_call(ctx, &mut builder, &mut stack, "dummy_load_array");
                }
                ir::Inst::StoreArray(_) => {
                    //translate_call(ctx, &mut builder, &mut stack, "dummy_store_array");
                    let array = stack.pop().unwrap();
                    let index = stack.pop().unwrap();
                    let value = stack.pop().unwrap();
                    let offset = builder.ins().imul_imm(index, 8);
                    let pointer = builder.ins().iadd(array, offset);
                    builder.ins().store(MemFlags::new().with_aligned(), value, pointer, 0);
                }
            }
        }
    }

    builder.seal_all_blocks();
    builder.finalize();

    //println!("{}", func.id);
    //println!("{}", translated.display());
    let res = verify_function(&translated, ctx.isa());
    if let Err(errors) = res {
        panic!("{}", errors);
    }
}