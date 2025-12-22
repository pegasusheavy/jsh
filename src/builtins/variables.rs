//! Variable management built-in commands: export, unset, set, local, readonly, declare

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};

/// export - export variables
pub fn builtin_export(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_unset(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    for name in args {
        if name.starts_with("-v") || name.starts_with("-f") {
            continue;
        }
        // Remove from interpreter's variable tables
        interp.vars.remove(name);
        interp.env.remove(name);
        interp.exports.remove(name);
        // Also remove from process environment
        unsafe { std::env::remove_var(name) };
    }
    Ok(ExitStatus::success())
}

/// set - set shell options
pub fn builtin_set(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
            arg[1..]
                .chars()
                .map(|c| {
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
                })
                .filter(|s| !s.is_empty())
                .collect()
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
pub fn builtin_local(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_readonly(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
pub fn builtin_declare(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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

/// Set a shell option by name (for set -o / set +o)
pub fn set_option_by_name(name: &str, enable: bool, interp: &mut Interpreter) {
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
pub fn print_shell_options(interp: &Interpreter) {
    let opts = &interp.options;
    println!(
        "errexit         {}",
        if opts.errexit { "on" } else { "off" }
    );
    println!(
        "nounset         {}",
        if opts.nounset { "on" } else { "off" }
    );
    println!("xtrace          {}", if opts.xtrace { "on" } else { "off" });
    println!("noexec          {}", if opts.noexec { "on" } else { "off" });
    println!(
        "allexport       {}",
        if opts.allexport { "on" } else { "off" }
    );
    println!(
        "noclobber       {}",
        if opts.noclobber { "on" } else { "off" }
    );
    println!("notify          {}", if opts.notify { "on" } else { "off" });
    println!("noglob          {}", if opts.noglob { "on" } else { "off" });
    println!("vi              {}", if opts.vi { "on" } else { "off" });
    println!("emacs           {}", if opts.emacs { "on" } else { "off" });
    println!(
        "ignoreeof       {}",
        if opts.ignoreeof { "on" } else { "off" }
    );
    println!("posix           {}", if opts.posix { "on" } else { "off" });
}

