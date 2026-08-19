//! Control flow built-in commands: exit, return, true, false

use crate::error::{JshError, Result};
use crate::interpreter::{ExitStatus, Interpreter};

/// exit - exit the shell
pub fn builtin_exit(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    Err(JshError::Exit(code))
}

/// return - return from function
pub fn builtin_return(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let value = args.first().cloned();
    Err(JshError::Return(value))
}

/// true - return success
pub fn builtin_true(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    Ok(ExitStatus::success())
}

/// false - return failure
pub fn builtin_false(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    Ok(ExitStatus::failure(1))
}
