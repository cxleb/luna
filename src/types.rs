#[derive(Debug, Clone, Default, PartialEq)]
pub enum Type {
    #[default]
    Unknown,
    Integer,
    Number,
    String,
    Bool, 
    UnknownReference, // An internal detail before generics is correctly implemented
    Array(Box<Type>),
    Identifier(String), //Function,
                        //Interface,
                        //Struct,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonResult {
    Same,
    Downcastable,
    Incompatible,
}

pub fn compare(a: &Type, b: &Type) -> ComparisonResult {
    if a == b {
        ComparisonResult::Same
    } else {
        ComparisonResult::Incompatible
    }
}

/// Is type numeric (integer or number)
pub fn is_numeric(ty: &Type) -> bool {
    matches!(ty, Type::Integer | Type::Number)
}

pub fn is_number(ty: &Type) -> bool {
    matches!(ty, Type::Number)
}

pub fn is_integer(ty: &Type) -> bool {
    matches!(ty, Type::Integer)
}

pub fn is_array(ty: &Type) -> bool {
    matches!(ty, Type::Array(_))
}

pub fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Bool)
}

pub fn unknown() -> Box<Type> {
    Box::new(Type::Unknown)
}

pub fn integer() -> Box<Type> {
    Box::new(Type::Integer)
}

pub fn number() -> Box<Type> {
    Box::new(Type::Number)
}

pub fn bool() -> Box<Type> {
    Box::new(Type::Bool)
}

pub fn string() -> Box<Type> {
    Box::new(Type::String)
}

pub fn unknown_reference() -> Box<Type> {
    Box::new(Type::UnknownReference)
}

pub fn array(element_type: Box<Type>) -> Box<Type> {
    Box::new(Type::Array(element_type))
}