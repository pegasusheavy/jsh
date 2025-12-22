//! Script execution built-in commands: source, eval, exec, trap, shift, read, getopts

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::io::{self, Read};

/// source - execute commands from file
pub fn builtin_source(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("jsh: source: filename argument required");
        return Ok(ExitStatus::failure(1));
    }

    interp.run_script(&args[0])
}

/// eval - evaluate string as command
pub fn builtin_eval(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let command = args.join(" ");
    interp.execute_string(&command)
}

/// exec - replace shell with command
pub fn builtin_exec(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        return Ok(ExitStatus::success());
    }

    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
    eprintln!("jsh: exec: {}: {}", args[0], err);
    Ok(ExitStatus::failure(126))
}

/// trap - set signal handlers
pub fn builtin_trap(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement signal trapping
    Ok(ExitStatus::success())
}

/// shift - shift positional parameters
pub fn builtin_shift(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let n = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    if n <= interp.positional_params.len() {
        interp.positional_params = interp.positional_params[n..].to_vec();
        Ok(ExitStatus::success())
    } else {
        eprintln!("jsh: shift: shift count out of range");
        Ok(ExitStatus::failure(1))
    }
}

/// read - read a line from input
pub fn builtin_read(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut prompt = String::new();
    let mut silent = false;
    let mut nchars: Option<usize> = None;
    let mut delimiter = '\n';
    let mut var_names: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                if i + 1 < args.len() {
                    prompt = args[i + 1].clone();
                    i += 1;
                }
            }
            "-s" => silent = true,
            "-n" => {
                if i + 1 < args.len() {
                    nchars = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "-d" => {
                if i + 1 < args.len() {
                    delimiter = args[i + 1].chars().next().unwrap_or('\n');
                    i += 1;
                }
            }
            arg if !arg.starts_with('-') => {
                var_names.push(arg);
            }
            _ => {}
        }
        i += 1;
    }

    // Print prompt if specified
    if !prompt.is_empty() && !silent {
        eprint!("{}", prompt);
        use std::io::Write;
        std::io::stderr().flush().ok();
    }

    let mut input = String::new();

    if let Some(n) = nchars {
        // Read exactly n characters
        let mut buf = vec![0u8; n];
        match io::stdin().read_exact(&mut buf) {
            Ok(_) => {
                input = String::from_utf8_lossy(&buf).to_string();
            }
            Err(_) => return Ok(ExitStatus::failure(1)),
        }
    } else {
        // Read until delimiter
        let stdin = io::stdin();
        for byte in stdin.bytes() {
            match byte {
                Ok(b) if b as char == delimiter => break,
                Ok(b) => input.push(b as char),
                Err(_) => return Ok(ExitStatus::failure(1)),
            }
        }
    }

    // Remove trailing newline if present
    let input = input.trim_end_matches('\n').trim_end_matches('\r');

    if var_names.is_empty() {
        // Default to REPLY
        interp.set_var("REPLY", input);
    } else if var_names.len() == 1 {
        interp.set_var(var_names[0], input);
    } else {
        // Split input by IFS and assign to multiple variables
        let ifs = interp.get_var("IFS").unwrap_or(" \t\n").to_string();
        let parts: Vec<&str> = input.split(|c| ifs.contains(c)).collect();

        for (i, var) in var_names.iter().enumerate() {
            if i < parts.len() - 1 {
                interp.set_var(var, parts[i]);
            } else {
                // Last variable gets all remaining
                interp.set_var(var, &parts[i..].join(" "));
                break;
            }
        }
    }

    if input.is_empty() && nchars.is_none() {
        Ok(ExitStatus::failure(1)) // EOF
    } else {
        Ok(ExitStatus::success())
    }
}

/// getopts - parse positional parameters
pub fn builtin_getopts(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement getopts
    Ok(ExitStatus::failure(1))
}

