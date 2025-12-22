//! Fish-compatible built-in commands: string, math, contains, status, functions, abbr

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::collections::HashMap;

// Thread-local abbreviations storage
thread_local! {
    pub(crate) static ABBREVIATIONS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

/// string - Fish-compatible string manipulation
pub fn builtin_string(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("string: missing subcommand");
        return Ok(ExitStatus::failure(1));
    }

    let subcommand = &args[0];
    let rest = &args[1..];

    match subcommand.as_str() {
        "length" => {
            if rest.is_empty() {
                eprintln!("string length: missing string");
                return Ok(ExitStatus::failure(1));
            }
            for s in rest {
                println!("{}", s.len());
            }
            Ok(ExitStatus::success())
        }
        "lower" => {
            for s in rest {
                println!("{}", s.to_lowercase());
            }
            Ok(ExitStatus::success())
        }
        "upper" => {
            for s in rest {
                println!("{}", s.to_uppercase());
            }
            Ok(ExitStatus::success())
        }
        "trim" => {
            for s in rest {
                println!("{}", s.trim());
            }
            Ok(ExitStatus::success())
        }
        "sub" | "substring" => {
            // string sub -s START -l LENGTH STRING
            let mut start: i64 = 1;
            let mut length: Option<usize> = None;
            let mut strings = Vec::new();
            let mut i = 0;

            while i < rest.len() {
                match rest[i].as_str() {
                    "-s" | "--start" => {
                        if i + 1 < rest.len() {
                            start = rest[i + 1].parse().unwrap_or(1);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "-l" | "--length" => {
                        if i + 1 < rest.len() {
                            length = rest[i + 1].parse().ok();
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        strings.push(&rest[i]);
                        i += 1;
                    }
                }
            }

            for s in strings {
                let chars: Vec<char> = s.chars().collect();
                let start_idx = if start < 0 {
                    (chars.len() as i64 + start).max(0) as usize
                } else {
                    (start - 1).max(0) as usize
                };

                let end_idx = if let Some(len) = length {
                    (start_idx + len).min(chars.len())
                } else {
                    chars.len()
                };

                let result: String = chars[start_idx..end_idx].iter().collect();
                println!("{}", result);
            }
            Ok(ExitStatus::success())
        }
        "split" => {
            // string split DELIMITER STRINGS...
            if rest.is_empty() {
                return Ok(ExitStatus::failure(1));
            }
            let delimiter = &rest[0];
            for s in &rest[1..] {
                for part in s.split(delimiter.as_str()) {
                    println!("{}", part);
                }
            }
            Ok(ExitStatus::success())
        }
        "join" => {
            // string join SEPARATOR STRINGS...
            if rest.is_empty() {
                return Ok(ExitStatus::failure(1));
            }
            let separator = &rest[0];
            let result = rest[1..].join(separator);
            println!("{}", result);
            Ok(ExitStatus::success())
        }
        "replace" => {
            // string replace [-a] PATTERN REPLACEMENT STRINGS...
            let mut all = false;
            let mut args_iter = rest.iter();
            let mut pattern = String::new();
            let mut replacement = String::new();
            let mut strings = Vec::new();

            while let Some(arg) = args_iter.next() {
                match arg.as_str() {
                    "-a" | "--all" => all = true,
                    _ if pattern.is_empty() => pattern = arg.clone(),
                    _ if replacement.is_empty() => replacement = arg.clone(),
                    _ => strings.push(arg.clone()),
                }
            }

            for s in &strings {
                let result = if all {
                    s.replace(&pattern, &replacement)
                } else {
                    s.replacen(&pattern, &replacement, 1)
                };
                println!("{}", result);
            }
            Ok(ExitStatus::success())
        }
        "match" => {
            // string match [-r] PATTERN STRING
            let mut regex_mode = false;
            let mut pattern = String::new();
            let mut strings = Vec::new();

            for arg in rest {
                match arg.as_str() {
                    "-r" | "--regex" => regex_mode = true,
                    _ if pattern.is_empty() => pattern = arg.clone(),
                    _ => strings.push(arg.clone()),
                }
            }

            let mut matched = false;
            for s in &strings {
                let is_match = if regex_mode {
                    regex::Regex::new(&pattern)
                        .map(|re| re.is_match(s))
                        .unwrap_or(false)
                } else {
                    // Glob-style matching
                    glob_match_pattern(&pattern, s)
                };

                if is_match {
                    println!("{}", s);
                    matched = true;
                }
            }

            Ok(if matched {
                ExitStatus::success()
            } else {
                ExitStatus::failure(1)
            })
        }
        "escape" => {
            for s in rest {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t");
                println!("{}", escaped);
            }
            Ok(ExitStatus::success())
        }
        "unescape" => {
            for s in rest {
                let unescaped = s
                    .replace("\\\\", "\x00")
                    .replace("\\'", "'")
                    .replace("\\\"", "\"")
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace('\x00', "\\");
                println!("{}", unescaped);
            }
            Ok(ExitStatus::success())
        }
        "repeat" => {
            // string repeat -n COUNT STRING
            let mut count: usize = 1;
            let mut strings = Vec::new();
            let mut i = 0;

            while i < rest.len() {
                match rest[i].as_str() {
                    "-n" | "--count" => {
                        if i + 1 < rest.len() {
                            count = rest[i + 1].parse().unwrap_or(1);
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    _ => {
                        strings.push(&rest[i]);
                        i += 1;
                    }
                }
            }

            for s in strings {
                println!("{}", s.repeat(count));
            }
            Ok(ExitStatus::success())
        }
        _ => {
            eprintln!("string: unknown subcommand '{}'", subcommand);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// Simple glob-style pattern matching for Fish string match
fn glob_match_pattern(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Match zero or more characters
                if pattern_chars.peek().is_none() {
                    return true;
                }
                let remaining_pattern: String = pattern_chars.collect();
                let mut remaining_text = String::new();
                while text_chars.peek().is_some() {
                    if glob_match_pattern(&remaining_pattern, &remaining_text) {
                        return true;
                    }
                    remaining_text.insert(0, text_chars.next().unwrap());
                }
                return glob_match_pattern(&remaining_pattern, &remaining_text);
            }
            '?' => {
                // Match exactly one character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    text_chars.peek().is_none()
}

/// contains - Fish-compatible contains check
pub fn builtin_contains(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.len() < 2 {
        return Ok(ExitStatus::failure(1));
    }

    let needle = &args[0];
    let haystack = &args[1..];

    for item in haystack {
        if item == needle {
            return Ok(ExitStatus::success());
        }
    }

    Ok(ExitStatus::failure(1))
}

/// status - Fish-compatible status information
pub fn builtin_status(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // Default: show if interactive
        if atty::is(atty::Stream::Stdin) {
            println!("This is an interactive fish session.");
        } else {
            println!("This is not an interactive session.");
        }
        return Ok(ExitStatus::success());
    }

    match args[0].as_str() {
        "is-interactive" | "--is-interactive" => {
            if atty::is(atty::Stream::Stdin) {
                Ok(ExitStatus::success())
            } else {
                Ok(ExitStatus::failure(1))
            }
        }
        "is-login" | "--is-login" => {
            // Check if this is a login shell
            Ok(ExitStatus::failure(1)) // Simplified - assume not login
        }
        "is-command-substitution" | "--is-command-substitution" => {
            Ok(ExitStatus::failure(1)) // Simplified
        }
        "filename" | "current-filename" | "--current-filename" => {
            println!("{}", interp.get_var("0").unwrap_or("jsh"));
            Ok(ExitStatus::success())
        }
        "function" | "current-function" | "--current-function" => {
            // Would need call stack to implement properly
            println!();
            Ok(ExitStatus::success())
        }
        "line-number" | "current-line-number" | "--current-line-number" => {
            println!("1"); // Simplified
            Ok(ExitStatus::success())
        }
        "fish-path" | "--fish-path" => {
            if let Ok(exe) = std::env::current_exe() {
                println!("{}", exe.display());
            }
            Ok(ExitStatus::success())
        }
        _ => {
            eprintln!("status: unknown subcommand '{}'", args[0]);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// functions - Fish-compatible function listing
pub fn builtin_functions(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // List all functions
        let mut names: Vec<_> = interp.functions.keys().collect();
        names.sort();
        for name in names {
            println!("{}", name);
        }
        return Ok(ExitStatus::success());
    }

    match args[0].as_str() {
        "-q" | "--query" => {
            // Check if function exists
            for name in &args[1..] {
                if !interp.functions.contains_key(name) {
                    return Ok(ExitStatus::failure(1));
                }
            }
            Ok(ExitStatus::success())
        }
        "-e" | "--erase" => {
            // Erase function
            for name in &args[1..] {
                interp.functions.remove(name);
            }
            Ok(ExitStatus::success())
        }
        "-n" | "--names" => {
            let mut names: Vec<_> = interp.functions.keys().collect();
            names.sort();
            for name in names {
                println!("{}", name);
            }
            Ok(ExitStatus::success())
        }
        name => {
            // Show function definition
            if let Some(func) = interp.functions.get(name) {
                println!("function {}", func.name);
                // Would need to pretty-print the body
                println!("end");
            } else {
                eprintln!("functions: Unknown function '{}'", name);
                return Ok(ExitStatus::failure(1));
            }
            Ok(ExitStatus::success())
        }
    }
}

/// abbr - Fish-compatible abbreviations
pub fn builtin_abbr(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // List all abbreviations
        ABBREVIATIONS.with(|abbrs| {
            let abbrs = abbrs.borrow();
            for (name, expansion) in abbrs.iter() {
                println!("abbr -a {} -- {}", name, expansion);
            }
        });
        return Ok(ExitStatus::success());
    }

    match args[0].as_str() {
        "-a" | "--add" => {
            if args.len() < 3 {
                eprintln!("abbr: --add requires name and expansion");
                return Ok(ExitStatus::failure(1));
            }
            let name = &args[1];
            let expansion = args[2..].join(" ");
            ABBREVIATIONS.with(|abbrs| {
                abbrs.borrow_mut().insert(name.clone(), expansion);
            });
            Ok(ExitStatus::success())
        }
        "-e" | "--erase" => {
            for name in &args[1..] {
                ABBREVIATIONS.with(|abbrs| {
                    abbrs.borrow_mut().remove(name);
                });
            }
            Ok(ExitStatus::success())
        }
        "-l" | "--list" => {
            ABBREVIATIONS.with(|abbrs| {
                let abbrs = abbrs.borrow();
                for name in abbrs.keys() {
                    println!("{}", name);
                }
            });
            Ok(ExitStatus::success())
        }
        "-s" | "--show" => {
            ABBREVIATIONS.with(|abbrs| {
                let abbrs = abbrs.borrow();
                for (name, expansion) in abbrs.iter() {
                    println!("abbr -a {} -- {}", name, expansion);
                }
            });
            Ok(ExitStatus::success())
        }
        _ => {
            // Treat as shorthand for --add
            if args.len() >= 2 {
                let name = &args[0];
                let expansion = args[1..].join(" ");
                ABBREVIATIONS.with(|abbrs| {
                    abbrs.borrow_mut().insert(name.clone(), expansion);
                });
            }
            Ok(ExitStatus::success())
        }
    }
}

/// math - Fish-compatible math evaluation
pub fn builtin_math(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("math: missing expression");
        return Ok(ExitStatus::failure(1));
    }

    let mut scale = 0;
    let mut expr_parts = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--scale" => {
                if i + 1 < args.len() {
                    scale = args[i + 1].parse().unwrap_or(0);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                expr_parts.push(args[i].clone());
                i += 1;
            }
        }
    }

    let expr = expr_parts.join(" ");

    // Simple expression evaluation
    // This is a simplified version - Fish's math is more powerful
    let result = evaluate_simple_math(&expr);

    match result {
        Ok(n) => {
            if scale > 0 {
                println!("{:.1$}", n, scale);
            } else if n.fract() == 0.0 {
                println!("{}", n as i64);
            } else {
                println!("{}", n);
            }
            Ok(ExitStatus::success())
        }
        Err(e) => {
            eprintln!("math: {}", e);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// Simple math expression evaluator
fn evaluate_simple_math(expr: &str) -> std::result::Result<f64, String> {
    let expr = expr.replace(' ', "");

    // Handle basic binary operations
    if let Some(pos) = expr.rfind('+') {
        if pos > 0 {
            let left = evaluate_simple_math(&expr[..pos])?;
            let right = evaluate_simple_math(&expr[pos + 1..])?;
            return Ok(left + right);
        }
    }

    if let Some(pos) = expr.rfind('-') {
        if pos > 0 && !matches!(expr.chars().nth(pos - 1), Some('*') | Some('/') | Some('^')) {
            let left = evaluate_simple_math(&expr[..pos])?;
            let right = evaluate_simple_math(&expr[pos + 1..])?;
            return Ok(left - right);
        }
    }

    if let Some(pos) = expr.rfind('*') {
        let left = evaluate_simple_math(&expr[..pos])?;
        let right = evaluate_simple_math(&expr[pos + 1..])?;
        return Ok(left * right);
    }

    if let Some(pos) = expr.rfind('/') {
        let left = evaluate_simple_math(&expr[..pos])?;
        let right = evaluate_simple_math(&expr[pos + 1..])?;
        if right == 0.0 {
            return Err("division by zero".to_string());
        }
        return Ok(left / right);
    }

    if let Some(pos) = expr.rfind('%') {
        let left = evaluate_simple_math(&expr[..pos])?;
        let right = evaluate_simple_math(&expr[pos + 1..])?;
        return Ok(left % right);
    }

    if let Some(pos) = expr.rfind('^') {
        let left = evaluate_simple_math(&expr[..pos])?;
        let right = evaluate_simple_math(&expr[pos + 1..])?;
        return Ok(left.powf(right));
    }

    // Handle parentheses
    if expr.starts_with('(') && expr.ends_with(')') {
        return evaluate_simple_math(&expr[1..expr.len() - 1]);
    }

    // Handle unary minus
    if expr.starts_with('-') {
        return Ok(-evaluate_simple_math(&expr[1..])?);
    }

    // Try to parse as number
    expr.parse::<f64>()
        .map_err(|_| format!("invalid number: {}", expr))
}

