use core::panic;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::ir;

use crate::builtins::Builtins;

pub mod ast;
pub mod checker;
pub mod emit;
pub mod mangle;
pub mod parser;
pub mod source;
pub mod token;
pub mod tokeniser;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceLoc {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

pub struct Compiler {
    parse_tasks: Vec<Box<dyn FnMut()>>,
    scheduled_files: HashSet<(String, String)>,
    errors: Vec<String>,
    program: ast::Program,
}

pub fn new_compiler() -> Arc<Mutex<Compiler>> {
    Arc::new(Mutex::new(Compiler {
        parse_tasks: Vec::new(),
        scheduled_files: HashSet::new(),
        errors: Vec::new(),
        program: ast::Program::default(),
    }))
}

fn ensure_package<'a>(
    program: &'a mut ast::Program,
    package_id: &str,

) -> &'a mut Box<ast::Package> {
    if let Some(index) = program
        .packages
        .iter()
        .position(|package| package.id == package_id)
    {
        &mut program.packages[index]
    } else {
        program.packages.push(Box::new(ast::Package {
            id: package_id.into(),
            files: Vec::new(),
            base_path: None,
        }));
        program.packages.last_mut().unwrap()
    }
}

pub fn add_file(compiler: Arc<Mutex<Compiler>>, package_id: &str, filename: &str) {
    let package_id = package_id.to_string();
    let filename = filename.to_string();

    {
        let mut compiler_guard = compiler.lock().unwrap();
        if !compiler_guard
            .scheduled_files
            .insert((package_id.clone(), filename.clone()))
        {
            return;
        }
    }

    let full_path = {
        let mut compiler_guard = compiler.lock().unwrap();
        let package = ensure_package(&mut compiler_guard.program, &package_id);
        if let Some(base_path) = &package.base_path {
            base_path.join(&filename).to_str().unwrap().to_string()
        } else {
            filename.clone()
        }
    };

    let compiler_clone = Arc::clone(&compiler);
    let parse_task = Box::new(move || match std::fs::read_to_string(&full_path) {
        Ok(src) => {
            let mut parser = parser::Parser::new(&package_id, &src);
            match parser.parse_file() {
                Ok(mut file) => {
                    file.id = filename.clone();
                    let imports = file.imports.clone();
                    {
                        let mut compiler_guard = compiler_clone.lock().unwrap();
                        let package =
                            ensure_package(&mut compiler_guard.program, &package_id);
                        package.files.push(file);
                    }

                    for import in imports {
                        add_file(Arc::clone(&compiler_clone), &import.package, &import.file);
                    }
                }
                Err(e) => {
                    compiler_clone.lock().unwrap().errors.push(format!(
                        "Error parsing file {} in package {}: {:?}",
                        filename, package_id, e
                    ));
                }
            }
        }
        Err(e) => {
            compiler_clone.lock().unwrap().errors.push(format!(
                "Could not read file {} in package {}: {}",
                filename, package_id, e
            ));
        }
    });
    compiler.lock().unwrap().parse_tasks.push(parse_task);
}

pub fn add_root_file(compiler: Arc<Mutex<Compiler>>, filename: &str) -> String {
    let base_path = std::path::absolute(filename).unwrap().parent().unwrap().to_path_buf();
    {
        let mut compiler_guard = compiler.lock().unwrap();
        let package = ensure_package(&mut compiler_guard.program, "main");
        package.base_path = Some(base_path);
    }
    // strip the base path from the filename for the root file
    let filename = std::path::Path::new(filename).file_name().and_then(|f| f.to_str()).unwrap();
    add_file(compiler, "main", filename);
    filename.into()
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
