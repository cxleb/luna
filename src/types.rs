use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Unknown,
    Integer,
    Number,
    String,
    Bool, 
    UnknownReference, // An internal detail before generics is correctly implemented
    Array(Type),
    Struct(String, Vec<(String, Type)>),
    Identifier(String), 
    //Function,
    //Interface,
    //Struct,
}

#[derive(Debug, PartialEq)]
struct Inner {
    kind: TypeKind,
}

#[derive(Debug, PartialEq)]
pub struct Type {
    inner: Arc<Inner>,
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
        unknown()
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
    matches!(ty.inner.kind, TypeKind::Struct(_, _))
}

pub fn create_type(kind: TypeKind) -> Type {
    Type {
        inner: Arc::new(Inner { kind }),
    }
}

pub fn unknown() -> Type {
    static UNKNOWN_TYPE: OnceLock<Type> = OnceLock::new();
    UNKNOWN_TYPE.get_or_init(|| create_type(TypeKind::Unknown)).clone()
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

pub fn struct_type(id: &str, fields: Vec<(String, Type)>) -> Type {
    create_type(TypeKind::Struct(id.to_string(), fields))
}