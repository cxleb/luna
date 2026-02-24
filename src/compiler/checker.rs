use std::collections::HashMap;

use crate::builtins::Builtins;
use crate::compiler::{SourceLoc, ast};
use crate::types::{self, Type, clone_struct_fields, NameSpecification};

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
    NonBoolInLogicalExpression,
    TypeNotFound,
    StructFieldNotFound,
    InvalidUsageOfSelector,
    CannotFindSelectorInStruct,
    CannotFindVariantInEnum,
    CannotUseSelfOutsideOfMethod,
    EnumVariantValueTypesIncompatible,
}

#[derive(Debug)]
pub struct SemaError {
    reason: SemaErrorReason,
    loc: SourceLoc,
}

type SemaResult<X> = Result<X, SemaError>;

pub fn error_loc<T>(reason: SemaErrorReason, loc: SourceLoc) -> SemaResult<T> {
    Err(SemaError { reason, loc })
}

struct TypeCollection {
    types: HashMap<NameSpecification, Type>,
}

impl TypeCollection {
    // Find a type in the collection
    pub fn get(&self, imports: &Vec<String>, package_id: &str, name: &String) -> Option<&Type> {
        if let Some(typ) = self.get_exact(package_id, name) {
            return Some(typ);
        } 

        for import in imports {
            let name_spec = NameSpecification {
                package: import.clone(),
                name: name.clone(),
            };
            if let Some(typ) = self.types.get(&name_spec) {
                return Some(typ);
            }
        }
        None
    }

    pub fn get_exact(&self, package: &str, name: &str) -> Option<&Type> {
        let name_spec = NameSpecification {
            package: package.into(),
            name: name.into(),
        };
        self.types.get(&name_spec)
    }
}

fn collect_types(program: &ast::Program) -> TypeCollection {
    let mut collection = TypeCollection {
        types: HashMap::new(),
    };

    for package in program.packages.iter() {
        for file in package.files.iter() {
            for struct_ in file.structs.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    name: struct_.id.clone(),
                };
                let typ = types::struct_type(name_spec.clone(), Vec::new(), Vec::new());
                collection.types.insert(name_spec, typ);
            }
            for enum_ in file.enums.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    name: enum_.id.clone(),
                };
                let typ = types::enum_type(name_spec.clone(), Vec::new());
                collection.types.insert(name_spec, typ);
            }
        }
    }

    collection
}

fn check_types(program: &ast::Program, collection: &TypeCollection) -> SemaResult<()> {
    for package in program.packages.iter() {
        for file in package.files.iter() {
            for struct_ in file.structs.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    name: struct_.id.clone(),
                };
                let typ = collection.types.get(&name_spec).unwrap().clone();
                for field in struct_.fields.iter() {
                    let field_type = type_lookup(&field.type_annotation, &collection, &package.id, &file.imports)?;
                    // resolve field types
                    if let types::TypeKind::Struct(struct_type) = &typ.inner.kind {
                        struct_type.fields.write().unwrap().push((field.id.clone(), field_type));
                    } else {
                        unreachable!();
                    }
                }
                for func in struct_.functions.iter() {
                    let mut params = Vec::new();
                    for param in func.signature.params.iter() {
                        params.push(type_lookup(&param.type_annotation, &collection, &package.id, &file.imports)?);
                    }
                    let mut returns = Vec::new();
                    for return_type in func.signature.return_type.iter() {
                        returns.push(type_lookup(&return_type, &collection, &package.id, &file.imports)?);
                    }
                    let function_type = types::FunctionType {
                        params,
                        returns,
                    };
                    if let types::TypeKind::Struct(struct_type) = &typ.inner.kind {
                        struct_type.functions.write().unwrap().push((func.signature.id.clone(), function_type));
                    } else {
                        unreachable!();
                    }
                }
            }
            for enum_ in file.enums.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    name: enum_.id.clone(),
                };
                let typ = collection.types.get(&name_spec).unwrap().clone();
                for variant in enum_.variants.iter() {
                    let mut field_types = Vec::new();
                    for field in variant.variant_types.iter() {
                        let field_type = type_lookup(&field, &collection, &package.id, &file.imports)?;
                        field_types.push(field_type);
                    }
                    if let types::TypeKind::Enum(enum_type) = &typ.inner.kind {
                        enum_type.variants.write().unwrap().push((variant.id.clone(), field_types));
                    }
                }
            }
        }
    }
    Ok(())
}

fn type_lookup(ast_type: &ast::Type, collection: &TypeCollection, package_id: &str, imports: &Vec<String>) -> SemaResult<Type> {
    match ast_type {
        ast::Type::Integer => Ok(types::integer()),
        ast::Type::Number => Ok(types::number()),
        ast::Type::String => Ok(types::string()),
        ast::Type::Bool => Ok(types::bool()),
        ast::Type::Identifier(id) => collection.get(imports, package_id, id).cloned().ok_or(SemaError { reason: SemaErrorReason::TypeNotFound, loc: SourceLoc::default() }),
        ast::Type::Array(element_type) => Ok(types::array(type_lookup(element_type, collection, package_id, imports)?)),
        _ => Err(SemaError { reason: SemaErrorReason::TypeNotFound, loc: SourceLoc::default() }),
    }
}

struct FunctionCollection {
    functions: HashMap<NameSpecification, types::FunctionType>,
}

impl FunctionCollection {
    // Find a function in the collection
    pub fn get(&self, imports: &Vec<String>, package: &str, name: &String) -> Option<(&types::FunctionType, NameSpecification)> {
        if let Some(typ) = self.get_exact(package, name) {
            return Some((typ, NameSpecification {
                package: package.into(),
                name: name.clone(),
            }));
        }
        for import in imports {
            let name_spec = NameSpecification {
                package: import.clone(),
                name: name.clone(),
            };
            if let Some(typ) = self.functions.get(&name_spec) {
                return Some((typ, name_spec));
            }
        }
        None
    }

    pub fn get_exact(&self, package: &str, name: &str) -> Option<&types::FunctionType> {
        let name_spec = NameSpecification {
            package: package.into(),
            name: name.into(),
        };
        self.functions.get(&name_spec)
    }
}

fn collect_functions(program: &ast::Program, builtins: &Builtins, collection: &TypeCollection) -> SemaResult<FunctionCollection> {
    let mut function_collection = FunctionCollection {
        functions: HashMap::new(),
    };

    // collect the builtin functions into the builtin package(which is implicitly imported)
    for builtin in builtins.functions.iter() {
        let name_spec = NameSpecification {
            package: "builtins".into(),
            name: builtin.id.clone(),
        };
        let mut params = Vec::new();
        for param in builtin.parameters.iter() {
            params.push(param.clone());
        }
        let mut returns = Vec::new();
        for return_type in builtin.returns.iter() {
            returns.push(return_type.clone());
        }
        let function_type = types::FunctionType {
            params,
            returns,
        };
        function_collection.functions.insert(name_spec, function_type);
    }
     
    for package in program.packages.iter() {
        for file in package.files.iter() {
            for func in file.functions.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    name: func.signature.id.clone(),
                };
                let mut params = Vec::new();
                for param in func.signature.params.iter() {
                    params.push(type_lookup(&param.type_annotation, &collection, &package.id, &file.imports)?);
                }
                let mut returns = Vec::new();
                for return_type in func.signature.return_type.iter() {
                    returns.push(type_lookup(&return_type, &collection, &package.id, &file.imports)?);
                }
                let function_type = types::FunctionType {
                    params,
                    returns,
                };
                function_collection.functions.insert(name_spec, function_type);
            }
        }
    }

    Ok(function_collection)
}

fn mangle_name(name_spec: &NameSpecification) -> String {
    // For now, we will just mangle by prefixing with the package name, but in the future we will need to mangle generics and stuff too
    format!("_L{}_{}", name_spec.package, name_spec.name)
}

fn mangle_name_struct(name_spec: &NameSpecification, name: &str) -> String {
    // For now, we will just mangle by prefixing with the package name, but in the future we will need to mangle generics and stuff too
    format!("_L{}_{}_{}", name_spec.package, name_spec.name, name)
}

struct FuncTypeInference<'a> {
    ///structs: &'a Vec<Box<ast::Struct>>,
    imports: &'a Vec<String>,
    types: &'a TypeCollection,
    //own_signature:&'a ast::FuncSignature,
    //signatures: &'a Vec<ast::FuncSignature>,
    self_type: Option<Type>,
    functions: &'a FunctionCollection,
    own_signature: &'a types::FunctionType,
    package_id: &'a str,
    variable_scopes: Vec<HashMap<String, Box<Type>>>,
}

impl<'a> FuncTypeInference<'a> {
    fn new(imports: &'a Vec<String>, types: &'a TypeCollection, own_signature: &'a types::FunctionType, functions: &'a FunctionCollection, package_id: &'a str) -> Self {
        Self {
            imports,
            types,
            own_signature,
            functions,
            package_id,
            variable_scopes: Vec::new(),
            self_type: None,
        }
    }

    fn new_for_method(imports: &'a Vec<String>, types: &'a TypeCollection, own_signature: &'a types::FunctionType, functions: &'a FunctionCollection, package_id: &'a str, self_type: Type) -> Self {
        Self {
            imports,
            types,
            own_signature,
            functions,
            package_id,
            variable_scopes: Vec::new(),
            self_type: Some(self_type),
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

    pub fn error_loc<T>(&self, reason: SemaErrorReason, loc: SourceLoc) -> SemaResult<T> {
        Err(SemaError { reason, loc })
    }

    pub fn find_type(&self, name: &str) -> Option<Type> {
        self.types.get(&self.imports, self.package_id, &name.to_string()).cloned()
    }

    fn binary_expr(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
        let b = match &mut e.kind {
            ast::ExprKind::BinaryExpr(b) => b,
            _ => panic!()
        };
        
        self.expr(&mut b.lhs, type_hint.clone())?;
        self.expr(&mut b.rhs, type_hint.clone())?;

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
                        return self.error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
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
                    return self.error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
                }

                e.typ = types::bool();
            }
            ast::BinaryExprKind::LogicalAnd |
            ast::BinaryExprKind::LogicalOr => {
                if !types::is_bool(&b.lhs.typ) {
                    return self.error_loc(SemaErrorReason::NonBoolInLogicalExpression, b.lhs.loc);
                }
                if !types::is_bool(&b.rhs.typ) {
                    return self.error_loc(SemaErrorReason::NonBoolInLogicalExpression, b.rhs.loc);
                }

                e.typ = types::bool();
            }
            
        }
        self.ok()
    }

    fn unary_expr(&mut self, _u: &mut Box<ast::UnaryExpr>, _type_hint: Option<types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn assign(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
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

    fn call(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let c = match &mut e.kind {
            ast::ExprKind::Call(c) => c,
            _ => panic!()
        };

        match &mut c.function.kind {
            ast::ExprKind::Identifier(i) => {
                let (func_signature, name_spec) = match self.functions.get(self.imports, self.package_id, &i.id) {
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
                    self.expr(arg, Some(param.clone()))?;
                    if types::compare(&arg.typ, &param) == types::ComparisonResult::Incompatible {
                        return self.error_loc(SemaErrorReason::CallArgumentTypeMismatch, e.loc);
                    }
                }

                c.symbol_name = Some(mangle_name(&name_spec));

                // Assign this expr the return type of the function
                if let Some(typ) = func_signature.returns.first() {
                    e.typ = typ.clone();
                } else {
                    e.typ = types::bad();
                }

                self.ok()
            }
            ast::ExprKind::Selector(s) => {
                self.expr(&mut s.value, None)?;

                if let types::TypeKind::Struct(struct_type) = s.value.typ.kind() {
                    match struct_type.functions.read().unwrap().iter().find(|f| f.0 == s.selector.id) {
                        Some(f) => {
                            let func_signature = &f.1;

                            // Do some basic argument count checking
                            if func_signature.params.len() < c.parameters.len() {
                                return self.error_loc(SemaErrorReason::CallTooManyArguments, e.loc);
                            }
                            if func_signature.params.len() > c.parameters.len() {
                                return self.error_loc(SemaErrorReason::CallNotEnoughArguments, e.loc);
                            }

                            // Sema check arguements, then check argument types
                            for (arg, param) in c.parameters.iter_mut().zip(func_signature.params.iter()) {
                                self.expr(arg, Some(param.clone()))?;
                                if types::compare(&arg.typ, &param) == types::ComparisonResult::Incompatible {
                                    return self.error_loc(SemaErrorReason::CallArgumentTypeMismatch, e.loc);
                                }
                            }

                            // Assign this expr the return type of the function
                            if let Some(typ) = func_signature.returns.first() {
                                e.typ = typ.clone();
                            } else {
                                e.typ = types::bad();
                            }

                            c.symbol_name = Some(mangle_name_struct(&struct_type.spec, &s.selector.id));

                            self.ok()
                        },
                        None => return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc),
                    }
                } else if let types::TypeKind::Enum(enum_type) = s.value.typ.kind() {
                    match enum_type.variants.read().unwrap().iter().enumerate().find(|v| v.1.0 == s.selector.id) {
                        Some((i, values)) => {
                            // Assign this expr the type of the enum
                            e.typ = s.value.typ.clone();

                            for (arg, param) in c.parameters.iter_mut().zip(values.1.iter()) {
                                self.expr(arg, Some(param.clone()))?;
                                if types::compare(&arg.typ, &param) == types::ComparisonResult::Incompatible {
                                    return self.error_loc(SemaErrorReason::EnumVariantValueTypesIncompatible, e.loc);
                                }
                            }

                            c.enum_idx = Some(i);
                            self.ok()
                        },
                        None => return self.error_loc(SemaErrorReason::CannotFindVariantInEnum, e.loc),
                    }
                } else {
                    return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, s.value.loc);
                }
            },
            _ => unimplemented!("Function calls must be direct(by name) for now"),
        }
    }

    fn integer(&mut self, _i: &mut Box<ast::Integer>, _type_hint: Option<types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn number(&mut self, _f: &mut Box<ast::Number>, _type_hint: Option<types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn boolean(&mut self, _b: &mut Box<ast::Bool>, _type_hint: Option<types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn string_literal(&mut self, _s: &mut Box<ast::StringLiteral>, _type_hint: Option<types::Type>) -> SemaResult<()> {
        self.ok()
    }

    fn identifier(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> { 
        let i = match &e.kind {
            ast::ExprKind::Identifier(i) => i,
            _ => panic!()
        };
        if let Some(typ) = self.find_var(&i.id) {
            e.typ = typ.clone().into();
            self.ok()
        } else if let Some(typ) = self.find_type(&i.id) {
            e.typ = typ;
            self.ok()
        } else {
            self.error_loc(SemaErrorReason::VariableNotFound, e.loc)
        }
    }

    fn subscript(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
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

        e.typ = match &s.value.typ.kind() {
            types::TypeKind::Array(element_type) => element_type.clone(),
            _ => panic!() // already checked above
        };
        self.ok()
    }

    fn selector(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let s = match &mut e.kind {
            ast::ExprKind::Selector(s) => s,
            _ => panic!()
        };

        self.expr(&mut s.value, None)?;

        if let types::TypeKind::Struct(struct_type) = s.value.typ.kind() {
            if let Some((i, (_, ty))) = struct_type.fields.read().unwrap().iter().enumerate().find(|a| a.1.0 == s.selector.id) {
                e.typ = ty.clone();
                s.idx = i;
            } else {
                return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc);
            }
        }        
        else if let types::TypeKind::Enum(enum_type) = s.value.typ.kind() {
           if let Some((i, _)) = enum_type.variants.read().unwrap().iter().enumerate().find(|a| a.1.0 == s.selector.id) {
               e.typ = s.value.typ.clone();
               s.enum_idx = Some(i);
           } else {
               return self.error_loc(SemaErrorReason::CannotFindVariantInEnum, e.loc);
           }
        }  
        else {
            return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, e.loc);
        }

        self.ok()
    }

    fn array_literal(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
        // get type from first element, check all are the same, then assign type as array of that type
        let l = match &mut e.kind {
            ast::ExprKind::ArrayLiteral(l) => l,
            _ => panic!()
        };

        if l.literals.is_empty() {
            // empty array literal, assign type as array of unknown
            match type_hint {
                Some(typ) => {
                    e.typ = typ.clone();
                }
                _ => {
                    e.typ = types::array(types::bad());
                }
            }
        } else {
            self.expr(&mut l.literals[0], type_hint)?;
            let element_type = l.literals[0].typ.clone();
            for literal in l.literals.iter_mut().skip(1) {
                self.expr(literal, Some(element_type.clone()))?;
                if types::compare(&literal.typ, &element_type) == types::ComparisonResult::Incompatible {
                    return self.error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
                }
            }
            e.typ = types::array(element_type);
        }

        self.ok()
    }

    fn object_literal(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let l = match &mut e.kind {
            ast::ExprKind::ObjectLiteral(l) => l,
            _ => panic!()
        };

        let id = match &l.id {
            Some(id) => id,
            None => unimplemented!("Anonymous object literals not supported yet"),
        };

        // maybe there need to be a module look up mapping to know which structs we want
        // does the struct exist?
        let struct_def = match self.find_type(&id.id) {
            Some(s) => s.clone(),
            None => return self.error_loc(SemaErrorReason::TypeNotFound, e.loc),
        };

        // check the type is actually a struct
        if !types::is_struct(&struct_def) {
            return self.error_loc(SemaErrorReason::TypeNotFound, e.loc);
        }

        let struct_fields = clone_struct_fields(&struct_def);

        // once we find the struct, we need to check what fields we are setting and if they exist
        // run the expr sema for the fields
        // then we need to type check them
        for field in l.fields.iter_mut() {
            let struct_field = match struct_fields.iter().find(|f| f.0 == field.id) {
                Some(f) => f,
                None => return self.error_loc(SemaErrorReason::StructFieldNotFound, field.loc),
            };
            // run sema on the field value
            self.expr(&mut field.value, Some(struct_field.1.clone()))?;
            // check the type matches
            if types::compare(&field.value.typ, &struct_field.1) == types::ComparisonResult::Incompatible {
                return self.error_loc(SemaErrorReason::AssignmentTypesIncompatible, field.loc);
            }
        }

        e.typ = struct_def;

        self.ok()
    }

    fn _self(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        let self_type = match &self.self_type {
            Some(t) => t.clone(),
            None => return self.error_loc(SemaErrorReason::CannotUseSelfOutsideOfMethod, e.loc),
        };
        e.typ = self_type;
        self.ok()
    } 

    fn expr(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
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
            ast::ExprKind::Selector(_) => self.selector(e, type_hint),
            ast::ExprKind::ArrayLiteral(_) => self.array_literal(e, type_hint),
            ast::ExprKind::ObjectLiteral(_) => self.object_literal(e, type_hint),
            ast::ExprKind::_Self => self._self(e),
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

        e.typ = match &s.value.typ.kind() {
            types::TypeKind::Array(element_type) => element_type.clone(),
            _ => panic!() // already checked above
        };
        self.ok()
    }

    fn store_selector(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        let s = match &mut e.kind {
            ast::ExprKind::Selector(s) => s,
            _ => panic!()
        };

        self.expr(&mut s.value, None)?;

        if !types::is_struct(&s.value.typ) {
            return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, e.loc);
        }

        if let types::TypeKind::Struct(struct_type) = s.value.typ.kind() {
            if let Some((i, (_, ty))) = struct_type.fields.read().unwrap().iter().enumerate().find(|a| a.1.0 == s.selector.id) {
                e.typ = ty.clone();
                s.idx = i;
            } else {
                return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc);
            }
        }

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
            self.error_loc(SemaErrorReason::VariableNotFound, e.loc)
        }
    }

    // These are expressions which are going to be used to "store" a value
    // aka l values
    fn store_expr(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        match &e.kind {
            ast::ExprKind::Subscript(_) => self.store_subscript(e),
            ast::ExprKind::Selector(_) => self.store_selector(e),
            ast::ExprKind::Identifier(_) => self.store_identifier(e),
            _ => self.error_loc(SemaErrorReason::CannotUseExpressionInLeftHandExpression, e.loc)
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
        self.expr(&mut f.test, Some(types::bool()))?;
        if types::compare(&f.test.typ, &types::bool()) != types::ComparisonResult::Same {
            return self.error_loc(SemaErrorReason::ExpectedBooleanInTestCondition, f.test.loc);
        }
        self.stmt(&mut f.consequent)?;
        if let Some(a) = &mut f.alternate{
            self.stmt(a)?;
        }
        self.ok()
    }

    fn return_stmt(&mut self, r: &mut Box<ast::ReturnStmt>) -> SemaResult<()> {
        if let Some(return_type) = self.own_signature.returns.first().cloned() {
            if let Some(r) = &mut r.value {
                self.expr(r, Some(return_type.clone()))?;
                if types::compare(&r.typ, &return_type) == types::ComparisonResult::Incompatible {
                    return self.error_loc(SemaErrorReason::IncompatibleTypesInReturnValue, r.loc);
                }
            } else {
                return self.error_loc(SemaErrorReason::MissingReturnValue, r.loc);
            }
        } else {
            if r.value.is_some() {
                return self.error_loc(SemaErrorReason::UnexpectedReturnValue, r.loc);
            }
        }
        self.ok()
    }

    fn var_decl_stmt(&mut self, v: &mut Box<ast::VarDeclStmt>) -> SemaResult<()> {
        if let Some(annotation) = &v.type_annotation {
            let annotation = type_lookup(&annotation, self.types, self.package_id, self.imports)?;
            self.expr(&mut v.value, Some(annotation.clone()))?;
            let ret = &v.value.typ;
            if types::compare(&annotation, ret) == types::ComparisonResult::Incompatible {
                return self.error_loc(SemaErrorReason::IncompatibleTypesInVariableDefinition, v.loc);
            }
            self.create_var(v.id.clone(), ret);
        } else {
            self.expr(&mut v.value, None)?;
            let ret = &v.value.typ;
            //v.type_annotation = Some(ret.clone());
            self.create_var(v.id.clone(), ret);
        }
        self.ok()
    }

    fn while_stmt(&mut self, w: &mut Box<ast::WhileStmt>) -> SemaResult<()> {
        self.expr(&mut w.condition, Some(types::bool()))?;
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
            let annotation = type_lookup(&p.type_annotation, self.types, self.package_id, self.imports)?;
            self.create_var(p.id.clone(), &annotation);
        }
        self.block_stmt(&mut func.body)?;
        self.pop_scope();
        self.ok()
    }
}

fn check_file(file: &mut Box<ast::File>, package_id: &str, collection: &TypeCollection, functions: &FunctionCollection) -> SemaResult<()> {
    file.imports.push("builtins".into());
    for func in file.functions.iter_mut() {
        let own_signature = functions.get_exact(package_id, &func.signature.id).unwrap();
        func.typ_ = own_signature.clone();
        FuncTypeInference::new(&file.imports, collection, own_signature, functions, package_id).check(func)?;

        func.signature.symbol_name = mangle_name(&NameSpecification {
            package: package_id.into(),
            name: func.signature.id.clone(),
        });
    }

    for _struct in file.structs.iter_mut() {
        let typ = collection.get_exact(package_id, &_struct.id).unwrap().clone();
        _struct.typ = typ.clone();
        for func in _struct.functions.iter_mut() {
            let own_signature = match typ.kind() {
                types::TypeKind::Struct(struct_type) => struct_type.functions.read().unwrap().iter().find(|f| f.0 == func.signature.id).unwrap().1.clone(),
                _ => panic!()
            };
            func.typ_ = own_signature.clone();
            func.signature.symbol_name = mangle_name_struct(&NameSpecification {
                package: package_id.into(),
                name: _struct.id.clone(),
            }, &func.signature.id);
            FuncTypeInference::new_for_method(&file.imports, collection, &own_signature, functions, package_id, typ.clone()).check(func)?;
        }
    }

    Ok(())
}

fn check_package(package: &mut ast::Package, collection: &TypeCollection, functions: &FunctionCollection) -> SemaResult<()> {
    for file in package.files.iter_mut() {
        check_file(file, &package.id, collection, functions)?;
    }
    Ok(())
}

pub fn check_program(program: &mut ast::Program, builtins: &Builtins) -> SemaResult<()> {
    let collection = collect_types(program);
    check_types(program, &collection)?;

    let function_collection = collect_functions(program, builtins, &collection)?;

    for file in program.packages.iter_mut() {
        check_package(file, &collection, &function_collection)?;
    }
    Ok(())
}

