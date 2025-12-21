//! jsh - A ZSH/Bash-compatible shell with enhanced scripting features
//!
//! jsh is a shell that combines compatibility with existing shell scripts
//! with modern, easy-to-understand syntax extensions for flow control.
//!
//! # Features
//!
//! ## Bash/ZSH Compatible
//! - Full support for common shell constructs
//! - If-then-else, for loops, while loops, case statements
//! - Variable expansion, command substitution
//! - Pipes, redirections, background jobs
//!
//! ## Enhanced Flow Control
//! - `match` expressions with pattern matching
//! - `loop` for infinite loops
//! - `let` and `const` bindings
//! - `try`/`catch`/`finally` error handling
//! - `fn` shorthand for functions
//!
//! # Example: Traditional Bash Style
//!
//! ```bash
//! # If-then-else
//! if [ "$x" -eq 1 ]; then
//!     echo "one"
//! elif [ "$x" -eq 2 ]; then
//!     echo "two"
//! else
//!     echo "other"
//! fi
//!
//! # For loop
//! for i in 1 2 3; do
//!     echo $i
//! done
//!
//! # Case statement
//! case "$x" in
//!     1) echo "one" ;;
//!     2|3) echo "two or three" ;;
//!     *) echo "other" ;;
//! esac
//! ```
//!
//! # Example: jsh Enhanced Syntax
//!
//! ```bash
//! # Match expression (Rust-like pattern matching)
//! match $x {
//!     1 => echo "one"
//!     2 | 3 => echo "two or three"
//!     4..10 => echo "between 4 and 10"
//!     /^hello/ => echo "starts with hello"
//!     * => echo "anything else"
//! }
//!
//! # Let bindings
//! let name = "world"
//! const PI = "3.14159"
//!
//! # Infinite loop with break
//! loop {
//!     read line
//!     if [ "$line" = "quit" ]; then
//!         break
//!     fi
//!     echo "You said: $line"
//! }
//!
//! # Try-catch-finally
//! try {
//!     risky_command
//! } catch err {
//!     echo "Error: $err"
//! } finally {
//!     cleanup
//! }
//!
//! # Function shorthand
//! fn greet {
//!     echo "Hello, $1!"
//! }
//! ```

pub mod argparse;
pub mod ast;
pub mod builtins;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod shell;
pub mod theme;
pub mod token;

pub use argparse::{ArgParser, ParsedArgs, ParseError};
pub use error::{JshError, Result};
pub use interpreter::{ExitStatus, Interpreter};
pub use shell::Shell;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

