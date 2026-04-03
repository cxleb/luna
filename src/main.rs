#![allow(dead_code)]

use luna_rs::{builtins, compiler, runtime, types::NameSpecification};
use std::sync::Arc;

fn main() {
    let mut args = std::env::args();
    args.next();
    let name = args.next().expect("Expected file");

    let builtins = builtins::default_builtins();

    let compiler = compiler::new_compiler();
    let main_file_name = compiler::add_root_file(Arc::clone(&compiler), &name);
    let module = compiler::run_compiler(Arc::clone(&compiler), &builtins);

    let mut jit = runtime::JitContext::new(builtins);
    //print!("compiling... ");  std::io::stdout().flush().unwrap();
    jit.compile_ir_module(&module);
    //println!("done.");
    //println!("running...\n");
    let main_symbol = compiler::mangle::mangle_name(&NameSpecification {
        package: "main".into(),
        file: main_file_name,
        name: "main".into(),
    });
    jit.call_function_no_params_no_return(&main_symbol);
    //println!("Returned {}", returned);
}
