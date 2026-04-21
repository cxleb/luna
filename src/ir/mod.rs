pub mod builder;
pub mod iter;

pub type BlockRef = usize;
pub type VariableRef = usize;
pub type StringRef = usize;
pub type GlobalRef = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Integer,
    Byte,
    Number,
    Bool,
    String,
    Reference,
    Array
}

#[derive(Debug, Clone, PartialEq)]
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
    EquString,
    NeqString,
    And,
    Or,
    LoadConstInt(i64),
    LoadConstByte(u8),
    LoadConstNumber(f64),
    LoadConstBool(bool),
    LoadConstString(StringRef),
    LoadGlobal(GlobalRef),
    Truncate, // Convert number to integer
    Promote,  // Convert integer to number
    Load(VariableRef),
    Store(VariableRef),
    Tee(VariableRef),
    //LoadNumber(u64),
    //StoreNumber(u64),
    CondBr(BlockRef, BlockRef), // branch if 0
    Br(BlockRef),
    BrTable(BlockRef, Vec<BlockRef>),
    Ret,
    // Pops the necessary arguments off the stack, e.g a(10, 10) will pop 2
    Call(String),
    IndirectCall(Signature), // top of the stack is the function to call, arity is the number of arguments (not including the function pointer)

    NewArray(usize, Type),
    LoadArray(Type),  // Pops the index and array
    StoreArray(Type), // Pops the value, index, and array
    CreateSlice(Type),
    ArrayLen,

    NewObject(usize),
    GetObject(usize, Type),
    SetObject(usize, Type),

    /// Check if this task should yield control (inserted at loop backedges and function prologue)
    /// This is a no-op if no yield is needed, or calls __yield() if the scheduler has marked this task
    CheckYield,

    Assert,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceLoc {
    pub file: StringRef,
    pub line: usize,
    pub col: usize,
}

// Contains all source locations because cranelift gives us a single integer for a source location!! argh
#[derive(Debug, Clone, Default)]
pub struct SourceLocs {
    pub locations: Vec<SourceLoc>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockRef,
    pub source_locs: Vec<(usize, usize)>,
    pub ins: Vec<Inst>,
}

impl<'a> Block {
    pub fn iter(&'a self) -> iter::BlockIter<'a> {
        iter::BlockIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: VariableRef,
    pub typ: Type,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub string_map: StringMap,
    pub global_value_map: GlobalValueMap,
    pub source_locs: SourceLocs,
}

#[derive(Debug, Clone)]
pub struct StringMap {
    map: Vec<String>,
}

impl StringMap {
    pub fn new() -> Self {
        StringMap { map: Vec::new() }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalValue {
    VirtualTable(Vec<String>), // method names in order
}

#[derive(Debug, Clone)]
pub struct GlobalValueMap {
    map: Vec<GlobalValue>,
}

impl GlobalValueMap {
    pub fn new() -> Self {
        GlobalValueMap { map: Vec::new() }
    }

    pub fn intern(&mut self, v: GlobalValue) -> GlobalRef {
        if let Some((i, _)) = self
            .map
            .iter()
            .enumerate()
            .find(|(_, existing)| *existing == &v)
        {
            return i;
        }
        self.map.push(v);
        self.map.len() - 1
    }

    pub fn get(&self, index: GlobalRef) -> &GlobalValue {
        self.map.get(index).unwrap()
    }
}
