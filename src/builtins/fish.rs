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
            let args_iter = rest.iter();
            let mut pattern = String::new();
            let mut replacement = String::new();
            let mut strings = Vec::new();

            for arg in args_iter {
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
/// Uses iterative approach to avoid stack overflow and infinite loops
fn glob_match_pattern(pattern: &str, text: &str) -> bool {
    glob_match_helper(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_helper(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0; // pattern index
    let mut ti = 0; // text index
    let mut star_pi = None; // position in pattern after last *
    let mut star_ti = None; // position in text when we hit last *

    while ti < text.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == text[ti]) {
            // Match single character or ?
            pi += 1;
            ti += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            // Found *, save state and try to match with zero chars
            star_pi = Some(pi);
            star_ti = Some(ti);
            pi += 1;
        } else if let Some(spi) = star_pi {
            // No match, but we have a * to backtrack to
            pi = spi + 1;
            star_ti = Some(star_ti.unwrap() + 1);
            ti = star_ti.unwrap();
        } else {
            // No match possible
            return false;
        }
    }

    // Check remaining pattern (should only be *s)
    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
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
            println!("{}", interp.get_var("0").unwrap_or("fsh"));
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

/// Simple math expression evaluator with depth limit to prevent stack overflow
fn evaluate_simple_math(expr: &str) -> std::result::Result<f64, String> {
    evaluate_math_with_depth(expr, 0)
}

const MAX_MATH_DEPTH: usize = 50;

fn evaluate_math_with_depth(expr: &str, depth: usize) -> std::result::Result<f64, String> {
    if depth > MAX_MATH_DEPTH {
        return Err("expression too complex".to_string());
    }

    let expr = expr.trim().replace(' ', "");

    if expr.is_empty() {
        return Err("empty expression".to_string());
    }

    // Handle parentheses first (find matching pair)
    if expr.starts_with('(')
        && let Some(end) = find_matching_paren(&expr)
            && end == expr.len() - 1 {
                return evaluate_math_with_depth(&expr[1..end], depth + 1);
            }

    // Handle addition/subtraction (lowest precedence, process right-to-left)
    // Skip operators inside parentheses
    if let Some(pos) = find_top_level_op(&expr, &['+', '-'])
        && pos > 0 {
            let left = evaluate_math_with_depth(&expr[..pos], depth + 1)?;
            let op = expr.chars().nth(pos).unwrap();
            let right = evaluate_math_with_depth(&expr[pos + 1..], depth + 1)?;
            return match op {
                '+' => Ok(left + right),
                '-' => Ok(left - right),
                _ => unreachable!(),
            };
        }

    // Handle multiplication/division/modulo
    if let Some(pos) = find_top_level_op(&expr, &['*', '/', '%']) {
        let left = evaluate_math_with_depth(&expr[..pos], depth + 1)?;
        let op = expr.chars().nth(pos).unwrap();
        let right = evaluate_math_with_depth(&expr[pos + 1..], depth + 1)?;
        return match op {
            '*' => Ok(left * right),
            '/' => {
                if right == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
            '%' => Ok(left % right),
            _ => unreachable!(),
        };
    }

    // Handle exponentiation (right associative)
    if let Some(pos) = find_top_level_op(&expr, &['^']) {
        let left = evaluate_math_with_depth(&expr[..pos], depth + 1)?;
        let right = evaluate_math_with_depth(&expr[pos + 1..], depth + 1)?;
        return Ok(left.powf(right));
    }

    // Handle unary minus
    if expr.starts_with('-') && expr.len() > 1 {
        return Ok(-evaluate_math_with_depth(&expr[1..], depth + 1)?);
    }

    // Handle unary plus
    if expr.starts_with('+') && expr.len() > 1 {
        return evaluate_math_with_depth(&expr[1..], depth + 1);
    }

    // Try to parse as number
    expr.parse::<f64>()
        .map_err(|_| format!("invalid number: {}", expr))
}

/// Find matching closing parenthesis
fn find_matching_paren(expr: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in expr.chars().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Find top-level operator (not inside parentheses), searching right-to-left
fn find_top_level_op(expr: &str, ops: &[char]) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = expr.chars().collect();

    // Search right-to-left for proper precedence
    for i in (0..chars.len()).rev() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if depth == 0 && ops.contains(&c) => {
                // Skip if this is part of a number (e.g., -5)
                if c == '-' && (i == 0 || matches!(chars.get(i - 1), Some(&'(') | Some(&'+') | Some(&'-') | Some(&'*') | Some(&'/') | Some(&'%') | Some(&'^'))) {
                    continue;
                }
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

