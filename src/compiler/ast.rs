use crate::{compiler::SourceLoc, types};

// This is used in the AST to store a reference to a type.
// This will be used with the file's lookup rules to resolve to a type
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Type {
    #[default]
    Unknown,
    Integer,
    Number,
    String,
    Bool, 
    Identifier(String),
    Array(Box<Type>),
    //UnknownReference, // An internal detail before generics is correctly implemented
}

#[derive(Debug, Clone)]
pub enum BinaryExprKind {
    LogicalAnd,
    LogicalOr,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub lhs: Expr,
    pub rhs: Expr,
    pub kind: BinaryExprKind,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {}

#[derive(Debug, Clone)]
pub struct Assign {
    pub destination: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub function: Expr,
    pub parameters: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Integer {
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Number {
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct Bool {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Subscript {
    pub value: Expr,
    pub index: Expr,
}

#[derive(Debug, Clone)]
pub struct Selector {
    pub value: Expr,
    pub selector: Identifier,
    pub idx: usize // the offset of the field within the struct, 
                 // saves us computing it twice and some complexity
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub literals: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralField {
    pub loc: SourceLoc,
    pub id: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub id: Option<Identifier>,
    pub fields: Vec<ObjectLiteralField>,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    BinaryExpr(Box<BinaryExpr>),
    UnaryExpr(Box<UnaryExpr>),
    Assign(Box<Assign>),
    Call(Box<Call>),
    Integer(Box<Integer>),
    Number(Box<Number>),
    StringLiteral(Box<StringLiteral>),
    Boolean(Box<Bool>),
    Identifier(Box<Identifier>),
    Subscript(Box<Subscript>),
    Selector(Box<Selector>),
    ArrayLiteral(Box<ArrayLiteral>),
    ObjectLiteral(Box<ObjectLiteral>),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub loc: SourceLoc,
    pub typ: types::Type,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub loc: SourceLoc,
    pub test: Expr,
    pub consequent: Stmt,
    pub alternate: Option<Stmt>,
    pub not: bool
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub loc: SourceLoc,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub loc: SourceLoc,
    pub is_const: bool,
    pub id: String,
    pub type_annotation: Option<Box<Type>>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub loc: SourceLoc,
    pub condition: Expr,
    pub consequent: Stmt,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub loc: SourceLoc,
    pub id: String,
    pub iterator: Expr,
    pub consequent: Stmt,
}

#[derive(Debug, Clone, Default)]
pub struct BlockStmt {
    pub loc: SourceLoc,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub loc: SourceLoc,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Box<IfStmt>),
    Return(Box<ReturnStmt>),
    VarDecl(Box<VarDeclStmt>),
    While(Box<WhileStmt>),
    For(Box<ForStmt>),
    Block(Box<BlockStmt>),
    ExprStmt(Box<ExprStmt>),
}

#[derive(Debug, Default, Clone)]
pub struct Param {
    pub id: String,
    pub type_annotation: Box<Type>,
}

#[derive(Debug, Default, Clone)]
pub struct FuncSignature {
    pub id: String,
    pub params: Vec<Param>,
    pub return_type: Option<Box<Type>>,
}

#[derive(Debug, Default, Clone)]
pub struct Func {
    pub loc: SourceLoc,
    pub signature: FuncSignature,
    pub typ_: types::FunctionType,
    pub body: Box<BlockStmt>,
}

#[derive(Debug, Default, Clone)]
pub struct StructField {
    pub loc: SourceLoc,
    pub id: String,
    pub type_annotation: Box<Type>,
}

#[derive(Debug, Default, Clone)]
pub struct Struct {
    pub loc: SourceLoc,
    pub id: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Default, Clone)]
pub struct File {
    pub id: String,
    pub functions: Vec<Box<Func>>,
    pub structs: Vec<Box<Struct>>,
    pub imports: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Package {
    pub id: String,
    pub files: Vec<Box<File>>,
}

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub packages: Vec<Box<Package>>,
}