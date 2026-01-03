//! Unit tests for the shell module

use franken_shell::shell::{fsh_cache_dir, fsh_config_dir, fsh_data_dir, fsh_state_dir};
use franken_shell::shell::{xdg_cache_home, xdg_config_home, xdg_data_home, xdg_state_home};
use franken_shell::Shell;

#[test]
fn test_shell_new() {
    let shell = Shell::with_options(false, false);
    assert!(shell.is_ok());
}

#[test]
fn test_xdg_config_home() {
    let path = xdg_config_home();
    assert!(!path.to_string_lossy().is_empty());
}

#[test]
fn test_xdg_data_home() {
    let path = xdg_data_home();
    assert!(!path.to_string_lossy().is_empty());
}

#[test]
fn test_xdg_cache_home() {
    let path = xdg_cache_home();
    assert!(!path.to_string_lossy().is_empty());
}

#[test]
fn test_xdg_state_home() {
    let path = xdg_state_home();
    assert!(!path.to_string_lossy().is_empty());
}

#[test]
fn test_fsh_config_dir() {
    let path = fsh_config_dir();
    assert!(!path.to_string_lossy().is_empty());
    assert!(path.to_string_lossy().contains("fsh"));
}

#[test]
fn test_fsh_data_dir() {
    let path = fsh_data_dir();
    assert!(!path.to_string_lossy().is_empty());
    assert!(path.to_string_lossy().contains("fsh"));
}

#[test]
fn test_fsh_cache_dir() {
    let path = fsh_cache_dir();
    assert!(!path.to_string_lossy().is_empty());
    assert!(path.to_string_lossy().contains("fsh"));
}

#[test]
fn test_fsh_state_dir() {
    let path = fsh_state_dir();
    assert!(!path.to_string_lossy().is_empty());
    assert!(path.to_string_lossy().contains("fsh"));
}

#[test]
fn test_shell_set_var() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("TEST_VAR", "test_value");
    assert_eq!(shell.interpreter.get_var("TEST_VAR"), Some("test_value"));
}

#[test]
fn test_shell_get_undefined_var() {
    let shell = Shell::with_options(false, false).unwrap();
    assert_eq!(shell.interpreter.get_var("UNDEFINED_VAR_XYZ"), None);
}

#[test]
fn test_shell_execute_simple_command() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo test");
    assert!(result.is_ok());
}

#[test]
fn test_shell_execute_assignment() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("MY_VAR=my_value");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("MY_VAR"), Some("my_value"));
}

#[test]
fn test_shell_execute_multiple_commands() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("A=1; B=2");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("A"), Some("1"));
    assert_eq!(shell.interpreter.get_var("B"), Some("2"));
}

#[test]
fn test_shell_execute_empty_string() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("");
    assert!(result.is_ok());
}

#[test]
fn test_shell_execute_whitespace_only() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("   ");
    assert!(result.is_ok());
}

#[test]
fn test_shell_execute_comment() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("# This is a comment");
    assert!(result.is_ok());
}

#[test]
fn test_shell_interpreter_has_pid() {
    let shell = Shell::with_options(false, false).unwrap();
    let pid = shell.interpreter.shell_pid;
    assert!(pid > 0);
    assert_eq!(pid, std::process::id());
}

#[test]
fn test_shell_positional_params_empty() {
    let shell = Shell::with_options(false, false).unwrap();
    assert!(shell.interpreter.positional_params.is_empty());
}

#[test]
fn test_shell_last_status_default() {
    let shell = Shell::with_options(false, false).unwrap();
    assert_eq!(shell.interpreter.last_status.code, 0);
}

#[test]
fn test_shell_builtin_echo() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo hello world");
    assert!(result.is_ok());
}

#[test]
fn test_shell_builtin_true() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("true");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.last_status.code, 0);
}

#[test]
fn test_shell_builtin_false() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("false");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.last_status.code, 1);
}

#[test]
fn test_shell_builtin_test_eq() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("test 1 -eq 1");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.last_status.code, 0);
}

#[test]
fn test_shell_builtin_test_ne() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("test 1 -ne 2");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.last_status.code, 0);
}

#[test]
fn test_shell_for_loop() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("for i in 1 2 3; do echo $i; done");
    assert!(result.is_ok());
}

#[test]
fn test_shell_while_loop() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("x=0; while test $x -lt 3; do x=$((x+1)); done");
    assert!(result.is_ok());
}

#[test]
fn test_shell_if_statement() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("if true; then echo yes; fi");
    assert!(result.is_ok());
}

#[test]
fn test_shell_if_else_statement() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("if false; then echo no; else echo yes; fi");
    assert!(result.is_ok());
}

#[test]
fn test_shell_case_statement() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("x=a; case $x in a) echo matched;; esac");
    assert!(result.is_ok());
}

#[test]
fn test_shell_function_definition() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("myfunc() { echo hello; }");
    assert!(result.is_ok());
}

#[test]
fn test_shell_function_call() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let _ = shell.run_command("myfunc() { echo hello; }");
    let result = shell.run_command("myfunc");
    assert!(result.is_ok());
}

#[test]
fn test_shell_pipeline() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo hello | cat");
    assert!(result.is_ok());
}

#[test]
fn test_shell_command_substitution() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Command substitution parsing test - just verify it doesn't crash
    let result = shell.run_command("VAR=$(echo hello)");
    assert!(result.is_ok());
}

#[test]
fn test_shell_arithmetic_expansion() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $((2 + 3))");
    assert!(result.is_ok());
}
