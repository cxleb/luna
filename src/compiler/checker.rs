use std::collections::HashMap;

use crate::builtins::Builtins;
use crate::compiler::mangle;
use crate::compiler::{SourceLoc, ast};
use crate::types::{self, NameSpecification, Type, clone_struct_fields};

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
    CannotAssignToConst,
    NonBoolInLogicalExpression,
    TypeNotFound,
    StructFieldNotFound,
    InvalidUsageOfSelector,
    CannotFindSelectorInStruct,
    CannotFindVariantInEnum,
    CannotUseSelfOutsideOfMethod,
    EnumVariantValueTypesIncompatible,
    InvalidSwitchCasePattern,
    EnumVariantNotFound,
    EnumVariantPatternFieldCountMismatch,
    InvalidPatternKind,
    ExpressionCannotBeCasted,
    TemplateSubstitutionMustBeString,
}

#[derive(Debug)]
pub struct SemaError {
    reason: SemaErrorReason,
    loc: SourceLoc,
    file: String,
    package: String,
}

type SemaResult<X> = Result<X, SemaError>;

pub fn error_loc<T>(reason: SemaErrorReason, loc: SourceLoc) -> SemaResult<T> {
    Err(SemaError { reason, loc, file: String::new(), package: String::new() })
}

struct TypeCollection {
    types: HashMap<NameSpecification, Type>,
}

impl TypeCollection {
    // Find a type in the collection
    pub fn get(
        &self,
        imports: &Vec<ast::Import>,
        package_id: &str,
        file_id: &str,
        name: &String,
    ) -> Option<&Type> {
        if let Some(typ) = self.get_exact(package_id, file_id, name) {
            return Some(typ);
        }

        if let Some(typ) = self.get_exact("builtins", "builtins", name) {
            return Some(typ);
        }

        for import in imports {
            let name_spec = NameSpecification {
                package: import.package.clone(),
                file: import.file.clone(),
                name: name.clone(),
            };
            if let Some(typ) = self.types.get(&name_spec) {
                return Some(typ);
            }
        }
        None
    }

    pub fn get_exact(&self, package: &str, file: &str, name: &str) -> Option<&Type> {
        let name_spec = NameSpecification {
            package: package.into(),
            file: file.into(),
            name: name.into(),
        };
        self.types.get(&name_spec)
    }
}

fn builtin_types(collection: &mut TypeCollection) {
    let string_name_spec = NameSpecification {
        package: "builtins".into(),
        file: "builtins".into(),
        name: "String".into(),
    };
    let string_interface = types::interface_type(
        string_name_spec.clone(),
        vec![(
            "string".into(),
            types::FunctionType {
                params: Vec::new(),
                returns: vec![types::string()],
            },
        )],
    );
    collection
        .types
        .insert(string_name_spec.clone(), string_interface);
}

fn collect_types(program: &ast::Program) -> TypeCollection {
    let mut collection = TypeCollection {
        types: HashMap::new(),
    };

    builtin_types(&mut collection);

    for package in program.packages.iter() {
        for file in package.files.iter() {
            for struct_ in file.structs.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: struct_.id.clone(),
                };
                let typ = types::struct_type(name_spec.clone(), Vec::new());
                collection.types.insert(name_spec, typ);
            }
            for enum_ in file.enums.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: enum_.id.clone(),
                };
                let typ = types::enum_type(name_spec.clone(), Vec::new());
                collection.types.insert(name_spec, typ);
            }
            for enum_ in file.enums.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: enum_.id.clone(),
                };
                let typ = types::enum_type(name_spec.clone(), Vec::new());
                collection.types.insert(name_spec, typ);
            }
            for interface in file.interfaces.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: interface.id.clone(),
                };
                let typ = types::interface_type(name_spec.clone(), Vec::new());
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
                    file: file.id.clone(),
                    name: struct_.id.clone(),
                };
                let typ = collection.types.get(&name_spec).unwrap().clone();
                for field in struct_.fields.iter() {
                    let field_type = type_lookup(
                        &field.type_annotation,
                        &collection,
                        &package.id,
                        &file.id,
                        &file.imports,
                    )?;
                    // resolve field types
                    if let types::TypeKind::Struct(struct_type) = &typ.inner.kind {
                        struct_type
                            .fields
                            .write()
                            .unwrap()
                            .push((field.id.clone(), field_type));
                    } else {
                        unreachable!();
                    }
                }
            }
            for enum_ in file.enums.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: enum_.id.clone(),
                };
                let typ = collection.types.get(&name_spec).unwrap().clone();
                for variant in enum_.variants.iter() {
                    let mut field_types = Vec::new();
                    for field in variant.variant_types.iter() {
                        let field_type =
                            type_lookup(&field, &collection, &package.id, &file.id, &file.imports)?;
                        field_types.push(field_type);
                    }
                    if let types::TypeKind::Enum(enum_type) = &typ.inner.kind {
                        enum_type
                            .variants
                            .write()
                            .unwrap()
                            .push((variant.id.clone(), field_types));
                    }
                }
            }
            for interface in file.interfaces.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: interface.id.clone(),
                };
                let typ = collection.types.get(&name_spec).unwrap().clone();
                for method in interface.methods.iter() {
                    let mut params = Vec::new();
                    for param in method.params.iter() {
                        let param_type = type_lookup(
                            &param.type_annotation,
                            &collection,
                            &package.id,
                            &file.id,
                            &file.imports,
                        )?;
                        params.push(param_type);
                    }
                    let mut returns = Vec::new();
                    for return_type in method.return_type.iter() {
                        let return_type = type_lookup(
                            &return_type,
                            &collection,
                            &package.id,
                            &file.id,
                            &file.imports,
                        )?;
                        returns.push(return_type);
                    }
                    if let types::TypeKind::Interface(interface_type) = &typ.inner.kind {
                        interface_type
                            .methods
                            .write()
                            .unwrap()
                            .push((method.id.clone(), types::FunctionType { params, returns }));
                    }
                }
            }
        }
    }
    Ok(())
}

fn type_lookup(
    ast_type: &ast::Type,
    collection: &TypeCollection,
    package_id: &str,
    file_id: &str,
    imports: &Vec<ast::Import>,
) -> SemaResult<Type> {
    match ast_type {
        ast::Type::Integer => Ok(types::integer()),
        ast::Type::Number => Ok(types::number()),
        ast::Type::String => Ok(types::string()),
        ast::Type::Bool => Ok(types::bool()),
        ast::Type::Identifier(id) => collection
            .get(imports, package_id, file_id, id)
            .cloned()
            .ok_or(SemaError {
                reason: SemaErrorReason::TypeNotFound,
                loc: SourceLoc::default(),
                file: file_id.into(),
                package: package_id.into(),
            }),
        ast::Type::Array(element_type) => Ok(types::array(type_lookup(
            element_type,
            collection,
            package_id,
            file_id,
            imports,
        )?)),
        _ => Err(SemaError {
            reason: SemaErrorReason::TypeNotFound,
            loc: SourceLoc::default(),
            file: file_id.into(),
            package: package_id.into(),
        }),
    }
}

struct FunctionCollection {
    functions: HashMap<NameSpecification, types::FunctionType>,
    //methods: HashMap<MethodSpecification, types::FunctionType>,
}

impl FunctionCollection {
    // Find a function in the collection
    pub fn get(
        &self,
        imports: &Vec<ast::Import>,
        package: &str,
        file: &str,
        name: &String,
    ) -> Option<(&types::FunctionType, NameSpecification)> {
        if let Some(typ) = self.get_exact(package, file, name) {
            return Some((
                typ,
                NameSpecification {
                    package: package.into(),
                    file: file.into(),
                    name: name.clone(),
                },
            ));
        }
        if let Some(typ) = self.get_exact("builtins", "builtins", name) {
            return Some((
                typ,
                NameSpecification {
                    package: "builtins".into(),
                    file: "builtins".into(),
                    name: name.clone(),
                },
            ));
        }
        for import in imports {
            let name_spec = NameSpecification {
                package: import.package.clone(),
                file: import.file.clone(),
                name: name.clone(),
            };
            if let Some(typ) = self.functions.get(&name_spec) {
                return Some((typ, name_spec));
            }
        }
        None
    }

    pub fn get_exact(&self, package: &str, file: &str, name: &str) -> Option<&types::FunctionType> {
        let name_spec = NameSpecification {
            package: package.into(),
            file: file.into(),
            name: name.into(),
        };
        self.functions.get(&name_spec)
    }
}

fn collect_functions(
    program: &ast::Program,
    builtins: &Builtins,
    collection: &TypeCollection,
) -> SemaResult<FunctionCollection> {
    let mut function_collection = FunctionCollection {
        functions: HashMap::new(),
    };

    // collect the builtin functions into the builtin package(which is implicitly imported)
    for builtin in builtins.functions.iter() {
        let name_spec = NameSpecification {
            package: "builtins".into(),
            file: "builtins".into(),
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
        let function_type = types::FunctionType { params, returns };
        function_collection
            .functions
            .insert(name_spec, function_type);
    }

    for package in program.packages.iter() {
        for file in package.files.iter() {
            for func in file.functions.iter() {
                let name_spec = NameSpecification {
                    package: package.id.clone(),
                    file: file.id.clone(),
                    name: func.signature.id.clone(),
                };
                let mut params = Vec::new();
                for param in func.signature.params.iter() {
                    params.push(type_lookup(
                        &param.type_annotation,
                        &collection,
                        &package.id,
                        &file.id,
                        &file.imports,
                    )?);
                }
                let mut returns = Vec::new();
                for return_type in func.signature.return_type.iter() {
                    returns.push(type_lookup(
                        &return_type,
                        &collection,
                        &package.id,
                        &file.id,
                        &file.imports,
                    )?);
                }
                let function_type = types::FunctionType { params, returns };
                function_collection
                    .functions
                    .insert(name_spec, function_type);
            }

            for struct_ in file.structs.iter() {
                let typ = collection
                    .types
                    .get(&NameSpecification {
                        package: package.id.clone(),
                        file: file.id.clone(),
                        name: struct_.id.clone(),
                    })
                    .unwrap()
                    .clone();

                for func in struct_.functions.iter() {
                    let mut params = Vec::new();
                    for param in func.signature.params.iter() {
                        params.push(type_lookup(
                            &param.type_annotation,
                            &collection,
                            &package.id,
                            &file.id,
                            &file.imports,
                        )?);
                    }
                    let mut returns = Vec::new();
                    for return_type in func.signature.return_type.iter() {
                        returns.push(type_lookup(
                            &return_type,
                            &collection,
                            &package.id,
                            &file.id,
                            &file.imports,
                        )?);
                    }
                    let function_type = types::FunctionType { params, returns };

                    typ.add_method(&func.signature.id, function_type)
                }
            }
        }
    }

    Ok(function_collection)
}

struct FuncTypeInference<'a> {
    ///structs: &'a Vec<Box<ast::Struct>>,
    imports: &'a Vec<ast::Import>,
    types: &'a TypeCollection,
    //own_signature:&'a ast::FuncSignature,
    //signatures: &'a Vec<ast::FuncSignature>,
    self_type: Option<Type>,
    functions: &'a FunctionCollection,
    own_signature: &'a types::FunctionType,
    package_id: &'a str,
    file_id: &'a str,
    variable_scopes: Vec<HashMap<String, VariableBinding>>,
}

struct VariableBinding {
    typ: Type,
    is_const: bool,
}

impl<'a> FuncTypeInference<'a> {
    fn new(
        imports: &'a Vec<ast::Import>,
        types: &'a TypeCollection,
        own_signature: &'a types::FunctionType,
        functions: &'a FunctionCollection,
        package_id: &'a str,
        file_id: &'a str,
    ) -> Self {
        Self {
            imports,
            types,
            own_signature,
            functions,
            package_id,
            file_id,
            variable_scopes: Vec::new(),
            self_type: None,
        }
    }

    fn new_for_method(
        imports: &'a Vec<ast::Import>,
        types: &'a TypeCollection,
        own_signature: &'a types::FunctionType,
        functions: &'a FunctionCollection,
        package_id: &'a str,
        file_id: &'a str,
        self_type: Type,
    ) -> Self {
        Self {
            imports,
            types,
            own_signature,
            functions,
            package_id,
            file_id,
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

    pub fn create_var(&mut self, name: String, typ: &Type, is_const: bool) {
        self.variable_scopes.last_mut().unwrap().insert(
            name,
            VariableBinding {
                typ: typ.clone(),
                is_const,
            },
        );
    }

    pub fn find_var(&self, name: &String) -> Option<&VariableBinding> {
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
        Err(SemaError { 
            reason, 
            loc,
            file: self.file_id.into(),
            package: self.package_id.into(),
        })
    }

    pub fn find_type(&self, name: &str) -> Option<Type> {
        self.types
            .get(
                &self.imports,
                self.package_id,
                self.file_id,
                &name.to_string(),
            )
            .cloned()
    }

    fn binary_expr(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
        let b = match &mut e.kind {
            ast::ExprKind::BinaryExpr(b) => b,
            _ => panic!(),
        };

        match b.kind {
            ast::BinaryExprKind::Add
            | ast::BinaryExprKind::Subtract
            | ast::BinaryExprKind::Multiply
            | ast::BinaryExprKind::Divide => {
                self.expr(&mut b.lhs, type_hint.clone())?;
                self.expr(&mut b.rhs, type_hint.clone())?;

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
                        return self.error_loc(
                            SemaErrorReason::IncompatibleTypesInBinaryExpression,
                            e.loc,
                        );
                    }
                }

                if !types::is_numeric(&b.rhs.typ) {
                    return self
                        .error_loc(SemaErrorReason::NonNumericTypeInBinaryExpression, e.loc);
                }
                if !types::is_numeric(&b.lhs.typ) {
                    return self
                        .error_loc(SemaErrorReason::NonNumericTypeInBinaryExpression, e.loc);
                }
                e.typ = typ;
            }
            ast::BinaryExprKind::Equal
            | ast::BinaryExprKind::NotEqual
            | ast::BinaryExprKind::LessThan
            | ast::BinaryExprKind::GreaterThan
            | ast::BinaryExprKind::LessThanEqual
            | ast::BinaryExprKind::GreaterThanEqual => {
                self.expr(&mut b.lhs, None)?;
                self.expr(&mut b.rhs, None)?;

                if types::compare(&b.lhs.typ, &b.rhs.typ) == types::ComparisonResult::Incompatible {
                    return self
                        .error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
                }

                e.typ = types::bool();
            }
            ast::BinaryExprKind::LogicalAnd | ast::BinaryExprKind::LogicalOr => {
                self.expr(&mut b.lhs, None)?;
                self.expr(&mut b.rhs, None)?;

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

    fn unary_expr(
        &mut self,
        _u: &mut Box<ast::UnaryExpr>,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        self.ok()
    }

    fn assign(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
        let a = match &mut e.kind {
            ast::ExprKind::Assign(a) => a,
            _ => panic!(),
        };

        self.expr(&mut a.value, type_hint)?;
        self.store_expr(&mut a.destination)?;
        if types::compare(&a.destination.typ, &a.value.typ) == types::ComparisonResult::Incompatible
        {
            return self.error_loc(SemaErrorReason::AssignmentTypesIncompatible, e.loc);
        }
        e.typ = a.destination.typ.clone();
        self.ok()
    }

    fn call(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let c = match &mut e.kind {
            ast::ExprKind::Call(c) => c,
            _ => panic!(),
        };

        match &mut c.function.kind {
            ast::ExprKind::Identifier(i) => {
                if i.id == "assert" {
                    if c.parameters.len() != 1 {
                        return self.error_loc(SemaErrorReason::CallTooManyArguments, e.loc);
                    }
                    self.expr(&mut c.parameters[0], None)?;
                    return self.ok();
                }

                let (func_signature, name_spec) =
                    match self
                        .functions
                        .get(self.imports, self.package_id, self.file_id, &i.id)
                    {
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
                    if types::compare(&param, &arg.typ) == types::ComparisonResult::Incompatible {
                        return self.error_loc(SemaErrorReason::CallArgumentTypeMismatch, arg.loc);
                    }
                }

                c.symbol_name = Some(mangle::mangle_name(&name_spec));

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

                if types::is_struct(&s.value.typ) {
                    if let Some(func_signature) = s.value.typ.get_method(&s.selector.id) {
                        //let func_signature = &f;

                        // Do some basic argument count checking
                        if func_signature.params.len() < c.parameters.len() {
                            return self.error_loc(SemaErrorReason::CallTooManyArguments, e.loc);
                        }
                        if func_signature.params.len() > c.parameters.len() {
                            return self.error_loc(SemaErrorReason::CallNotEnoughArguments, e.loc);
                        }

                        // Sema check arguements, then check argument types
                        for (arg, param) in
                            c.parameters.iter_mut().zip(func_signature.params.iter())
                        {
                            self.expr(arg, Some(param.clone()))?;
                            if types::compare(&param, &arg.typ)
                                == types::ComparisonResult::Incompatible
                            {
                                return self
                                    .error_loc(SemaErrorReason::CallArgumentTypeMismatch, e.loc);
                            }
                        }

                        // Assign this expr the return type of the function
                        if let Some(typ) = func_signature.returns.first() {
                            e.typ = typ.clone();
                        } else {
                            e.typ = types::bad();
                        }

                        c.symbol_name =
                            Some(mangle::mangle_method_name(&s.selector.id, &s.value.typ));

                        return self.ok();
                    } else {
                        return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc);
                    }
                } else if types::is_interface(&s.value.typ) {
                    if let Some(func_signature) = s.value.typ.get_method(&s.selector.id) {
                        //let func_signature = &f;

                        // Do some basic argument count checking
                        if func_signature.params.len() < c.parameters.len() {
                            return self.error_loc(SemaErrorReason::CallTooManyArguments, e.loc);
                        }
                        if func_signature.params.len() > c.parameters.len() {
                            return self.error_loc(SemaErrorReason::CallNotEnoughArguments, e.loc);
                        }

                        // Sema check arguements, then check argument types
                        for (arg, param) in
                            c.parameters.iter_mut().zip(func_signature.params.iter())
                        {
                            self.expr(arg, Some(param.clone()))?;
                            if types::compare(&arg.typ, &param)
                                == types::ComparisonResult::Incompatible
                            {
                                return self
                                    .error_loc(SemaErrorReason::CallArgumentTypeMismatch, e.loc);
                            }
                        }

                        // Assign this expr the return type of the function
                        if let Some(typ) = func_signature.returns.first() {
                            e.typ = typ.clone();
                        } else {
                            e.typ = types::bad();
                        }
                        c.function.typ = s.value.typ.clone();

                        return self.ok();
                    } else {
                        return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc);
                    }
                } else if let types::TypeKind::Enum(enum_type) = s.value.typ.kind() {
                    match enum_type
                        .variants
                        .read()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .find(|v| v.1.0 == s.selector.id)
                    {
                        Some((i, values)) => {
                            // Assign this expr the type of the enum
                            e.typ = s.value.typ.clone();

                            for (arg, param) in c.parameters.iter_mut().zip(values.1.iter()) {
                                self.expr(arg, Some(param.clone()))?;
                                if types::compare(&arg.typ, &param)
                                    == types::ComparisonResult::Incompatible
                                {
                                    return self.error_loc(
                                        SemaErrorReason::EnumVariantValueTypesIncompatible,
                                        e.loc,
                                    );
                                }
                            }

                            c.enum_idx = Some(i);
                            self.ok()
                        }
                        None => {
                            return self.error_loc(SemaErrorReason::CannotFindVariantInEnum, e.loc);
                        }
                    }
                } else {
                    return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, s.value.loc);
                }
            }
            _ => unimplemented!("Function calls must be direct(by name) for now"),
        }
    }

    fn integer(
        &mut self,
        _i: &mut Box<ast::Integer>,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        self.ok()
    }

    fn number(
        &mut self,
        _f: &mut Box<ast::Number>,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        self.ok()
    }

    fn boolean(
        &mut self,
        _b: &mut Box<ast::Bool>,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        self.ok()
    }

    fn string_literal(
        &mut self,
        _s: &mut Box<ast::StringLiteral>,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        self.ok()
    }

    fn identifier(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let i = match &e.kind {
            ast::ExprKind::Identifier(i) => i,
            _ => panic!(),
        };
        if let Some(binding) = self.find_var(&i.id) {
            e.typ = binding.typ.clone();
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
            _ => panic!(),
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
            _ => panic!(), // already checked above
        };
        self.ok()
    }

    fn selector(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let s = match &mut e.kind {
            ast::ExprKind::Selector(s) => s,
            _ => panic!(),
        };

        self.expr(&mut s.value, None)?;

        if let types::TypeKind::Struct(struct_type) = s.value.typ.kind() {
            if let Some((i, (_, ty))) = struct_type
                .fields
                .read()
                .unwrap()
                .iter()
                .enumerate()
                .find(|a| a.1.0 == s.selector.id)
            {
                e.typ = ty.clone();
                s.idx = i;
            } else {
                return self.error_loc(SemaErrorReason::CannotFindSelectorInStruct, e.loc);
            }
        } else if let types::TypeKind::Enum(enum_type) = s.value.typ.kind() {
            if let Some((i, _)) = enum_type
                .variants
                .read()
                .unwrap()
                .iter()
                .enumerate()
                .find(|a| a.1.0 == s.selector.id)
            {
                e.typ = s.value.typ.clone();
                s.enum_idx = Some(i);
            } else {
                return self.error_loc(SemaErrorReason::CannotFindVariantInEnum, e.loc);
            }
        } else {
            return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, e.loc);
        }

        self.ok()
    }

    fn array_literal(
        &mut self,
        e: &mut ast::Expr,
        type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        // get type from first element, check all are the same, then assign type as array of that type
        let l = match &mut e.kind {
            ast::ExprKind::ArrayLiteral(l) => l,
            _ => panic!(),
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
            let inner = type_hint.and_then(|t| {
                if let types::TypeKind::Array(element_type) = t.kind() {
                    Some(element_type.clone())
                } else {
                    None
                }
            });

            // Process the first element to determine the type of the array,
            // then check the rest of the elements are the same type
            self.expr(&mut l.literals[0], inner)?;
            let element_type = l.literals[0].typ.clone();

            for literal in l.literals.iter_mut().skip(1) {
                self.expr(literal, Some(element_type.clone()))?;
                if types::compare(&element_type, &literal.typ)
                    == types::ComparisonResult::Incompatible
                {
                    return self
                        .error_loc(SemaErrorReason::IncompatibleTypesInBinaryExpression, e.loc);
                }
            }
            e.typ = types::array(element_type);
        }

        self.ok()
    }

    fn object_literal(
        &mut self,
        e: &mut ast::Expr,
        _type_hint: Option<types::Type>,
    ) -> SemaResult<()> {
        let l = match &mut e.kind {
            ast::ExprKind::ObjectLiteral(l) => l,
            _ => panic!(),
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
            if types::compare(&struct_field.1, &field.value.typ)
                == types::ComparisonResult::Incompatible
            {
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

    fn template(&mut self, e: &mut ast::Expr, _type_hint: Option<types::Type>) -> SemaResult<()> {
        let t = match &mut e.kind {
            ast::ExprKind::Template(t) => t,
            _ => panic!(),
        };
        // check there is at least one substitution, then check all substitutions are strings, then assign type as string
        let string_interface = self
            .types
            .get_exact("builtins", "builtins", "String")
            .expect("String interface not found");

        for expr in t.expressions.iter_mut() {
            self.expr(expr, None)?;
            if types::compare(string_interface, &expr.typ) == types::ComparisonResult::Incompatible
            {
                return self.error_loc(SemaErrorReason::TemplateSubstitutionMustBeString, expr.loc);
            }

            // wrap the expression in a call to the string function on the string interface
            let original_expr = expr.clone();
            *expr = ast::Expr {
                kind: ast::ExprKind::Call(Box::new(ast::Call {
                    function: ast::Expr {
                        kind: ast::ExprKind::Selector(Box::new(ast::Selector {
                            value: original_expr,
                            selector: ast::Identifier {
                                id: "string".into(),
                            },
                            idx: 0,
                            enum_idx: None,
                        })),
                        typ: types::bad(), // will be filled in by the selector sema
                        loc: expr.loc,
                    },
                    parameters: Vec::new(),
                    symbol_name: None,
                    enum_idx: None,
                })),
                typ: types::bad(), // will be filled in by the call sema
                loc: expr.loc,
            };
            // sema check the new call expression
            self.expr(expr, None)?;
        }

        e.typ = types::string();
        self.ok()
    }

    fn expr(&mut self, e: &mut ast::Expr, type_hint: Option<types::Type>) -> SemaResult<()> {
        let checked_e = match &mut e.kind {
            ast::ExprKind::BinaryExpr(_) => self.binary_expr(e, type_hint.clone()),
            ast::ExprKind::UnaryExpr(u) => self.unary_expr(u, type_hint.clone()),
            ast::ExprKind::Assign(_) => self.assign(e, type_hint.clone()),
            ast::ExprKind::Call(_) => self.call(e, type_hint.clone()),
            ast::ExprKind::Integer(i) => self.integer(i, type_hint.clone()),
            ast::ExprKind::Number(f) => self.number(f, type_hint.clone()),
            ast::ExprKind::Boolean(b) => self.boolean(b, type_hint.clone()),
            ast::ExprKind::StringLiteral(s) => self.string_literal(s, type_hint.clone()),
            ast::ExprKind::Identifier(_) => self.identifier(e, type_hint.clone()),
            ast::ExprKind::Subscript(_) => self.subscript(e, type_hint.clone()),
            ast::ExprKind::Selector(_) => self.selector(e, type_hint.clone()),
            ast::ExprKind::ArrayLiteral(_) => self.array_literal(e, type_hint.clone()),
            ast::ExprKind::ObjectLiteral(_) => self.object_literal(e, type_hint.clone()),
            ast::ExprKind::_Self => self._self(e),
            ast::ExprKind::Template(_) => self.template(e, type_hint.clone()),
            ast::ExprKind::Cast(_) => unimplemented!("Cast expressions not implemented yet"),
        };

        if checked_e.is_err() {
            return checked_e;
        }

        // Wraps implicit casts
        if let Some(expected) = type_hint {
            match types::compare(&expected, &e.typ) {
                types::ComparisonResult::Incompatible => {
                    return self.error_loc(SemaErrorReason::ExpressionCannotBeCasted, e.loc);
                }
                types::ComparisonResult::Upcastable => {
                    // wrap the expression in a cast expression
                    let original_expr = e.clone();
                    e.kind = ast::ExprKind::Cast(Box::new(ast::Cast {
                        value: original_expr,
                        target_type: expected.clone(),
                    }));
                    e.typ = expected;
                }
                _ => {}
            }
        }

        self.ok()
    }

    fn store_subscript(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        // check the array is an array, then check the index is an integer
        // then set the type to the array element type
        let s = match &mut e.kind {
            ast::ExprKind::Subscript(s) => s,
            _ => panic!(),
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
            _ => panic!(), // already checked above
        };
        self.ok()
    }

    fn store_selector(&mut self, e: &mut ast::Expr) -> SemaResult<()> {
        let s = match &mut e.kind {
            ast::ExprKind::Selector(s) => s,
            _ => panic!(),
        };

        self.expr(&mut s.value, None)?;

        if !types::is_struct(&s.value.typ) {
            return self.error_loc(SemaErrorReason::InvalidUsageOfSelector, e.loc);
        }

        if let types::TypeKind::Struct(struct_type) = s.value.typ.kind() {
            if let Some((i, (_, ty))) = struct_type
                .fields
                .read()
                .unwrap()
                .iter()
                .enumerate()
                .find(|a| a.1.0 == s.selector.id)
            {
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
            _ => panic!(),
        };
        if let Some(binding) = self.find_var(&i.id) {
            if binding.is_const {
                return self.error_loc(SemaErrorReason::CannotAssignToConst, e.loc);
            }
            e.typ = binding.typ.clone();
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
            _ => self.error_loc(
                SemaErrorReason::CannotUseExpressionInLeftHandExpression,
                e.loc,
            ),
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
        if let Some(a) = &mut f.alternate {
            self.stmt(a)?;
        }
        self.ok()
    }

    fn return_stmt(&mut self, r: &mut Box<ast::ReturnStmt>) -> SemaResult<()> {
        if let Some(return_type) = self.own_signature.returns.first().cloned() {
            if let Some(r) = &mut r.value {
                self.expr(r, Some(return_type.clone()))?;
                if types::compare(&return_type, &r.typ) == types::ComparisonResult::Incompatible {
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
            let annotation = type_lookup(
                &annotation,
                self.types,
                self.package_id,
                self.file_id,
                self.imports,
            )?;
            self.expr(&mut v.value, Some(annotation.clone()))?;
            let ret = &v.value.typ;
            if types::compare(&annotation, ret) == types::ComparisonResult::Incompatible {
                return self.error_loc(
                    SemaErrorReason::IncompatibleTypesInVariableDefinition,
                    v.loc,
                );
            }
            self.create_var(v.id.clone(), ret, v.is_const);
        } else {
            self.expr(&mut v.value, None)?;
            let ret = &v.value.typ;
            //v.type_annotation = Some(ret.clone());
            self.create_var(v.id.clone(), ret, v.is_const);
        }
        self.ok()
    }

    fn while_stmt(&mut self, w: &mut Box<ast::WhileStmt>) -> SemaResult<()> {
        self.expr(&mut w.condition, Some(types::bool()))?;
        self.stmt(&mut w.consequent)?;
        self.ok()
    }

    fn switch_stmt(&mut self, s: &mut ast::SwitchStmt) -> SemaResult<()> {
        self.expr(&mut s.value, None)?;
        if types::is_enum(&s.value.typ) {
            let enum_type = match &s.value.typ.kind() {
                types::TypeKind::Enum(enum_type) => enum_type,
                _ => unreachable!(),
            };
            for case in s.cases.iter_mut() {
                match &mut case.pattern.kind {
                    ast::PatternKind::CatchAll => {
                        self.block_stmt(&mut case.block)?;
                    }
                    ast::PatternKind::EnumVariant { id, values } => {
                        match enum_type
                            .variants
                            .read()
                            .unwrap()
                            .iter()
                            .enumerate()
                            .find(|v| v.1.0 == *id)
                        {
                            Some(v) => {
                                if values.len() != v.1.1.len() {
                                    return self.error_loc(
                                        SemaErrorReason::EnumVariantPatternFieldCountMismatch,
                                        case.pattern.loc,
                                    );
                                }
                                self.push_scope();
                                for (typ, (name, val_typ)) in v.1.1.iter().zip(values.iter_mut()) {
                                    self.create_var(name.clone(), typ, true);
                                    *val_typ = typ.clone();
                                }
                                self.block_stmt(&mut case.block)?;
                                self.pop_scope();
                                case.case_idx = v.0 as i64;
                            }
                            None => {
                                return self.error_loc(
                                    SemaErrorReason::EnumVariantNotFound,
                                    case.pattern.loc,
                                );
                            }
                        };
                    }
                    _ => {
                        return self
                            .error_loc(SemaErrorReason::InvalidPatternKind, case.pattern.loc);
                    }
                }
            }
        } else if types::is_integer(&s.value.typ) {
            for case in s.cases.iter_mut() {
                match &case.pattern.kind {
                    ast::PatternKind::CatchAll => {
                        self.block_stmt(&mut case.block)?;
                    }
                    ast::PatternKind::Integer(_) => {
                        self.block_stmt(&mut case.block)?;
                    }
                    ast::PatternKind::IntegerRange(_, _) => {
                        self.block_stmt(&mut case.block)?;
                    }
                    _ => {
                        return self
                            .error_loc(SemaErrorReason::InvalidPatternKind, case.pattern.loc);
                    }
                }
            }
        } else {
            unimplemented!();
        }
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
            ast::Stmt::Switch(s) => self.switch_stmt(s),
        }
    }

    fn check(&mut self, func: &mut ast::Func) -> SemaResult<()> {
        self.push_scope();
        for p in func.signature.params.iter() {
            let annotation = type_lookup(
                &p.type_annotation,
                self.types,
                self.package_id,
                self.file_id,
                self.imports,
            )?;
            self.create_var(p.id.clone(), &annotation, false);
        }
        self.block_stmt(&mut func.body)?;
        self.pop_scope();
        self.ok()
    }
}

fn check_file(
    file: &mut Box<ast::File>,
    package_id: &str,
    collection: &TypeCollection,
    functions: &FunctionCollection,
) -> SemaResult<()> {
    for func in file.functions.iter_mut() {
        let own_signature = functions
            .get_exact(package_id, &file.id, &func.signature.id)
            .unwrap();
        func.typ_ = own_signature.clone();
        FuncTypeInference::new(
            &file.imports,
            collection,
            own_signature,
            functions,
            package_id,
            &file.id,
        )
        .check(func)?;

        func.signature.symbol_name = mangle::mangle_name(&NameSpecification {
            package: package_id.into(),
            file: file.id.clone(),
            name: func.signature.id.clone(),
        });
    }

    for _struct in file.structs.iter_mut() {
        let typ = collection
            .get_exact(package_id, &file.id, &_struct.id)
            .unwrap()
            .clone();
        _struct.typ = typ.clone();
        for func in _struct.functions.iter_mut() {
            let own_signature = typ.get_method(&func.signature.id).unwrap();
            func.typ_ = own_signature.clone();
            func.signature.symbol_name = mangle::mangle_method_name(&func.signature.id, &typ);
            FuncTypeInference::new_for_method(
                &file.imports,
                collection,
                &own_signature,
                functions,
                package_id,
                &file.id,
                typ.clone(),
            )
            .check(func)?;
        }
    }

    Ok(())
}

fn check_package(
    package: &mut ast::Package,
    collection: &TypeCollection,
    functions: &FunctionCollection,
) -> SemaResult<()> {
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
