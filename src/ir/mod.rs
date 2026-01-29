use crate::types::Type;

pub mod builder;

type BlockRef = usize;
type VariableRef = usize;
type StringRef = usize;

#[derive(Debug, Clone)]
pub enum Inst {
    Nop,
    Dup(usize), // Duplicates the top - i of the stack
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
    And,
    Or,
    LoadConstInt(i64),
    LoadConstNumber(f64),
    LoadConstBool(bool),
    LoadConstString(StringRef),
    Truncate, // Convert number to integer
    Promote,   // Convert integer to number
    Load(VariableRef),
    Store(VariableRef),
    Tee(VariableRef),
    //LoadNumber(u64),
    //StoreNumber(u64),
    CondBr(BlockRef, BlockRef), // branch if 0
    Br(BlockRef),
    Ret,
    // Pops the necessary arguments off the stack, e.g a(10, 10) will pop 2 
    Call(String),
    IndirectCall, // top of the stack is the function to call.

    NewArray(usize),
    LoadArray(Type), // Pops the index and array
    StoreArray(Type), // Pops the value, index, and array

    NewObject(usize),
    GetObject(usize, Type),
    SetObject(usize, Type),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockRef,
    pub ins: Vec<Inst>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: VariableRef,
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub ret_types: Vec<Type>,
    pub parameters: Vec<Type>,
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
    pub string_map: StringMap
}

#[derive(Debug, Clone)]
pub struct StringMap {
    map: Vec<String>
}

impl StringMap {
    pub fn new() -> Self {
        StringMap {
            map: Vec::new()
        }
    }

    pub fn intern(&mut self, s: &str) -> StringRef {
        if let Some((i, _)) = self.map.iter().enumerate().find(|(_, v)| *v == s) {
            return i;
        }
        self.map.push(s.to_string());
        self.map.len() - 1
    }

    pub fn get(&self, index: StringRef) -> &str {
        self.map.get(index).map(|s| s.as_str()).unwrap()
    }
}