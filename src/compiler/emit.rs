use crate::compiler::ast;
use crate::ir::builder::FuncBuilder;
use crate::ir::{self, StringMap};
use crate::types;

struct FuncGen<'a> {
    bld: FuncBuilder,
    str_map: &'a mut StringMap,
    self_var: Option<ir::VariableRef>,
}

impl<'a> FuncGen<'a> {
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

    fn call(&mut self, c: &Box<ast::Call>) {
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
        self.bld.load_array(e.typ.clone());
    }
    
    fn selector(&mut self, e: &ast::Expr, s: &ast::Selector) {
        self.expr(&s.value);
        self.bld.get_object(s.idx, e.typ.clone());
    }

    fn array_literal(&mut self, a: &Box<ast::ArrayLiteral>) {
        self.bld.new_array(a.literals.len());
        for (i, literal) in a.literals.iter().enumerate() {
            self.expr(literal);
            self.bld.load_const_int(i as i64);
            self.bld.dup(2);
            self.bld.store_array(literal.typ.clone());
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
                self.bld.set_object(i, field_type.clone());
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
        match &e.kind {
            ast::ExprKind::BinaryExpr(b) => self.binary_expr(e, b),
            ast::ExprKind::UnaryExpr(u) => self.unary_expr(u),
            ast::ExprKind::Assign(a) => self.assign(a),
            ast::ExprKind::Call(c) => self.call(c),
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
        self.bld.store_array(e.typ.clone());
    }

    fn store_selector(&mut self, e: &ast::Expr, s: &ast::Selector) {
        self.expr(&s.value);
        self.bld.set_object(s.idx, e.typ.clone());
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
        self.expr(&e.expr);
        // todo: pop value off stack if not used, there may be multiple values on stack
        false
    }

    fn for_stmt(&mut self, _f: &Box<ast::ForStmt>) -> bool {
        unimplemented!()
    }

    fn if_stmt(&mut self, f: &Box<ast::IfStmt>) -> bool {
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
        if let Some(r) = &r.value {
            self.expr(r);
        }
        self.bld.ret();
        true
    }

    fn var_decl_stmt(&mut self, v: &Box<ast::VarDeclStmt>) -> bool {
        self.expr(&v.value);
        let id = self
            .bld
            .create_var(v.id.clone(), v.value.typ.clone());
        self.bld.store(id);
        false
    }

    fn while_stmt(&mut self, w: &Box<ast::WhileStmt>) -> bool {
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

    fn stmt(&mut self, s: &ast::Stmt) -> bool {
        match s {
            ast::Stmt::Block(b) => self.block_stmt(&b),
            ast::Stmt::ExprStmt(e) => self.expr_stmt(&e),
            ast::Stmt::For(f) => self.for_stmt(&f),
            ast::Stmt::If(i) => self.if_stmt(&i),
            ast::Stmt::Return(r) => self.return_stmt(&r),
            ast::Stmt::VarDecl(v) => self.var_decl_stmt(&v),
            ast::Stmt::While(w) => self.while_stmt(&w),
        }
    }

    fn generate(func: &Box<ast::Func>, str_map: &'a mut StringMap) -> Self {
        let signature = ir::Signature {
            ret_types: func.typ_.returns.iter().cloned().collect(),
            parameters: func.typ_.params.iter().cloned().collect()
        };
        let mut s = Self {
            str_map,
            bld: FuncBuilder::new(func.signature.symbol_name.clone(), signature),
            self_var: None,
        };
        s.bld.push_scope();
        // add params as variables for scope purposes
        for (p, sig_p) in func.signature.params.iter().zip(func.typ_.params.iter()) {
            s.bld.create_var(p.id.clone(), sig_p.clone());
        }
        if !s.block_stmt(&func.body) {
            s.bld.ret();
        }
        s.bld.pop_scope();
        s
    }

    fn generate_struct_func(func: &Box<ast::Func>, str_map: &'a mut StringMap, struct_type: types::Type) -> Self {
        let mut signature = ir::Signature {
            ret_types: func.typ_.returns.iter().cloned().collect(),
            parameters: func.typ_.params.iter().cloned().collect()
        };
        signature.parameters.insert(0, struct_type.clone());
        let mut s = Self {
            str_map,
            bld: FuncBuilder::new(func.signature.symbol_name.clone(), signature),
            self_var: None,
        };
        s.bld.push_scope();
        // add params as variables for scope purposes
        s.self_var = Some(s.bld.create_var("".into(), struct_type.clone()));
        for (p, sig_p) in func.signature.params.iter().zip(func.typ_.params.iter()) {
            s.bld.create_var(p.id.clone(), sig_p.clone());
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
    let mut ir_module = ir::Module { string_map: StringMap::new(), funcs: vec![] };
    
    for package in program.packages.iter() {
        for file in package.files.iter() {
            for func in file.functions.iter() {
                let ir_func = FuncGen::generate(func, &mut ir_module.string_map).finish();
                ir_module.funcs.push(*ir_func);
            }
            for _struct in file.structs.iter() {
                for func in _struct.functions.iter() {
                    let ir_func = FuncGen::generate_struct_func(func, &mut ir_module.string_map, _struct.typ.clone()).finish();
                    ir_module.funcs.push(*ir_func);
                }
            }
        }
    }

    Box::new(ir_module)
}