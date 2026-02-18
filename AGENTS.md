# AGENTS.md

This file contains guidelines and commands for agentic coding agents working on the Luna compiler project.

## Build/Lint/Test Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode

### Testing
- `cargo test` - Run Rust unit tests
- `./scripts/run_tests.sh` - Run the complete test suite including Luna language tests
- `./target/debug/luna-rs <filename.luna>` - Run a single Luna test file

### Single Test Execution
To run a specific Luna test file:
```bash
./target/debug/luna-rs tests/expr_literals.luna
./target/debug/luna-rs tests/stmt_if.luna
```

First ensure the project is built with `cargo build`, then run the test file.

## Project Structure

This is a compiler for the Luna programming language written in Rust with the following architecture:
- `src/compiler/` - Parsing, tokenization, AST, and type checking
- `src/runtime/` - JIT compilation using Cranelift and garbage collection
- `src/ir/` - Intermediate representation
- `src/types.rs` - Type system
- `src/builtins.rs` - Built-in functions and types
- `tests/` - Luna language test files (.luna extension)
- `examples/` - Example Luna programs

## Code Style Guidelines

### Rust Code Style

#### Imports and Modules
- Use `use crate::` for internal imports with explicit module paths
- Group imports: standard library first, then external crates, then internal modules
- Prefer `use std::sync::{Arc, Mutex};` over separate lines for related imports
- Module declarations at the top of files should be in alphabetical order

#### Formatting
- Use standard Rust formatting (`rustfmt` if available)
- 4-space indentation
- Line length under 100 characters when practical
- Use trailing commas in multi-line arrays/structs

#### Naming Conventions
- `snake_case` for functions and variables
- `PascalCase` for types and structs
- `SCREAMING_SNAKE_CASE` for constants
- Module names use `snake_case`
- Function names should be descriptive and verbs (e.g., `parse_file`, `add_root_file`)

#### Types and Pattern Matching
- Use explicit type annotations for public APIs
- Prefer `Result<T, E>` over panic for error handling
- Use `match` statements exhaustively
- Use `Option<T>` for nullable values

#### Error Handling
- Custom error types with `Debug` derive
- Use `panic!` only for unrecoverable errors
- Return `Result` types from functions that can fail
- Include context in error messages with file names and locations

#### Unsafe Code
- Use unsafe blocks sparingly and only when necessary
- Add comments explaining why unsafe code is needed
- Prefer safe alternatives when available

#### Memory Management
- Use `Arc<Mutex<T>>` for shared mutable state
- Prefer references over cloning when possible
- Use `Box::into_raw()` and `Box::from_raw()` for FFI boundaries

### Luna Language Style

#### File Extensions
- Use `.luna` extension for Luna source files
- Test files go in `tests/` directory
- Example programs go in `examples/` directory

#### Syntax Patterns
- Functions: `func name(): return_type { }`
- Variables: `let name = value;`
- Struct definitions: `struct Name { field: type, }`
- Type annotations after colons: `name: type`

## Testing Guidelines

### Test Organization
- Unit tests for Rust code in `#[cfg(test)]` modules within source files
- Integration tests as `.luna` files in `tests/` directory
- Each test file should test a specific feature or language construct

### Test File Naming
- Use descriptive names: `expr_literals.luna`, `stmt_if.luna`, `decl_struct.luna`
- Prefix with category: `expr_` for expressions, `stmt_` for statements, `decl_` for declarations

### Test Structure
- Each test should have a `main()` function
- Use `assert()` for testing conditions
- Test should exercise the specific feature thoroughly
- Include positive and negative test cases where appropriate

## Development Workflow

### Adding New Features
1. Add AST nodes in `src/compiler/ast.rs` if needed
2. Update parser in `src/compiler/parser.rs`
3. Add type checking logic in `src/compiler/checker.rs`
4. Update IR generation if needed
5. Add runtime support if required
6. Write comprehensive tests
7. Run `./run_tests.sh` to verify all tests pass

### Debugging
- Use VS Code launch configuration (`lldb` launch) for debugging
- The executable is at `target/debug/luna-rs`
- Pass Luna file as argument to test specific functionality

### Dependencies
- Cranelift for JIT compilation
- Standard library for I/O and collections
- No external formatting/linting tools currently configured

## Architecture Notes

### Compiler Pipeline
1. **Tokenization** (`tokeniser.rs`) - Convert source to tokens
2. **Parsing** (`parser.rs`) - Build AST from tokens
3. **Type Checking** (`checker.rs`) - Validate types and semantics
4. **IR Generation** (`emit.rs`) - Generate intermediate representation
5. **JIT Compilation** (`runtime/`) - Compile to native code using Cranelift

### Key Design Patterns
- Arc<Mutex<>> for shared compiler state
- Result-based error handling throughout
- Separate phases in compilation pipeline
- Built-in functions registered in `builtins.rs`

### Performance Considerations
- JIT compilation for runtime performance
- Efficient memory management with garbage collection
- Minimal allocations in hot paths