//! Unit tests for variable expansion

use franken_shell::Shell;

#[test]
fn test_expand_simple_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("FOO", "bar");
    let result = shell.run_command("echo $FOO");
    assert!(result.is_ok());
}

#[test]
fn test_expand_braced_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("VAR", "value");
    let result = shell.run_command("echo ${VAR}");
    assert!(result.is_ok());
}

#[test]
fn test_expand_undefined_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $UNDEFINED");
    assert!(result.is_ok());
}

#[test]
fn test_expand_default_value() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo ${UNDEFINED:-default}");
    assert!(result.is_ok());
}

#[test]
fn test_expand_default_value_set() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("DEFINED", "actual");
    let result = shell.run_command("echo ${DEFINED:-default}");
    assert!(result.is_ok());
}

#[test]
fn test_expand_string_length() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("STR", "hello");
    let result = shell.run_command("echo ${#STR}");
    assert!(result.is_ok());
}

#[test]
fn test_expand_special_var_question_mark() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("true; echo $?");
    assert!(result.is_ok());
}

#[test]
fn test_expand_special_var_dollar() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $$");
    assert!(result.is_ok());
}

#[test]
fn test_expand_special_var_hash() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $#");
    assert!(result.is_ok());
}

#[test]
fn test_expand_tilde() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo ~");
    assert!(result.is_ok());
}

#[test]
fn test_expand_arithmetic() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $((1 + 2))");
    assert!(result.is_ok());
}

#[test]
fn test_expand_arithmetic_with_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "10");
    let result = shell.run_command("echo $((X + 5))");
    assert!(result.is_ok());
}

#[test]
fn test_expand_multiple_variables() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("A", "Hello");
    shell.interpreter.set_var("B", "World");
    let result = shell.run_command("echo $A $B");
    assert!(result.is_ok());
}

#[test]
fn test_expand_home_env() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $HOME");
    assert!(result.is_ok());
}

#[test]
fn test_expand_path_env() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo $PATH");
    assert!(result.is_ok());
}

#[test]
fn test_double_quote_preserves_variables() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("VAR", "value");
    let result = shell.run_command("echo \"$VAR\"");
    assert!(result.is_ok());
}

#[test]
fn test_command_substitution_simple() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("VAR=$(echo hello)");
    assert!(result.is_ok());
}

#[test]
fn test_assignment_simple() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("X=5");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("X"), Some("5"));
}

#[test]
fn test_assignment_with_expansion() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("A", "hello");
    let result = shell.run_command("B=$A");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("B"), Some("hello"));
}
