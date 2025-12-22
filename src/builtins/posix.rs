//! POSIX-required built-in commands

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use nix::libc;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::str::FromStr;

/// kill - send signal to process (POSIX required)
pub fn builtin_kill(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("kill: usage: kill [-s sigspec | -n signum | -sigspec] pid | jobspec ... or kill -l [sigspec]");
        return Ok(ExitStatus::failure(1));
    }

    let mut signal = Signal::SIGTERM;
    let mut list_signals = false;
    let mut pids = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "-l" || arg == "-L" {
            list_signals = true;
            i += 1;
            // If there's a number after -l, show signal name for that number
            if i < args.len() {
                if let Ok(num) = args[i].parse::<i32>() {
                    if let Ok(sig) = Signal::try_from(num) {
                        println!("{}", format!("{:?}", sig).strip_prefix("SIG").unwrap_or(&format!("{:?}", sig)));
                    }
                    return Ok(ExitStatus::success());
                }
            }
        } else if arg == "-s" {
            i += 1;
            if i < args.len() {
                signal = parse_signal(&args[i])?;
            }
        } else if arg == "-n" {
            i += 1;
            if i < args.len() {
                if let Ok(num) = args[i].parse::<i32>() {
                    signal = Signal::try_from(num).map_err(|_| {
                        crate::error::JshError::InvalidArgument(format!("invalid signal: {}", num))
                    })?;
                }
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            // -SIGNAME or -signum
            let sig_part = &arg[1..];
            if let Ok(num) = sig_part.parse::<i32>() {
                signal = Signal::try_from(num).map_err(|_| {
                    crate::error::JshError::InvalidArgument(format!("invalid signal: {}", num))
                })?;
            } else {
                signal = parse_signal(sig_part)?;
            }
        } else {
            // PID or job spec
            if let Ok(pid) = arg.parse::<i32>() {
                pids.push(pid);
            } else if arg.starts_with('%') {
                // Job spec - not implemented
                eprintln!("kill: job control not implemented");
            }
        }
        i += 1;
    }

    if list_signals {
        // List all signal names
        let signals = [
            "HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "BUS", "FPE",
            "KILL", "USR1", "SEGV", "USR2", "PIPE", "ALRM", "TERM",
            "STKFLT", "CHLD", "CONT", "STOP", "TSTP", "TTIN", "TTOU",
            "URG", "XCPU", "XFSZ", "VTALRM", "PROF", "WINCH", "IO",
            "PWR", "SYS",
        ];
        for (i, name) in signals.iter().enumerate() {
            print!("{:2}) SIG{:<8}", i + 1, name);
            if (i + 1) % 5 == 0 {
                println!();
            }
        }
        println!();
        return Ok(ExitStatus::success());
    }

    if pids.is_empty() {
        eprintln!("kill: no process ID specified");
        return Ok(ExitStatus::failure(1));
    }

    let mut status = ExitStatus::success();
    for pid in pids {
        match signal::kill(Pid::from_raw(pid), signal) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("kill: ({}) - {}", pid, e);
                status = ExitStatus::failure(1);
            }
        }
    }

    Ok(status)
}

/// Parse a signal name or number
fn parse_signal(s: &str) -> Result<Signal> {
    // Try as number first
    if let Ok(num) = s.parse::<i32>() {
        return Signal::try_from(num).map_err(|_| {
            crate::error::JshError::InvalidArgument(format!("invalid signal: {}", s))
        });
    }

    // Try as name (with or without SIG prefix)
    let name = s.to_uppercase();
    let name = if name.starts_with("SIG") {
        name
    } else {
        format!("SIG{}", name)
    };

    Signal::from_str(&name).map_err(|_| {
        crate::error::JshError::InvalidArgument(format!("invalid signal: {}", s))
    })
}

/// fc - process the command history (POSIX required)
/// This is a minimal implementation
pub fn builtin_fc(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut list_mode = false;
    let mut reverse = false;
    let mut suppress_numbers = false;
    let mut editor = interp.get_var("FCEDIT")
        .or_else(|| interp.get_var("EDITOR"))
        .unwrap_or("vi")
        .to_string();
    let mut first: Option<String> = None;
    let mut last: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                match c {
                    'l' => list_mode = true,
                    'n' => suppress_numbers = true,
                    'r' => reverse = true,
                    's' => {
                        // Execute command directly (re-execute)
                        // fc -s [old=new] [command]
                        // Minimal implementation - just show message
                        eprintln!("fc: -s not fully implemented");
                        return Ok(ExitStatus::failure(1));
                    }
                    'e' => {
                        i += 1;
                        if i < args.len() {
                            editor = args[i].clone();
                        }
                    }
                    _ => {}
                }
            }
        } else if first.is_none() {
            first = Some(arg.clone());
        } else {
            last = Some(arg.clone());
        }
        i += 1;
    }

    if list_mode {
        // List history entries
        // This would require access to the shell's history
        // For now, show a placeholder message
        if suppress_numbers {
            println!("(history listing without numbers - requires history integration)");
        } else {
            println!("(history listing - requires history integration)");
        }
        if reverse {
            println!("(would be in reverse order)");
        }
        return Ok(ExitStatus::success());
    }

    // Edit mode - would normally invoke editor on history
    eprintln!("fc: edit mode requires history integration (would use {})", editor);
    let _ = (first, last); // Suppress unused warnings

    Ok(ExitStatus::failure(1))
}

/// colon (:) - null command, always succeeds (POSIX special builtin)
/// Note: Already implemented as alias for true in control.rs
#[allow(dead_code)]
pub fn builtin_colon(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    Ok(ExitStatus::success())
}

/// dot (.) - source a file (POSIX special builtin)
/// Note: Already implemented as builtin_source in scripting.rs
#[allow(dead_code)]
pub fn builtin_dot(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    crate::builtins::scripting::builtin_source(args, interp)
}

/// times - print accumulated user and system times (POSIX special builtin)
pub fn builtin_times_posix(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // Get process times using libc
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;

        let mut tms = MaybeUninit::<libc::tms>::uninit();
        let clock_tick = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;

        if unsafe { libc::times(tms.as_mut_ptr()) } != -1 {
            let tms = unsafe { tms.assume_init() };
            let user_time = tms.tms_utime as f64 / clock_tick;
            let sys_time = tms.tms_stime as f64 / clock_tick;
            let child_user = tms.tms_cutime as f64 / clock_tick;
            let child_sys = tms.tms_cstime as f64 / clock_tick;

            // Shell times
            let user_min = (user_time / 60.0) as u64;
            let user_sec = user_time % 60.0;
            let sys_min = (sys_time / 60.0) as u64;
            let sys_sec = sys_time % 60.0;
            println!("{}m{:.3}s {}m{:.3}s", user_min, user_sec, sys_min, sys_sec);

            // Children times
            let cuser_min = (child_user / 60.0) as u64;
            let cuser_sec = child_user % 60.0;
            let csys_min = (child_sys / 60.0) as u64;
            let csys_sec = child_sys % 60.0;
            println!("{}m{:.3}s {}m{:.3}s", cuser_min, cuser_sec, csys_min, csys_sec);
        } else {
            println!("0m0.000s 0m0.000s");
            println!("0m0.000s 0m0.000s");
        }
    }

    #[cfg(not(unix))]
    {
        println!("0m0.000s 0m0.000s");
        println!("0m0.000s 0m0.000s");
    }

    Ok(ExitStatus::success())
}

/// newgrp - change to a new group (POSIX)
/// Note: This typically needs to exec a new shell, so it's a placeholder
pub fn builtin_newgrp(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("newgrp: usage: newgrp [-] [group]");
        return Ok(ExitStatus::failure(1));
    }

    // newgrp needs to be an external command as it needs to set the group ID
    // and exec a new shell. This is a placeholder.
    eprintln!("newgrp: must be run as external command (use /usr/bin/newgrp)");
    Ok(ExitStatus::failure(1))
}

