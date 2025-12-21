//! jsh - A ZSH/Bash-compatible shell with enhanced scripting features

use colored::Colorize;
use jsh::Shell;
use std::env;
use std::process::ExitCode;

fn print_version() {
    println!(
        "{} {} - A ZSH/Bash-compatible shell",
        "jsh".cyan().bold(),
        jsh::VERSION
    );
    println!("Copyright (c) 2025 Pegasus Heavy Industries LLC");
    println!("Licensed under MIT OR Apache-2.0");
}

fn print_help() {
    print_version();
    println!();
    println!("{}", "USAGE:".yellow().bold());
    println!("    jsh [OPTIONS] [SCRIPT] [ARGS...]");
    println!();
    println!("{}", "OPTIONS:".yellow().bold());
    println!("    -c <command>    Execute command string and exit");
    println!("    -s              Read commands from stdin");
    println!("    -i              Force interactive mode");
    println!("    -l, --login     Start as login shell");
    println!("    -n              Parse but don't execute (syntax check)");
    println!("    -v, --verbose   Print commands before execution");
    println!("    -x              Print commands as they execute (debug)");
    println!("    -h, --help      Print this help message");
    println!("    -V, --version   Print version information");
    println!();
    println!("{}", "SCRIPT:".yellow().bold());
    println!("    Path to script file to execute");
    println!();
    println!("{}", "EXAMPLES:".yellow().bold());
    println!("    jsh                     Start interactive shell");
    println!("    jsh script.sh           Run a script");
    println!("    jsh -c 'echo hello'     Run a command");
    println!("    jsh script.sh arg1 arg2 Run script with arguments");
    println!();
    println!("{}", "JSH FEATURES:".yellow().bold());
    println!("    jsh extends Bash/ZSH with modern syntax:");
    println!();
    println!("    {} - Pattern matching like Rust", "match".magenta());
    println!("        match $val {{");
    println!("            1 => echo one");
    println!("            2|3 => echo two_or_three");
    println!("            * => echo other");
    println!("        }}");
    println!();
    println!("    {} - Infinite loop", "loop".magenta());
    println!("        loop {{ read x; [[ $x == quit ]] && break }}");
    println!();
    println!("    {}/{} - Variable bindings", "let".magenta(), "const".magenta());
    println!("        let name = value");
    println!("        const PI = 3.14159");
    println!();
    println!("    {} - Error handling", "try/catch/finally".magenta());
    println!("        try {{ risky }} catch e {{ echo $e }} finally {{ cleanup }}");
    println!();
    println!("    {} - Function shorthand", "fn".magenta());
    println!("        fn greet {{ echo \"Hello $1\" }}");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Check if invoked as login shell (argv[0] starts with '-' or --login flag)
    let invoked_as_login = args.first()
        .map(|s| s.starts_with('-') || s.ends_with("-jsh"))
        .unwrap_or(false);

    let mut command: Option<String> = None;
    let mut script: Option<String> = None;
    let mut script_args: Vec<String> = Vec::new();
    let mut interactive = false;
    let mut from_stdin = false;
    let mut login_shell = invoked_as_login;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                print_version();
                return ExitCode::SUCCESS;
            }
            "-c" => {
                if i + 1 < args.len() {
                    command = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("{}: -c requires a command string", "error".red().bold());
                    return ExitCode::FAILURE;
                }
            }
            "-s" => {
                from_stdin = true;
            }
            "-i" => {
                interactive = true;
            }
            "-l" | "--login" => {
                login_shell = true;
            }
            "-n" => {
                // TODO: Parse-only mode
            }
            "-v" | "--verbose" | "-x" => {
                // TODO: Verbose/trace mode
            }
            arg if !arg.starts_with('-') => {
                script = Some(arg.to_string());
                // Rest are script arguments
                script_args = args[i + 1..].to_vec();
                break;
            }
            _ => {
                eprintln!("{}: unknown option: {}", "warning".yellow(), args[i]);
            }
        }
        i += 1;
    }

    // Determine if shell is interactive
    let is_interactive = interactive || (command.is_none() && script.is_none() && atty::is(atty::Stream::Stdin));

    // Create shell with appropriate options
    let mut shell = match Shell::with_options(login_shell, is_interactive) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: failed to initialize shell: {}", "error".red().bold(), e);
            return ExitCode::FAILURE;
        }
    };

    // Execute based on mode
    let exit_code = if let Some(cmd) = command {
        // -c mode: run command string
        match shell.run_command(&cmd) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                1
            }
        }
    } else if let Some(script_path) = script {
        // Script mode: run script file
        match shell.run_script(&script_path, &script_args) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}: {}: {}", "error".red().bold(), script_path, e);
                1
            }
        }
    } else if from_stdin && !atty::is(atty::Stream::Stdin) {
        // Read from stdin
        use std::io::Read;
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("{}: failed to read stdin: {}", "error".red().bold(), e);
            return ExitCode::FAILURE;
        }
        match shell.run_command(&input) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                1
            }
        }
    } else if interactive || atty::is(atty::Stream::Stdin) {
        // Interactive mode
        match shell.run() {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                1
            }
        }
    } else {
        // Non-interactive stdin
        use std::io::Read;
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("{}: failed to read stdin: {}", "error".red().bold(), e);
            return ExitCode::FAILURE;
        }
        match shell.run_command(&input) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                1
            }
        }
    };

    if exit_code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(exit_code as u8)
    }
}
