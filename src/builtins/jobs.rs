//! Job control built-in commands: jobs, fg, bg, wait

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};

/// jobs - list background jobs
pub fn builtin_jobs(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement job control
    Ok(ExitStatus::success())
}

/// fg - bring job to foreground
pub fn builtin_fg(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    eprintln!("franken: fg: job control not implemented");
    Ok(ExitStatus::failure(1))
}

/// bg - send job to background
pub fn builtin_bg(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    eprintln!("franken: bg: job control not implemented");
    Ok(ExitStatus::failure(1))
}

/// wait - wait for background jobs
pub fn builtin_wait(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Wait for background processes
    Ok(ExitStatus::success())
}

