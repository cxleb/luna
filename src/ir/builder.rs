use std::collections::HashMap;

use crate::ir::{Block, BlockRef, Signature, VariableRef};
use crate::types::Type;

pub struct FuncBuilder {
    func: super::Function,
    current_block: usize,
    variables: VariableRef,
    variable_scopes: Vec<HashMap<String, VariableRef>>,
}

impl FuncBuilder {
    pub fn new(id: String, signature: Signature) -> Self {
        Self {
            func: super::Function {
                id,
                signature,
                variables: Vec::new(),
                blocks: vec![Block {
                    id: 0,
                    ins: Vec::new(),
                }],
            },
            current_block: 0,
            variables: 0,
            variable_scopes: Vec::new(),
        }
    }

    /// Creates a new block, but does not switch to it. Use `switch_to_block` to
    /// do that
    pub fn new_block(&mut self) -> BlockRef {
        let r = self.func.blocks.len();
        self.func.blocks.push(Block {
            id: r,
            ins: Vec::new(),
        });
        r
    }

    pub fn switch_to_block(&mut self, block: BlockRef) {
        self.current_block = block;
    }

    pub fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.variable_scopes.pop();
    }

    pub fn create_var(&mut self, name: String, typ: Box<Type>) -> VariableRef {
        let id = self.variables;
        self.variables += 1;
        self.variable_scopes.last_mut().unwrap().insert(name, id);
        self.func.variables.push(super::Variable { id, typ });
        id
    }

    pub fn find_var(&self, name: &String) -> Option<VariableRef> {
        for scope in self.variable_scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn append_inst(&mut self, inst: super::Inst) {
        self.func.blocks[self.current_block].ins.push(inst);
    }

    pub fn dup(&mut self) {
        self.append_inst(super::Inst::Dup);
    }

    pub fn add_int(&mut self) {
        self.append_inst(super::Inst::AddInt);
    }

    pub fn sub_int(&mut self) {
        self.append_inst(super::Inst::SubInt);
    }

    pub fn mul_int(&mut self) {
        self.append_inst(super::Inst::MulInt);
    }

    pub fn div_int(&mut self) {
        self.append_inst(super::Inst::DivInt);
    }

    pub fn mod_int(&mut self) {
        self.append_inst(super::Inst::ModInt);
    }

    pub fn eq_int(&mut self) {
        self.append_inst(super::Inst::EquInt);
    }

    pub fn neq_int(&mut self) {
        self.append_inst(super::Inst::NeqInt);
    }

    pub fn lt_int(&mut self) {
        self.append_inst(super::Inst::LtInt);
    }

    pub fn gt_int(&mut self) {
        self.append_inst(super::Inst::GtInt);
    }

    pub fn leq_int(&mut self) {
        self.append_inst(super::Inst::LeqInt);
    }

    pub fn geq_int(&mut self) {
        self.append_inst(super::Inst::GeqInt);
    }

    pub fn add_number(&mut self) {
        self.append_inst(super::Inst::AddNumber);
    }

    pub fn sub_number(&mut self) {
        self.append_inst(super::Inst::SubNumber);
    }

    pub fn mul_number(&mut self) {
        self.append_inst(super::Inst::MulNumber);
    }

    pub fn div_number(&mut self) {
        self.append_inst(super::Inst::DivNumber);
    }

    pub fn load_const_int(&mut self, i: i64) {
        self.append_inst(super::Inst::LoadConstInt(i));
    }

    pub fn load_const_number(&mut self, n: f64) {
        self.append_inst(super::Inst::LoadConstNumber(n));
    }

    pub fn load(&mut self, v: VariableRef) {
        self.append_inst(super::Inst::Load(v));
    }

    pub fn store(&mut self, v: VariableRef) {
        self.append_inst(super::Inst::Store(v));
    }

    // pub fn load_number(&mut self, v: u64) {
    //     self.append_inst(super::Inst::LoadNumber(v));
    // }

    // pub fn store_number(&mut self, v: u64) {
    //     self.append_inst(super::Inst::StoreNumber(v));
    // }

    pub fn nop(&mut self) {
        self.append_inst(super::Inst::Nop);
    }

    pub fn ret(&mut self) {
        self.append_inst(super::Inst::Ret);
    }

    pub fn br(&mut self, block: BlockRef) {
        self.append_inst(super::Inst::Br(block));
    }

    pub fn condbr(&mut self, then_block: BlockRef, else_block: BlockRef) {
        self.append_inst(super::Inst::CondBr(then_block, else_block));
    }

    pub fn call(&mut self, id: String) {
        self.append_inst(super::Inst::Call(id));
    }

    pub fn indirect_call(&mut self) {
        self.append_inst(super::Inst::IndirectCall);
    }

    pub fn finish(self) -> Box<super::Function> {
        return Box::new(self.func);
    }
}
