use std::collections::HashMap;

use crate::builtins::Builtins;
use crate::compiler::{SourceLoc, ast};
use crate::types::{self, Type};

#[derive(Debug)]
pub enum SemaErrorReason {
    GenericError, // todo(caleb): Remove me!
    VariableNotFound,
    NonNumericTypeInBinaryExpression,
    IncompatibleTypesInBinaryExpression,
    IncompatibleTypesInVariableDefinition,
    CannotUseExpressionInLeftHandExpression,
    UnexpectedReturnValue,
    MissingReturnValue,
    IncompatibleTypesInReturnValue,
    ExpectedBooleanInTestCondition,
    FunctionNotFound,
    CallNotEnoughArguments,
    CallTooManyArguments,
    CallArgumentTypeMismatch,
    ValueIsNotIndexable,
    ValueCannotBeUsedAsIndex,
    AssignmentTypesIncompatible,
}

#[derive(Debug)]
pub struct SemaError {
    reason: SemaErrorReason,
    loc: SourceLoc,
}

type SemaResult<X> = Result<X, SemaError>;

struct FuncTypeInference<'a> {
    own_signature:&'a ast::FuncSignature,
    signatures: &'a Vec<ast::FuncSignature>,
    variable_scopes: Vec<HashMap<String, Box<Type>>>,
}

impl<'a> FuncTypeInference<'a> {
    fn new(own_signature: &'a ast::FuncSignature, signatures: &'a Vec<ast::FuncSignature>) -> Self {
        Self {
            own_signature,
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
        Err(SemaError { reason, loc: SourceLoc::default() })
    }

    pub fn error_loc<T>(&self, reason: SemaErrorReason, loc: SourceLoc) -> SemaResult<T> {
        Err(SemaError { reason, loc })
    }

    fn binary_expr(&mut self, e: &mut ast::Expr, type_hint: Option<&types::Type>) -> SemaResult<()> {
        let b = match &mut e.kind {
            ast::ExprKind::BinaryExpr(b) => b,
            _ => panic!()
        };
        
        self.expr(&mut b.lhs, type_hint)?;
        self.expr(&mut b.rhs, type_hint)?;

        match b.kind {
            ast::BinaryExprKind::Add |
            ast::BinaryExprKind::Subtract |
            ast::BinaryExprKind::Multiply |
            ast::BinaryExprKind::Divide => {
                let mut typ = b.lhs.typ.clone();
                
                if types::compare(&b.lhs.typ, &b.rhs.typ) == types::ComparisonResult::Incompatible {
                    // If we are doing something with an int and number, promote the int to a number
                    if types::is_numeric(&b.lhs.typ) && types::is_numeric(&b.rhs.typ) {
                        if types::is_number(&b.lhs.typ) || types::is_number(&b.rhs.typ) {
                            typ = types::number();
                        } else {
                            typ = types::integer();
                        }
                    } else {
                        return self.error(SemaErrorReason::IncompatibleTypesInBinaryExpression);
                    }
                }

                if !types::is_numeric(&b.rhs.typ) {
                    return self.error_loc(SemaErrorReason::NonNumericTypeInBinaryExpression, e.loc);
                }
                if !types::is_numeric(&b.lhs.typ) {
                    return self.error_loc(SemaErrorReason::NonNumericTypeInBinaryExpression, e.loc);
                }
                e.typ = typ;
            }
            ast::BinaryExprKind::Equal |
            ast::BinaryExprKind::NotEqual |
            ast::BinaryExprKind::LessThan |
            ast::BinaryExprKind::GreaterThan |
            ast::BinaryExprKind::LessThanEqual |
            ast::BinaryExprKind::GreaterThanEqual => {
                if types::compare(&b.lhs.typ, &b.rhs.typ) == types::ComparisonResult::Incompatible {
                    return self.error(SemaErrorReason::IncompatibleTypesInBinaryExpression);
                }

                e.typ = types::bool();
            }
        }
        self.ok()
    }

    fn unary_expr(&mut self, _u: &mut Box<ast::UnaryExpr>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn assign(&mut self, e: &mut ast::Expr, type_hint: Option<&types::Type>) -> SemaResult<()> {
        let a = match &mut e.kind {
            ast::ExprKind::Assign(a) => a,
            _ => panic!()
        };
        
        self.expr(&mut a.value, type_hint)?;
        self.store_expr(&mut a.destination)?;
        if types::compare(&a.value.typ, &a.destination.typ) == types::ComparisonResult::Incompatible {
            println!("Assignment type mismatch: {:?} != {:?}", a.value.typ, a.destination.typ);
            return self.error_loc(SemaErrorReason::AssignmentTypesIncompatible, e.loc);
        }
        e.typ = a.destination.typ.clone();  
        self.ok()
    }

    fn call(&mut self, e: &mut ast::Expr, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        let c = match &mut e.kind {
            ast::ExprKind::Call(c) => c,
            _ => panic!()
        };

        // load the function call name if its an identifier
        // (for now, we only support direct calls)
        let name = match &c.function.kind {
            ast::ExprKind::Identifier(i) => &i.id,
            _ => unimplemented!("Function calls must be direct(by name) for now"),
        };

        // find the called function
        //println!("Looking for function: {}", name);
        //println!("Available functions: {}", self.signatures.iter().map(|s| s.id.clone()).collect::<Vec<_>>().join(", "));
        let func_signature = match self.signatures.iter().find(|s| s.id == *name) {
            Some(s) => s,
            None => return self.error_loc(SemaErrorReason::FunctionNotFound, e.loc),
        };

        // Do some basic argument count checking
        if func_signature.params.len() < c.parameters.len() {
            return self.error_loc(SemaErrorReason::CallTooManyArguments, e.loc);
        }
        if func_signature.params.len() > c.parameters.len() {
            return self.error_loc(SemaErrorReason::CallNotEnoughArguments, e.loc);
        }

        // Sema check arguements, then check argument types
        for (arg, param) in c.parameters.iter_mut().zip(func_signature.params.iter()) {
            self.expr(arg, Some(&param.type_annotation))?;
            if types::compare(&arg.typ, &param.type_annotation) == types::ComparisonResult::Incompatible {
                return self.error_loc(SemaErrorReason::CallArgumentTypeMismatch, e.loc);
            }
        }

        // Assign this expr the return type of the function
        if let Some(typ) = &func_signature.return_type {
            e.typ = typ.clone();
        } else {
            e.typ = types::unknown();
        }
        self.ok()
    }

    fn integer(&mut self, _i: &mut Box<ast::Integer>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn number(&mut self, _f: &mut Box<ast::Number>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn boolean(&mut self, _b: &mut Box<ast::Bool>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn string_literal(&mut self, _s: &mut Box<ast::StringLiteral>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn identifier(&mut self, e: &mut ast::Expr, _type_hint: Option<&types::Type>) -> SemaResult<()> { 
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

    fn subscript(&mut self, e: &mut ast::Expr, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        // check the array is an array, then check the index is an integer
        // then set the type to the array element type
        let s = match &mut e.kind {
            ast::ExprKind::Subscript(s) => s,
            _ => panic!()
        };
        
        self.expr(&mut s.value, None)?;
        self.expr(&mut s.index, None)?;
        
        if !types::is_array(&s.value.typ) {
            return self.error_loc(SemaErrorReason::ValueIsNotIndexable, e.loc);
        }

        if !types::is_integer(&s.index.typ) {
            return self.error_loc(SemaErrorReason::ValueCannotBeUsedAsIndex, e.loc);
        }

        e.typ = match &*s.value.typ {
            types::Type::Array(element_type) => element_type.clone(),
            _ => panic!() // already checked above
        };
        self.ok()
    }

    fn selector(&mut self, _l: &Box<ast::Selector>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn array_literal(&mut self, e: &mut ast::Expr, type_hint: Option<&types::Type>) -> SemaResult<()> {
        // get type from first element, check all are the same, then assign type as array of that type
        let l = match &mut e.kind {
            ast::ExprKind::ArrayLiteral(l) => l,
            _ => panic!()
        };

        if l.literals.is_empty() {
            // empty array literal, assign type as array of unknown
            match type_hint {
                Some(typ) => {
                    e.typ = Box::new(typ.clone());
                }
                _ => {
                    e.typ = types::array(types::unknown());
                }
            }
        } else {
            self.expr(&mut l.literals[0], type_hint)?;
            let element_type = l.literals[0].typ.clone();
            for literal in l.literals.iter_mut().skip(1) {
                self.expr(literal, Some(&element_type))?;
                if types::compare(&literal.typ, &element_type) == types::ComparisonResult::Incompatible {
                    return self.error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
                }
            }
            e.typ = types::array(element_type);
        }

        self.ok()
    }

    fn object_literal(&mut self, _o: &Box<ast::ObjectLiteral>, _type_hint: Option<&types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn expr(&mut self, e: &mut ast::Expr, type_hint: Option<&types::Type>) -> SemaResult<()> {
        match &mut e.kind {
            ast::ExprKind::BinaryExpr(_) => self.binary_expr(e, type_hint),
            ast::ExprKind::UnaryExpr(u) => self.unary_expr(u, type_hint),
            ast::ExprKind::Assign(_) => self.assign(e, type_hint),
            ast::ExprKind::Call(_) => self.call(e, type_hint),
            ast::ExprKind::Integer(i) => self.integer(i, type_hint),
            ast::ExprKind::Number(f) => self.number(f, type_hint),
            ast::ExprKind::Boolean(b) => self.boolean(b, type_hint),
            ast::ExprKind::StringLiteral(s) => self.string_literal(s, type_hint),
            ast::ExprKind::Identifier(_) => self.identifier(e, type_hint),
            ast::ExprKind::Subscript(_) => self.subscript(e, type_hint),
            ast::ExprKind::Selector(l) => self.selector(l, type_hint),
            ast::ExprKind::ArrayLiteral(_) => self.array_literal(e, type_hint),
            ast::ExprKind::ObjectLiteral(o) => self.object_literal(o, type_hint),
        }
    }

    fn store_subscript(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        // check the array is an array, then check the index is an integer
        // then set the type to the array element type
        let s = match &mut e.kind {
            ast::ExprKind::Subscript(s) => s,
            _ => panic!()
        };
        
        self.expr(&mut s.value, None)?;
        self.expr(&mut s.index, None)?;
        
        if !types::is_array(&s.value.typ) {
            return self.error_loc(SemaErrorReason::ValueIsNotIndexable, e.loc);
        }

        if !types::is_integer(&s.index.typ) {
            return self.error_loc(SemaErrorReason::ValueCannotBeUsedAsIndex, e.loc);
        }

        e.typ = match &*s.value.typ {
            types::Type::Array(element_type) => element_type.clone(),
            _ => panic!() // already checked above
        };
        self.ok()
    }

    fn store_selector(&mut self, _e: &ast::Expr, _l: &Box<ast::Selector>) -> SemaResult<()> {
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

    // These are expressions which are going to be used to "store" a value
    // aka l values
    fn store_expr(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        match &e.kind {
            ast::ExprKind::Subscript(_) => self.store_subscript(e),
            ast::ExprKind::Selector(l) => self.store_selector(e, l),
            ast::ExprKind::Identifier(_) => self.store_identifier(e),
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
        self.expr(&mut e.expr, None)
    }

    fn for_stmt(&mut self, _f: &mut Box<ast::ForStmt>) -> SemaResult<()> {
        unimplemented!()
    }

    fn if_stmt(&mut self, f: &mut Box<ast::IfStmt>) -> SemaResult<()> {
        self.expr(&mut f.test, Some(&Type::Bool))?;
        if types::compare(&f.test.typ, &Type::Bool) != types::ComparisonResult::Same {
            return self.error_loc(SemaErrorReason::ExpectedBooleanInTestCondition, f.test.loc);
        }
        self.stmt(&mut f.consequent)?;
        if let Some(a) = &mut f.alternate{
            self.stmt(a)?;
        }
        self.ok()
    }

    fn return_stmt(&mut self, r: &mut Box<ast::ReturnStmt>) -> SemaResult<()> {
        if let Some(return_type) = &self.own_signature.return_type {
            if let Some(r) = &mut r.value {
                self.expr(r, Some(return_type))?;
                if types::compare(&r.typ, return_type) == types::ComparisonResult::Incompatible {
                    return self.error(SemaErrorReason::IncompatibleTypesInReturnValue);
                }
            } else {
                return self.error(SemaErrorReason::MissingReturnValue);
            }
        } else {
            if r.value.is_some() {
                return self.error(SemaErrorReason::UnexpectedReturnValue);
            }
        }
        self.ok()
    }

    fn var_decl_stmt(&mut self, v: &mut Box<ast::VarDeclStmt>) -> SemaResult<()> {
        if let Some(annotation) = &v.type_annotation {
            self.expr(&mut v.value, Some(annotation))?;
            let ret = &v.value.typ;
            if types::compare(&annotation, ret) == types::ComparisonResult::Incompatible {
                return self.error(SemaErrorReason::IncompatibleTypesInVariableDefinition);
            }
            self.create_var(v.id.clone(), ret);
        } else {
            self.expr(&mut v.value, None)?;
            let ret = &v.value.typ;
            v.type_annotation = Some(ret.clone());
            self.create_var(v.id.clone(), ret);
        }
        self.ok()
    }

    fn while_stmt(&mut self, w: &mut Box<ast::WhileStmt>) -> SemaResult<()> {
        self.expr(&mut w.condition, Some(&Type::Bool))?;
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
    let own_signature = signatures.iter().find(|s| s.id == func.signature.id).unwrap();
    FuncTypeInference::new(own_signature, signatures).check(func)?;
    Ok(())
}

pub fn sema_module(module: &mut Box<ast::Module>, builtins: &Builtins) -> SemaResult<()> {
    let mut signatures = module.functions.iter().map(|f| f.signature.clone()).collect::<Vec<_>>();
    // add builtin signatures
    for builtin in builtins.functions.iter() {
        let signature = ast::FuncSignature {
            id: builtin.id.clone(),
            params: builtin.parameters.iter().enumerate().map(|(i, p)| ast::Param {
                id: format!("arg{}", i),
                type_annotation: p.clone(),
            }).collect(),
            return_type: builtin.returns.clone(),
        };
        signatures.push(signature);
        //// avoid duplicates
        //if !signatures.iter().any(|s| s.id == signature.id) {
        //    // println!("Adding builtin function signature: {:?}", signature);
        //}
    }
    for func in module.functions.iter_mut() {
        sema_function(&signatures, func)?;
    }

    Ok(())
}
