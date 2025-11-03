use std::collections::HashMap;

use crate::ir::{self};
use cranelift_codegen::{Context, ir::{AbiParam, Block, InstBuilder, Signature, condcodes::IntCC, types::I64}, isa::CallConv, verify_function};
use cranelift_frontend::Variable;
use cranelift_module::{Linkage, Module};

pub(crate) struct TranslateSignature {
    pub id: String,
    pub signature: ir::Signature,
}

fn translate_signature(ctx: &super::JitContext, signature: &crate::ir::Signature, call_conv: CallConv) -> Signature {
    Signature { 
        params: signature.parameters.iter().map(|a| AbiParam::new(ctx.translate_type(a))).collect(), 
        returns: signature.ret_types.iter().map(|a| AbiParam::new(ctx.translate_type(a))).collect(), 
        call_conv
    }
}

pub fn translate_function(ctx: &mut super::JitContext, func: &ir::Function, dest: &mut Context, signatures: &Vec<TranslateSignature>) {
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

    let mut declared_signatures: HashMap<String, cranelift_codegen::ir::FuncRef> = HashMap::new();

    builder.append_block_params_for_function_params(blocks[0]);
    builder.seal_block(blocks[0]);
    
    builder.switch_to_block(blocks[0]); 
    for i in 0..func.signature.parameters.len() {
        builder.def_var(variables[i], builder.block_params(blocks[0])[i]);
    }

    let mut stack = Vec::new();
    for (i, block) in func.blocks.iter().enumerate() {
        builder.switch_to_block(blocks[i]);
        for inst in block.ins.iter() {
            match inst {
                ir::Inst::Nop => {}
                ir::Inst::Dup => {
                    let val = *stack.last().unwrap();
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
                ir::Inst::EquInt =>{
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
                ir::Inst::LoadConstInt(value) => {
                    let val = builder.ins().iconst(I64, *value);
                    stack.push(val);
                }
                ir::Inst::LoadConstNumber(value) => {
                    let val = builder.ins().f64const(*value);
                    stack.push(val);
                }
                ir::Inst::Load(var) => {
                    stack.push(builder.use_var(variables[*var]));
                }
                ir::Inst::Store(var) => {
                    let val = stack.pop().unwrap();
                    builder.def_var(variables[*var], val);
                }
                ir::Inst::CondBr(c, a) => {
                    let cond = stack.pop().unwrap();
                    builder.ins().brif(cond, blocks[*c], &[], blocks[*a], &[]);
                    break;
                }
                &ir::Inst::Br(target) => {
                    builder.ins().jump(blocks[target], &[]);
                    break;
                }
                ir::Inst::Ret => {
                    let ret_vals = stack.iter().rev().take(func.signature.ret_types.len()).cloned().collect::<Vec<_>>();
                    stack.truncate(stack.len() - ret_vals.len());
                    builder.ins().return_(&ret_vals);
                    break;
                }
                ir::Inst::Call(id) => {
                    let sig = signatures.iter().find(|s| s.id == *id).unwrap();
                    if !declared_signatures.contains_key(id) {
                        let signature = translate_signature(ctx, &sig.signature, call_conv);
                        let func_id = ctx.module.declare_function(&sig.id, Linkage::Import, &signature).expect("Failed to declare function");
                        let func_ref = ctx.module.declare_func_in_func(func_id, builder.func);
                        declared_signatures.insert(id.clone(), func_ref);
                    }

                    println!("Taking {}", sig.signature.parameters.len());

                    let args = stack.iter().rev().take(sig.signature.parameters.len()).cloned().collect::<Vec<_>>();
                    stack.truncate(stack.len() - args.len());
                    let call = builder.ins().call(declared_signatures[id], &args);
                    for r in builder.inst_results(call) {
                        stack.push(*r);
                    }
                }
                ir::Inst::IndirectCall => {

                }
                _ => {}
            }
        }
    }

    builder.seal_all_blocks();
    builder.finalize();

    println!("{}", translated.display());
    let res = verify_function(&translated, ctx.isa());
    if let Err(errors) = res {
        panic!("{}", errors);
    }
}