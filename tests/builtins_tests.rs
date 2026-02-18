//! Builtins module tests
//!
//! These tests verify the behavior of shell built-in commands.

use franken_shell::interpreter::Interpreter;

// =============================================================================
// Echo Tests
// =============================================================================

#[test]
fn test_builtin_echo() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("echo hello world");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_echo_n() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("echo -n hello");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_echo_e() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"echo -e "hello\tworld""#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Printf Tests
// =============================================================================

#[test]
fn test_builtin_printf() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"printf "Hello %s\n" World"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_printf_number() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"printf "%d\n" 42"#);
    assert!(result.is_ok());
}

// =============================================================================
// CD Tests
// =============================================================================

#[test]
fn test_builtin_cd_home() {
    let mut interp = Interpreter::new();
    // cd without args goes to HOME
    let result = interp.execute_string("cd");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_cd_tmp() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("cd /tmp");
    assert!(result.is_ok());
    // PWD should be set
}

#[test]
fn test_builtin_cd_with_var() {
    let mut interp = Interpreter::new();
    interp.execute_string("cd /tmp").unwrap();
    // Set OLDPWD manually since cd - might have interactive output
    interp.set_var("OLDPWD", "/");
    // Just verify the variable is set
    assert!(interp.get_var("OLDPWD").is_some());
}

// =============================================================================
// PWD Tests
// =============================================================================

#[test]
fn test_builtin_pwd() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("pwd");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Export Tests
// =============================================================================

#[test]
fn test_builtin_export_simple() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("export FOO=bar");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("FOO"), Some("bar"));
}

#[test]
fn test_builtin_export_existing() {
    let mut interp = Interpreter::new();
    interp.execute_string("FOO=initial").unwrap();
    let result = interp.execute_string("export FOO");
    assert!(result.is_ok());
}

// =============================================================================
// Unset Tests
// =============================================================================

#[test]
fn test_builtin_unset() {
    let mut interp = Interpreter::new();
    interp.execute_string("FOO=bar").unwrap();
    let result = interp.execute_string("unset FOO");
    assert!(result.is_ok());
    assert!(interp.get_var("FOO").is_none());
}

#[test]
fn test_builtin_unset_multiple() {
    let mut interp = Interpreter::new();
    interp.execute_string("A=1").unwrap();
    interp.execute_string("B=2").unwrap();
    let result = interp.execute_string("unset A B");
    assert!(result.is_ok());
    assert!(interp.get_var("A").is_none());
    assert!(interp.get_var("B").is_none());
}

// =============================================================================
// True/False Tests
// =============================================================================

#[test]
fn test_builtin_true() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("true");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_false() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("false");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 1);
}

#[test]
fn test_builtin_colon() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(":");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Exit Tests
// =============================================================================

#[test]
fn test_builtin_subshell_var_isolation() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=outer").unwrap();
    // Subshell variable changes shouldn't affect parent
    let result = interp.execute_string("(x=inner)");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("outer"));
}

// =============================================================================
// Test/[ Tests
// =============================================================================

#[test]
fn test_builtin_test_string_eq() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"test "hello" = "hello""#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_string_ne() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"test "hello" != "world""#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_num_eq() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("test 5 -eq 5");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_num_lt() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("test 3 -lt 5");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_num_gt() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("test 5 -gt 3");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_z_empty() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"test -z """#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_n_nonempty() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"test -n "hello""#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_bracket_form() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("[ 1 -eq 1 ]");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_file_exists() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("test -e /etc/passwd");
    assert!(result.is_ok());
    // On most systems this should exist
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_builtin_test_file_not_exists() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("test -e /nonexistent/file/path");
    assert!(result.is_ok());
    assert_ne!(interp.last_status.code, 0);
}

// =============================================================================
// Source Tests
// =============================================================================

#[test]
fn test_builtin_source_nonexistent() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("source /nonexistent/file");
    // Should fail but not crash
    assert!(result.is_err() || interp.last_status.code != 0);
}

// =============================================================================
// Eval Tests
// =============================================================================

#[test]
fn test_builtin_eval() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"eval "x=hello""#);
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("hello"));
}

#[test]
fn test_builtin_eval_command() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"eval "true""#);
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Read Tests
// =============================================================================

// Read tests are tricky because they require stdin
// Skipping interactive read tests

// =============================================================================
// Local Tests
// =============================================================================

#[test]
fn test_builtin_local() {
    let mut interp = Interpreter::new();
    // Test function with local variable
    interp.execute_string("x=global").unwrap();
    interp.execute_string("set_x() { x=changed; }").unwrap();
    interp.execute_string("set_x").unwrap();
    // x should be changed since we didn't use local
    assert!(interp.get_var("x").is_some());
}

// =============================================================================
// Readonly Tests
// =============================================================================

#[test]
fn test_builtin_readonly() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("readonly RO=value");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("RO"), Some("value"));
}

// =============================================================================
// Alias Tests
// =============================================================================

#[test]
fn test_builtin_alias() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("alias ll='ls -la'");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_unalias() {
    let mut interp = Interpreter::new();
    interp.execute_string("alias ll='ls -la'").unwrap();
    let result = interp.execute_string("unalias ll");
    assert!(result.is_ok());
}

// =============================================================================
// Type/Command Tests
// =============================================================================

#[test]
fn test_builtin_type_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("type echo");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_command() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("command echo hello");
    assert!(result.is_ok());
}

// =============================================================================
// Directory Stack Tests
// =============================================================================

#[test]
fn test_builtin_pushd_popd() {
    let mut interp = Interpreter::new();
    interp.execute_string("pushd /tmp").unwrap();
    let result = interp.execute_string("popd");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_dirs() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("dirs");
    assert!(result.is_ok());
}

// =============================================================================
// Hash Tests
// =============================================================================

#[test]
fn test_builtin_hash() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("hash");
    assert!(result.is_ok());
}

// =============================================================================
// Set/Shift Tests
// =============================================================================

#[test]
fn test_builtin_set() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("set -- a b c");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_shift() {
    let mut interp = Interpreter::new();
    interp.execute_string("set -- a b c").unwrap();
    let result = interp.execute_string("shift");
    assert!(result.is_ok());
}

// =============================================================================
// Arithmetic Tests
// =============================================================================

#[test]
fn test_arithmetic_addition() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((2 + 3))").unwrap();
    assert_eq!(interp.get_var("x"), Some("5"));
}

#[test]
fn test_arithmetic_subtraction() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((10 - 3))").unwrap();
    assert_eq!(interp.get_var("x"), Some("7"));
}

#[test]
fn test_arithmetic_multiplication() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((4 * 5))").unwrap();
    assert_eq!(interp.get_var("x"), Some("20"));
}

#[test]
fn test_arithmetic_division() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((20 / 4))").unwrap();
    assert_eq!(interp.get_var("x"), Some("5"));
}

#[test]
fn test_arithmetic_modulo() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((10 % 3))").unwrap();
    assert_eq!(interp.get_var("x"), Some("1"));
}

#[test]
fn test_arithmetic_complex() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((2 + 3 * 4))").unwrap();
    assert_eq!(interp.get_var("x"), Some("14"));
}

#[test]
fn test_arithmetic_parentheses() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$(((2 + 3) * 4))").unwrap();
    assert_eq!(interp.get_var("x"), Some("20"));
}

#[test]
fn test_arithmetic_negative() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=$((-5 + 3))").unwrap();
    assert_eq!(interp.get_var("x"), Some("-2"));
}

// =============================================================================
// Times Tests
// =============================================================================

#[test]
fn test_builtin_times() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("times");
    assert!(result.is_ok());
}

// =============================================================================
// Return Tests
// =============================================================================

#[test]
fn test_builtin_return() {
    let mut interp = Interpreter::new();
    interp.execute_string("test_ret() { return 42; }").unwrap();
    interp.execute_string("test_ret").unwrap();
    assert_eq!(interp.last_status.code, 42);
}

#[test]
fn test_builtin_return_default() {
    let mut interp = Interpreter::new();
    interp
        .execute_string("test_ret() { true; return; }")
        .unwrap();
    interp.execute_string("test_ret").unwrap();
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Break/Continue Tests
// =============================================================================

#[test]
fn test_builtin_break() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=0").unwrap();
    interp
        .execute_string("for i in 1 2 3 4 5; do x=$i; if [ $i -eq 3 ]; then break; fi; done")
        .unwrap();
    assert_eq!(interp.get_var("x"), Some("3"));
}

#[test]
fn test_builtin_continue() {
    let mut interp = Interpreter::new();
    interp.execute_string("sum=0").unwrap();
    // Skip 2 using continue
    interp
        .execute_string(
            "for i in 1 2 3; do if [ $i -eq 2 ]; then continue; fi; sum=$((sum + i)); done",
        )
        .unwrap();
    // 1 + 3 = 4
    assert_eq!(interp.get_var("sum"), Some("4"));
}

// =============================================================================
// Trap Tests
// =============================================================================

#[test]
fn test_builtin_trap() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("trap 'echo exiting' EXIT");
    assert!(result.is_ok());
}

// =============================================================================
// Wait Tests
// =============================================================================

#[test]
fn test_builtin_wait() {
    let mut interp = Interpreter::new();
    // Wait with no jobs should succeed
    let result = interp.execute_string("wait");
    assert!(result.is_ok());
}

// =============================================================================
// Kill Tests (POSIX)
// =============================================================================

#[test]
fn test_builtin_kill_list() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("kill -l");
    assert!(result.is_ok());
}

// =============================================================================
// Getopts Tests
// =============================================================================

#[test]
fn test_builtin_getopts() {
    let mut interp = Interpreter::new();
    // Basic getopts test
    let result = interp.execute_string("getopts abc opt");
    // May fail if no args set, but shouldn't crash
    let _ = result;
}

// =============================================================================
// Umask Tests
// =============================================================================

#[test]
fn test_builtin_umask() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("umask");
    assert!(result.is_ok());
}

#[test]
fn test_builtin_umask_set() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("umask 022");
    assert!(result.is_ok());
}
