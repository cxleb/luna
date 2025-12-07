use crate::compiler::ast;
use crate::ir::builder::FuncBuilder;
use crate::ir::{self, StringMap};
use crate::types;

struct FuncGen<'a> {
    bld: FuncBuilder,
    str_map: &'a mut StringMap,
}

impl<'a> FuncGen<'a> {
    fn binary_expr(&mut self, e: &ast::Expr, b: &Box<ast::BinaryExpr>) {
        match e.typ.as_ref() {
            crate::types::Type::Number => {
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
            crate::types::Type::Integer => {
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
            crate::types::Type::Bool => {
                self.expr(&b.lhs);
                self.expr(&b.rhs);
                match b.lhs.typ.as_ref() {
                    crate::types::Type::Integer => {
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
                    crate::types::Type::Number => {
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
                    crate::types::Type::Bool => {
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
        for arg in c.parameters.iter() {
            self.expr(arg);
        }
        if let ast::ExprKind::Identifier(name) = &c.function.kind {
            self.bld.call(name.id.clone());
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
    
    fn selector(&mut self, _l: &Box<ast::Selector>) {
        unimplemented!()
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

    fn object_literal(&mut self, _o: &Box<ast::ObjectLiteral>) {
        todo!()
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
            ast::ExprKind::Selector(l) => self.selector(l),
            ast::ExprKind::ArrayLiteral(a) => self.array_literal(a),
            ast::ExprKind::ObjectLiteral(o) => self.object_literal(o),
        }
    }

    fn store_subscript(&mut self, e: &ast::Expr, l: &Box<ast::Subscript>) {
        self.expr(&l.index);
        self.expr(&l.value);
        self.bld.store_array(e.typ.clone());
    }

    fn store_selector(&mut self, _e: &ast::Expr, _l: &Box<ast::Selector>) {
        unimplemented!()
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
        let id = self
            .bld
            .create_var(v.id.clone(), v.type_annotation.clone().unwrap());
        self.expr(&v.value);
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
        let mut signature = ir::Signature {
            ret_types: Vec::new(),
            parameters: func.signature.params.iter().map(|p| p.type_annotation.clone()).collect()
        };
        if let Some(ret_type) = &func.signature.return_type {
            signature.ret_types.push(ret_type.clone());
        }
        let mut s = Self {
            str_map,
            bld: FuncBuilder::new(func.signature.id.clone(), signature),
        };
        s.bld.push_scope();
        // add params as variables for scope purposes
        for p in func.signature.params.iter() {
            s.bld.create_var(p.id.clone(), p.type_annotation.clone());
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

pub fn gen_function(func: &Box<ast::Func>, str_map: &mut StringMap) -> Box<ir::Function> {
    FuncGen::generate(func, str_map).finish()
}

pub fn gen_module(module: Box<ast::Module>) -> Box<ir::Module> {
    let mut ir_module = ir::Module { string_map: StringMap::new(), funcs: vec![] };

    for func in module.functions.iter() {
        let ir_func = gen_function(&func, &mut ir_module.string_map);
        ir_module.funcs.push(*ir_func);
    }

    Box::new(ir_module)
}
