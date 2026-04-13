use std::collections::HashMap;

use crate::ir::{Block, BlockRef, GlobalRef, Signature, SourceLocs, StringRef, Type, VariableRef};
pub struct FuncBuilder<'a> {
    func: super::Function,
    current_block: usize,
    variables: VariableRef,
    variable_scopes: Vec<HashMap<String, VariableRef>>,
    source_locs: &'a mut SourceLocs,
    next_source_loc: Option<usize>,
}

impl<'a> FuncBuilder<'a> {
    pub fn new(id: String, signature: Signature, source_locs: &'a mut SourceLocs) -> Self {
        Self {
            func: super::Function {
                id,
                signature,
                variables: Vec::new(),
                blocks: vec![Block {
                    id: 0,
                    ins: Vec::new(),
                    source_locs: Default::default(),
                }],
            },
            current_block: 0,
            variables: 0,
            variable_scopes: Vec::new(),
            source_locs,
            next_source_loc: None,
        }
    }

    /// Creates a new block, but does not switch to it. Use `switch_to_block` to
    /// do that
    pub fn new_block(&mut self) -> BlockRef {
        let r = self.func.blocks.len();
        self.func.blocks.push(Block {
            id: r,
            ins: Default::default(),
            source_locs: Vec::new(),
        });
        r
    }

    pub fn switch_to_block(&mut self, block: BlockRef) {
        self.current_block = block;
    }

    pub fn current_block(&mut self) -> BlockRef {
        self.current_block
    }

    pub fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.variable_scopes.pop();
    }

    pub fn create_var(&mut self, name: String, typ: Type) -> VariableRef {
        let id = self.variables;
        self.variables += 1;
        self.variable_scopes.last_mut().unwrap().insert(name, id);
        self.func.variables.push(super::Variable { id, typ });
        id
    }

    pub fn create_temp(&mut self, typ: Type) -> VariableRef {
        let id = self.variables;
        self.variables += 1;
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

    pub fn source_loc(&mut self, source_loc: super::SourceLoc) {
        let next_source_loc = self.source_locs.locations.len();
        self.next_source_loc = Some(next_source_loc);
        self.source_locs.locations.push(source_loc);
    }

    pub fn append_inst(&mut self, inst: super::Inst) {
        if let Some(source_loc) = self.next_source_loc {
            let next_ins = self.func.blocks[self.current_block].ins.len();
            // check if the last source loc is the same, if it is not append the source loc
            if let Some((_, last_source_loc)) =
                self.func.blocks[self.current_block].source_locs.last()
            {
                if *last_source_loc != source_loc {
                    self.func.blocks[self.current_block]
                        .source_locs
                        .push((next_ins, source_loc));
                }
            } else {
                self.func.blocks[self.current_block]
                    .source_locs
                    .push((next_ins, source_loc));
            }
        }
        self.func.blocks[self.current_block].ins.push(inst);
    }

    pub fn dup(&mut self, i: usize) {
        self.append_inst(super::Inst::Dup(i));
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

    pub fn eq_number(&mut self) {
        self.append_inst(super::Inst::EquNumber);
    }

    pub fn neq_number(&mut self) {
        self.append_inst(super::Inst::NeqNumber);
    }

    pub fn lt_number(&mut self) {
        self.append_inst(super::Inst::LtNumber);
    }

    pub fn gt_number(&mut self) {
        self.append_inst(super::Inst::GtNumber);
    }

    pub fn leq_number(&mut self) {
        self.append_inst(super::Inst::LeqNumber);
    }

    pub fn geq_number(&mut self) {
        self.append_inst(super::Inst::GeqNumber);
    }

    pub fn eq_string(&mut self) {
        self.append_inst(super::Inst::EquString);
    }

    pub fn neq_string(&mut self) {
        self.append_inst(super::Inst::NeqString);
    }

    pub fn and(&mut self) {
        self.append_inst(super::Inst::And);
    }

    pub fn or(&mut self) {
        self.append_inst(super::Inst::Or);
    }

    pub fn load_const_int(&mut self, i: i64) {
        self.append_inst(super::Inst::LoadConstInt(i));
    }

    pub fn load_const_byte(&mut self, b: u8) {
        self.append_inst(super::Inst::LoadConstByte(b));
    }

    pub fn load_const_number(&mut self, n: f64) {
        self.append_inst(super::Inst::LoadConstNumber(n));
    }

    pub fn load_const_bool(&mut self, b: bool) {
        self.append_inst(super::Inst::LoadConstBool(b));
    }

    pub fn load_const_string(&mut self, s: StringRef) {
        self.append_inst(super::Inst::LoadConstString(s));
    }

    pub fn load_global(&mut self, g: GlobalRef) {
        self.append_inst(super::Inst::LoadGlobal(g));
    }

    pub fn truncate(&mut self) {
        self.append_inst(super::Inst::Truncate);
    }

    pub fn promote(&mut self) {
        self.append_inst(super::Inst::Promote);
    }

    pub fn load(&mut self, v: VariableRef) {
        self.append_inst(super::Inst::Load(v));
    }

    pub fn store(&mut self, v: VariableRef) {
        self.append_inst(super::Inst::Store(v));
    }

    pub fn tee(&mut self, v: VariableRef) {
        self.append_inst(super::Inst::Tee(v));
    }

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

    pub fn br_table(&mut self, default_block: BlockRef, blocks: Vec<BlockRef>) {
        self.append_inst(super::Inst::BrTable(default_block, blocks));
    }

    pub fn call(&mut self, id: String) {
        self.append_inst(super::Inst::Call(id));
    }

    pub fn indirect_call(&mut self, signature: Signature) {
        self.append_inst(super::Inst::IndirectCall(signature));
    }

    pub fn new_array(&mut self, size: usize, typ: Type) {
        self.append_inst(super::Inst::NewArray(size, typ));
    }

    pub fn load_array(&mut self, typ: Type) {
        self.append_inst(super::Inst::LoadArray(typ));
    }

    pub fn store_array(&mut self, typ: Type) {
        self.append_inst(super::Inst::StoreArray(typ));
    }

    pub fn new_object(&mut self, size: usize) {
        self.append_inst(super::Inst::NewObject(size));
    }

    pub fn set_object(&mut self, size: usize, typ: Type) {
        self.append_inst(super::Inst::SetObject(size, typ));
    }

    pub fn get_object(&mut self, size: usize, typ: Type) {
        self.append_inst(super::Inst::GetObject(size, typ));
    }

    pub fn check_yield(&mut self) {
        self.append_inst(super::Inst::CheckYield);
    }

    pub fn assert(&mut self) {
        self.append_inst(super::Inst::Assert);
    }

    pub fn finish(self) -> Box<super::Function> {
        return Box::new(self.func);
    }
}
