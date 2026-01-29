#![allow(dead_code)]

use std::sync::Arc;

mod compiler;
mod ir;
mod runtime;
mod types;
mod builtins;

fn main() {

    let mut args = std::env::args();
    args.next();
    let name = args.next().expect("Expected file");

    let builtins = builtins::default_builtins();

    let compiler = compiler::new_compiler();
    compiler::add_root_file(Arc::clone(&compiler), &name);
    let module = compiler::run_compiler(Arc::clone(&compiler), &builtins);

    let mut jit = runtime::JitContext::new(builtins);
    //print!("compiling... ");  std::io::stdout().flush().unwrap();
    jit.compile_ir_module(&module);
    //println!("done.");
    //println!("running...\n");
    _ = jit.call_function_no_params::<i64>("main");
    //println!("Returned {}", returned);
}
