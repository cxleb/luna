use crate::types;

pub struct BuiltinFunction {
    pub id: String,
    pub parameters: Vec<types::Type>,
    pub returns: Option<types::Type>,
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

    pub fn push_function<P, R>(
        &mut self,
        id: &str,
        params: Vec<types::Type>,
        ret: Option<types::Type>,
        implementation: fn(*mut crate::runtime::RuntimeContext, P) -> R,
    ) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }

    pub fn push_function_0<R>(
        &mut self,
        id: &str,
        params: Vec<types::Type>,
        ret: Option<types::Type>,
        implementation: fn(*mut crate::runtime::RuntimeContext) -> R,
    ) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }

    pub fn push_function_2<P1, P2, R>(
        &mut self,
        id: &str,
        params: Vec<types::Type>,
        ret: Option<types::Type>,
        implementation: fn(*mut crate::runtime::RuntimeContext, P1, P2) -> R,
    ) {
        let func = BuiltinFunction {
            id: id.into(),
            parameters: params,
            returns: ret,
            implementation: implementation as *const u8,
        };
        self.functions.push(func);
    }

    pub fn push_function_3<P1, P2, P3, R>(
        &mut self,
        id: &str,
        params: Vec<types::Type>,
        ret: Option<types::Type>,
        implementation: fn(*mut crate::runtime::RuntimeContext, P1, P2, P3) -> R,
    ) {
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
    print!(
        "{}",
        crate::runtime::string::convert_from_internal_string(value)
    );
}

pub fn builtin_println(_: *mut crate::runtime::RuntimeContext, value: *const u8) {
    println!(
        "{}",
        crate::runtime::string::convert_from_internal_string(value)
    );
}

pub fn builtin_printint(_: *mut crate::runtime::RuntimeContext, value: i64) {
    print!("{}", value);
}

pub fn builtin_printnum(_: *mut crate::runtime::RuntimeContext, value: f64) {
    print!("{}", value);
}

pub fn builtin_printarray(_: *mut crate::runtime::RuntimeContext, value: *const i64) {
    print!("{:?} {}", value, unsafe { *value });
}

pub fn builtin_create_template_builder(ctx: *mut crate::runtime::RuntimeContext) -> *const u8 {
    //Box::into_raw(Box::new(String::new())) as *const u8
    unsafe {
        (*ctx).gc.create_boxed_value(String::new()) as *const u8
    }
}

pub fn builtin_add_number(_: *mut crate::runtime::RuntimeContext, builder: *const u8, value: f64) {
    let builder = unsafe { &mut *(builder as *mut String) };
    builder.push_str(&value.to_string());
}

pub fn builtin_add_integer(_: *mut crate::runtime::RuntimeContext, builder: *const u8, value: i64) {
    let builder = unsafe { &mut *(builder as *mut String) };
    builder.push_str(&value.to_string());
}

pub fn builtin_add_boolean(
    _: *mut crate::runtime::RuntimeContext,
    builder: *const u8,
    value: bool,
) {
    let builder = unsafe { &mut *(builder as *mut String) };
    builder.push_str(if value { "true" } else { "false" });
}

pub fn builtin_add_string(
    _: *mut crate::runtime::RuntimeContext,
    builder: *const u8,
    value: *const u8,
) {
    let builder = unsafe { &mut *(builder as *mut String) };
    builder.push_str(crate::runtime::string::convert_from_internal_string(value));
}

pub fn builtin_get_string(_: *mut crate::runtime::RuntimeContext, builder: *const u8) -> *const u8 {
    let builder = unsafe { &mut *(builder as *mut String) };
    let internal = crate::runtime::string::convert_to_interal_string(&builder);
    Box::into_raw(internal) as *const u8
}

pub fn default_builtins() -> Builtins {
    let mut builtins = Builtins::new();
    builtins.push_function("print", vec![types::string()], None, builtin_print);
    builtins.push_function("println", vec![types::string()], None, builtin_println);
    builtins.push_function("printint", vec![types::integer()], None, builtin_printint);
    builtins.push_function("printnum", vec![types::number()], None, builtin_printnum);
    builtins.push_function(
        "printarray",
        vec![types::array(types::integer())],
        None,
        builtin_printarray,
    );
    builtins.push_function_0(
        "create_template_builder",
        Vec::new(),
        Some(types::unknown_reference()),
        builtin_create_template_builder,
    );
    builtins.push_function_2(
        "add_number",
        vec![types::unknown_reference(), types::number()],
        None,
        builtin_add_number,
    );
    builtins.push_function_2(
        "add_integer",
        vec![types::unknown_reference(), types::integer()],
        None,
        builtin_add_integer,
    );
    builtins.push_function_2(
        "add_boolean",
        vec![types::unknown_reference(), types::bool()],
        None,
        builtin_add_boolean,
    );
    builtins.push_function_2(
        "add_string",
        vec![types::unknown_reference(), types::string()],
        None,
        builtin_add_string,
    );
    builtins.push_function(
        "get_string",
        vec![types::unknown_reference()],
        Some(types::string()),
        builtin_get_string,
    );

    builtins
}
