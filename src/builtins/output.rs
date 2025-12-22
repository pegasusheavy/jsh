//! Output built-in commands: echo, printf

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::io::{self, Write};

/// echo - print arguments
pub fn builtin_echo(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut newline = true;
    let mut interpret_escapes = false;
    let mut start = 0;

    // Parse options
    for (i, arg) in args.iter().enumerate() {
        if arg == "-n" {
            newline = false;
            start = i + 1;
        } else if arg == "-e" {
            interpret_escapes = true;
            start = i + 1;
        } else if arg == "-E" {
            interpret_escapes = false;
            start = i + 1;
        } else if arg.starts_with('-') && arg.chars().skip(1).all(|c| "neE".contains(c)) {
            if arg.contains('n') {
                newline = false;
            }
            if arg.contains('e') {
                interpret_escapes = true;
            }
            if arg.contains('E') {
                interpret_escapes = false;
            }
            start = i + 1;
        } else {
            break;
        }
    }

    let output = args[start..].join(" ");

    let output = if interpret_escapes {
        interpret_escape_sequences(&output)
    } else {
        output
    };

    print!("{}", output);
    if newline {
        println!();
    }
    io::stdout().flush()?;
    Ok(ExitStatus::success())
}

/// Interpret escape sequences
pub fn interpret_escape_sequences(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                let escaped = match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    'a' => '\x07',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'v' => '\x0b',
                    '\\' => '\\',
                    'e' | 'E' => '\x1b',
                    '0' => {
                        // Octal
                        chars.next();
                        let mut val = 0u32;
                        for _ in 0..3 {
                            if let Some(&c @ '0'..='7') = chars.peek() {
                                val = val * 8 + (c as u32 - '0' as u32);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        result.push(char::from_u32(val).unwrap_or('\0'));
                        continue;
                    }
                    'x' => {
                        // Hex
                        chars.next();
                        let mut val = 0u32;
                        for _ in 0..2 {
                            if let Some(c) = chars.peek().and_then(|c| c.to_digit(16)) {
                                val = val * 16 + c;
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        result.push(char::from_u32(val).unwrap_or('\0'));
                        continue;
                    }
                    _ => {
                        result.push('\\');
                        continue;
                    }
                };
                chars.next();
                result.push(escaped);
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// printf - formatted output
pub fn builtin_printf(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("jsh: printf: usage: printf format [arguments]");
        return Ok(ExitStatus::failure(1));
    }

    let format = &args[0];
    let mut arg_idx = 1;
    let mut chars = format.chars().peekable();
    let mut output = String::new();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    '%' => {
                        output.push('%');
                        chars.next();
                    }
                    's' => {
                        chars.next();
                        if arg_idx < args.len() {
                            output.push_str(&args[arg_idx]);
                            arg_idx += 1;
                        }
                    }
                    'd' | 'i' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Ok(n) = args[arg_idx].parse::<i64>() {
                                output.push_str(&n.to_string());
                            } else {
                                output.push('0');
                            }
                            arg_idx += 1;
                        }
                    }
                    'f' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Ok(n) = args[arg_idx].parse::<f64>() {
                                output.push_str(&format!("{:.6}", n));
                            } else {
                                output.push_str("0.000000");
                            }
                            arg_idx += 1;
                        }
                    }
                    'c' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Some(c) = args[arg_idx].chars().next() {
                                output.push(c);
                            }
                            arg_idx += 1;
                        }
                    }
                    'x' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Ok(n) = args[arg_idx].parse::<i64>() {
                                output.push_str(&format!("{:x}", n));
                            } else {
                                output.push('0');
                            }
                            arg_idx += 1;
                        }
                    }
                    'X' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Ok(n) = args[arg_idx].parse::<i64>() {
                                output.push_str(&format!("{:X}", n));
                            } else {
                                output.push('0');
                            }
                            arg_idx += 1;
                        }
                    }
                    'o' => {
                        chars.next();
                        if arg_idx < args.len() {
                            if let Ok(n) = args[arg_idx].parse::<i64>() {
                                output.push_str(&format!("{:o}", n));
                            } else {
                                output.push('0');
                            }
                            arg_idx += 1;
                        }
                    }
                    _ => {
                        output.push('%');
                    }
                }
            }
        } else if c == '\\' {
            if let Some(&next) = chars.peek() {
                let escaped = match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    _ => {
                        output.push('\\');
                        continue;
                    }
                };
                chars.next();
                output.push(escaped);
            }
        } else {
            output.push(c);
        }
    }

    print!("{}", output);
    io::stdout().flush()?;
    Ok(ExitStatus::success())
}

