//! Miscellaneous built-in commands: alias, hash, umask, ulimit, let, history, etc.

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};

/// alias - define or display aliases
pub fn builtin_alias(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement aliases
    Ok(ExitStatus::success())
}

/// unalias - remove aliases
pub fn builtin_unalias(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement aliases
    Ok(ExitStatus::success())
}

/// hash - remember or display command locations
pub fn builtin_hash(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement command hashing
    Ok(ExitStatus::success())
}

/// umask - set file creation mask
pub fn builtin_umask(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    use nix::sys::stat::{umask, Mode};

    if args.is_empty() {
        let current = umask(Mode::empty());
        umask(current);
        println!("{:04o}", current.bits());
    } else if let Ok(mask) = u32::from_str_radix(&args[0], 8) {
        umask(Mode::from_bits_truncate(mask));
    }

    Ok(ExitStatus::success())
}

/// ulimit - get/set resource limits
pub fn builtin_ulimit(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement resource limits
    println!("unlimited");
    Ok(ExitStatus::success())
}

/// times - display process times (stub, real implementation in posix.rs)
#[allow(dead_code)]
pub fn builtin_times(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // Stub - the real implementation is in posix::builtin_times_posix
    println!("0m0.000s 0m0.000s");
    println!("0m0.000s 0m0.000s");
    Ok(ExitStatus::success())
}

/// enable - enable/disable builtins
pub fn builtin_enable(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement enable
    Ok(ExitStatus::success())
}

/// shopt - set shell options
pub fn builtin_shopt(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement shell options
    Ok(ExitStatus::success())
}

/// let - evaluate arithmetic expression
pub fn builtin_let_arith(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    for expr in args {
        // Simple arithmetic evaluation
        let expr = expr.replace(' ', "");

        // Handle assignment
        if let Some(eq_pos) = expr.find('=') {
            let var = &expr[..eq_pos];
            let value_str = &expr[eq_pos + 1..];

            // Try to evaluate as number
            if let Ok(value) = value_str.parse::<i64>() {
                interp.set_var(var, &value.to_string());
            }
        }
    }

    Ok(ExitStatus::success())
}

/// history - display command history
pub fn builtin_history(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement history
    Ok(ExitStatus::success())
}

