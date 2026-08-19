//! 🧟 Franken Shell - The vibe-coded shell that does everything stupidly™
//!
//! A monstrous creation stitched together from the best parts of Bash, ZSH,
//! Fish, and POSIX shells — held together with duct tape, good intentions,
//! and an alarming amount of caffeine.
//!
//! # Features
//!
//! ## Bash/ZSH Compatible (Ish)
//! - Full support for common shell constructs
//! - If-then-else, for loops, while loops, case statements
//! - Variable expansion, command substitution
//! - Pipes, redirections, background jobs
//!
//! ## Enhanced Flow Control (The Fun Stuff)
//! - `match` expressions with pattern matching
//! - `loop` for infinite loops
//! - `let` and `const` bindings
//! - `try`/`catch`/`finally` error handling
//! - `fn` shorthand for functions with named parameters
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
//! # Example: Franken Shell's Weird Syntax
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
//! # Function shorthand with named parameters
//! fn greet(name, greeting="Hello") {
//!     echo "$greeting, $name!"
//! }
//! ```

pub mod arena;
pub mod argparse;
pub mod ast;
pub mod builtins;
pub mod bytecode;
pub mod error;
pub mod intern;
pub mod interpreter;
pub mod jit;
pub mod lexer;
pub mod parser;
pub mod plugins;
pub mod shell;
pub mod theme;
pub mod tmux;
pub mod token;

pub use argparse::{ArgParser, ParseError, ParsedArgs};
pub use error::{FrankenError, Result};
pub use interpreter::{ExitStatus, Interpreter};
pub use shell::Shell;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Shell name
pub const SHELL_NAME: &str = "fsh";

/// Shell tagline
pub const TAGLINE: &str = "The vibe-coded shell that does everything stupidly™";
