//! Unit tests for info builtins via shell commands

use franken_shell::Shell;

#[test]
fn test_type_echo() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type echo");
    assert!(result.is_ok());
}

#[test]
fn test_type_cd() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type cd");
    assert!(result.is_ok());
}

#[test]
fn test_type_not_found() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type nonexistent_command_xyz");
    assert!(result.is_ok());
}

#[test]
fn test_type_multiple() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type echo cd true");
    assert!(result.is_ok());
}

#[test]
fn test_type_a_flag() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type -a echo");
    assert!(result.is_ok());
}

#[test]
fn test_type_t_flag() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type -t echo");
    assert!(result.is_ok());
}

#[test]
fn test_which_ls() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("which ls");
    assert!(result.is_ok());
}

#[test]
fn test_which_builtin() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("which cd");
    assert!(result.is_ok());
}

#[test]
fn test_which_not_found() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("which nonexistent_command_xyz");
    assert!(result.is_ok());
}

#[test]
fn test_which_multiple() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("which ls cat grep");
    assert!(result.is_ok());
}

#[test]
fn test_help_no_args() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("help");
    assert!(result.is_ok());
}

#[test]
fn test_help_specific() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("help cd");
    assert!(result.is_ok());
}

#[test]
fn test_help_echo() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("help echo");
    assert!(result.is_ok());
}

#[test]
fn test_command_v_flag() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("command -v echo");
    assert!(result.is_ok());
}

#[test]
fn test_command_execute() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("command echo hello");
    assert!(result.is_ok());
}

#[test]
fn test_hash_no_args() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("hash");
    assert!(result.is_ok());
}

#[test]
fn test_hash_r_flag() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("hash -r");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_command() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("builtin echo hello");
    assert!(result.is_ok());
}
