//! Unit tests for fzf integration via shell commands

use franken_shell::Shell;

// Note: These tests verify that fzf commands don't crash when called.
// Actual fzf functionality depends on fzf being installed on the system.

#[test]
fn test_fzf_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Check that the fzf builtin is registered
    let result = shell.run_command("type fzf");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_history_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-history");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_file_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-file");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_dir_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-dir");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_cd_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-cd");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_kill_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-kill");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_git_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-git");
    assert!(result.is_ok());
}

#[test]
fn test_fzf_env_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type fzf-env");
    assert!(result.is_ok());
}
