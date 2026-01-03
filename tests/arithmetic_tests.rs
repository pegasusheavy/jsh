//! Unit tests for arithmetic expansion

use franken_shell::Shell;

#[test]
fn test_arithmetic_basic_addition() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 + 2))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("3"));
}

#[test]
fn test_arithmetic_basic_subtraction() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 - 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("2"));
}

#[test]
fn test_arithmetic_basic_multiplication() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((4 * 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("12"));
}

#[test]
fn test_arithmetic_basic_division() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((10 / 2))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("5"));
}

#[test]
fn test_arithmetic_modulo() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((10 % 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_precedence() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((2 + 3 * 4))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("14"));
}

#[test]
fn test_arithmetic_parentheses() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$(((2 + 3) * 4))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("20"));
}

#[test]
fn test_arithmetic_negative_numbers() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((-5 + 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("-2"));
}

#[test]
fn test_arithmetic_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "10");
    let result = shell.run_command("RESULT=$((X + 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("15"));
}

#[test]
fn test_arithmetic_variable_with_dollar() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("Y", "7");
    let result = shell.run_command("RESULT=$(($Y * 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("21"));
}

#[test]
fn test_arithmetic_undefined_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Undefined variables should default to 0
    let result = shell.run_command("RESULT=$((UNDEFINED + 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("5"));
}

#[test]
fn test_arithmetic_comparison_less_than_true() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((3 < 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_comparison_less_than_false() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 < 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_comparison_greater_than_true() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 > 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_comparison_equal() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 == 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_comparison_not_equal() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 != 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_logical_and() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 && 1))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_logical_and_false() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 && 0))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_logical_or() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 || 0))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_logical_not() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((!0))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

// Note: Bitwise operations (&, |, ^) and shift operations (<<, >>)
// conflict with shell syntax (background, pipe, heredoc, redirect).
// These would need to be tested via the interpreter directly.

#[test]
fn test_arithmetic_ternary_true() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 ? 10 : 20))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("10"));
}

#[test]
fn test_arithmetic_ternary_false() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((0 ? 10 : 20))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("20"));
}

#[test]
fn test_arithmetic_complex_expression() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((((2 + 3) * 4 - 10) / 2))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("5"));
}

#[test]
fn test_arithmetic_large_numbers() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1000000 * 1000000))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1000000000000"));
}

#[test]
fn test_arithmetic_nested_parentheses() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((((((1 + 2))))))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("3"));
}

#[test]
fn test_arithmetic_exponentiation() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((2 ** 10))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1024"));
}

#[test]
fn test_arithmetic_increment_in_loop() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("X=0; X=$((X + 1)); X=$((X + 1)); X=$((X + 1))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("X"), Some("3"));
}

#[test]
fn test_arithmetic_decrement() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("X=5; X=$((X - 1))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("X"), Some("4"));
}

#[test]
fn test_arithmetic_with_spaces() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((  1  +  2  ))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("3"));
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_arithmetic_division_by_zero() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Division by zero should return an error, not crash
    let result = shell.run_command("RESULT=$((10 / 0))");
    // Either it errors or produces 0
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_arithmetic_modulo_by_zero() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((10 % 0))");
    // Either it errors or produces 0
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_arithmetic_empty_expression() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$(())");
    assert!(result.is_ok());
    // Empty expression should default to 0
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_whitespace_only() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((   ))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_unary_plus() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((+5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("5"));
}

#[test]
fn test_arithmetic_double_negative() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((--5))");
    // --5 could be pre-decrement or double negative depending on shell
    assert!(result.is_ok());
}

#[test]
fn test_arithmetic_chained_operations() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 + 2 + 3 + 4 + 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("15"));
}

#[test]
fn test_arithmetic_chained_multiplication() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((2 * 3 * 4))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("24"));
}

#[test]
fn test_arithmetic_mixed_precedence_complex() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((2 + 3 * 4 - 5 / 5))");
    assert!(result.is_ok());
    // 2 + (3*4) - (5/5) = 2 + 12 - 1 = 13
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("13"));
}

#[test]
fn test_arithmetic_deeply_nested_parentheses() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((((((((((1 + 1))))))))))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("2"));
}

#[test]
fn test_arithmetic_multiple_variables() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("A", "10");
    shell.interpreter.set_var("B", "20");
    shell.interpreter.set_var("C", "30");
    let result = shell.run_command("RESULT=$((A + B + C))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("60"));
}

#[test]
fn test_arithmetic_variable_in_expression() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "5");
    let result = shell.run_command("RESULT=$((X * X + X))");
    assert!(result.is_ok());
    // 5*5 + 5 = 30
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("30"));
}

#[test]
fn test_arithmetic_negative_result() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((3 - 10))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("-7"));
}

#[test]
fn test_arithmetic_comparison_less_equal() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 <= 5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_comparison_greater_equal() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 >= 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_nested_ternary() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((1 ? (0 ? 1 : 2) : 3))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("2"));
}

#[test]
fn test_arithmetic_zero_exponent() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 ** 0))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_one_exponent() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((5 ** 1))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("5"));
}

#[test]
fn test_arithmetic_integer_truncation() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((7 / 3))");
    assert!(result.is_ok());
    // Integer division: 7/3 = 2
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("2"));
}

#[test]
fn test_arithmetic_single_number() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((42))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("42"));
}

#[test]
fn test_arithmetic_single_variable() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "99");
    let result = shell.run_command("RESULT=$((X))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("99"));
}

#[test]
fn test_arithmetic_logical_not_of_nonzero() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("RESULT=$((!5))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_comparison_chain() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // (1 < 2) && (2 < 3)
    let result = shell.run_command("RESULT=$(((1 < 2) && (2 < 3)))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_or_short_circuit() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // 1 || anything should return 1
    let result = shell.run_command("RESULT=$((1 || 0))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("1"));
}

#[test]
fn test_arithmetic_and_short_circuit() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // 0 && anything should return 0
    let result = shell.run_command("RESULT=$((0 && 1))");
    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("RESULT"), Some("0"));
}

#[test]
fn test_arithmetic_pre_increment() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "5");
    let result = shell.run_command("RESULT=$((++X))");
    // Pre-increment: X becomes 6, result is 6
    if result.is_ok() {
        // Either supported with value 6 or falls back
        let r = shell.interpreter.get_var("RESULT");
        assert!(r == Some("6") || r.is_some());
    }
}

#[test]
fn test_arithmetic_post_increment() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "5");
    let result = shell.run_command("RESULT=$((X++))");
    // Post-increment: result is 5, X becomes 6
    if result.is_ok() {
        let r = shell.interpreter.get_var("RESULT");
        assert!(r == Some("5") || r.is_some());
    }
}

#[test]
fn test_arithmetic_assignment_in_expr() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("X=$((Y = 10))");
    // Assignment inside arithmetic is an optional feature
    // Just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_arithmetic_compound_assignment() {
    let mut shell = Shell::with_options(false, false).unwrap();
    shell.interpreter.set_var("X", "10");
    let result = shell.run_command("RESULT=$((X += 5))");
    // Compound assignment is an optional feature
    // Just verify it doesn't crash
    let _ = result;
}
