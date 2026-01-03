//! Unit tests for output builtins via shell commands

use franken_shell::Shell;

#[test]
fn test_printf_basic() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf 'Hello, %s!\\n' World");
    assert!(result.is_ok());
}

#[test]
fn test_printf_integer() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf 'Number: %d\\n' 42");
    assert!(result.is_ok());
}

#[test]
fn test_printf_hex() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf 'Hex: %x\\n' 255");
    assert!(result.is_ok());
}

#[test]
fn test_printf_octal() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf 'Octal: %o\\n' 8");
    assert!(result.is_ok());
}

#[test]
fn test_printf_percent() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf '100%%\\n'");
    assert!(result.is_ok());
}

#[test]
fn test_printf_width() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf '%10s' test");
    assert!(result.is_ok());
}

#[test]
fn test_printf_no_args() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf");
    assert!(result.is_ok());
}

#[test]
fn test_printf_multiple_args() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("printf '%s %s %d' Hello World 42");
    assert!(result.is_ok());
}

#[test]
fn test_echo_basic() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo hello world");
    assert!(result.is_ok());
}

#[test]
fn test_echo_no_newline() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo -n hello");
    assert!(result.is_ok());
}

#[test]
fn test_echo_escape_sequences() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo -e 'hello\\nworld'");
    assert!(result.is_ok());
}

#[test]
fn test_echo_empty() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo");
    assert!(result.is_ok());
}

#[test]
fn test_echo_quoted() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo 'hello world'");
    assert!(result.is_ok());
}

#[test]
fn test_echo_double_quoted() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("echo \"hello world\"");
    assert!(result.is_ok());
}

#[test]
fn test_read_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type read");
    assert!(result.is_ok());
}
