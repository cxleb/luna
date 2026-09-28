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
    verify_function,
};
use cranelift_frontend::Variable;

pub(crate) struct TranslateSignature {
    pub id: String,
    pub signature: ir::Signature,
}

fn translate_signature(
    ctx: &super::JitContext,
    signature: &crate::ir::Signature
) -> Signature {
    let mut sig = ctx.module.make_signature();
    sig.params.push(AbiParam::new(ctx.context_type()));
    for param in signature
        .parameters
        .iter() {
        sig.params.push(AbiParam::new(ctx.translate_type(param).root));
    }
    for ret_type in signature
        .ret_types
        .iter() {
        sig.returns.push(AbiParam::new(ctx.translate_type(ret_type).root));
    }

    sig

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
    stack: Vec<cranelift_codegen::ir::Value>,
}

impl ValueStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, value: cranelift_codegen::ir::Value) {
        self.stack.push(value);
    }
    
    pub fn pop(&mut self) -> cranelift_codegen::ir::Value {
        self.stack.pop().expect("Stack underflow")
    }

    pub fn duplicate(&mut self, i: usize) {
        let val = *self.stack.iter().rev().nth(i).unwrap();
        self.stack.push(val);
    }

    pub fn take(&mut self, n: usize) -> Vec<cranelift_codegen::ir::Value> {
        let len = self.stack.len();
        if n > len {
            panic!("Stack underflow");
        }
        let mut taken = Vec::new();
        for v in self.stack.iter().rev().take(n) {
            taken.push(*v);
        }
        self.stack.truncate(len - n);
        taken
    }
}

pub struct TranslationContext<'a> {
    ctx: &'a mut super::JitContext,
    func: &'a ir::Function,
    signatures: &'a Vec<TranslateSignature>,
    str_map: &'a StringMap,
    source_locs: &'a SourceLocs,
    globals: &'a GlobalValueMap,

    declared_signatures: HashMap<String, cranelift_codegen::ir::FuncRef>,
}

impl<'a> TranslationContext<'a> {
    pub fn new(
        ctx: &'a mut super::JitContext,
        func: &'a ir::Function,
        signatures: &'a Vec<TranslateSignature>,
        str_map: &'a StringMap,
        source_locs: &'a SourceLocs,
        globals: &'a GlobalValueMap,
    ) -> Self {
        Self {
            ctx,
            func,
            signatures,
            str_map,
            source_locs,
            globals,
            declared_signatures: HashMap::new(),
        }
    }

    fn ensure_sig(&mut self, builder: &mut cranelift_frontend::FunctionBuilder, id: &str) -> (cranelift_codegen::ir::FuncRef, usize, usize) {
        let sig = self.signatures
            .iter()
            .find(|s| s.id == *id)
            .expect(format!("Could not find signature for {}", id).as_str());
        if !self.declared_signatures.contains_key(id) {
            let signature = translate_signature(self.ctx, &sig.signature);
            let func_id = self.ctx
                .module
                .declare_function(&sig.id, Linkage::Import, &signature)
                .expect("Failed to declare function");
            let func_ref = self.ctx.module.declare_func_in_func(func_id, &mut builder.func);
            self.declared_signatures.insert(id.to_string(), func_ref);
        }

        // func_ref, arity, ret_count
        (self.declared_signatures[id], sig.signature.parameters.len(), sig.signature.ret_types.len())
    }

    fn translate_call (&mut self, builder: &mut cranelift_frontend::FunctionBuilder, runtime_ctx: cranelift_codegen::ir::Value, stack: &mut ValueStack, id: &str) {
        let (func_ref, arity, ret_count) = self.ensure_sig(builder, id);

        let mut args = stack.take(arity);
        args.reverse();
        args.insert(0, runtime_ctx);

        let call = builder.ins().call(func_ref, &args);
        let mut results = builder.inst_results(call).iter();
        for _ in 0..ret_count {
            stack.push(*results.next().unwrap());
        }
    }

    pub fn translate_function(&mut self, dest: &mut Context) {
        //let mut translated = &mut self.dest.func;
        dest.func.signature = translate_signature(self.ctx, &self.func.signature);
        //translated.name = cranelift_codegen::ir::UserFuncName::user(0, func.id);

        let mut func_bld_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut builder = cranelift_frontend::FunctionBuilder::new(&mut dest.func, &mut func_bld_ctx);

        let blocks: Vec<Block> = self.func.blocks.iter().map(|_| builder.create_block()).collect();

        let variables: Vec<Variable> = self.func
            .variables
            .iter()
            .map(|var| {
                let abi_type = self.ctx.translate_type(&var.typ);
                let var = builder.declare_var(abi_type.root);
                var
            })
            .collect();
        let mut data_desc = DataDescription::new();

        builder.append_block_params_for_function_params(blocks[0]);
        builder.seal_block(blocks[0]);

        builder.switch_to_block(blocks[0]);
        for i in 0..self.func.signature.parameters.len() {
            builder.def_var(variables[i], builder.block_params(blocks[0])[1 + i]);
        }

        for (var, func_var) in variables.iter().zip(self.func.variables.iter()) {
            if matches!(func_var.typ, crate::ir::Type::Reference | crate::ir::Type::Array) {
                builder.declare_var_needs_stack_map(*var);
            }
        }

        let runtime_ctx = builder.block_params(blocks[0])[0];
        let mut stack = ValueStack::new();
        let frontend_config = self.ctx.isa().frontend_config();

        let panic_block = builder.create_block();

        for (i, block) in self.func.blocks.iter().enumerate() {
            builder.switch_to_block(blocks[i]);

            for (inst, source_loc) in block.iter() {
                match &inst {
                    ir::Inst::Nop => {}
                    ir::Inst::Dup(i) => {
                        stack.duplicate(*i);
                    }
                    ir::Inst::AddInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().iadd(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::SubInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().isub(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::MulInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().imul(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::DivInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().sdiv(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::ModInt => todo!(),
                    ir::Inst::EquInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::NeqInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::LtInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::GtInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::LeqInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::GeqInt => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::AddNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fadd(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::SubNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fsub(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::MulNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fmul(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::DivNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fdiv(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::EquNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::Equal, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::NeqNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::LtNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::LessThan, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::GtNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::LeqNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::GeqNumber => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::EquString | ir::Inst::NeqString => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();

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
                        let rhs = stack.pop();
                        let lhs = stack.pop();
                        let res = builder.ins().band(lhs, rhs);
                        stack.push(res);
                    }
                    ir::Inst::Or => {
                        let rhs = stack.pop();
                        let lhs = stack.pop();
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
                        let data_id = self.ctx
                            .module
                            .declare_anonymous_data(false, false)
                            .expect("Failed to create anonymous data");
                        data_desc.clear();
                        data_desc.define(string::convert_to_interal_string(self.str_map.get(*value)));
                        self.ctx.module
                            .define_data(data_id, &data_desc)
                            .expect("Could not define data");
                        let local_data_id = self.ctx.module.declare_data_in_func(data_id, builder.func);
                        let addr = builder.ins().symbol_value(I64, local_data_id);
                        stack.push(addr);
                    }
                    ir::Inst::LoadGlobal(value) => {
                        let data_id = self.ctx
                            .module
                            .declare_anonymous_data(false, false)
                            .expect("Failed to create anonymous data");
                        data_desc.clear();
                        match self.globals.get(*value) {
                            ir::GlobalValue::VirtualTable(functions) => {
                                data_desc.define_zeroinit(functions.len() * 8);
                                for (i, func) in functions.iter().enumerate() {
                                    let sig = self.signatures.iter().find(|s| s.id == *func).expect(
                                        format!("Could not find signature for {}", func).as_str(),
                                    );
                                    let signature = translate_signature(self.ctx, &sig.signature);
                                    let func_id = self.ctx
                                        .module
                                        .declare_function(&sig.id, Linkage::Import, &signature)
                                        .expect("Failed to declare function");
                                    let func_ref =
                                        self.ctx.module.declare_func_in_data(func_id, &mut data_desc);
                                    data_desc.write_function_addr((i * 8) as CodeOffset, func_ref);
                                }
                            }
                        }
                        self.ctx.module
                            .define_data(data_id, &data_desc)
                            .expect("Could not define data");
                        let local_data_id = self.ctx.module.declare_data_in_func(data_id, builder.func);
                        let addr = builder.ins().symbol_value(I64, local_data_id);
                        stack.push(addr);
                    }
                    ir::Inst::Truncate => {
                        let val = stack.pop();
                        let res = builder.ins().fcvt_to_sint(I64, val);
                        stack.push(res);
                    }
                    ir::Inst::Promote => {
                        let val = stack.pop();
                        let res = builder
                            .ins()
                            .fcvt_from_sint(cranelift_codegen::ir::types::F64, val);
                        stack.push(res);
                    }
                    ir::Inst::Load(var) => {
                        let var = variables[*var];
                        stack.push(builder.use_var(var));
                    }
                    ir::Inst::Store(var) => {
                        let var = variables[*var];
                        let val = stack.pop();
                        builder.def_var(var, val);
                    }
                    ir::Inst::Tee(var) => {
                        let var = variables[*var];
                        let val = stack.pop();
                        builder.def_var(var, val);
                        stack.push(val);
                    }
                    ir::Inst::CondBr(c, a) => {
                        let cond = stack.pop();
                        builder.ins().brif(cond, blocks[*c], &[], blocks[*a], &[]);
                    }
                    ir::Inst::Br(target) => {
                        builder.ins().jump(blocks[*target], &[]);
                    }
                    ir::Inst::BrTable(def, a) => {
                        let val = stack.pop();
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
                        let ret_vals = stack.take(self.func.signature.ret_types.len());
                        builder.ins().return_(&ret_vals);
                    }
                    ir::Inst::Call(id) => {
                        self.translate_call(&mut builder, runtime_ctx, &mut stack, &id);
                    }
                    ir::Inst::IndirectCall(signature) => {
                        let mut args = stack.take(signature.parameters.len());
                        args.reverse();
                        args.insert(0, runtime_ctx);
                        let func = stack.pop();
                        let sig = translate_signature(self.ctx, signature);
                        let sig_ref = builder.import_signature(sig);
                        let call = builder.ins().call_indirect(sig_ref, func, &args);
                        let mut results = builder.inst_results(call).iter();
                        for _ in signature.ret_types.iter() {
                            stack.push(*results.next().unwrap());
                        }
                    }
                    ir::Inst::FuncAddr(id) => {
                        let (func_ref, _, _) = self.ensure_sig(&mut builder, id);
                        stack.push(builder.ins().func_addr(I64, func_ref));
                    }
                    ir::Inst::LoadPointer(typ) => {
                        let pointer = stack.pop();
                        let typ = self.ctx.translate_type(typ).root;
                        stack.push(builder.ins().load(typ, MemFlags::new(), pointer, 0));
                    }
                    ir::Inst::StorePointer => {
                        let value = stack.pop();
                        let pointer = stack.pop();
                        builder.ins().store(MemFlags::new(), value, pointer, 0);
                    }
                    ir::Inst::OffsetPointer(typ) => {
                        let count = stack.pop();
                        let pointer = stack.pop();
                        let pointer_type = self.ctx.context_type();
                        let count = if pointer_type == I64 {
                            count
                        } else {
                            builder.ins().ireduce(pointer_type, count)
                        };
                        let size = self.ctx.translate_type(typ).bytes();
                        let offset = builder.ins().imul_imm(count, size as i64);
                        stack.push(builder.ins().iadd(pointer, offset));
                    }
                    ir::Inst::IsNullPointer => {
                        let pointer = stack.pop();
                        stack.push(builder.ins().icmp_imm(IntCC::Equal, pointer, 0));
                    }
                    ir::Inst::NewArray(size, typ) => {
                        // allocate
                        let clir_typ = self.ctx.translate_type(&typ);
                        let val = builder.ins().iconst(I64, *size as i64);
                        stack.push(val);
                        let val = builder.ins().iconst(I64, clir_typ.bytes() as i64);
                        stack.push(val);
                        let scan_elements = matches!(typ, ir::Type::Reference);
                        stack.push(builder.ins().iconst(I8, if scan_elements { 1 } else { 0 }));
                        self.translate_call(&mut builder, runtime_ctx, &mut stack, "__create_array");
                    }
                    ir::Inst::LoadArray(typ) => {
                        let array = stack.pop();
                        let index = stack.pop();
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
                            self.ctx,
                            &mut builder,
                            self.source_locs,
                            source_loc.unwrap(),
                            self.str_map,
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
                        let clir_typ = self.ctx.translate_type(&typ);
                        let offset = builder.ins().imul_imm(index, clir_typ.bytes() as i64);
                        let array_begin = builder.ins().iadd_imm(array, 8);
                        let pointer = builder.ins().iadd(array_begin, offset);
                        let value =
                            builder
                                .ins()
                                .load(clir_typ.root, MemFlags::new(), pointer, 0);
                        stack.push(value);
                    }
                    ir::Inst::StoreArray(typ) => {
                        let array = stack.pop();
                        let index = stack.pop();
                        let value = stack.pop();
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
                            self.ctx,
                            &mut builder,
                            self.source_locs,
                            source_loc.unwrap(),
                            self.str_map,
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
                        let clir_typ = self.ctx.translate_type(&typ);
                        let offset = builder.ins().imul_imm(index, clir_typ.bytes() as i64);
                        let array_begin = builder.ins().iadd_imm(array, 8);
                        let pointer = builder.ins().iadd(array_begin, offset);
                        builder
                            .ins()
                            .store(MemFlags::new().with_aligned(), value, pointer, 0);
                    }
                    ir::Inst::CreateSlice(typ) => {
                        let array = stack.pop();
                        let end = stack.pop();
                        let start = stack.pop();
                        
                        // load the array size, which is directly at the array pointer
                        let array_size = builder.ins().load(I64, MemFlags::new(), array, 0);
                        
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
                            self.ctx,
                            &mut builder,
                            self.source_locs,
                            source_loc.unwrap(),
                            self.str_map,
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

                        // create the new array
                        let clir_typ = self.ctx.translate_type(&typ);
                        let slice_size = builder.ins().isub(end, start);
                        stack.push(slice_size);
                        let val = builder.ins().iconst(I64, clir_typ.bytes() as i64);
                        stack.push(val);
                        let scan_elements = matches!(typ, ir::Type::Reference);
                        stack.push(builder.ins().iconst(I8, if scan_elements { 1 } else { 0 }));
                        self.translate_call(&mut builder, runtime_ctx, &mut stack, "__create_array");

                        // copy the contents
                        let slice = stack.pop();
                        let byte_size = builder.ins().imul_imm(slice_size, clir_typ.bytes() as i64);
                        let offset = builder.ins().imul_imm(start, clir_typ.bytes() as i64);
                        let src_pointer = builder.ins().iadd_imm(array, 8);
                        let src_pointer = builder.ins().iadd(src_pointer, offset);
                        let dest_pointer = builder.ins().iadd_imm(slice, 8);
                        builder.call_memcpy(frontend_config, dest_pointer, src_pointer, byte_size);

                        stack.push(slice);
                    }
                    ir::Inst::ArrayLen => {
                        let array = stack.pop();
                        let array_size = builder.ins().load(I64, MemFlags::new(), array, 0);
                        stack.push(array_size);
                    }   
                    ir::Inst::NewObject(size) => {
                        let val = builder.ins().iconst(I64, *size as i64);
                        stack.push(val);
                        self.translate_call(&mut builder, runtime_ctx, &mut stack, "__create_object");
                    }
                    ir::Inst::GetObject(i, typ) => {
                        let object = stack.pop();
                        let offset = *i as i64 * 8;
                        let pointer = builder.ins().iadd_imm(object, offset);
                        let abi_type = self.ctx.translate_type(&typ);
                        let value =
                            builder
                                .ins()
                                .load(abi_type.root, MemFlags::new(), pointer, 0);
                        stack.push(value);
                    }
                    ir::Inst::SetObject(i, _) => {
                        let object = stack.pop();
                        let value = stack.pop();
                        let offset = *i as i64 * 8;
                        let pointer = builder.ins().iadd_imm(object, offset);
                        builder
                            .ins()
                            .store(MemFlags::new().with_aligned(), value, pointer, 0);
                    }
                    ir::Inst::CheckYield => {
                        self.translate_call(&mut builder, runtime_ctx, &mut stack, "__check_yield");
                    }
                    ir::Inst::Assert => {
                        let condition = stack.pop();
                        let continue_block = builder.create_block();
                        let panic_message = construct_panic_message(
                            self.ctx,
                            &mut builder,
                            self.source_locs,
                            source_loc.unwrap(),
                            self.str_map,
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
        let message = builder.append_block_param(panic_block, self.ctx.translate_type(&ir::Type::Reference).root);
        stack.push(message);
        self.translate_call(&mut builder, runtime_ctx, &mut stack, "__panic");
        builder
            .ins()
            .trap(TrapCode::user(1).expect("Could not get trap code"));

        builder.seal_all_blocks();
        builder.finalize();

        //println!("{}", func.id);
        //println!("{}", translated.display());
        let res = verify_function(&dest.func, self.ctx.isa());
        if let Err(errors) = res {
            println!("Error verifying function {}: {}", self.func.id, errors);
            println!("{}", dest.func.display());
        }
    }
}
