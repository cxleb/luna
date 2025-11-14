use crate::types;

pub struct BuiltinFunction {
    pub id: String,
    pub parameters: Vec<Box<types::Type>>,
    pub returns: Option<Box<types::Type>>,
    pub implementation: *const u8,
}

pub struct Builtins {
    pub functions: Vec<BuiltinFunction>,
}

impl Builtins {
    pub fn new() -> Self {
        Builtins {
            functions: Vec::new(),
        }
    }

    pub fn push_function<P, R>(&mut self, id: &str, params: Vec<Box<types::Type>>, ret: Option<Box<types::Type>>, implementation: fn(P) -> R) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }
}

// Default built-in functions like print, input, etc.

pub fn builtin_print(value: i64) {
    println!("{}", value);
}

pub fn builtin_assert(cond: i8) {
    if cond == 0 {
        // todo: handle this correctly instead of fucking panicing lol
        panic!("Assertion failed");    
    }
}

pub fn default_builtins() -> Builtins {
    let mut builtins = Builtins::new();
    builtins.push_function("print", vec![types::integer()], None, builtin_print);
    builtins.push_function("assert", vec![types::bool()], None, builtin_assert); 
    builtins
}