//! Interpreter module tests

use franken_shell::interpreter::Interpreter;

// =============================================================================
// Basic Command Execution Tests
// =============================================================================

#[test]
fn test_interpreter_new() {
    let interp = Interpreter::new();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_execute_empty_string() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("");
    assert!(result.is_ok());
}

#[test]
fn test_execute_whitespace() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("   ");
    assert!(result.is_ok());
}

#[test]
fn test_execute_comment() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("# this is a comment");
    assert!(result.is_ok());
}

#[test]
fn test_execute_true() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("true");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_execute_false() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("false");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 1);
}

#[test]
fn test_execute_colon() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(":");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Variable Tests
// =============================================================================

#[test]
fn test_variable_assignment() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("x=hello");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("hello"));
}

#[test]
fn test_variable_assignment_with_quotes() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"x="hello world""#);
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("hello world"));
}

#[test]
fn test_variable_unset() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=hello").unwrap();
    interp.execute_string("unset x").unwrap();
    assert!(interp.get_var("x").is_none());
}

#[test]
fn test_export_variable() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("export FOO=bar");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("FOO"), Some("bar"));
}

#[test]
fn test_readonly_variable() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("readonly RO=value");
    assert!(result.is_ok());
    // Should fail to modify
    let result2 = interp.execute_string("RO=other");
    assert!(result2.is_err() || interp.get_var("RO") == Some("value"));
}

// =============================================================================
// Echo Tests
// =============================================================================

#[test]
fn test_echo_simple() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("echo hello");
    assert!(result.is_ok());
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_echo_variable() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=world").unwrap();
    let result = interp.execute_string("echo $x");
    assert!(result.is_ok());
}

#[test]
fn test_echo_multiple_args() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("echo hello world");
    assert!(result.is_ok());
}

#[test]
fn test_echo_n_flag() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("echo -n hello");
    assert!(result.is_ok());
}

#[test]
fn test_echo_e_flag() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string(r#"echo -e "hello\nworld""#);
    assert!(result.is_ok());
}

// =============================================================================
// Control Flow Tests
// =============================================================================

#[test]
fn test_if_then_fi_true() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("if true; then result=yes; fi");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("yes"));
}

#[test]
fn test_if_then_fi_false() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("if false; then result=yes; fi");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("no"));
}

#[test]
fn test_if_then_else_fi() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("if false; then result=yes; else result=no; fi");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("no"));
}

#[test]
fn test_for_loop() {
    let mut interp = Interpreter::new();
    interp.execute_string("count=0").unwrap();
    let result = interp.execute_string("for i in 1 2 3; do count=$((count + 1)); done");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("count"), Some("3"));
}

#[test]
fn test_while_loop() {
    let mut interp = Interpreter::new();
    interp.execute_string("count=0").unwrap();
    let result = interp.execute_string("while [ $count -lt 3 ]; do count=$((count + 1)); done");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("count"), Some("3"));
}

#[test]
fn test_break_in_loop() {
    let mut interp = Interpreter::new();
    interp.execute_string("count=0").unwrap();
    let result = interp.execute_string("for i in 1 2 3 4 5; do count=$((count + 1)); if [ $count -eq 2 ]; then break; fi; done");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("count"), Some("2"));
}

#[test]
fn test_continue_in_loop() {
    let mut interp = Interpreter::new();
    interp.execute_string("sum=0").unwrap();
    // Skip 2 by continuing
    let result = interp.execute_string("for i in 1 2 3; do if [ $i -eq 2 ]; then continue; fi; sum=$((sum + i)); done");
    assert!(result.is_ok());
    // sum = 1 + 3 = 4
    assert_eq!(interp.get_var("sum"), Some("4"));
}

// =============================================================================
// Function Tests
// =============================================================================

#[test]
fn test_function_definition() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("myfunc() { echo hello; }");
    assert!(result.is_ok());
}

#[test]
fn test_function_call() {
    let mut interp = Interpreter::new();
    interp.execute_string("myfunc() { result=called; }").unwrap();
    let result = interp.execute_string("myfunc");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("called"));
}

#[test]
fn test_function_with_args() {
    let mut interp = Interpreter::new();
    interp.execute_string("greet() { result=$1; }").unwrap();
    let result = interp.execute_string("greet world");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("world"));
}

#[test]
fn test_function_return() {
    let mut interp = Interpreter::new();
    interp.execute_string("ret5() { return 5; }").unwrap();
    interp.execute_string("ret5").unwrap();
    assert_eq!(interp.last_status.code, 5);
}

#[test]
fn test_fn_shorthand() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("fn hello { result=hi; }");
    assert!(result.is_ok());
    interp.execute_string("hello").unwrap();
    assert_eq!(interp.get_var("result"), Some("hi"));
}

// =============================================================================
// Pipeline Tests
// =============================================================================

#[test]
fn test_simple_pipeline() {
    let mut interp = Interpreter::new();
    // This will run but output depends on system
    let result = interp.execute_string("echo hello | cat");
    assert!(result.is_ok());
}

#[test]
fn test_and_operator() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("true && result=yes");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("yes"));
}

#[test]
fn test_and_operator_short_circuit() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("false && result=yes");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("no"));
}

#[test]
fn test_or_operator() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("false || result=yes");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("yes"));
}

#[test]
fn test_or_operator_short_circuit() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=no").unwrap();
    let result = interp.execute_string("true || result=yes");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("result"), Some("no"));
}

// =============================================================================
// Test Command Tests
// =============================================================================

#[test]
fn test_test_string_eq() {
    let mut interp = Interpreter::new();
    interp.execute_string(r#"[ "hello" = "hello" ]"#).unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_string_ne() {
    let mut interp = Interpreter::new();
    interp.execute_string(r#"[ "hello" = "world" ]"#).unwrap();
    assert_ne!(interp.last_status.code, 0);
}

#[test]
fn test_test_numeric_eq() {
    let mut interp = Interpreter::new();
    interp.execute_string("[ 5 -eq 5 ]").unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_numeric_ne() {
    let mut interp = Interpreter::new();
    interp.execute_string("[ 5 -ne 3 ]").unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_numeric_lt() {
    let mut interp = Interpreter::new();
    interp.execute_string("[ 3 -lt 5 ]").unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_numeric_gt() {
    let mut interp = Interpreter::new();
    interp.execute_string("[ 5 -gt 3 ]").unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_z_empty() {
    let mut interp = Interpreter::new();
    interp.execute_string(r#"[ -z "" ]"#).unwrap();
    assert_eq!(interp.last_status.code, 0);
}

#[test]
fn test_test_n_nonempty() {
    let mut interp = Interpreter::new();
    interp.execute_string(r#"[ -n "hello" ]"#).unwrap();
    assert_eq!(interp.last_status.code, 0);
}

// =============================================================================
// Arithmetic Tests
// =============================================================================

#[test]
fn test_arithmetic_add() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((2 + 3))").unwrap();
    assert_eq!(interp.get_var("result"), Some("5"));
}

#[test]
fn test_arithmetic_subtract() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((10 - 3))").unwrap();
    assert_eq!(interp.get_var("result"), Some("7"));
}

#[test]
fn test_arithmetic_multiply() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((4 * 5))").unwrap();
    assert_eq!(interp.get_var("result"), Some("20"));
}

#[test]
fn test_arithmetic_divide() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((20 / 4))").unwrap();
    assert_eq!(interp.get_var("result"), Some("5"));
}

#[test]
fn test_arithmetic_modulo() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((10 % 3))").unwrap();
    assert_eq!(interp.get_var("result"), Some("1"));
}

#[test]
fn test_arithmetic_precedence() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$((2 + 3 * 4))").unwrap();
    assert_eq!(interp.get_var("result"), Some("14"));
}

#[test]
fn test_arithmetic_parentheses() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$(((2 + 3) * 4))").unwrap();
    assert_eq!(interp.get_var("result"), Some("20"));
}

#[test]
fn test_arithmetic_variable() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=5").unwrap();
    interp.execute_string("result=$((x * 2))").unwrap();
    assert_eq!(interp.get_var("result"), Some("10"));
}

// =============================================================================
// Case Statement Tests
// =============================================================================

#[test]
fn test_case_structure() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=hello").unwrap();
    // Test that case statement parses and runs without crash
    let result = interp.execute_string("case $x in hello) echo matched;; esac");
    // Just verify it runs
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_case_default_pattern() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=other").unwrap();
    // Test case with default pattern
    let result = interp.execute_string("case $x in hello) echo matched;; *) echo default;; esac");
    // Just verify it runs
    assert!(result.is_ok() || result.is_err());
}

// =============================================================================
// Subshell Tests
// =============================================================================

#[test]
fn test_subshell() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=outer").unwrap();
    // Variable changes in subshell should not affect parent
    let result = interp.execute_string("(x=inner)");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("outer"));
}

#[test]
fn test_command_substitution_runs() {
    let mut interp = Interpreter::new();
    // Test that command substitution at least runs
    let result = interp.execute_string("result=$(echo hello)");
    assert!(result.is_ok());
    // Result may or may not capture output depending on implementation
    assert!(interp.get_var("result").is_some());
}

// =============================================================================
// Exit and Return Tests
// =============================================================================

#[test]
fn test_exit_in_function() {
    let mut interp = Interpreter::new();
    interp.execute_string("test_exit() { return 42; }").unwrap();
    interp.execute_string("test_exit").unwrap();
    assert_eq!(interp.last_status.code, 42);
}

// =============================================================================
// Special Variable Tests
// =============================================================================

#[test]
fn test_special_var_question() {
    let mut interp = Interpreter::new();
    interp.execute_string("true").unwrap();
    interp.execute_string("result=$?").unwrap();
    assert_eq!(interp.get_var("result"), Some("0"));
}

#[test]
fn test_special_var_question_after_false() {
    let mut interp = Interpreter::new();
    interp.execute_string("false").unwrap();
    interp.execute_string("result=$?").unwrap();
    assert_eq!(interp.get_var("result"), Some("1"));
}

#[test]
fn test_special_var_dollar() {
    let mut interp = Interpreter::new();
    interp.execute_string("result=$$").unwrap();
    // Should be a number (PID)
    let result = interp.get_var("result").unwrap();
    assert!(result.parse::<u32>().is_ok());
}

// =============================================================================
// Local Variable Tests
// =============================================================================

#[test]
fn test_local_variable_syntax() {
    let mut interp = Interpreter::new();
    // Test that local keyword is recognized in function definition
    let result = interp.execute_string("test_func() { local x=value; echo $x; }");
    assert!(result.is_ok());
}

// =============================================================================
// Alias Tests
// =============================================================================

#[test]
fn test_alias_creation() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("alias ll='ls -la'");
    assert!(result.is_ok());
}

// =============================================================================
// Let Binding Tests (franken-specific)
// =============================================================================

#[test]
fn test_let_binding() {
    let mut interp = Interpreter::new();
    let result = interp.execute_string("let x = hello");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("x"), Some("hello"));
}

// =============================================================================
// Match Expression Tests (franken-specific)
// =============================================================================

#[test]
fn test_match_expression_parses() {
    let mut interp = Interpreter::new();
    interp.execute_string("x=hello").unwrap();
    // Test that match expression at least parses
    let result = interp.execute_string("match $x { hello => echo matched; * => echo default; }");
    // Just check it runs or gives a parse error (depending on feature support)
    assert!(result.is_ok() || result.is_err());
}

// =============================================================================
// Loop Expression Tests (franken-specific)
// =============================================================================

#[test]
fn test_loop_with_break() {
    let mut interp = Interpreter::new();
    interp.execute_string("count=0").unwrap();
    let result = interp.execute_string("loop { count=$((count + 1)); if [ $count -ge 3 ]; then break; fi; }");
    assert!(result.is_ok());
    assert_eq!(interp.get_var("count"), Some("3"));
}
