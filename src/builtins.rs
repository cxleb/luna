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

    pub fn push_function<P, R>(&mut self, id: &str, params: Vec<Box<types::Type>>, ret: Option<Box<types::Type>>, implementation: fn(*mut crate::runtime::RuntimeContext, P) -> R) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }

    pub fn push_function_2<P1, P2, R>(&mut self, id: &str, params: Vec<Box<types::Type>>, ret: Option<Box<types::Type>>, implementation: fn(*mut crate::runtime::RuntimeContext, P1, P2) -> R) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }

    pub fn push_function_3<P1, P2, P3, R>(&mut self, id: &str, params: Vec<Box<types::Type>>, ret: Option<Box<types::Type>>, implementation: fn(*mut crate::runtime::RuntimeContext, P1, P2, P3) -> R) {
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
pub fn builtin_print(_: *mut crate::runtime::RuntimeContext, value: *const u8) {
    print!("{}", crate::runtime::string::convert_from_internal_string(value));
}

pub fn builtin_println(_: *mut crate::runtime::RuntimeContext, value: *const u8) {
    println!("{}", crate::runtime::string::convert_from_internal_string(value));
}

pub fn builtin_printint(_: *mut crate::runtime::RuntimeContext, value: i64) {
    println!("{}", value);
}

pub fn builtin_printarray(_: *mut crate::runtime::RuntimeContext, value: *const i64) {
    println!("{:?} {}", value, unsafe { *value });
}

pub fn builtin_assert(_: *mut crate::runtime::RuntimeContext, cond: i8) {
    if cond == 0 {
        // todo: handle this correctly instead of fucking panicing lol
        panic!("Assertion failed");    
    }
}

pub fn default_builtins() -> Builtins {
    let mut builtins = Builtins::new();
    builtins.push_function("print", vec![types::string()], None, builtin_print);
    builtins.push_function("println", vec![types::string()], None, builtin_println);
    builtins.push_function("printint", vec![types::integer()], None, builtin_printint);
    builtins.push_function("printarray", vec![types::array(types::integer())], None, builtin_printarray);
    builtins.push_function("assert", vec![types::bool()], None, builtin_assert); 

    builtins
}