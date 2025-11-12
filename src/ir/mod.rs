use crate::types::Type;

pub mod builder;

type BlockRef = usize;
type VariableRef = usize;

#[derive(Debug, Clone)]
pub enum Inst {
    Nop,
    Dup, // Duplicates the top of the stack
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    ModInt,
    EquInt,
    NeqInt,
    LtInt,
    GtInt,
    LeqInt,
    GeqInt,
    AddNumber,
    SubNumber,
    MulNumber,
    DivNumber,
    EquNumber,
    NeqNumber,
    LtNumber,
    GtNumber,
    LeqNumber,
    GeqNumber,
    LoadConstInt(i64),
    LoadConstNumber(f64),
    LoadConstBool(bool),
    Load(VariableRef),
    Store(VariableRef),
    //LoadNumber(u64),
    //StoreNumber(u64),
    CondBr(BlockRef, BlockRef), // branch if 0
    Br(BlockRef),
    Ret,
    // Pops the necessary arguments off the stack, e.g a(10, 10) will pop 2 
    Call(String),
    IndirectCall, // top of the stack is the function to call.
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockRef,
    pub ins: Vec<Inst>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: VariableRef,
    pub typ: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub ret_types: Vec<Box<Type>>,
    pub parameters: Vec<Box<Type>>,
}

// Logically, a function will start at block 0 and execute
#[derive(Debug, Clone)]
pub struct Function {
    pub id: String,
    pub signature: Signature,
    pub variables: Vec<Variable>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub funcs: Vec<Function>,
}
