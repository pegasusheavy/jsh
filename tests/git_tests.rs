//! Unit tests for git integration builtins via shell commands

use franken_shell::Shell;

#[test]
fn test_git_status_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-status");
    assert!(result.is_ok());
}

#[test]
fn test_git_branch_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-branch");
    assert!(result.is_ok());
}

#[test]
fn test_git_log_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-log");
    assert!(result.is_ok());
}

#[test]
fn test_git_diff_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-diff");
    assert!(result.is_ok());
}

#[test]
fn test_git_add_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-add");
    assert!(result.is_ok());
}

#[test]
fn test_git_commit_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-commit");
    assert!(result.is_ok());
}

#[test]
fn test_git_checkout_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-checkout");
    assert!(result.is_ok());
}

#[test]
fn test_git_pull_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-pull");
    assert!(result.is_ok());
}

#[test]
fn test_git_push_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-push");
    assert!(result.is_ok());
}

#[test]
fn test_git_stash_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-stash");
    assert!(result.is_ok());
}

#[test]
fn test_git_reset_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-reset");
    assert!(result.is_ok());
}

#[test]
fn test_git_merge_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-merge");
    assert!(result.is_ok());
}

#[test]
fn test_git_rebase_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-rebase");
    assert!(result.is_ok());
}

#[test]
fn test_git_remote_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-remote");
    assert!(result.is_ok());
}

#[test]
fn test_git_fetch_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-fetch");
    assert!(result.is_ok());
}

#[test]
fn test_git_clone_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-clone");
    assert!(result.is_ok());
}

#[test]
fn test_git_init_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-init");
    assert!(result.is_ok());
}

#[test]
fn test_git_tag_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-tag");
    assert!(result.is_ok());
}

#[test]
fn test_git_show_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-show");
    assert!(result.is_ok());
}

#[test]
fn test_git_clean_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type git-clean");
    assert!(result.is_ok());
}
