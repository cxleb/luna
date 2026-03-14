use std::collections::HashMap;

use crate::compiler::{SourceLoc, ast};
use crate::ir::builder::FuncBuilder;
use crate::ir::{self, BlockRef, StringMap, StringRef, Type, VariableRef};
use crate::types;


// This implementation is a copy of cranelift's switch api.
// Redone to target our IR, which has the same building blocks as CLIR but 
// we dont want to implement this in the runtime translate, its better here 
// so its runtime agnostic 
// https://github.com/bytecodealliance/wasmtime/blob/f6b2700cc89118308faadc08e72092642928f9cf/cranelift/frontend/src/switch.rs#L42
struct SwitchEmitter {
    cases: HashMap<i64, BlockRef>,
}

impl SwitchEmitter {
    /// Create a new empty switch
    pub fn new() -> Self {
        Self {
            cases: HashMap::new(),
        }
    }

    /// Set a switch entry
    pub fn set_entry(&mut self, index: i64, block: BlockRef) {
        let prev = self.cases.insert(index, block);
        assert!(prev.is_none(), "Tried to set the same entry {index} twice");
    }



    /// Turn the `cases` `HashMap` into a list of `ContiguousCaseRange`s.
    ///
    /// # Postconditions
    ///
    /// * Every entry will be represented.
    /// * The `ContiguousCaseRange`s will not overlap.
    /// * Between two `ContiguousCaseRange`s there will be at least one entry index.
    /// * No `ContiguousCaseRange`s will be empty.
    fn collect_contiguous_case_ranges(self) -> Vec<ContiguousCaseRange> {
        let mut cases = self.cases.into_iter().collect::<Vec<(_, _)>>();
        cases.sort_by_key(|&(index, _)| index);

        let mut contiguous_case_ranges: Vec<ContiguousCaseRange> = vec![];
        let mut last_index = None;
        for (index, block) in cases {
            match last_index {
                None => contiguous_case_ranges.push(ContiguousCaseRange::new(index)),
                Some(last_index) => {
                    if index > last_index + 1 {
                        contiguous_case_ranges.push(ContiguousCaseRange::new(index));
                    }
                }
            }
            contiguous_case_ranges
                .last_mut()
                .unwrap()
                .blocks
                .push(block);
            last_index = Some(index);
        }

        contiguous_case_ranges
    }

    /// Binary search for the right `ContiguousCaseRange`.
    fn build_search_tree<'a>(
        bx: &mut FuncBuilder,
        temp: VariableRef,
        otherwise: BlockRef,
        contiguous_case_ranges: &'a [ContiguousCaseRange],
    ) {
        // If no switch cases were added to begin with, we can just emit `jump otherwise`.
        if contiguous_case_ranges.is_empty() {
            bx.br(otherwise);
            return;
        }

        // Avoid allocation in the common case
        if contiguous_case_ranges.len() <= 3 {
            Self::build_search_branches(bx, otherwise, temp, contiguous_case_ranges);
            return;
        }

        let mut stack = Vec::new();
        stack.push((None, contiguous_case_ranges));

        while let Some((block, contiguous_case_ranges)) = stack.pop() {
            if let Some(block) = block {
                bx.switch_to_block(block);
            }

            if contiguous_case_ranges.len() <= 3 {
                Self::build_search_branches(bx, temp, otherwise, contiguous_case_ranges);
            } else {
                let split_point = contiguous_case_ranges.len() / 2;
                let (left, right) = contiguous_case_ranges.split_at(split_point);

                let left_block = bx.new_block();
                let right_block = bx.new_block();

                let first_index = right[0].first_index;
                bx.load(temp);
                bx.load_const_int(first_index);
                bx.geq_int();
                bx.condbr(right_block, left_block);

                stack.push((Some(left_block), left));
                stack.push((Some(right_block), right));
            }
        }
    }

    /// Linear search for the right `ContiguousCaseRange`.
    fn build_search_branches<'a>(
        bx: &mut FuncBuilder,
        otherwise: BlockRef,
        temp: VariableRef,
        contiguous_case_ranges: &'a [ContiguousCaseRange],
    ) {
        for (ix, range) in contiguous_case_ranges.iter().enumerate().rev() {
            let alternate = if ix == 0 {
                otherwise
            } else {
                bx.new_block()
            };

            if range.first_index == 0 {
                assert_eq!(alternate, otherwise);

                if let Some(block) = range.single_block() {
                    bx.load(temp);
                    bx.condbr(otherwise, block);
                } else {
                    Self::build_jump_table(bx, otherwise, temp, 0, &range.blocks);
                }
            } else {
                if let Some(block) = range.single_block() {
                    bx.load(temp);
                    bx.load_const_int(range.first_index);
                    bx.eq_int();
                    bx.condbr(block, alternate);
                } else {
                    let jt_block = bx.new_block();
                    bx.load(temp);
                    bx.load_const_int(range.first_index);
                    bx.geq_int();
                    bx.condbr(jt_block, alternate);
                    bx.switch_to_block(jt_block);
                    Self::build_jump_table(bx, otherwise, temp, range.first_index, &range.blocks);
                }
            }

            if alternate != otherwise {
                bx.switch_to_block(alternate);
            }
        }
    }

    fn build_jump_table(
        bx: &mut FuncBuilder,
        otherwise: BlockRef,
        temp: VariableRef,
        first_index: i64,
        blocks: &[BlockRef],
    ) {
        bx.load(temp);

        if first_index != 0 {
            bx.load_const_int(first_index.wrapping_neg());
            bx.sub_int();
        };

        bx.br_table(otherwise, blocks.into());
    }

    /// Build the switch
    ///
    /// # Arguments
    ///
    /// * The function builder to emit to
    /// * The default block
    pub fn emit(self, bx: &mut FuncBuilder, default: BlockRef) {
        let contiguous_case_ranges = self.collect_contiguous_case_ranges();
        let temp = bx.create_temp(Type::Integer);
        bx.store(temp);
        Self::build_search_tree(bx, temp, default, &contiguous_case_ranges);
    }
}

/// This represents a contiguous range of cases to switch on.
///
/// For example 10 => block1, 11 => block2, 12 => block7 will be represented as:
///
/// ```plain
/// ContiguousCaseRange {
///     first_index: 10,
///     blocks: vec![Block::from_u32(1), Block::from_u32(2), Block::from_u32(7)]
/// }
/// ```
#[derive(Debug)]
struct ContiguousCaseRange {
    /// The entry index of the first case. Eg. 10 when the entry indexes are 10, 11, 12 and 13.
    first_index: i64,

    /// The blocks to jump to sorted in ascending order of entry index.
    blocks: Vec<BlockRef>,
}

impl ContiguousCaseRange {
    fn new(first_index: i64) -> Self {
        Self {
            first_index,
            blocks: Vec::new(),
        }
    }

    /// Returns `Some` block when there is only a single block in this range.
    fn single_block(&self) -> Option<BlockRef> {
        if self.blocks.len() == 1 {
            Some(self.blocks[0])
        } else {
            None
        }
    }
}

fn translate_type(ty: &crate::types::Type) -> ir::Type {
    match ty.kind() {
        crate::types::TypeKind::Integer => ir::Type::Integer,
        crate::types::TypeKind::Bool => ir::Type::Bool,
        crate::types::TypeKind::Number => ir::Type::Number,
        _ => ir::Type::Reference
    }
}

impl Into<ir::Type> for crate::types::Type {
    fn into(self) -> ir::Type {
        translate_type(&self)
    }
}

struct FuncGen<'a> {
    interned_file_name: StringRef,
    bld: FuncBuilder<'a>,
    str_map: &'a mut StringMap,
    self_var: Option<ir::VariableRef>,
}

impl<'a> FuncGen<'a> {
    fn emit_source_loc(&mut self, source_loc: SourceLoc) {
        self.bld.source_loc(crate::ir::SourceLoc {
            file: self.interned_file_name,
            line: source_loc.line,
            col: source_loc.col
        });
    }

    fn binary_expr(&mut self, e: &ast::Expr, b: &Box<ast::BinaryExpr>) {
        match e.typ.kind() {
            crate::types::TypeKind::Number => {
                self.expr(&b.lhs);
                // auto cast to number
                if types::is_integer(&b.lhs.typ) {
                    self.bld.promote();
                }
                self.expr(&b.rhs);
                // auto cast to number
                if types::is_integer(&b.rhs.typ) {
                    self.bld.promote();
                }
                match b.kind {
                    ast::BinaryExprKind::Add => self.bld.add_number(),
                    ast::BinaryExprKind::Subtract => self.bld.sub_number(),
                    ast::BinaryExprKind::Multiply => self.bld.mul_number(),
                    ast::BinaryExprKind::Divide => self.bld.div_number(),
                    _ => { panic!("Invalid condition for type"); }
                }
            },
            crate::types::TypeKind::Integer => {
                // auto cast to integer is not supported
                self.expr(&b.lhs);
                self.expr(&b.rhs);
                match b.kind {
                    ast::BinaryExprKind::Add => self.bld.add_int(),
                    ast::BinaryExprKind::Subtract => self.bld.sub_int(),
                    ast::BinaryExprKind::Multiply => self.bld.mul_int(),
                    ast::BinaryExprKind::Divide => self.bld.div_int(),
                    _ => { panic!("Invalid condition for type"); }
                }
            },
            crate::types::TypeKind::Bool => {
                self.expr(&b.lhs);
                self.expr(&b.rhs);
                match b.lhs.typ.kind() {
                    crate::types::TypeKind::Integer => {
                        match b.kind {
                            ast::BinaryExprKind::Equal => self.bld.eq_int(),
                            ast::BinaryExprKind::NotEqual => self.bld.neq_int(),
                            ast::BinaryExprKind::LessThan => self.bld.lt_int(),
                            ast::BinaryExprKind::GreaterThan => self.bld.gt_int(),
                            ast::BinaryExprKind::LessThanEqual => self.bld.leq_int(),
                            ast::BinaryExprKind::GreaterThanEqual => self.bld.geq_int(),
                            _ => panic!("Invalid binary operation for bool type"),
                        }
                    },
                    crate::types::TypeKind::Number => {
                        match b.kind {
                            ast::BinaryExprKind::Equal => self.bld.eq_number(),
                            ast::BinaryExprKind::NotEqual => self.bld.neq_number(),
                            ast::BinaryExprKind::LessThan => self.bld.lt_number(),
                            ast::BinaryExprKind::GreaterThan => self.bld.gt_number(),
                            ast::BinaryExprKind::LessThanEqual => self.bld.leq_number(),
                            ast::BinaryExprKind::GreaterThanEqual => self.bld.geq_number(),
                            _ => panic!("Invalid binary operation for bool type"),
                        }
                    },
                    crate::types::TypeKind::Bool => {
                        match b.kind {
                            ast::BinaryExprKind::LogicalAnd => self.bld.and(),
                            ast::BinaryExprKind::LogicalOr => self.bld.or(),
                            _ => panic!("Invalid binary operation for bool type"),
                        }
                    },
                    _ => { panic!("Invalid lhs type for bool binary expr"); }
                }
                
            },
            _ => { panic!("Cant generate IR for {:?}", e.typ); }
        }
    }

    fn unary_expr(&mut self, _u: &Box<ast::UnaryExpr>) {
        todo!()
    }

    fn assign(&mut self, a: &Box<ast::Assign>) {
        self.expr(&a.value);
        self.store_expr(&a.destination);
    }

    fn enum_literal(&mut self, typ: &types::Type, i: usize, values: &Vec<ast::Expr>) {
        let enum_size = types::get_max_enum_values(typ);
        self.bld.new_object(enum_size + 1);
        self.bld.load_const_int(i as i64);
        self.bld.dup(1);
        self.bld.set_object(0, Type::Integer);

        for (i, value) in values.iter().enumerate() {
            self.expr(value);
            self.bld.dup(1);
            self.bld.set_object(i + 1, value.typ.clone().into());
        }
    }

    fn call(&mut self, c: &ast::Call, e: &ast::Expr) {
        if let Some(i) = c.enum_idx {
            self.enum_literal(&e.typ, i, &c.parameters);
            return;
        }

        if let ast::ExprKind::Identifier(id) = &c.function.kind {
            if id.id == "assert" {
                self.expr(&c.parameters[0]);
                self.bld.assert();
                return;
            }
        }

        self.bld.check_yield();
        // when it is a struct/interface function call, we need to load self first
        if let ast::ExprKind::Selector(s) = &c.function.kind {
            self.expr(&s.value);
        }
        for arg in c.parameters.iter() {
            self.expr(arg);
        }
        if let Some(name) = &c.symbol_name {
            self.bld.call(name.clone());
        } else {
            self.expr(&c.function);
            self.bld.indirect_call();
        }
    }

    fn integer(&mut self, i: &Box<ast::Integer>) {
        self.bld.load_const_int(i.value);
    }

    fn number(&mut self, f: &Box<ast::Number>) {
        self.bld.load_const_number(f.value);
    }

    fn boolean(&mut self, b: &Box<ast::Bool>) {
        self.bld.load_const_bool(b.value);
    }

    fn string_literal(&mut self, s: &Box<ast::StringLiteral>) {
        let s = self.str_map.intern(&s.value);
        self.bld.load_const_string(s);
    }

    fn identifier(&mut self, i: &Box<ast::Identifier>) {
        if let Some(var_id) = self.bld.find_var(&i.id) {
            self.bld.load(var_id);
        } else {
            panic!("Undefined variable {}", i.id);
        }
    }

    fn subscript(&mut self, e: &ast::Expr, l: &Box<ast::Subscript>) {
        self.expr(&l.index);
        self.expr(&l.value);
        self.bld.load_array(e.typ.clone().into());
    }
    
    fn selector(&mut self, e: &ast::Expr, s: &ast::Selector) {
        if let Some(i) = s.enum_idx {
            self.enum_literal(&e.typ, i, &Vec::new());
            return;
        }
        self.expr(&s.value);
        self.bld.get_object(s.idx, e.typ.clone().into());
    }

    fn array_literal(&mut self, a: &Box<ast::ArrayLiteral>) {
        self.bld.new_array(a.literals.len());
        for (i, literal) in a.literals.iter().enumerate() {
            self.expr(literal);
            self.bld.load_const_int(i as i64);
            self.bld.dup(2);
            self.bld.store_array(literal.typ.clone().into());
        }
    }

    fn object_literal(&mut self, typ: &types::Type, o: &Box<ast::ObjectLiteral>) {
        self.bld.new_object(o.fields.len());
        // We need to set all the fields which we got then we need to provide defaults for the rest
        if let crate::types::TypeKind::Struct(struct_fields) = typ.kind() {
            for (i, (field_name, field_type)) in struct_fields.fields.read().unwrap().iter().enumerate() {
                if let Some(value) = o.fields.iter().find(|field| &field.id == field_name) {
                    self.expr(&value.value);
                } else { 
                    // provide default value
                    match field_type.kind() {
                        crate::types::TypeKind::Integer => self.bld.load_const_int(0),
                        crate::types::TypeKind::Number => self.bld.load_const_number(0.0),
                        crate::types::TypeKind::Bool => self.bld.load_const_bool(false),
                        crate::types::TypeKind::String => {
                            let s = self.str_map.intern("");
                            self.bld.load_const_string(s);
                        },
                        crate::types::TypeKind::Array(_) => {
                            self.bld.new_array(0);
                        },
                        _ => { panic!("Cannot provide default value for field type {:?}", field_type); }
                    }
                }
                self.bld.dup(1);
                self.bld.set_object(i, field_type.clone().into());
            }
        } else {
            panic!("Trying to emit object literal but type was not struct!")
        }
    }

    fn _self(&mut self) {
        if let Some(self_var) = self.self_var {
            self.bld.load(self_var);
        } else {
            panic!("Trying to use self in a non struct function");
        }
    }

    fn expr(&mut self, e: &ast::Expr) {
        self.emit_source_loc(e.loc);
        match &e.kind {
            ast::ExprKind::BinaryExpr(b) => self.binary_expr(e, b),
            ast::ExprKind::UnaryExpr(u) => self.unary_expr(u),
            ast::ExprKind::Assign(a) => self.assign(a),
            ast::ExprKind::Call(c) => self.call(c, e),
            ast::ExprKind::Integer(i) => self.integer(i),
            ast::ExprKind::Number(f) => self.number(f),
            ast::ExprKind::Boolean(b) => self.boolean(b),
            ast::ExprKind::StringLiteral(s) => self.string_literal(s),
            ast::ExprKind::Identifier(i) => self.identifier(i),
            ast::ExprKind::Subscript(l) => self.subscript(e, l),
            ast::ExprKind::Selector(l) => self.selector(e, l),
            ast::ExprKind::ArrayLiteral(a) => self.array_literal(a),
            ast::ExprKind::ObjectLiteral(o) => self.object_literal(&e.typ, o),
            ast::ExprKind::_Self => self._self(),
        }
    }

    fn store_subscript(&mut self, e: &ast::Expr, l: &Box<ast::Subscript>) {
        self.expr(&l.index);
        self.expr(&l.value);
        self.bld.store_array(e.typ.clone().into());
    }

    fn store_selector(&mut self, e: &ast::Expr, s: &ast::Selector) {
        self.expr(&s.value);
        self.bld.set_object(s.idx, e.typ.clone().into());
    }

    fn store_identifier(&mut self, _e: &ast::Expr, i: &Box<ast::Identifier>) {
        if let Some(var_id) = self.bld.find_var(&i.id) {
            self.bld.tee(var_id);
        } else {
            panic!("Undefined variable {}", i.id);
        }
    }

    // These are expressions which are going to be used to "store" a value
    // aka l values
    fn store_expr(&mut self, e: &ast::Expr) {
        //println!("here {:?}", e);
        self.emit_source_loc(e.loc);
        match &e.kind {
            ast::ExprKind::Subscript(l) => self.store_subscript(e, l),
            ast::ExprKind::Selector(l) => self.store_selector(e, l),
            ast::ExprKind::Identifier(i) => self.store_identifier(e, i),
            //ast::ExprKind::Assign(a) => self.store_assign(e, a),
            //ast::Expr::BinaryExpr(b)
            //ast::Expr::UnaryExpr(u)
            //ast::Expr::Call(c)
            //ast::Expr::Integer(i)
            //ast::Expr::Number(f)
            //ast::Expr::StringLiteral(s)
            //ast::Expr::ArrayLiteral(a)
            //ast::Expr::ObjectLiteral(o)
            _ => {}
        }
    }

    fn block_stmt(&mut self, b: &Box<ast::BlockStmt>) -> bool {
        self.emit_source_loc(b.loc);
        let mut did_return = false;
        self.bld.push_scope();
        for s in b.stmts.iter() {
            if self.stmt(&s) {
                did_return = true;
                break;
            }
        }
        self.bld.pop_scope();
        did_return
    }

    fn expr_stmt(&mut self, e: &Box<ast::ExprStmt>) -> bool {
        self.emit_source_loc(e.loc);
        self.expr(&e.expr);
        // todo: pop value off stack if not used, there may be multiple values on stack
        false
    }

    fn for_stmt(&mut self, f: &Box<ast::ForStmt>) -> bool {
        self.emit_source_loc(f.loc);
        unimplemented!()
    }

    fn if_stmt(&mut self, f: &Box<ast::IfStmt>) -> bool {
        self.emit_source_loc(f.loc);
        if let Some(alternate) = &f.alternate {
            let consequent_block = self.bld.new_block();
            let alternate_block = self.bld.new_block();
            self.expr(&f.test);
            if f.not {
                self.bld.condbr(alternate_block, consequent_block);
            } else {
                self.bld.condbr(consequent_block, alternate_block);
            }
            self.bld.switch_to_block(consequent_block);
            let consequent_returned = self.stmt(&f.consequent);
            self.bld.switch_to_block(alternate_block);
            let alternate_returned = self.stmt(alternate);
            let needs_finish = !(consequent_returned && alternate_returned);
            if needs_finish {
                let finish_block = self.bld.new_block();
                if !consequent_returned {
                    self.bld.switch_to_block(consequent_block);
                    self.bld.br(finish_block);
                }
                if !alternate_returned {
                    self.bld.switch_to_block(alternate_block);
                    self.bld.br(finish_block)
                };
                self.bld.switch_to_block(finish_block);
                false
            } else {
                true
            }
        } else {
            let consequent_block = self.bld.new_block();
            let finish_block = self.bld.new_block();
            self.expr(&f.test);
            if f.not {
                self.bld.condbr(finish_block, consequent_block);
            } else {
                self.bld.condbr(consequent_block, finish_block);
            }
            self.bld.switch_to_block(consequent_block);
            self.stmt(&f.consequent);
            // In this situation, we always need to branch to finish
            self.bld.br(finish_block);
            self.bld.switch_to_block(finish_block);
            false
        }
    }

    fn return_stmt(&mut self, r: &Box<ast::ReturnStmt>) -> bool {
        self.emit_source_loc(r.loc);
        if let Some(r) = &r.value {
            self.expr(r);
        }
        self.bld.ret();
        true
    }

    fn var_decl_stmt(&mut self, v: &Box<ast::VarDeclStmt>) -> bool {
        self.emit_source_loc(v.loc);
        self.expr(&v.value);
        let id = self
            .bld
            .create_var(v.id.clone(), v.value.typ.clone().into());
        self.bld.store(id);
        false
    }

    fn while_stmt(&mut self, w: &Box<ast::WhileStmt>) -> bool {
        self.emit_source_loc(w.loc);
        let condition_block = self.bld.new_block();
        let body_block = self.bld.new_block();
        let finish_block = self.bld.new_block();

        self.bld.br(condition_block);
        self.bld.switch_to_block(condition_block);
        self.expr(&w.condition);
        self.bld.condbr(body_block, finish_block);
        self.bld.switch_to_block(body_block);
        let did_return = self.stmt(&w.consequent);
        if did_return {
            true
        } else {
            self.bld.check_yield();
            self.bld.br(condition_block);
            self.bld.switch_to_block(finish_block);
            false
        }
    }

    fn switch_stmt(&mut self, s: &ast::SwitchStmt) -> bool {
        self.emit_source_loc(s.loc);
        let mut did_return = s.cases.len() > 0;
        let prev_block = self.bld.current_block();

        let finish_block = self.bld.new_block();
        //let mut blocks = Vec::new();

        self.expr(&s.value);

        let default = if let Some(c) = s.cases.iter().find(|c| matches!(c.pattern.kind, ast::PatternKind::CatchAll)) {
            let catch_all = self.bld.new_block();
            self.bld.switch_to_block(catch_all);
            did_return &= self.block_stmt(&c.block);
            self.bld.br(finish_block);
            self.bld.switch_to_block(prev_block);
            catch_all
        } else {
            finish_block
        };

        let mut switch_emitter = SwitchEmitter::new();

        if types::is_enum(&s.value.typ) {
            let temp = self.bld.create_temp(Type::Reference);
            self.bld.store(temp);
            self.bld.load(temp);
            self.bld.get_object(0, Type::Integer);

            for case in s.cases.iter() {
                match &case.pattern.kind {
                    ast::PatternKind::EnumVariant { id:_, values } => {
                        let block = self.bld.new_block();
                        self.bld.switch_to_block(block);
                        self.bld.push_scope();
                        for (i, (name, typ)) in values.iter().enumerate() {
                            let var = self.bld.create_var(name.clone(), typ.clone().into());
                            self.bld.load(temp);
                            self.bld.get_object(i + 1, typ.clone().into());
                            self.bld.store(var);
                        }
                        did_return &= self.block_stmt(&case.block);
                        self.bld.pop_scope();
                        self.bld.br(finish_block);
                        switch_emitter.set_entry(case.case_idx, block);
                    }
                    _ => {}
                }
            }
        } else if types::is_integer(&s.value.typ) {
            for case in s.cases.iter() {
                match &case.pattern.kind {
                    ast::PatternKind::Integer(i) => {
                        let block = self.bld.new_block();
                        self.bld.switch_to_block(block);
                        did_return &= self.block_stmt(&case.block);
                        self.bld.br(finish_block);
                        self.bld.switch_to_block(prev_block);
                        
                        switch_emitter.set_entry(*i, block);
                    },
                    ast::PatternKind::IntegerRange(_, _) => unimplemented!(),
                    _ => {}
                }
            }
        } else {
            unimplemented!();
        }

        self.bld.switch_to_block(prev_block);
        switch_emitter.emit(&mut self.bld, default);
        self.bld.switch_to_block(finish_block);

        did_return
    }

    fn stmt(&mut self, s: &ast::Stmt) -> bool {
        match s {
            ast::Stmt::Block(b) => self.block_stmt(&b),
            ast::Stmt::ExprStmt(e) => self.expr_stmt(&e),
            ast::Stmt::For(f) => self.for_stmt(&f),
            ast::Stmt::If(i) => self.if_stmt(&i),
            ast::Stmt::Return(r) => self.return_stmt(&r),
            ast::Stmt::VarDecl(v) => self.var_decl_stmt(&v),
            ast::Stmt::While(w) => self.while_stmt(&w),
            ast::Stmt::Switch(s) => self.switch_stmt(&s),
        }
    }

    fn generate(func: &Box<ast::Func>, ir_module: &'a mut ir::Module, interned_file_name: StringRef) -> Self {
        let signature = ir::Signature {
            ret_types: func.typ_.returns.iter().map(|t| t.clone().into()).collect(),
            parameters: func.typ_.params.iter().map(|t| t.clone().into()).collect()
        };
        let mut s = Self {
            str_map: &mut ir_module.string_map,
            bld: FuncBuilder::new(func.signature.symbol_name.clone(), signature, &mut ir_module.source_locs),
            self_var: None,
            interned_file_name
        };
        s.bld.push_scope();
        // add params as variables for scope purposes
        for (p, sig_p) in func.signature.params.iter().zip(func.typ_.params.iter()) {
            s.bld.create_var(p.id.clone(), sig_p.clone().into());
        }
        if !s.block_stmt(&func.body) {
            s.bld.ret();
        }
        s.bld.pop_scope();
        s
    }

    fn generate_struct_func(func: &Box<ast::Func>, ir_module: &'a mut ir::Module, struct_type: types::Type, interned_file_name: StringRef) -> Self {
        let mut signature = ir::Signature {
            ret_types: func.typ_.returns.iter().map(|t| t.clone().into()).collect(),
            parameters: func.typ_.params.iter().map(|t| t.clone().into()).collect()
        };
        signature.parameters.insert(0, struct_type.clone().into());
        let mut s = Self {
            str_map: &mut ir_module.string_map,
            bld: FuncBuilder::new(func.signature.symbol_name.clone(), signature, &mut ir_module.source_locs),
            self_var: None,
            interned_file_name
        };
        s.bld.push_scope();
        // add params as variables for scope purposes
        s.self_var = Some(s.bld.create_var("".into(), struct_type.clone().into()));
        for (p, sig_p) in func.signature.params.iter().zip(func.typ_.params.iter()) {
            s.bld.create_var(p.id.clone(), sig_p.clone().into());
        }
        if !s.block_stmt(&func.body) {
            s.bld.ret();
        }
        s.bld.pop_scope();
        s
    }

    fn finish(self) -> Box<ir::Function> {
        self.bld.finish()
    }
}

pub fn emit_program(program: &ast::Program) -> Box<ir::Module> {
    let mut ir_module = ir::Module { string_map: StringMap::new(), funcs: vec![], source_locs: Default::default() };
    
    for package in program.packages.iter() {
        for file in package.files.iter() {
            let interned_file_name = ir_module.string_map.intern(&file.id);
            for func in file.functions.iter() {
                let ir_func = FuncGen::generate(func, &mut ir_module, interned_file_name).finish();
                ir_module.funcs.push(*ir_func);
            }
            for _struct in file.structs.iter() {
                for func in _struct.functions.iter() {
                    let ir_func = FuncGen::generate_struct_func(func, &mut ir_module, _struct.typ.clone(), interned_file_name).finish();
                    ir_module.funcs.push(*ir_func);
                }
            }
        }
    }

    //println!("Generated IR: {:#?}", ir_module);

    Box::new(ir_module)
}