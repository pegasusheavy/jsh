//! FZF (fuzzy finder) integration for Franken Shell
//!
//! Provides shell functions and utilities for fzf integration:
//! - fzf_history: Fuzzy search command history
//! - fzf_file: Fuzzy file selection
//! - fzf_dir: Fuzzy directory selection
//! - fzf_git_branch: Fuzzy git branch selection
//! - fzf_git_log: Fuzzy git log selection
//! - fzf_process: Fuzzy process selection

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::io::Write;
use std::process::{Command, Stdio};

/// Check if fzf is available in PATH
pub fn fzf_available() -> bool {
    which::which("fzf").is_ok()
}

/// Run fzf with the given input and options
fn run_fzf(input: &str, opts: &[&str]) -> Option<String> {
    let mut cmd = Command::new("fzf");
    cmd.args(opts)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// fzf_history - fuzzy search command history
/// Usage: fzf_history [--multi]
pub fn builtin_fzf_history(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_history: fzf not found in PATH");
        eprintln!("Install fzf: https://github.com/junegunn/fzf#installation");
        return Ok(ExitStatus::failure(1));
    }

    // Get history file
    let history_file = crate::shell::default_history_file();
    let history = std::fs::read_to_string(&history_file).unwrap_or_default();

    // Reverse lines so newest is at top
    let lines: Vec<&str> = history.lines().rev().collect();
    let input = lines.join("\n");

    let mut opts = vec![
        "--height=40%",
        "--reverse",
        "--prompt=history> ",
        "--preview-window=hidden",
    ];

    if args.contains(&"--multi".to_string()) {
        opts.push("--multi");
    }

    if let Some(selected) = run_fzf(&input, &opts) {
        println!("{}", selected);
        interp.set_var("FZF_RESULT", &selected);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130)) // Cancelled
    }
}

/// fzf_file - fuzzy file selection
/// Usage: fzf_file [directory] [--multi] [--preview]
pub fn builtin_fzf_file(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_file: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let dir = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or(".");

    // Use fd if available, otherwise find
    let files = if which::which("fd").is_ok() {
        Command::new("fd")
            .args(["--type", "f", "--hidden", "--exclude", ".git", ".", dir])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        Command::new("find")
            .args([dir, "-type", "f", "-not", "-path", "*/.git/*"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    };

    let input = files.unwrap_or_default();

    let mut opts = vec!["--height=40%", "--reverse", "--prompt=file> "];

    if args.contains(&"--multi".to_string()) {
        opts.push("--multi");
    }

    if args.contains(&"--preview".to_string()) {
        opts.push("--preview");
        opts.push("head -100 {}");
    }

    if let Some(selected) = run_fzf(&input, &opts) {
        println!("{}", selected);
        interp.set_var("FZF_RESULT", &selected);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf_dir - fuzzy directory selection
/// Usage: fzf_dir [base_directory] [--preview]
pub fn builtin_fzf_dir(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_dir: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let dir = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or(".");

    // Use fd if available, otherwise find
    let dirs = if which::which("fd").is_ok() {
        Command::new("fd")
            .args(["--type", "d", "--hidden", "--exclude", ".git", ".", dir])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        Command::new("find")
            .args([dir, "-type", "d", "-not", "-path", "*/.git/*"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    };

    let input = dirs.unwrap_or_default();

    let mut opts = vec!["--height=40%", "--reverse", "--prompt=dir> "];

    if args.contains(&"--preview".to_string()) {
        opts.push("--preview");
        opts.push("ls -la {}");
    }

    if let Some(selected) = run_fzf(&input, &opts) {
        println!("{}", selected);
        interp.set_var("FZF_RESULT", &selected);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf_git_branch - fuzzy git branch selection
/// Usage: fzf_git_branch [--all]
pub fn builtin_fzf_git_branch(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_git_branch: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let branch_args = if args.contains(&"--all".to_string()) {
        vec!["branch", "-a", "--format=%(refname:short)"]
    } else {
        vec!["branch", "--format=%(refname:short)"]
    };

    let branches = Command::new("git")
        .args(&branch_args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

    let input = match branches {
        Some(b) if !b.is_empty() => b,
        _ => {
            eprintln!("franken: fzf_git_branch: not a git repository or no branches found");
            return Ok(ExitStatus::failure(1));
        }
    };

    let opts = vec![
        "--height=40%",
        "--reverse",
        "--prompt=branch> ",
        "--preview",
        "git log --oneline -20 {}",
    ];

    if let Some(selected) = run_fzf(&input, &opts) {
        println!("{}", selected);
        interp.set_var("FZF_RESULT", &selected);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf_git_log - fuzzy git log selection (returns commit hash)
/// Usage: fzf_git_log [--all]
pub fn builtin_fzf_git_log(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_git_log: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let log_args = if args.contains(&"--all".to_string()) {
        vec![
            "log",
            "--all",
            "--oneline",
            "--graph",
            "--color=always",
            "-100",
        ]
    } else {
        vec!["log", "--oneline", "--graph", "--color=always", "-100"]
    };

    let logs = Command::new("git")
        .args(&log_args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

    let input = match logs {
        Some(l) if !l.is_empty() => l,
        _ => {
            eprintln!("franken: fzf_git_log: not a git repository or no commits found");
            return Ok(ExitStatus::failure(1));
        }
    };

    let opts = vec![
        "--height=50%",
        "--reverse",
        "--ansi",
        "--prompt=commit> ",
        "--preview",
        "git show --color=always {1}",
    ];

    if let Some(selected) = run_fzf(&input, &opts) {
        // Extract commit hash (first word after any graph characters)
        let hash = selected
            .split_whitespace()
            .find(|s| s.chars().all(|c| c.is_ascii_hexdigit()))
            .unwrap_or(&selected);
        println!("{}", hash);
        interp.set_var("FZF_RESULT", hash);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf_process - fuzzy process selection
/// Usage: fzf_process [--multi]
pub fn builtin_fzf_process(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_process: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let ps_output = Command::new("ps")
        .args(["aux"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut opts = vec![
        "--height=50%",
        "--reverse",
        "--header-lines=1",
        "--prompt=process> ",
    ];

    if args.contains(&"--multi".to_string()) {
        opts.push("--multi");
    }

    if let Some(selected) = run_fzf(&ps_output, &opts) {
        // Extract PID (second column)
        let pid = selected.split_whitespace().nth(1).unwrap_or(&selected);
        println!("{}", pid);
        interp.set_var("FZF_RESULT", pid);
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf - generic fzf wrapper
/// Usage: echo "input" | fzf [options...]
/// Or: fzf [options...] (reads from stdin)
pub fn builtin_fzf(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf: fzf not found in PATH");
        eprintln!("Install fzf: https://github.com/junegunn/fzf#installation");
        return Ok(ExitStatus::failure(1));
    }

    // Pass through to fzf directly
    let status = Command::new("fzf")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            if code == 0 {
                Ok(ExitStatus::success())
            } else {
                Ok(ExitStatus::failure(code))
            }
        }
        Err(e) => {
            eprintln!("franken: fzf: {}", e);
            Ok(ExitStatus::failure(1))
        }
    }
}

/// fzf_cd - fuzzy cd with directory preview
/// Usage: fzf_cd [base_directory]
pub fn builtin_fzf_cd(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_cd: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let dir = args.first().map(|s| s.as_str()).unwrap_or(".");

    // Use fd if available, otherwise find
    let dirs = if which::which("fd").is_ok() {
        Command::new("fd")
            .args(["--type", "d", "--hidden", "--exclude", ".git", ".", dir])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        Command::new("find")
            .args([dir, "-type", "d", "-not", "-path", "*/.git/*"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    };

    let input = dirs.unwrap_or_default();

    let opts = vec![
        "--height=40%",
        "--reverse",
        "--prompt=cd> ",
        "--preview",
        "ls -la {}",
    ];

    if let Some(selected) = run_fzf(&input, &opts) {
        // Change to the selected directory
        if std::env::set_current_dir(&selected).is_ok() {
            let pwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            interp.set_var("PWD", &pwd);
            interp.set_var("FZF_RESULT", &selected);
            Ok(ExitStatus::success())
        } else {
            eprintln!("franken: fzf_cd: cannot cd to '{}'", selected);
            Ok(ExitStatus::failure(1))
        }
    } else {
        Ok(ExitStatus::failure(130))
    }
}

/// fzf_kill - fuzzy kill process
/// Usage: fzf_kill [signal]
pub fn builtin_fzf_kill(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if !fzf_available() {
        eprintln!("franken: fzf_kill: fzf not found in PATH");
        return Ok(ExitStatus::failure(1));
    }

    let signal = args.first().map(|s| s.as_str()).unwrap_or("-15");

    let ps_output = Command::new("ps")
        .args(["aux"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let opts = vec![
        "--height=50%",
        "--reverse",
        "--header-lines=1",
        "--prompt=kill> ",
        "--multi",
    ];

    if let Some(selected) = run_fzf(&ps_output, &opts) {
        let mut killed = Vec::new();
        for line in selected.lines() {
            if let Some(pid) = line.split_whitespace().nth(1) {
                let status = Command::new("kill").args([signal, pid]).status();

                if status.map(|s| s.success()).unwrap_or(false) {
                    killed.push(pid.to_string());
                }
            }
        }

        if !killed.is_empty() {
            println!("Killed: {}", killed.join(", "));
            interp.set_var("FZF_RESULT", &killed.join(" "));
            Ok(ExitStatus::success())
        } else {
            Ok(ExitStatus::failure(1))
        }
    } else {
        Ok(ExitStatus::failure(130))
    }
}
