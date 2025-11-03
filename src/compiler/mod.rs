pub mod ast;
pub mod generate;
pub mod parser;
pub mod source;
pub mod token;
pub mod tokeniser;
pub mod sema;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceLoc {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}
