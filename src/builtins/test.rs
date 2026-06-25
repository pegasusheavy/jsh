//! Test built-in commands: test, [

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};

/// test - evaluate conditional expression
pub fn builtin_test(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    // Handle [ ... ] form
    let args: Vec<&str> = if !args.is_empty() && args.last() == Some(&"]".to_string()) {
        args[..args.len() - 1].iter().map(|s| s.as_str()).collect()
    } else {
        args.iter().map(|s| s.as_str()).collect()
    };

    if args.is_empty() {
        return Ok(ExitStatus::failure(1));
    }

    let result = evaluate_test(&args, interp)?;
    Ok(if result {
        ExitStatus::success()
    } else {
        ExitStatus::failure(1)
    })
}

/// Evaluate test expression
pub fn evaluate_test(args: &[&str], interp: &Interpreter) -> Result<bool> {
    if args.is_empty() {
        return Ok(false);
    }

    // Unary operators
    if args.len() >= 2 {
        match args[0] {
            "!" => return Ok(!evaluate_test(&args[1..], interp)?),
            "-z" => return Ok(args[1].is_empty()),
            "-n" => return Ok(!args[1].is_empty()),
            "-e" => return Ok(std::path::Path::new(args[1]).exists()),
            "-f" => return Ok(std::path::Path::new(args[1]).is_file()),
            "-d" => return Ok(std::path::Path::new(args[1]).is_dir()),
            "-r" => {
                // Check readable - simplified
                return Ok(std::path::Path::new(args[1]).exists());
            }
            "-w" => {
                // Check writable - simplified
                return Ok(std::path::Path::new(args[1]).exists());
            }
            "-x" => {
                // Check executable - simplified
                return Ok(std::path::Path::new(args[1]).exists());
            }
            "-s" => {
                // Check non-empty file
                return Ok(std::fs::metadata(args[1])
                    .map(|m| m.len() > 0)
                    .unwrap_or(false));
            }
            "-L" | "-h" => {
                return Ok(std::fs::symlink_metadata(args[1])
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false));
            }
            "-v" => {
                return Ok(interp.get_var(args[1]).is_some());
            }
            "-b" => {
                // Block special file
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    return Ok(std::fs::metadata(args[1])
                        .map(|m| m.file_type().is_block_device())
                        .unwrap_or(false));
                }
                #[cfg(not(unix))]
                return Ok(false);
            }
            "-c" => {
                // Character special file
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    return Ok(std::fs::metadata(args[1])
                        .map(|m| m.file_type().is_char_device())
                        .unwrap_or(false));
                }
                #[cfg(not(unix))]
                return Ok(false);
            }
            "-p" => {
                // Named pipe (FIFO)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    return Ok(std::fs::metadata(args[1])
                        .map(|m| m.file_type().is_fifo())
                        .unwrap_or(false));
                }
                #[cfg(not(unix))]
                return Ok(false);
            }
            "-S" => {
                // Socket
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    return Ok(std::fs::metadata(args[1])
                        .map(|m| m.file_type().is_socket())
                        .unwrap_or(false));
                }
                #[cfg(not(unix))]
                return Ok(false);
            }
            "-t" => {
                // File descriptor is a terminal
                if let Ok(fd) = args[1].parse::<i32>() {
                    return Ok(atty::is(match fd {
                        0 => atty::Stream::Stdin,
                        1 => atty::Stream::Stdout,
                        2 => atty::Stream::Stderr,
                        _ => return Ok(false),
                    }));
                }
                return Ok(false);
            }
            _ => {}
        }
    }

    // Binary operators
    if args.len() >= 3 {
        let left = args[0];
        let op = args[1];
        let right = args[2];

        match op {
            "=" | "==" => return Ok(left == right),
            "!=" => return Ok(left != right),
            "-eq" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l == r);
            }
            "-ne" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l != r);
            }
            "-lt" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l < r);
            }
            "-le" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l <= r);
            }
            "-gt" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l > r);
            }
            "-ge" => {
                let l: i64 = left.parse().unwrap_or(0);
                let r: i64 = right.parse().unwrap_or(0);
                return Ok(l >= r);
            }
            "<" => return Ok(left < right),
            ">" => return Ok(left > right),
            "-nt" => {
                // Newer than
                let left_time = std::fs::metadata(left).and_then(|m| m.modified()).ok();
                let right_time = std::fs::metadata(right).and_then(|m| m.modified()).ok();
                return Ok(left_time > right_time);
            }
            "-ot" => {
                // Older than
                let left_time = std::fs::metadata(left).and_then(|m| m.modified()).ok();
                let right_time = std::fs::metadata(right).and_then(|m| m.modified()).ok();
                return Ok(left_time < right_time);
            }
            "-ef" => {
                // Same file (same device and inode)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    let left_meta = std::fs::metadata(left).ok();
                    let right_meta = std::fs::metadata(right).ok();
                    return Ok(match (left_meta, right_meta) {
                        (Some(l), Some(r)) => l.dev() == r.dev() && l.ino() == r.ino(),
                        _ => false,
                    });
                }
                #[cfg(not(unix))]
                return Ok(false);
            }
            "-a" => {
                // And
                let left_result = evaluate_test(&[left], interp)?;
                let right_result = evaluate_test(&[right], interp)?;
                return Ok(left_result && right_result);
            }
            "-o" => {
                // Or
                let left_result = evaluate_test(&[left], interp)?;
                let right_result = evaluate_test(&[right], interp)?;
                return Ok(left_result || right_result);
            }
            "=~" => {
                // Regex match
                return Ok(regex::Regex::new(right)
                    .map(|re| re.is_match(left))
                    .unwrap_or(false));
            }
            _ => {}
        }
    }

    // Single argument - true if non-empty
    if args.len() == 1 {
        return Ok(!args[0].is_empty());
    }

    Ok(false)
}
