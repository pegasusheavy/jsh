//! Navigation built-in commands: cd, pwd, pushd, popd, dirs

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::env;
use std::path::PathBuf;

// Directory stack for pushd/popd
thread_local! {
    pub(crate) static DIR_STACK: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// cd - change directory
pub fn builtin_cd(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let path = if args.is_empty() {
        interp
            .get_var("HOME")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "/".to_string())
    } else if args[0] == "-" {
        interp
            .get_var("OLDPWD")
            .map(|s| s.to_string())
            .unwrap_or_else(|| interp.cwd.to_string_lossy().to_string())
    } else {
        args[0].clone()
    };

    let new_path = if path.starts_with('/') {
        PathBuf::from(&path)
    } else if path.starts_with('~') {
        let home = interp.get_var("HOME").unwrap_or("/");
        PathBuf::from(path.replacen('~', home, 1))
    } else {
        interp.cwd.join(&path)
    };

    match new_path.canonicalize() {
        Ok(canonical) => {
            let old_pwd = interp.cwd.to_string_lossy().to_string();
            interp.set_var("OLDPWD", &old_pwd);
            interp.cwd = canonical.clone();
            interp.set_var("PWD", &canonical.to_string_lossy());
            env::set_current_dir(&canonical)?;
            Ok(ExitStatus::success())
        }
        Err(e) => {
            eprintln!("franken: cd: {}: {}", path, e);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// pwd - print working directory
pub fn builtin_pwd(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    println!("{}", interp.cwd.display());
    Ok(ExitStatus::success())
}

/// pushd - push directory onto stack
pub fn builtin_pushd(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let current = interp.cwd.clone();

    if args.is_empty() {
        // Swap with top of stack
        DIR_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(top) = stack.pop() {
                stack.push(current);
                builtin_cd(&[top.to_string_lossy().to_string()], interp)
            } else {
                eprintln!("franken: pushd: no other directory");
                Ok(ExitStatus::failure(1))
            }
        })
    } else {
        DIR_STACK.with(|stack| {
            stack.borrow_mut().push(current);
        });
        builtin_cd(args, interp)
    }
}

/// popd - pop directory from stack
pub fn builtin_popd(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    DIR_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(dir) = stack.pop() {
            builtin_cd(&[dir.to_string_lossy().to_string()], interp)
        } else {
            eprintln!("franken: popd: directory stack empty");
            Ok(ExitStatus::failure(1))
        }
    })
}

/// dirs - display directory stack
pub fn builtin_dirs(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    print!("{}", interp.cwd.display());
    DIR_STACK.with(|stack| {
        for dir in stack.borrow().iter().rev() {
            print!(" {}", dir.display());
        }
    });
    println!();
    Ok(ExitStatus::success())
}
