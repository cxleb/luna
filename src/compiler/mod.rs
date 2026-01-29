use core::panic;
use std::sync::{Arc, Mutex};

use crate::ir;

use crate::builtins::Builtins;

mod taskgraph;
pub mod ast;
pub mod emit;
pub mod parser;
pub mod source;
pub mod token;
pub mod tokeniser;
pub mod checker;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceLoc {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

pub struct Compiler {
    parse_tasks: Vec<Box <dyn FnMut()>>,
    errors: Vec<String>,
    program: ast::Program,
}

pub fn new_compiler() -> Arc<Mutex<Compiler>> {
    Arc::new(Mutex::new(Compiler {
        parse_tasks: Vec::new(),
        errors: Vec::new(),
        program: ast::Program::default()
    }))
}

pub fn add_root_file(compiler: Arc<Mutex<Compiler>>, filename: &str) {
    let filename = filename.to_string();
    let compiler_clone = Arc::clone(&compiler); 
    let parse_task = Box::new(move || {
        match std::fs::read_to_string(&filename) {
            Ok(src) => {
                let mut parser = parser::Parser::new(&src);
                match parser.parse_file() {
                    Ok(file) => {
                        compiler_clone.lock().unwrap().program.packages[0].files.push(file);
                    }
                    Err(e) => {
                        compiler_clone.lock().unwrap().errors.push(format!("Error parsing file {}: {:?}", filename, e));
                    }
                }
            }
            Err(e) => {
                compiler_clone.lock().unwrap().errors.push(format!("Could not read file {}: {}", filename, e));
            }
        }
    });
    let compiler = Arc::clone(&compiler);
    compiler.lock().unwrap().program.packages.push(Box::new(ast::Package {
        id: "main".into(),
        files: Vec::new(),
    }));
    compiler.lock().unwrap().parse_tasks.push(parse_task);
}

pub fn run_compiler(compiler: Arc<Mutex<Compiler>>, builtins: &Builtins) -> Box<ir::Module> {
    let compiler = Arc::clone(&compiler);
    
    while let Some(mut task) = { compiler.lock().unwrap().parse_tasks.pop() } {
        task();
    }

    if !compiler.lock().unwrap().errors.is_empty() {
        for error in &compiler.lock().unwrap().errors {
            eprintln!("{}", error);
        }
        panic!("Compilation failed due to errors.");
    }

    // By this point we should be ok to maintain a mutable reference
    let mut compiler = compiler.lock().unwrap();

    checker::check_program(&mut compiler.program, builtins).unwrap();

    emit::emit_program(&compiler.program)
}