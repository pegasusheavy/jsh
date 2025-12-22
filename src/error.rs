//! Error types for jsh shell

use thiserror::Error;

/// Result type for jsh operations
pub type Result<T> = std::result::Result<T, JshError>;

/// Main error type for jsh
#[derive(Error, Debug)]
pub enum JshError {
    #[error("Readline error: {0}")]
    Readline(#[from] rustyline::error::ReadlineError),

    #[error("Syntax error: {0}")]
    Syntax(String),

    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Arithmetic error: {0}")]
    Arithmetic(String),

    #[error("Exit with code {0}")]
    Exit(i32),

    #[error("Break from loop")]
    Break,

    #[error("Continue loop")]
    Continue,

    #[error("Return with value")]
    Return(Option<String>),
}

impl JshError {
    pub fn syntax(msg: impl Into<String>) -> Self {
        JshError::Syntax(msg.into())
    }

    pub fn parse(msg: impl Into<String>, line: usize, column: usize) -> Self {
        JshError::Parse {
            message: msg.into(),
            line,
            column,
        }
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        JshError::Runtime(msg.into())
    }

    pub fn type_error(msg: impl Into<String>) -> Self {
        JshError::Type(msg.into())
    }
}

