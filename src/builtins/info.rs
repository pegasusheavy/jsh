//! Information built-in commands: type, which, command, builtin, help

use super::Builtins;
use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};

/// type - describe a command
pub fn builtin_type(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_which(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_command(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_builtin(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_help(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        println!("jsh - Joseph's Shell - A ZSH/Bash-compatible shell with enhanced scripting");
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
        println!(
            "  string   - String manipulation (length, upper, lower, split, join, replace)"
        );
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
                    println!(
                        "  Colors: black, red, green, yellow, blue, magenta, cyan, white"
                    );
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

