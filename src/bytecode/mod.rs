//! Bytecode interpreter for Franken Shell
//!
//! Provides a stack-based virtual machine for faster script execution.
//! Scripts are compiled to bytecode once and can be executed multiple times
//! without re-parsing.
//!
//! ## Architecture
//!
//! ```text
//! Source -> Lexer -> Parser -> AST -> Compiler -> Bytecode -> VM
//! ```
//!
//! ## Performance
//!
//! The bytecode interpreter is faster than tree-walking because:
//! - No AST traversal overhead
//! - Cache-friendly linear bytecode
//! - Pre-computed jump targets
//! - Stack-based execution (no recursion)

mod compiler;
mod instruction;
mod vm;

pub use compiler::Compiler;
pub use instruction::{Chunk, Instruction, OpCode};
pub use vm::{VM, VMError, VMResult};

/// Bytecode module version
pub const BYTECODE_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_and_run_simple() {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile_string("x=5").unwrap();

        let mut vm = VM::new();
        let result = vm.run(&chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_and_run_arithmetic() {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile_string("echo $((2 + 3))").unwrap();

        let mut vm = VM::new();
        let result = vm.run(&chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_and_run_loop() {
        let mut compiler = Compiler::new();
        let chunk = compiler
            .compile_string(
                r#"
            for i in 1 2 3; do
                echo $i
            done
        "#,
            )
            .unwrap();

        let mut vm = VM::new();
        let result = vm.run(&chunk);
        assert!(result.is_ok());
    }
}
