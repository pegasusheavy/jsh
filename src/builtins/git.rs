//! Git integration built-in commands: git_branch, git_status, git_info, git_prompt, in_git_repo

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use std::process::{Command, Stdio};

/// Get the current git branch name
fn get_git_branch(path: &std::path::Path) -> Option<String> {
    // Try symbolic-ref first (for regular branches)
    let output = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(path)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // Fallback to describe for detached HEAD
    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .current_dir(path)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !tag.is_empty() {
            return Some(format!("tags/{}", tag));
        }
    }

    // Fallback to short commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .stderr(Stdio::null())
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
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the git repository root
fn get_git_root(path: &std::path::Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// git_branch - Print current git branch
pub fn builtin_git_branch(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
        let mut cmd = Command::new("git");
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
pub fn builtin_git_status(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
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
pub fn builtin_git_info(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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

    let short_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let full_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let remote = Command::new("git")
        .args(["remote"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let root = get_git_root(&interp.cwd).unwrap_or_default();

    // Get status info
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
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
    let (ahead, behind) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
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
            }
        })
        .unwrap_or((0, 0));

    // Get tag if we're on one
    let tag = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
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
pub fn builtin_git_prompt(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
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
    let (ahead, behind) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(&interp.cwd)
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
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
            }
        })
        .unwrap_or((0, 0));

    let mut result = format;
    result = result.replace("%b", &branch);
    result = result.replace("%d", if is_dirty { "✗" } else { "" });
    result = result.replace("%c", if !is_dirty { "✓" } else { "" });

    let status_symbols = format!(
        "{}{}{}",
        if has_staged { "●" } else { "" },
        if is_dirty && !has_staged { "✚" } else { "" },
        if has_untracked { "…" } else { "" }
    );
    result = result.replace("%s", &status_symbols);

    result = result.replace(
        "%a",
        &if ahead > 0 {
            format!("↑{}", ahead)
        } else {
            String::new()
        },
    );
    result = result.replace(
        "%A",
        &if behind > 0 {
            format!("↓{}", behind)
        } else {
            String::new()
        },
    );

    print!("{}", result);
    Ok(ExitStatus::success())
}

/// in_git_repo - Check if current directory is in a git repository
pub fn builtin_in_git_repo(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
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
