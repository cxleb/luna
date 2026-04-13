use std::net::Ipv4Addr;

use crate::types;
use libc::{self};

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

pub fn builtin_add_byte(
    _: *mut crate::runtime::RuntimeContext,
    builder: *const u8,
    value: u8,
) {
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

pub fn builtin_input(_: *mut crate::runtime::RuntimeContext) -> *const u8 {
    use std::io::{self, Write};
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let internal = crate::runtime::string::convert_to_interal_string(&input.trim_end());
    Box::into_raw(internal) as *const u8
}

pub fn builtin_tcp_connect(_: *mut crate::runtime::RuntimeContext, address: *const u8) -> i64 {
    let address = crate::runtime::string::convert_from_internal_string(address);

    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() != 2 {
        return -5;
    }

    let host = parts[0];
    let port = match parts[1].parse::<u16>() {
        Ok(p) => p,
        Err(_) => return -4,
    };

    let host = match host.parse::<Ipv4Addr>() {
        Ok(h) => h,
        Err(_) => return -3,
    };

    let socket = unsafe { libc::socket(libc::PF_INET, libc::SOCK_STREAM, libc::IPPROTO_TCP) };
    if socket == -1 {
        return -1;
    }

    unsafe {
        let sa = libc::sockaddr_in {
            sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8,
            sin_family: libc::AF_INET as u8,
            sin_port: port,
            sin_addr: libc::in_addr {
                s_addr: host.to_bits()
            },
            sin_zero: [0; 8],
        };

        if libc::bind(socket, &sa as *const libc::sockaddr_in as *const libc::sockaddr, std::mem::size_of::<libc::sockaddr_in>() as u32) == -1 {
            libc::close(socket);
            return -2;
        }
    }

    socket as i64
}

pub fn builtin_tcp_accept(_: *mut crate::runtime::RuntimeContext, socket: i64) -> i64 {
    let client_socket = unsafe { libc::accept(socket as libc::c_int, std::ptr::null_mut(), std::ptr::null_mut()) };
    if client_socket == -1 {
        return -1;
    }
    client_socket as i64
}

pub fn builtin_tcp_disconnect(_: *mut crate::runtime::RuntimeContext, socket: i64) -> i64 {
    if unsafe { libc::close(socket as libc::c_int) } == -1 {
        return -1;
    }
    0
}

pub fn builtin_stdin(_: *mut crate::runtime::RuntimeContext) -> i64 {
    libc::STDIN_FILENO as i64
}

pub fn builtin_stdout(_: *mut crate::runtime::RuntimeContext) -> i64 {
    libc::STDOUT_FILENO as i64
}

pub fn builtin_stderr(_: *mut crate::runtime::RuntimeContext) -> i64 {
    libc::STDERR_FILENO as i64
}

pub fn builtin_read(ctx: *mut crate::runtime::RuntimeContext, fd: i64) -> *const i64 {
    let mut buffer = [0u8; 1024];
    let mut copy_buffer = Vec::new();

    loop {
        let bytes_read = unsafe { libc::read(fd as libc::c_int, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len()) };
        if bytes_read == -1 {
            return unsafe {
                (*ctx).gc.create_array(0, 1, false)
            };
        }
        for i in 0..bytes_read {
            copy_buffer.push(buffer[i as usize] as i64);
        }
        if (bytes_read as usize) < buffer.len() {
            break;
        }
    }

    let array = unsafe {
        (*ctx).gc.create_array(buffer.len(), 1, false)
    };
    for (i, &b) in copy_buffer.iter().enumerate() {
        unsafe {
            let ptr = (array as usize + 8 + (i * 8)) as *mut usize;
            *ptr = b as usize;
        }
    }
    array
}
//
//pub fn builtin_write(ctx: *mut crate::runtime::RuntimeContext, fd: i64, array: *mut i64) {
//    loop {
//        let bytes_read = unsafe { libc::write(fd, array as *const libc::c_void, 8) };
//    }
//}

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
        "add_byte",
        vec![types::unknown_reference(), types::byte()],
        None,
        builtin_add_byte,
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
    builtins.push_function_0("stdin", vec![], Some(types::integer()), builtin_stdin);
    builtins.push_function_0("stdout", vec![], Some(types::integer()), builtin_stdout);
    builtins.push_function_0("stderr", vec![], Some(types::integer()), builtin_stderr);
    builtins.push_function("tcp_connect", vec![types::string()], Some(types::integer()), builtin_tcp_connect);
    builtins.push_function("tcp_accept", vec![types::integer()], Some(types::integer()), builtin_tcp_accept);
    builtins.push_function("tcp_disconnect", vec![types::integer()], None, builtin_tcp_disconnect);
    builtins.push_function("read", vec![types::integer()], Some(types::array(types::integer())), builtin_read);


    builtins
}
