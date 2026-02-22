use std::sync::{Arc, OnceLock, RwLock};


#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct NameSpecification {
    pub package: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub returns: Vec<Type>,
}

#[derive(Debug)]
pub struct StructType {
    pub spec: NameSpecification,
    pub fields: RwLock<Vec<(String, Type)>>,
    pub functions: RwLock<Vec<(String, FunctionType)>>,
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.spec == other.spec
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeKind {
    Bad,
    Integer,
    Number,
    String,
    Bool, 
    UnknownReference, // An internal detail before generics is correctly implemented
    Array(Type),
    Struct(StructType),
    Identifier(String), 
    Function(FunctionType),
    //Function,
    //Interface,
    //Struct,
}

#[derive(Debug, PartialEq)]
pub struct Inner {
    pub kind: TypeKind,
}

#[derive(Debug, PartialEq)]
pub struct Type {
    pub inner: Arc<Inner>,
}

impl Type {
    pub fn kind(&self) -> &TypeKind {
        &self.inner.kind
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        Type {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        bad()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonResult {
    Same,
    Downcastable,
    Incompatible,
}

pub fn compare(a: &Type, b: &Type) -> ComparisonResult {
    if a.inner.kind == b.inner.kind {
        ComparisonResult::Same
    } else {
        ComparisonResult::Incompatible
    }
}

/// Is type numeric (integer or number)
pub fn is_numeric(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Integer | TypeKind::Number)
}

pub fn is_number(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Number)
}

pub fn is_integer(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Integer)
}

pub fn is_array(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Array(_))
}

pub fn is_bool(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Bool)
}

pub fn is_string(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::String)
}

pub fn is_struct(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Struct(_))
}

pub fn is_function(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::Function(_))
}

pub fn is_reference(ty: &Type) -> bool {
    matches!(ty.inner.kind, TypeKind::UnknownReference | TypeKind::Struct(_) | TypeKind::Array(_))
}

pub fn clone_struct_fields(ty: &Type) -> Vec<(String, Type)> {
    if let TypeKind::Struct(struct_type) = &ty.inner.kind {
        struct_type.fields.read().unwrap().clone()
    } else {
        panic!("Type is not a struct");
    }
}

pub fn create_type(kind: TypeKind) -> Type {
    Type {
        inner: Arc::new(Inner { kind }),
    }
}

pub fn bad() -> Type {
    static UNKNOWN_TYPE: OnceLock<Type> = OnceLock::new();
    UNKNOWN_TYPE.get_or_init(|| create_type(TypeKind::Bad)).clone()
}

pub fn integer() -> Type {
    static INTEGER_TYPE: OnceLock<Type> = OnceLock::new();
    INTEGER_TYPE.get_or_init(|| create_type(TypeKind::Integer)).clone()
}

pub fn number() -> Type {
    static NUMBER_TYPE: OnceLock<Type> = OnceLock::new();
    NUMBER_TYPE.get_or_init(|| create_type(TypeKind::Number)).clone()
}

pub fn bool() -> Type {
    static BOOL_TYPE: OnceLock<Type> = OnceLock::new();
    BOOL_TYPE.get_or_init(|| create_type(TypeKind::Bool)).clone()
}

pub fn string() -> Type {
    static STRING_TYPE: OnceLock<Type> = OnceLock::new();
    STRING_TYPE.get_or_init(|| create_type(TypeKind::String)).clone()
}

pub fn unknown_reference() -> Type {
    static UNKNOWN_REFERENCE_TYPE: OnceLock<Type> = OnceLock::new();
    UNKNOWN_REFERENCE_TYPE.get_or_init(|| create_type(TypeKind::UnknownReference)).clone()
}

pub fn array(element_type: Type) -> Type {
    create_type(TypeKind::Array(element_type))
}

pub fn identifier(id: String) -> Type {
    create_type(TypeKind::Identifier(id))
}

pub fn struct_type(spec: NameSpecification, fields: Vec<(String, Type)>, functions: Vec<(String, FunctionType)>) -> Type {
    create_type(TypeKind::Struct(StructType {
        spec,
        fields: RwLock::new(fields),
        functions: RwLock::new(functions),
    }))
}

pub fn function_type(params: Vec<Type>, returns: Vec<Type>) -> Type {
    create_type(TypeKind::Function(FunctionType { params, returns }))
}