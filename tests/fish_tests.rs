//! Unit tests for fish-compatible builtins - command existence checks

use franken_shell::Shell;

#[test]
fn test_string_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type string");
    assert!(result.is_ok());
}

#[test]
fn test_math_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type math");
    assert!(result.is_ok());
}

#[test]
fn test_status_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type status");
    assert!(result.is_ok());
}

#[test]
fn test_contains_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type contains");
    assert!(result.is_ok());
}

#[test]
fn test_count_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type count");
    assert!(result.is_ok());
}

#[test]
fn test_path_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type path");
    assert!(result.is_ok());
}

#[test]
fn test_abbr_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type abbr");
    assert!(result.is_ok());
}

#[test]
fn test_set_color_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type set_color");
    assert!(result.is_ok());
}

#[test]
fn test_isatty_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type isatty");
    assert!(result.is_ok());
}

#[test]
fn test_seq_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type seq");
    assert!(result.is_ok());
}
