#![allow(dead_code)]

use std::io::Write;

mod compiler;
mod ir;
mod runtime;
mod types;

fn main() {
    let src = include_str!("../example.luna");
    print!("parsing... ");  std::io::stdout().flush().unwrap();
    let mut parser = compiler::parser::Parser::new(src);
    let mut ast_module = parser.parse_module().unwrap();
    println!("done.");
    print!("type checking... ");  std::io::stdout().flush().unwrap();
    let sema_error = compiler::sema::sema_module(&mut ast_module);
    if let Err(error) = sema_error  {
        println!("error: {:?}", error);
        return;
    }
    println!("done.");
    //println!("{:?}", ast_module);
    print!("generating ir... ");  std::io::stdout().flush().unwrap();
    let module = compiler::generate::gen_module(ast_module);
    println!("done.");
    println!("{:?}", module);
    //runtime::run();

    //runtime::start(&module);

    let mut jit = runtime::JitContext::new();
    print!("compiling... ");  std::io::stdout().flush().unwrap();
    jit.compile_ir_module(&module);
    println!("done.");
    println!("running...\n");
    let returned = jit.call_function_no_params::<i64>("main");
    println!("Returned {}", returned);
}
