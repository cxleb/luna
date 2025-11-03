use std::collections::HashMap;

use crate::compiler::ast;
use crate::types::{self, Type};

#[derive(Debug)]
pub enum SemaErrorReason {
    GenericError, // todo(caleb): Remove me!
    VariableNotFound,
    IncompatibleTypeInVariableDefinition,
    CannotUseExpressionInLeftHandExpression
}

#[derive(Debug)]
pub struct SemaError {
    reason: SemaErrorReason,
}

type SemaResult<X> = Result<X, SemaError>;

struct FuncTypeInference<'a> {
    signatures: &'a Vec<ast::FuncSignature>,
    variable_scopes: Vec<HashMap<String, Box<Type>>>,
}

impl<'a> FuncTypeInference<'a> {
    fn new(signatures: &'a Vec<ast::FuncSignature>) -> Self {
        Self {
            signatures,
            variable_scopes: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.variable_scopes.pop();
    }

    pub fn create_var(&mut self, name: String, typ: &Type) {
        self.variable_scopes.last_mut().unwrap().insert(name, Box::new(typ.clone()));
    }

    pub fn find_var(&self, name: &String) -> Option<&Type> {
        for scope in self.variable_scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }

    pub fn ok(&self) -> SemaResult<()> {
        Ok(())
    }

    pub fn error<T>(&self, reason: SemaErrorReason) -> SemaResult<T> {
        Err(SemaError { reason })
    }

    fn binary_expr(&mut self, b: &mut Box<ast::BinaryExpr>) -> SemaResult<()> {
        self.expr(&mut b.lhs)?;
        self.expr(&mut b.rhs)?;
        //match b.kind {
        //    ast::BinaryExprKind::Add => self.bld.add_int(),
        //    ast::BinaryExprKind::Subtract => self.bld.sub_int(),
        //    ast::BinaryExprKind::Multiply => self.bld.mul_int(),
        //    ast::BinaryExprKind::Divide => self.bld.div_int(),
        //    ast::BinaryExprKind::Equal => self.bld.eq_int(),
        //    ast::BinaryExprKind::NotEqual => self.bld.neq_int(),
        //    ast::BinaryExprKind::LessThan => self.bld.lt_int(),
        //    ast::BinaryExprKind::GreaterThan => self.bld.gt_int(),
        //    ast::BinaryExprKind::LessThanEqual => self.bld.leq_int(),
        //    ast::BinaryExprKind::GreaterThanEqual => self.bld.geq_int(),
        //}
        self.ok()
    }

    fn unary_expr(&mut self, _u: &mut Box<ast::UnaryExpr>) -> SemaResult<()> {
        self.ok()
    }

    fn assign(&mut self, a: &mut Box<ast::Assign>) -> SemaResult<()> {
        self.expr(&mut a.value)?;
        self.store_expr(&mut a.destination)?;
        self.ok()
    }

    fn call(&mut self, _c: &mut Box<ast::Call>) -> SemaResult<()> {
        self.ok()
    }

    fn integer(&mut self, _i: &mut Box<ast::Integer>) -> SemaResult<()> {
        self.ok()
    }

    fn number(&mut self, _f: &mut Box<ast::Number>) -> SemaResult<()> {
        self.ok()
    }

    fn string_literal(&mut self, _s: &mut Box<ast::StringLiteral>) -> SemaResult<()> {
        self.ok()
    }

    fn identifier(&mut self, e: &mut ast::Expr) -> SemaResult<()> { 
        let i = match &e.kind {
            ast::ExprKind::Identifier(i) => i,
            _ => panic!()
        };
        if let Some(typ) = self.find_var(&i.id) {
            e.typ = typ.clone().into();
            self.ok()
        } else {
            self.error(SemaErrorReason::VariableNotFound)
        }
    }

    fn lookup(&mut self, _l: &Box<ast::Lookup>) -> SemaResult<()> {
        self.ok()
    }

    fn array_literal(&mut self, _a: &Box<ast::ArrayLiteral>) -> SemaResult<()> {
        self.ok()
    }

    fn object_literal(&mut self, _o: &Box<ast::ObjectLiteral>) -> SemaResult<()> {
        self.ok()
    }

    fn expr(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        match &mut e.kind {
            ast::ExprKind::BinaryExpr(b) => self.binary_expr(b),
            ast::ExprKind::UnaryExpr(u) => self.unary_expr(u),
            ast::ExprKind::Assign(a) => self.assign(a),
            ast::ExprKind::Call(c) => self.call(c),
            ast::ExprKind::Integer(i) => self.integer(i),
            ast::ExprKind::Number(f) => self.number(f),
            ast::ExprKind::StringLiteral(s) => self.string_literal(s),
            ast::ExprKind::Identifier(_) => self.identifier(e),
            ast::ExprKind::Lookup(l) => self.lookup(l),
            ast::ExprKind::ArrayLiteral(a) => self.array_literal(a),
            ast::ExprKind::ObjectLiteral(o) => self.object_literal(o),
        }
    }

    fn store_lookup(&mut self, _e: &ast::Expr, _l: &Box<ast::Lookup>) -> SemaResult<()> {
        self.ok()
    }

    fn store_identifier(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        let i = match &e.kind {
            ast::ExprKind::Identifier(i) => i,
            _ => panic!()
        };
        if let Some(typ) = self.find_var(&i.id) {
            e.typ = typ.clone().into();
            self.ok()
        } else {
            self.error(SemaErrorReason::VariableNotFound)
        }
    }

    fn store_assign(&mut self, _e: &ast::Expr, _a: &Box<ast::Assign>) -> SemaResult<()> {
        unimplemented!()
    }

    // These are expressions which are going to be used to "store" a value
    // aka l values
    fn store_expr(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        match &e.kind {
            ast::ExprKind::Lookup(l) => self.store_lookup(e, l),
            ast::ExprKind::Identifier(_) => self.store_identifier(e),
            ast::ExprKind::Assign(a) => self.store_assign(e, a),
            _ => self.error(SemaErrorReason::CannotUseExpressionInLeftHandExpression)
        }
    }

    fn block_stmt(&mut self, b: &mut Box<ast::BlockStmt>) -> SemaResult<()> {
        self.push_scope();
        for s in b.stmts.iter_mut() {
            self.stmt(s)?;
        }
        self.pop_scope();
        self.ok()
    }

    fn expr_stmt(&mut self, e: &mut Box<ast::ExprStmt>) -> SemaResult<()> {
        self.expr(&mut e.expr)
    }

    fn for_stmt(&mut self, _f: &mut Box<ast::ForStmt>) -> SemaResult<()> {
        unimplemented!()
    }

    fn if_stmt(&mut self, f: &mut Box<ast::IfStmt>) -> SemaResult<()> {
        self.expr(&mut f.test)?;
        self.stmt(&mut f.consequent)?;
        if let Some(a) = &mut f.alternate{
            self.stmt(a)?;
        }
        self.ok()
    }

    fn return_stmt(&mut self, r: &mut Box<ast::ReturnStmt>) -> SemaResult<()> {
        if let Some(r) = &mut r.value {
            self.expr(r)?;
        }
        self.ok()
    }

    fn var_decl_stmt(&mut self, v: &mut Box<ast::VarDeclStmt>) -> SemaResult<()> {
        self.expr(&mut v.value)?;
        let ret = &v.value.typ;
        if let Some(annotation) = &v.type_annotation {
            if types::compare(&annotation, ret) == types::ComparisonResult::Incompatible {
                return self.error(SemaErrorReason::IncompatibleTypeInVariableDefinition);
            }
        } else {
            v.type_annotation = Some(ret.clone());
        }
        self.create_var(v.id.clone(), ret);
        self.ok()
    }

    fn while_stmt(&mut self, w: &mut Box<ast::WhileStmt>) -> SemaResult<()> {
        self.expr(&mut w.condition)?;
        self.stmt(&mut w.consequent)?;
        self.ok()
    }

    fn stmt(&mut self, s: &mut ast::Stmt) -> SemaResult<()> {
        match s {
            ast::Stmt::Block(b) => self.block_stmt(b),
            ast::Stmt::ExprStmt(e) => self.expr_stmt(e),
            ast::Stmt::For(f) => self.for_stmt(f),
            ast::Stmt::If(i) => self.if_stmt(i),
            ast::Stmt::Return(r) => self.return_stmt(r),
            ast::Stmt::VarDecl(v) => self.var_decl_stmt(v),
            ast::Stmt::While(w) => self.while_stmt(w),
        }
    }

    fn check(&mut self, func: & mut ast::Func) -> SemaResult<()> {
        self.push_scope();
        for p in func.signature.params.iter() {
            self.create_var(p.id.clone(), &p.type_annotation);
        }
        self.block_stmt(&mut func.body)?;
        self.pop_scope();
        self.ok()
    }
}

pub fn sema_function(signatures: &Vec<ast::FuncSignature>, func: & mut ast::Func) -> SemaResult<()> {
    FuncTypeInference::new(signatures).check(func)?;
    Ok(())
}

pub fn sema_module(module: &mut Box<ast::Module>) -> SemaResult<()> {
    let signatures = module.functions.iter().map(|f| f.signature.clone()).collect::<Vec<_>>();
    for func in module.functions.iter_mut() {
        sema_function(&signatures, func)?;
    }

    Ok(())
}
