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

/// ssh_agent - control ssh-agent integration
///
/// Usage:
///   ssh_agent start   - Start ssh-agent if not running
///   ssh_agent stop    - Stop the current ssh-agent
///   ssh_agent status  - Show ssh-agent status
///   ssh_agent add     - Add default SSH keys to the agent
pub fn builtin_ssh_agent(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    use std::process::{Command, Stdio};
    use std::path::PathBuf;

    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("status");

    match subcommand {
        "start" => {
            // Check if already running
            if let Some(sock) = interp.get_var("SSH_AUTH_SOCK") {
                let sock_path = PathBuf::from(&sock);
                if sock_path.exists() {
                    println!("ssh-agent is already running (socket: {})", sock);
                    return Ok(ExitStatus::success());
                }
            }

            // Start ssh-agent
            let output = Command::new("ssh-agent")
                .arg("-s")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    for line in stdout.lines() {
                        if line.starts_with("SSH_AUTH_SOCK=") {
                            if let Some(sock) = line
                                .strip_prefix("SSH_AUTH_SOCK=")
                                .and_then(|s| s.strip_suffix("; export SSH_AUTH_SOCK;"))
                            {
                                // SAFETY: Setting env var in user command context
                                unsafe { std::env::set_var("SSH_AUTH_SOCK", sock) };
                                interp.set_var("SSH_AUTH_SOCK", sock);
                                interp.export_var("SSH_AUTH_SOCK", Some(sock));
                            }
                        } else if line.starts_with("SSH_AGENT_PID=") {
                            if let Some(pid) = line
                                .strip_prefix("SSH_AGENT_PID=")
                                .and_then(|s| s.strip_suffix("; export SSH_AGENT_PID;"))
                            {
                                // SAFETY: Setting env var in user command context
                                unsafe { std::env::set_var("SSH_AGENT_PID", pid) };
                                interp.set_var("SSH_AGENT_PID", pid);
                                interp.export_var("SSH_AGENT_PID", Some(pid));
                            }
                        }
                    }

                    if let Some(pid) = interp.get_var("SSH_AGENT_PID") {
                        println!("ssh-agent started (pid {})", pid);
                    }
                    Ok(ExitStatus::success())
                }
                _ => {
                    eprintln!("franken: ssh_agent: failed to start ssh-agent");
                    Ok(ExitStatus::failure(1))
                }
            }
        }

        "stop" => {
            if let Some(pid) = interp.get_var("SSH_AGENT_PID").map(|s| s.to_string()) {
                let _ = Command::new("kill")
                    .arg(&pid)
                    .status();

                // SAFETY: Clearing env vars
                unsafe {
                    std::env::remove_var("SSH_AUTH_SOCK");
                    std::env::remove_var("SSH_AGENT_PID");
                }
                interp.vars.remove("SSH_AUTH_SOCK");
                interp.vars.remove("SSH_AGENT_PID");
                interp.exports.remove("SSH_AUTH_SOCK");
                interp.exports.remove("SSH_AGENT_PID");

                println!("ssh-agent stopped (pid {})", pid);
                Ok(ExitStatus::success())
            } else {
                eprintln!("franken: ssh_agent: no ssh-agent running");
                Ok(ExitStatus::failure(1))
            }
        }

        "status" => {
            if let Some(sock) = interp.get_var("SSH_AUTH_SOCK") {
                let sock_path = PathBuf::from(&sock);
                if sock_path.exists() {
                    println!("ssh-agent is running");
                    println!("  SSH_AUTH_SOCK={}", sock);
                    if let Some(pid) = interp.get_var("SSH_AGENT_PID") {
                        println!("  SSH_AGENT_PID={}", pid);
                    }

                    // List identities
                    let output = Command::new("ssh-add")
                        .arg("-l")
                        .output();

                    if let Ok(output) = output {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            println!("  Loaded keys:");
                            for line in stdout.lines() {
                                println!("    {}", line);
                            }
                        } else {
                            println!("  No keys loaded");
                        }
                    }
                    return Ok(ExitStatus::success());
                }
            }
            println!("ssh-agent is not running");
            Ok(ExitStatus::failure(1))
        }

        "add" => {
            // Check if agent is running
            if interp.get_var("SSH_AUTH_SOCK").is_none() {
                eprintln!("franken: ssh_agent: ssh-agent is not running");
                return Ok(ExitStatus::failure(1));
            }

            // Add default keys
            let status = Command::new("ssh-add")
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("Keys added to ssh-agent");
                    Ok(ExitStatus::success())
                }
                _ => {
                    eprintln!("franken: ssh_agent: failed to add keys");
                    Ok(ExitStatus::failure(1))
                }
            }
        }

        _ => {
            eprintln!("Usage: ssh_agent [start|stop|status|add]");
            eprintln!("  start  - Start ssh-agent if not running");
            eprintln!("  stop   - Stop the current ssh-agent");
            eprintln!("  status - Show ssh-agent status (default)");
            eprintln!("  add    - Add default SSH keys to the agent");
            Ok(ExitStatus::failure(1))
        }
    }
}

