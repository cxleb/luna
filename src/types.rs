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