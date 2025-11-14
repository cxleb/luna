#[derive(Debug, Clone, Default, PartialEq)]
pub enum Type {
    #[default]
    Unknown,
    Integer,
    Number,
    String,
    Bool,
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