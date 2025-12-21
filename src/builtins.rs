//! Built-in commands for jsh shell

use crate::error::{JshError, Result};
use crate::interpreter::{ExitStatus, Interpreter};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

/// Type for builtin command functions
type BuiltinFn = fn(&[String], &mut Interpreter) -> Result<ExitStatus>;

/// Built-in command handler
pub struct Builtins {
    commands: HashMap<String, BuiltinFn>,
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}

impl Builtins {
    pub fn new() -> Self {
        let mut commands: HashMap<String, BuiltinFn> = HashMap::new();

        // Navigation
        commands.insert("cd".to_string(), builtin_cd);
        commands.insert("pwd".to_string(), builtin_pwd);
        commands.insert("pushd".to_string(), builtin_pushd);
        commands.insert("popd".to_string(), builtin_popd);
        commands.insert("dirs".to_string(), builtin_dirs);

        // Output
        commands.insert("echo".to_string(), builtin_echo);
        commands.insert("printf".to_string(), builtin_printf);

        // Variables
        commands.insert("export".to_string(), builtin_export);
        commands.insert("unset".to_string(), builtin_unset);
        commands.insert("set".to_string(), builtin_set);
        commands.insert("local".to_string(), builtin_local);
        commands.insert("readonly".to_string(), builtin_readonly);
        commands.insert("declare".to_string(), builtin_declare);
        commands.insert("typeset".to_string(), builtin_declare); // alias

        // Control flow
        commands.insert("exit".to_string(), builtin_exit);
        commands.insert("return".to_string(), builtin_return);
        commands.insert("true".to_string(), builtin_true);
        commands.insert("false".to_string(), builtin_false);
        commands.insert(":".to_string(), builtin_true); // colon is true

        // Job control
        commands.insert("jobs".to_string(), builtin_jobs);
        commands.insert("fg".to_string(), builtin_fg);
        commands.insert("bg".to_string(), builtin_bg);
        commands.insert("wait".to_string(), builtin_wait);

        // Source and eval
        commands.insert("source".to_string(), builtin_source);
        commands.insert(".".to_string(), builtin_source);
        commands.insert("eval".to_string(), builtin_eval);

        // Test
        commands.insert("test".to_string(), builtin_test);
        commands.insert("[".to_string(), builtin_test);

        // Information
        commands.insert("type".to_string(), builtin_type);
        commands.insert("which".to_string(), builtin_which);
        commands.insert("command".to_string(), builtin_command);
        commands.insert("builtin".to_string(), builtin_builtin);
        commands.insert("help".to_string(), builtin_help);

        // Misc
        commands.insert("alias".to_string(), builtin_alias);
        commands.insert("unalias".to_string(), builtin_unalias);
        commands.insert("hash".to_string(), builtin_hash);
        commands.insert("read".to_string(), builtin_read);
        commands.insert("shift".to_string(), builtin_shift);
        commands.insert("exec".to_string(), builtin_exec);
        commands.insert("trap".to_string(), builtin_trap);
        commands.insert("umask".to_string(), builtin_umask);
        commands.insert("ulimit".to_string(), builtin_ulimit);
        commands.insert("times".to_string(), builtin_times);
        commands.insert("getopts".to_string(), builtin_getopts);
        commands.insert("enable".to_string(), builtin_enable);
        commands.insert("shopt".to_string(), builtin_shopt);
        commands.insert("let".to_string(), builtin_let_arith);
        commands.insert("history".to_string(), builtin_history);

        // Fish-compatible builtins
        commands.insert("string".to_string(), builtin_string);
        commands.insert("contains".to_string(), builtin_contains);
        commands.insert("status".to_string(), builtin_status);
        commands.insert("functions".to_string(), builtin_functions);
        commands.insert("abbr".to_string(), builtin_abbr);
        commands.insert("math".to_string(), builtin_math);

        // Git integration
        commands.insert("git_branch".to_string(), builtin_git_branch);
        commands.insert("git_status".to_string(), builtin_git_status);
        commands.insert("git_info".to_string(), builtin_git_info);
        commands.insert("git_prompt".to_string(), builtin_git_prompt);
        commands.insert("in_git_repo".to_string(), builtin_in_git_repo);

        Self { commands }
    }

    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Execute a builtin command
    pub fn execute(
        &self,
        name: &str,
        args: &[String],
        interp: &mut Interpreter,
    ) -> Result<Option<ExitStatus>> {
        if let Some(func) = self.commands.get(name) {
            Ok(Some(func(args, interp)?))
        } else {
            Ok(None)
        }
    }

    /// List all builtins
    pub fn list(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}

// ============================================================================
// Builtin implementations
// ============================================================================

/// cd - change directory
fn builtin_cd(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
            eprintln!("jsh: cd: {}: {}", path, e);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// pwd - print working directory
fn builtin_pwd(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    println!("{}", interp.cwd.display());
    Ok(ExitStatus::success())
}

// Directory stack for pushd/popd
thread_local! {
    static DIR_STACK: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// pushd - push directory onto stack
fn builtin_pushd(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let current = interp.cwd.clone();

    if args.is_empty() {
        // Swap with top of stack
        DIR_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(top) = stack.pop() {
                stack.push(current);
                builtin_cd(&[top.to_string_lossy().to_string()], interp)
            } else {
                eprintln!("jsh: pushd: no other directory");
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
fn builtin_popd(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    DIR_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(dir) = stack.pop() {
            builtin_cd(&[dir.to_string_lossy().to_string()], interp)
        } else {
            eprintln!("jsh: popd: directory stack empty");
            Ok(ExitStatus::failure(1))
        }
    })
}

/// dirs - display directory stack
fn builtin_dirs(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    print!("{}", interp.cwd.display());
    DIR_STACK.with(|stack| {
        for dir in stack.borrow().iter().rev() {
            print!(" {}", dir.display());
        }
    });
    println!();
    Ok(ExitStatus::success())
}

/// echo - print arguments
fn builtin_echo(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
            if arg.contains('n') { newline = false; }
            if arg.contains('e') { interpret_escapes = true; }
            if arg.contains('E') { interpret_escapes = false; }
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
fn interpret_escape_sequences(s: &str) -> String {
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
fn builtin_printf(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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

/// export - export variables
fn builtin_export(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // List all exports
        for name in &interp.exports {
            if let Some(value) = interp.get_var(name) {
                println!("export {}={:?}", name, value);
            }
        }
        return Ok(ExitStatus::success());
    }

    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let name = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];
            interp.export_var(name, Some(value));
        } else {
            interp.export_var(arg, None);
        }
    }

    Ok(ExitStatus::success())
}

/// unset - unset variables
fn builtin_unset(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    for name in args {
        if name.starts_with("-v") || name.starts_with("-f") {
            continue;
        }
        interp.vars.remove(name);
        interp.env.remove(name);
        interp.exports.remove(name);
    }
    Ok(ExitStatus::success())
}

/// set - set shell options
fn builtin_set(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    // Handle -o without argument (list all options)
    if args.len() == 1 && (args[0] == "-o" || args[0] == "+o") {
        print_shell_options(interp);
        return Ok(ExitStatus::success());
    }

    if args.is_empty() {
        // Print all variables
        for (name, value) in &interp.vars {
            println!("{}={:?}", name, value);
        }
        for (name, value) in &interp.env {
            println!("{}={:?}", name, value);
        }
        return Ok(ExitStatus::success());
    }

    // Fish-style flags
    let mut fish_export = false;
    let mut fish_erase = false;
    let mut fish_query = false;
    let mut fish_show = false;
    #[allow(unused_assignments)]
    let mut _local = false;
    let mut global = false;
    let mut universal = false;
    let mut idx = 0;

    while idx < args.len() {
        let arg = &args[idx];

        // Handle +o and -o options (POSIX/Ash)
        if arg == "-o" || arg == "+o" {
            let enable = arg == "-o";
            if idx + 1 < args.len() {
                set_option_by_name(&args[idx + 1], enable, interp);
                idx += 2;
                continue;
            } else {
                // List all options
                print_shell_options(interp);
                return Ok(ExitStatus::success());
            }
        }

        if !arg.starts_with('-') && !arg.starts_with('+') {
            break;
        }

        let enable = arg.starts_with('-');
        let flags = if arg.starts_with("--") {
            vec![arg.as_str()]
        } else {
            arg[1..].chars().map(|c| {
                // Return the flag as a single character string
                match c {
                    'e' => "-e",
                    'u' => "-u",
                    'x' => "-x",
                    'n' => "-n",
                    'a' => "-a",
                    'C' => "-C",
                    'b' => "-b",
                    'f' => "-f",
                    'v' => "-v",
                    'h' => "-h",
                    _ => "",
                }
            }).filter(|s| !s.is_empty()).collect()
        };

        for flag in flags {
            match flag {
                // POSIX/Ash shell options
                "-e" => interp.options.errexit = enable,
                "-u" => interp.options.nounset = enable,
                "-x" => interp.options.xtrace = enable,
                "-n" => interp.options.noexec = enable,
                "-a" => interp.options.allexport = enable,
                "-C" => interp.options.noclobber = enable,
                "-b" => interp.options.notify = enable,
                "-f" => interp.options.noglob = enable,
                // Fish-style options (only with -)
                "--export" => fish_export = true,
                "--global" => global = true,
                "--local" => _local = true,
                "--universal" => universal = true,
                "--erase" => fish_erase = true,
                "--query" => fish_query = true,
                "--show" => fish_show = true,
                _ => {}
            }
        }

        // Special case: -- sets positional params
        if arg == "--" {
            interp.positional_params = args[idx + 1..].to_vec();
            return Ok(ExitStatus::success());
        }

        idx += 1;
    }

    let remaining = &args[idx..];

    // Handle Fish-style set commands
    if fish_erase {
        for name in remaining {
            interp.vars.remove(name);
            interp.env.remove(name);
            interp.exports.remove(name);
        }
        return Ok(ExitStatus::success());
    }

    if fish_query {
        for name in remaining {
            if interp.get_var(name).is_none() {
                return Ok(ExitStatus::failure(1));
            }
        }
        return Ok(ExitStatus::success());
    }

    if fish_show {
        for name in remaining {
            if let Some(value) = interp.get_var(name) {
                let scope = if interp.exports.contains(name) {
                    "exported"
                } else if interp.env.contains_key(name) {
                    "environment"
                } else {
                    "local"
                };
                println!("${}='{}'  # {}", name, value, scope);
            }
        }
        return Ok(ExitStatus::success());
    }

    // Fish-style variable assignment: set varname value [value2 ...]
    if !remaining.is_empty() {
        let name = &remaining[0];

        // Check for Bash-style var=value
        if let Some(eq_pos) = name.find('=') {
            let var_name = &name[..eq_pos];
            let value = &name[eq_pos + 1..];
            if fish_export || global {
                interp.export_var(var_name, Some(value));
            } else {
                interp.set_var(var_name, value);
            }
        } else if remaining.len() >= 2 {
            // Fish-style: set name value
            let values = &remaining[1..];
            let value = values.join(" "); // Fish stores as list, we simplify to string

            if fish_export || global || universal {
                interp.export_var(name, Some(&value));
            } else {
                interp.set_var(name, &value);
            }
        } else {
            // set name (with no value) - show value or set to empty
            if let Some(value) = interp.get_var(name) {
                println!("{}", value);
            }
        }
    }

    Ok(ExitStatus::success())
}

/// local - declare local variables
fn builtin_local(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let name = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];
            interp.set_var(name, value);
        } else {
            interp.set_var(arg, "");
        }
    }
    Ok(ExitStatus::success())
}

/// readonly - declare readonly variables
fn builtin_readonly(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        for (name, value) in &interp.consts {
            println!("readonly {}={:?}", name, value);
        }
        return Ok(ExitStatus::success());
    }

    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let name = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];
            interp.consts.insert(name.to_string(), value.to_string());
        } else if let Some(value) = interp.get_var(arg) {
            interp.consts.insert(arg.to_string(), value.to_string());
        }
    }

    Ok(ExitStatus::success())
}

/// declare - declare variables with attributes
fn builtin_declare(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut export = false;
    let mut readonly = false;
    let mut idx = 0;

    // Parse options
    while idx < args.len() && args[idx].starts_with('-') {
        for c in args[idx].chars().skip(1) {
            match c {
                'x' => export = true,
                'r' => readonly = true,
                _ => {}
            }
        }
        idx += 1;
    }

    for arg in &args[idx..] {
        if let Some(eq_pos) = arg.find('=') {
            let name = &arg[..eq_pos];
            let value = &arg[eq_pos + 1..];

            if export {
                interp.export_var(name, Some(value));
            } else if readonly {
                interp.consts.insert(name.to_string(), value.to_string());
            } else {
                interp.set_var(name, value);
            }
        }
    }

    Ok(ExitStatus::success())
}

/// exit - exit the shell
fn builtin_exit(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    Err(JshError::Exit(code))
}

/// return - return from function
fn builtin_return(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let value = args.first().cloned();
    Err(JshError::Return(value))
}

/// true - return success
fn builtin_true(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    Ok(ExitStatus::success())
}

/// false - return failure
fn builtin_false(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    Ok(ExitStatus::failure(1))
}

/// jobs - list background jobs
fn builtin_jobs(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement job control
    Ok(ExitStatus::success())
}

/// fg - bring job to foreground
fn builtin_fg(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    eprintln!("jsh: fg: job control not implemented");
    Ok(ExitStatus::failure(1))
}

/// bg - send job to background
fn builtin_bg(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    eprintln!("jsh: bg: job control not implemented");
    Ok(ExitStatus::failure(1))
}

/// wait - wait for background jobs
fn builtin_wait(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Wait for background processes
    Ok(ExitStatus::success())
}

/// source - execute commands from file
fn builtin_source(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("jsh: source: filename argument required");
        return Ok(ExitStatus::failure(1));
    }

    interp.run_script(&args[0])
}

/// eval - evaluate string as command
fn builtin_eval(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let command = args.join(" ");
    interp.execute_string(&command)
}

/// test - evaluate conditional expression
fn builtin_test(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn evaluate_test(args: &[&str], interp: &Interpreter) -> Result<bool> {
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
                let left_time = std::fs::metadata(left)
                    .and_then(|m| m.modified())
                    .ok();
                let right_time = std::fs::metadata(right)
                    .and_then(|m| m.modified())
                    .ok();
                return Ok(left_time > right_time);
            }
            "-ot" => {
                // Older than
                let left_time = std::fs::metadata(left)
                    .and_then(|m| m.modified())
                    .ok();
                let right_time = std::fs::metadata(right)
                    .and_then(|m| m.modified())
                    .ok();
                return Ok(left_time < right_time);
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
            _ => {}
        }
    }

    // Single argument - true if non-empty
    if args.len() == 1 {
        return Ok(!args[0].is_empty());
    }

    Ok(false)
}

/// type - describe a command
fn builtin_type(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let builtins = Builtins::new();

    for name in args {
        if builtins.is_builtin(name) {
            println!("{} is a shell builtin", name);
        } else if interp.functions.contains_key(name) {
            println!("{} is a function", name);
        } else if let Ok(path) = which::which(name) {
            println!("{} is {}", name, path.display());
        } else {
            eprintln!("jsh: type: {}: not found", name);
        }
    }

    Ok(ExitStatus::success())
}

/// which - locate a command
fn builtin_which(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut status = ExitStatus::success();

    for name in args {
        match which::which(name) {
            Ok(path) => println!("{}", path.display()),
            Err(_) => {
                eprintln!("{} not found", name);
                status = ExitStatus::failure(1);
            }
        }
    }

    Ok(status)
}

/// command - execute a command
fn builtin_command(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        return Ok(ExitStatus::success());
    }

    // Skip -v, -V, -p options
    let mut idx = 0;
    while idx < args.len() && args[idx].starts_with('-') {
        if args[idx] == "-v" {
            if idx + 1 < args.len() {
                return builtin_type(&args[idx + 1..], interp);
            }
        }
        idx += 1;
    }

    if idx >= args.len() {
        return Ok(ExitStatus::success());
    }

    // Execute command, bypassing functions
    interp.execute_string(&args[idx..].join(" "))
}

/// builtin - run a shell builtin
fn builtin_builtin(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        return Ok(ExitStatus::success());
    }

    let builtins = Builtins::new();
    if let Some(status) = builtins.execute(&args[0], &args[1..], interp)? {
        Ok(status)
    } else {
        eprintln!("jsh: builtin: {}: not a shell builtin", args[0]);
        Ok(ExitStatus::failure(1))
    }
}

/// help - display help
fn builtin_help(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        println!("jsh - A ZSH/Bash-compatible shell with enhanced scripting features");
        println!();
        println!("Built-in commands:");
        let builtins = Builtins::new();
        let mut names: Vec<_> = builtins.list();
        names.sort();
        for chunk in names.chunks(8) {
            println!("  {}", chunk.join("  "));
        }
        println!();
        println!("jsh-specific features:");
        println!("  match    - Pattern matching expression");
        println!("  loop     - Infinite loop");
        println!("  let      - Variable binding");
        println!("  const    - Constant binding");
        println!("  try      - Try-catch-finally");
        println!("  fn       - Function definition");
        println!();
        println!("Theme system (oh-my-zsh compatible):");
        println!("  theme list           - List available themes");
        println!("  theme set <name>     - Set theme");
        println!("  theme preview        - Preview all themes");
        println!();
        println!("Theme variables:");
        println!("  JSH_THEME   - Set theme by name (in .jshrc)");
        println!("  PROMPT      - Custom prompt string (ZSH format)");
        println!("  RPROMPT     - Right prompt string");
        println!();
        println!("Prompt escape sequences (ZSH-style):");
        println!("  %n - username      %m - hostname");
        println!("  %~ - cwd (~)       %c - current dir");
        println!("  %F{{color}}...%f   - foreground color");
        println!("  %K{{color}}...%k   - background color");
        println!("  %B...%b            - bold");
        println!("  %(?.true.false)    - conditional on exit status");
        println!();
        println!("Fish-compatible builtins:");
        println!("  set      - Fish-style variable assignment");
        println!("  string   - String manipulation (length, upper, lower, split, join, replace)");
        println!("  math     - Arithmetic evaluation");
        println!("  contains - List membership test");
        println!("  status   - Shell status queries");
        println!("  functions - Function management");
        println!("  abbr     - Abbreviations");
        println!();
        println!("Git integration builtins:");
        println!("  git_branch   - Get current git branch name");
        println!("  git_status   - Git status in shell-friendly format");
        println!("  git_info     - Detailed git repository information");
        println!("  git_prompt   - Git info formatted for shell prompts");
        println!("  in_git_repo  - Check if in a git repository");
        println!();
        println!("Ash/POSIX-compatible options (set -o <name>):");
        println!("  errexit  (-e)  Exit on error");
        println!("  nounset  (-u)  Error on unset variables");
        println!("  xtrace   (-x)  Print commands before execution");
        println!("  noexec   (-n)  Parse only, don't execute");
        println!("  allexport (-a) Export all variables");
        println!("  noclobber (-C) Don't overwrite files with >");
        println!("  noglob   (-f)  Disable pathname expansion");
        println!("  posix         Strict POSIX mode");
    } else {
        for name in args {
            match name.as_str() {
                "theme" => {
                    println!("theme - manage shell themes");
                    println!();
                    println!("Usage:");
                    println!("  theme list           - List available themes");
                    println!("  theme set <name>     - Set theme");
                    println!("  theme <name>         - Set theme (shorthand)");
                    println!("  theme current        - Show current theme");
                    println!("  theme preview        - Preview all themes");
                    println!("  theme preview <name> - Preview specific theme");
                    println!();
                    println!("Available themes: robbyrussell, agnoster, minimal, jsh,");
                    println!("                  powerlevel, simple, pure");
                    println!();
                    println!("You can also set themes via environment variables:");
                    println!("  export JSH_THEME=<name>");
                    println!("  export PROMPT='<prompt string>'");
                }
                "prompt" | "PROMPT" => {
                    println!("PROMPT - Custom prompt string");
                    println!();
                    println!("ZSH-style prompt escape sequences:");
                    println!("  %n - username");
                    println!("  %m - short hostname    %M - full hostname");
                    println!("  %~ - cwd with ~ subst  %c - current dir only");
                    println!("  %T - 24hr time         %t - 12hr time");
                    println!("  %D{{fmt}} - date with strftime format");
                    println!("  %# - # if root, %% otherwise");
                    println!("  %? - last exit status");
                    println!("  %h - history number");
                    println!();
                    println!("Colors:");
                    println!("  %F{{color}}text%f - foreground color");
                    println!("  %K{{color}}text%k - background color");
                    println!("  Colors: black, red, green, yellow, blue, magenta, cyan, white");
                    println!("  Also: 0-255 for 256-color, or #RRGGBB for true color");
                    println!();
                    println!("Formatting:");
                    println!("  %B...%b - bold");
                    println!("  %U...%u - underline");
                    println!();
                    println!("Conditionals:");
                    println!("  %(?.true.false) - if last status was 0");
                    println!("  %(#.true.false) - if user is root");
                    println!();
                    println!("Git info:");
                    println!("  $(git_prompt_info) - git branch and status");
                }
                _ => println!("{}: Help for this command is not available", name),
            }
        }
    }

    Ok(ExitStatus::success())
}

/// alias - define or display aliases
fn builtin_alias(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement aliases
    Ok(ExitStatus::success())
}

/// unalias - remove aliases
fn builtin_unalias(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement aliases
    Ok(ExitStatus::success())
}

/// hash - remember or display command locations
fn builtin_hash(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement command hashing
    Ok(ExitStatus::success())
}

/// read - read a line from input
fn builtin_read(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let var_name = args.first().map(|s| s.as_str()).unwrap_or("REPLY");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => Ok(ExitStatus::failure(1)), // EOF
        Ok(_) => {
            let input = input.trim_end_matches('\n').trim_end_matches('\r');
            interp.set_var(var_name, input);
            Ok(ExitStatus::success())
        }
        Err(_) => Ok(ExitStatus::failure(1)),
    }
}

/// shift - shift positional parameters
fn builtin_shift(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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

/// exec - replace shell with command
fn builtin_exec(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        return Ok(ExitStatus::success());
    }

    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
    eprintln!("jsh: exec: {}: {}", args[0], err);
    Ok(ExitStatus::failure(126))
}

/// trap - set signal handlers
fn builtin_trap(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement signal trapping
    Ok(ExitStatus::success())
}

/// umask - set file creation mask
fn builtin_umask(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn builtin_ulimit(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement resource limits
    println!("unlimited");
    Ok(ExitStatus::success())
}

/// times - display process times
fn builtin_times(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement process times
    println!("0m0.000s 0m0.000s");
    println!("0m0.000s 0m0.000s");
    Ok(ExitStatus::success())
}

/// getopts - parse positional parameters
fn builtin_getopts(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement getopts
    Ok(ExitStatus::failure(1))
}

/// enable - enable/disable builtins
fn builtin_enable(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement enable
    Ok(ExitStatus::success())
}

/// shopt - set shell options
fn builtin_shopt(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement shell options
    Ok(ExitStatus::success())
}

/// let - evaluate arithmetic expression
fn builtin_let_arith(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn builtin_history(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    // TODO: Implement history
    Ok(ExitStatus::success())
}

// ============================================================================
// Fish-compatible builtins
// ============================================================================

/// string - Fish-compatible string manipulation
fn builtin_string(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn builtin_contains(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn builtin_status(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
            println!("");
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
fn builtin_functions(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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

// Thread-local abbreviations storage
thread_local! {
    static ABBREVIATIONS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

/// abbr - Fish-compatible abbreviations
fn builtin_abbr(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
fn builtin_math(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
    let expr = expr.replace(" ", "");

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

// ============================================================================
// Shell options helpers (POSIX/Ash compatible)
// ============================================================================

/// Set a shell option by name (for set -o / set +o)
fn set_option_by_name(name: &str, enable: bool, interp: &mut Interpreter) {
    match name {
        "errexit" => interp.options.errexit = enable,
        "nounset" => interp.options.nounset = enable,
        "xtrace" => interp.options.xtrace = enable,
        "noexec" => interp.options.noexec = enable,
        "allexport" => interp.options.allexport = enable,
        "noclobber" => interp.options.noclobber = enable,
        "notify" => interp.options.notify = enable,
        "noglob" => interp.options.noglob = enable,
        "vi" => {
            interp.options.vi = enable;
            if enable {
                interp.options.emacs = false;
            }
        }
        "emacs" => {
            interp.options.emacs = enable;
            if enable {
                interp.options.vi = false;
            }
        }
        "ignoreeof" => interp.options.ignoreeof = enable,
        "posix" => interp.options.posix = enable,
        _ => eprintln!("jsh: set: {}: invalid option name", name),
    }
}

/// Print all shell options (for set -o with no argument)
fn print_shell_options(interp: &Interpreter) {
    let opts = &interp.options;
    println!("errexit         {}", if opts.errexit { "on" } else { "off" });
    println!("nounset         {}", if opts.nounset { "on" } else { "off" });
    println!("xtrace          {}", if opts.xtrace { "on" } else { "off" });
    println!("noexec          {}", if opts.noexec { "on" } else { "off" });
    println!("allexport       {}", if opts.allexport { "on" } else { "off" });
    println!("noclobber       {}", if opts.noclobber { "on" } else { "off" });
    println!("notify          {}", if opts.notify { "on" } else { "off" });
    println!("noglob          {}", if opts.noglob { "on" } else { "off" });
    println!("vi              {}", if opts.vi { "on" } else { "off" });
    println!("emacs           {}", if opts.emacs { "on" } else { "off" });
    println!("ignoreeof       {}", if opts.ignoreeof { "on" } else { "off" });
    println!("posix           {}", if opts.posix { "on" } else { "off" });
}

// ============================================================================
// Git integration builtins
// ============================================================================

/// Get the current git branch name
fn get_git_branch(path: &std::path::Path) -> Option<String> {
    // Try symbolic-ref first (for regular branches)
    let output = std::process::Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(path)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // Fallback to describe for detached HEAD
    let output = std::process::Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .current_dir(path)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !tag.is_empty() {
            return Some(format!("tags/{}", tag));
        }
    }

    // Fallback to short commit hash
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !hash.is_empty() {
            return Some(hash);
        }
    }

    None
}

/// Check if we're in a git repository
fn is_in_git_repo(path: &std::path::Path) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the git repository root
fn get_git_root(path: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// git_branch - Print current git branch
fn builtin_git_branch(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut short = false;
    let mut show_all = false;
    let mut show_remote = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--short" => short = true,
            "-a" | "--all" => show_all = true,
            "-r" | "--remote" => show_remote = true,
            "-h" | "--help" => {
                println!("git_branch - Show current git branch");
                println!();
                println!("Usage: git_branch [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -s, --short   Show only the branch name (default)");
                println!("  -a, --all     Show all branches (local and remote)");
                println!("  -r, --remote  Show remote branches");
                println!("  -h, --help    Show this help message");
                println!();
                println!("Examples:");
                println!("  git_branch              # Print current branch");
                println!("  BRANCH=$(git_branch)    # Capture branch name");
                println!("  if [ \"$(git_branch)\" = \"main\" ]; then ...; fi");
                return Ok(ExitStatus::success());
            }
            _ => {}
        }
        i += 1;
    }

    if show_all || show_remote {
        // Use git branch command for listing
        let mut cmd = std::process::Command::new("git");
        cmd.arg("branch");
        if show_all {
            cmd.arg("-a");
        } else if show_remote {
            cmd.arg("-r");
        }
        cmd.current_dir(&interp.cwd);

        let output = cmd.output();
        if let Ok(output) = output {
            if output.status.success() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                return Ok(ExitStatus::success());
            }
        }
        return Ok(ExitStatus::failure(1));
    }

    // Default: show current branch
    if let Some(branch) = get_git_branch(&interp.cwd) {
        if short {
            print!("{}", branch);
        } else {
            println!("{}", branch);
        }
        Ok(ExitStatus::success())
    } else {
        if !short {
            eprintln!("git_branch: not a git repository");
        }
        Ok(ExitStatus::failure(1))
    }
}

/// git_status - Show git status in a shell-friendly format
fn builtin_git_status(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut porcelain = false;
    let mut check_dirty = false;
    let mut check_staged = false;
    let mut check_untracked = false;
    let mut check_clean = false;

    for arg in args {
        match arg.as_str() {
            "-p" | "--porcelain" => porcelain = true,
            "-d" | "--dirty" => check_dirty = true,
            "-s" | "--staged" => check_staged = true,
            "-u" | "--untracked" => check_untracked = true,
            "-c" | "--clean" => check_clean = true,
            "-h" | "--help" => {
                println!("git_status - Git status in shell-friendly format");
                println!();
                println!("Usage: git_status [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --porcelain   Show porcelain status output");
                println!("  -d, --dirty       Exit 0 if repo has uncommitted changes");
                println!("  -s, --staged      Exit 0 if repo has staged changes");
                println!("  -u, --untracked   Exit 0 if repo has untracked files");
                println!("  -c, --clean       Exit 0 if repo is clean");
                println!("  -h, --help        Show this help message");
                println!();
                println!("Examples:");
                println!("  if git_status --dirty; then echo 'Changes pending'; fi");
                println!("  git_status --porcelain | wc -l");
                return Ok(ExitStatus::success());
            }
            _ => {}
        }
    }

    if !is_in_git_repo(&interp.cwd) {
        eprintln!("git_status: not a git repository");
        return Ok(ExitStatus::failure(128));
    }

    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(ExitStatus::failure(1)),
    };

    if !output.status.success() {
        return Ok(ExitStatus::failure(1));
    }

    let status_text = String::from_utf8_lossy(&output.stdout);

    if check_dirty || check_staged || check_untracked || check_clean {
        let mut has_dirty = false;
        let mut has_staged = false;
        let mut has_untracked = false;

        for line in status_text.lines() {
            if line.starts_with("??") {
                has_untracked = true;
            } else if line.starts_with(' ') {
                has_dirty = true;
            } else if !line.is_empty() {
                has_staged = true;
                // Also check index for dirty
                if line.len() > 1 && !line[1..2].trim().is_empty() && &line[1..2] != " " {
                    has_dirty = true;
                }
            }
        }

        let is_clean = !has_dirty && !has_staged && !has_untracked;

        if check_dirty && has_dirty {
            return Ok(ExitStatus::success());
        }
        if check_staged && has_staged {
            return Ok(ExitStatus::success());
        }
        if check_untracked && has_untracked {
            return Ok(ExitStatus::success());
        }
        if check_clean && is_clean {
            return Ok(ExitStatus::success());
        }

        return Ok(ExitStatus::failure(1));
    }

    if porcelain {
        print!("{}", status_text);
    } else {
        // Human-friendly output
        let mut dirty_count = 0;
        let mut staged_count = 0;
        let mut untracked_count = 0;

        for line in status_text.lines() {
            if line.starts_with("??") {
                untracked_count += 1;
            } else if line.starts_with(' ') {
                dirty_count += 1;
            } else if !line.is_empty() {
                staged_count += 1;
            }
        }

        if staged_count > 0 {
            println!("staged: {}", staged_count);
        }
        if dirty_count > 0 {
            println!("modified: {}", dirty_count);
        }
        if untracked_count > 0 {
            println!("untracked: {}", untracked_count);
        }
        if staged_count == 0 && dirty_count == 0 && untracked_count == 0 {
            println!("clean");
        }
    }

    Ok(ExitStatus::success())
}

/// git_info - Get detailed git repository information
fn builtin_git_info(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut format = String::new();
    let mut query: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 1;
                }
            }
            "-q" | "--query" => {
                if i + 1 < args.len() {
                    query = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("git_info - Get git repository information");
                println!();
                println!("Usage: git_info [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -q, --query <field>   Query specific field:");
                println!("                        branch, hash, short_hash, remote,");
                println!("                        root, dirty, staged, untracked,");
                println!("                        ahead, behind, tag");
                println!("  -f, --format <fmt>    Custom format string");
                println!("  -h, --help            Show this help message");
                println!();
                println!("Format placeholders:");
                println!("  %b  branch name       %h  short hash");
                println!("  %H  full hash         %r  remote name");
                println!("  %R  repo root         %d  dirty indicator");
                println!("  %s  staged indicator  %u  untracked indicator");
                println!("  %a  ahead count       %A  behind count");
                println!("  %t  tag (if on tag)");
                println!();
                println!("Examples:");
                println!("  git_info --query branch");
                println!("  git_info --format '%b (%h)'");
                return Ok(ExitStatus::success());
            }
            _ => {}
        }
        i += 1;
    }

    if !is_in_git_repo(&interp.cwd) {
        eprintln!("git_info: not a git repository");
        return Ok(ExitStatus::failure(128));
    }

    // Gather all git info
    let branch = get_git_branch(&interp.cwd).unwrap_or_default();

    let short_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
        } else {
            None
        })
        .unwrap_or_default();

    let full_hash = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
        } else {
            None
        })
        .unwrap_or_default();

    let remote = std::process::Command::new("git")
        .args(["remote"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            String::from_utf8_lossy(&o.stdout).lines().next().map(|s| s.to_string())
        } else {
            None
        })
        .unwrap_or_default();

    let root = get_git_root(&interp.cwd).unwrap_or_default();

    // Get status info
    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output();

    let (is_dirty, has_staged, has_untracked) = if let Ok(output) = status_output {
        let status = String::from_utf8_lossy(&output.stdout);
        let mut dirty = false;
        let mut staged = false;
        let mut untracked = false;
        for line in status.lines() {
            if line.starts_with("??") {
                untracked = true;
            } else if line.starts_with(' ') {
                dirty = true;
            } else if !line.is_empty() {
                staged = true;
            }
        }
        (dirty || staged, staged, untracked)
    } else {
        (false, false, false)
    };

    // Get ahead/behind
    let (ahead, behind) = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            let counts = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = counts.trim().split('\t').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].parse::<usize>().unwrap_or(0),
                    parts[1].parse::<usize>().unwrap_or(0),
                ))
            } else {
                None
            }
        } else {
            None
        })
        .unwrap_or((0, 0));

    // Get tag if we're on one
    let tag = std::process::Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
        } else {
            None
        })
        .unwrap_or_default();

    // Handle query mode
    if let Some(q) = query {
        let result = match q.as_str() {
            "branch" => branch.clone(),
            "hash" => full_hash.clone(),
            "short_hash" => short_hash.clone(),
            "remote" => remote.clone(),
            "root" => root.clone(),
            "dirty" => if is_dirty { "true" } else { "false" }.to_string(),
            "staged" => if has_staged { "true" } else { "false" }.to_string(),
            "untracked" => if has_untracked { "true" } else { "false" }.to_string(),
            "ahead" => ahead.to_string(),
            "behind" => behind.to_string(),
            "tag" => tag.clone(),
            _ => {
                eprintln!("git_info: unknown field '{}'", q);
                return Ok(ExitStatus::failure(1));
            }
        };
        println!("{}", result);
        return Ok(ExitStatus::success());
    }

    // Handle format mode
    if !format.is_empty() {
        let mut result = format.clone();
        result = result.replace("%b", &branch);
        result = result.replace("%h", &short_hash);
        result = result.replace("%H", &full_hash);
        result = result.replace("%r", &remote);
        result = result.replace("%R", &root);
        result = result.replace("%d", if is_dirty { "✗" } else { "" });
        result = result.replace("%s", if has_staged { "●" } else { "" });
        result = result.replace("%u", if has_untracked { "?" } else { "" });
        result = result.replace("%a", &ahead.to_string());
        result = result.replace("%A", &behind.to_string());
        result = result.replace("%t", &tag);
        println!("{}", result);
        return Ok(ExitStatus::success());
    }

    // Default: show all info
    println!("branch: {}", branch);
    println!("commit: {} ({})", short_hash, full_hash);
    if !remote.is_empty() {
        println!("remote: {}", remote);
    }
    println!("root: {}", root);
    println!("dirty: {}", is_dirty);
    println!("staged: {}", has_staged);
    println!("untracked: {}", has_untracked);
    if ahead > 0 || behind > 0 {
        println!("ahead: {}, behind: {}", ahead, behind);
    }
    if !tag.is_empty() {
        println!("tag: {}", tag);
    }

    Ok(ExitStatus::success())
}

/// git_prompt - Output formatted git info for prompts
fn builtin_git_prompt(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut format = " (%b%d)".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("git_prompt - Git info formatted for shell prompts");
                println!();
                println!("Usage: git_prompt [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -f, --format <fmt>  Format string (default: ' (%b%d)')");
                println!("  -h, --help          Show this help message");
                println!();
                println!("Format placeholders:");
                println!("  %b  branch name     %d  dirty indicator (✗)");
                println!("  %c  clean indicator (✓)");
                println!("  %s  status symbols (●✚…)");
                println!("  %a  ahead (↑N)      %A  behind (↓N)");
                println!();
                println!("Examples:");
                println!("  PS1=\"\\u@\\h:\\w$(git_prompt)$ \"");
                println!("  git_prompt -f ' [%b%s]'");
                return Ok(ExitStatus::success());
            }
            _ => {}
        }
        i += 1;
    }

    if !is_in_git_repo(&interp.cwd) {
        // Silent exit - no output if not in repo
        return Ok(ExitStatus::success());
    }

    let branch = get_git_branch(&interp.cwd).unwrap_or_else(|| "unknown".to_string());

    // Get status
    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output();

    let (is_dirty, has_staged, has_untracked) = if let Ok(output) = status_output {
        let status = String::from_utf8_lossy(&output.stdout);
        let mut dirty = false;
        let mut staged = false;
        let mut untracked = false;
        for line in status.lines() {
            if line.starts_with("??") {
                untracked = true;
            } else if line.starts_with(' ') {
                dirty = true;
            } else if !line.is_empty() {
                staged = true;
            }
        }
        (dirty || staged, staged, untracked)
    } else {
        (false, false, false)
    };

    // Get ahead/behind
    let (ahead, behind) = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(&interp.cwd)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            let counts = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = counts.trim().split('\t').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].parse::<usize>().unwrap_or(0),
                    parts[1].parse::<usize>().unwrap_or(0),
                ))
            } else {
                None
            }
        } else {
            None
        })
        .unwrap_or((0, 0));

    let mut result = format;
    result = result.replace("%b", &branch);
    result = result.replace("%d", if is_dirty { "✗" } else { "" });
    result = result.replace("%c", if !is_dirty { "✓" } else { "" });

    let status_symbols = format!("{}{}{}",
        if has_staged { "●" } else { "" },
        if is_dirty && !has_staged { "✚" } else { "" },
        if has_untracked { "…" } else { "" }
    );
    result = result.replace("%s", &status_symbols);

    result = result.replace("%a", &if ahead > 0 { format!("↑{}", ahead) } else { String::new() });
    result = result.replace("%A", &if behind > 0 { format!("↓{}", behind) } else { String::new() });

    print!("{}", result);
    Ok(ExitStatus::success())
}

/// in_git_repo - Check if current directory is in a git repository
fn builtin_in_git_repo(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    for arg in args {
        if arg == "-h" || arg == "--help" {
            println!("in_git_repo - Check if in a git repository");
            println!();
            println!("Usage: in_git_repo");
            println!();
            println!("Returns exit code 0 if in a git repo, 1 otherwise.");
            println!();
            println!("Examples:");
            println!("  if in_git_repo; then echo 'In git repo'; fi");
            println!("  in_git_repo && git_branch");
            return Ok(ExitStatus::success());
        }
    }

    if is_in_git_repo(&interp.cwd) {
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

