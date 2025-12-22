//! Arithmetic expression evaluation for POSIX $(()) expansion

use crate::error::{JshError, Result};
use crate::interpreter::Interpreter;

impl Interpreter {
    /// Evaluate an arithmetic expression string (POSIX $(()) syntax)
    pub fn eval_arithmetic(&mut self, expr: &str) -> Result<i64> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(0);
        }

        // Handle comma operator (evaluate all, return last)
        if let Some(pos) = find_top_level_char(expr, ',') {
            self.eval_arithmetic(&expr[..pos])?;
            return self.eval_arithmetic(&expr[pos + 1..]);
        }

        // Handle ternary operator
        if let Some(q_pos) = find_top_level_char(expr, '?') {
            if let Some(c_pos) = find_top_level_char(&expr[q_pos + 1..], ':') {
                let condition = self.eval_arithmetic(&expr[..q_pos])?;
                let then_part = &expr[q_pos + 1..q_pos + 1 + c_pos];
                let else_part = &expr[q_pos + 1 + c_pos + 1..];
                return if condition != 0 {
                    self.eval_arithmetic(then_part)
                } else {
                    self.eval_arithmetic(else_part)
                };
            }
        }

        // Handle assignment operators
        for (op_str, op_fn) in &[
            ("=", None::<fn(i64, i64) -> i64>),
            ("+=", Some(|a: i64, b: i64| a + b)),
            ("-=", Some(|a: i64, b: i64| a - b)),
            ("*=", Some(|a: i64, b: i64| a * b)),
            ("/=", Some(|a: i64, b: i64| if b != 0 { a / b } else { 0 })),
            ("%=", Some(|a: i64, b: i64| if b != 0 { a % b } else { 0 })),
            ("<<=", Some(|a: i64, b: i64| a << b)),
            (">>=", Some(|a: i64, b: i64| a >> b)),
            ("&=", Some(|a: i64, b: i64| a & b)),
            ("^=", Some(|a: i64, b: i64| a ^ b)),
            ("|=", Some(|a: i64, b: i64| a | b)),
        ] {
            if *op_str == "=" {
                // Simple assignment needs special handling to not match <=, >=, ==, !=
                if let Some(pos) = find_assignment_equals(expr) {
                    let var = expr[..pos].trim();
                    if is_valid_var_name(var) {
                        let value = self.eval_arithmetic(&expr[pos + 1..])?;
                        self.set_var(var, &value.to_string());
                        return Ok(value);
                    }
                }
            } else if let Some(pos) = expr.find(op_str) {
                let var = expr[..pos].trim();
                if is_valid_var_name(var) {
                    let current = self.get_var(var)
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    let value = self.eval_arithmetic(&expr[pos + op_str.len()..])?;
                    let result = op_fn.unwrap()(current, value);
                    self.set_var(var, &result.to_string());
                    return Ok(result);
                }
            }
        }

        // Handle logical OR ||
        if let Some(pos) = find_top_level_str(expr, "||") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            if left != 0 {
                return Ok(1);
            }
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if right != 0 { 1 } else { 0 });
        }

        // Handle logical AND &&
        if let Some(pos) = find_top_level_str(expr, "&&") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            if left == 0 {
                return Ok(0);
            }
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if right != 0 { 1 } else { 0 });
        }

        // Handle bitwise OR |
        if let Some(pos) = find_top_level_char(expr, '|') {
            // Make sure it's not ||
            if pos + 1 >= expr.len() || expr.as_bytes()[pos + 1] != b'|' {
                let left = self.eval_arithmetic(&expr[..pos])?;
                let right = self.eval_arithmetic(&expr[pos + 1..])?;
                return Ok(left | right);
            }
        }

        // Handle bitwise XOR ^
        if let Some(pos) = find_top_level_char(expr, '^') {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 1..])?;
            return Ok(left ^ right);
        }

        // Handle bitwise AND &
        if let Some(pos) = find_top_level_char(expr, '&') {
            // Make sure it's not &&
            if pos + 1 >= expr.len() || expr.as_bytes()[pos + 1] != b'&' {
                let left = self.eval_arithmetic(&expr[..pos])?;
                let right = self.eval_arithmetic(&expr[pos + 1..])?;
                return Ok(left & right);
            }
        }

        // Handle equality == and !=
        if let Some(pos) = find_top_level_str(expr, "==") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if left == right { 1 } else { 0 });
        }
        if let Some(pos) = find_top_level_str(expr, "!=") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if left != right { 1 } else { 0 });
        }

        // Handle relational operators (must check <= and >= before < and >)
        if let Some(pos) = find_top_level_str(expr, "<=") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if left <= right { 1 } else { 0 });
        }
        if let Some(pos) = find_top_level_str(expr, ">=") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(if left >= right { 1 } else { 0 });
        }

        // Handle shift operators (must check before < and >)
        if let Some(pos) = find_top_level_str(expr, "<<") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(left << right);
        }
        if let Some(pos) = find_top_level_str(expr, ">>") {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(left >> right);
        }

        // Handle < and > (after checking for << >> <= >=)
        if let Some(pos) = find_top_level_char(expr, '<') {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 1..])?;
            return Ok(if left < right { 1 } else { 0 });
        }
        if let Some(pos) = find_top_level_char(expr, '>') {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let right = self.eval_arithmetic(&expr[pos + 1..])?;
            return Ok(if left > right { 1 } else { 0 });
        }

        // Handle addition and subtraction (lowest precedence of arithmetic ops)
        // Scan from right to left to get left-to-right evaluation
        if let Some(pos) = find_top_level_additive(expr) {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let op = expr.as_bytes()[pos] as char;
            let right = self.eval_arithmetic(&expr[pos + 1..])?;
            return Ok(if op == '+' { left + right } else { left - right });
        }

        // Handle multiplication, division, modulo
        if let Some(pos) = find_top_level_multiplicative(expr) {
            let left = self.eval_arithmetic(&expr[..pos])?;
            let op = expr.as_bytes()[pos] as char;
            let right = self.eval_arithmetic(&expr[pos + 1..])?;
            return match op {
                '*' => Ok(left * right),
                '/' => {
                    if right == 0 {
                        Err(JshError::Arithmetic("division by zero".to_string()))
                    } else {
                        Ok(left / right)
                    }
                }
                '%' => {
                    if right == 0 {
                        Err(JshError::Arithmetic("modulo by zero".to_string()))
                    } else {
                        Ok(left % right)
                    }
                }
                _ => unreachable!(),
            };
        }

        // Handle exponentiation **
        if let Some(pos) = find_top_level_str(expr, "**") {
            let base = self.eval_arithmetic(&expr[..pos])?;
            let exp = self.eval_arithmetic(&expr[pos + 2..])?;
            return Ok(base.pow(exp as u32));
        }

        // Handle unary operators
        let expr = expr.trim();
        if expr.starts_with('!') {
            let operand = self.eval_arithmetic(&expr[1..])?;
            return Ok(if operand == 0 { 1 } else { 0 });
        }
        if expr.starts_with('~') {
            let operand = self.eval_arithmetic(&expr[1..])?;
            return Ok(!operand);
        }
        if expr.starts_with('-') && !expr[1..].starts_with(|c: char| c.is_ascii_digit()) {
            let operand = self.eval_arithmetic(&expr[1..])?;
            return Ok(-operand);
        }
        if expr.starts_with('+') && !expr[1..].starts_with(|c: char| c.is_ascii_digit()) {
            return self.eval_arithmetic(&expr[1..]);
        }

        // Handle pre-increment/decrement
        if expr.starts_with("++") {
            let var = expr[2..].trim();
            if is_valid_var_name(var) {
                let value = self.get_var(var)
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0) + 1;
                self.set_var(var, &value.to_string());
                return Ok(value);
            }
        }
        if expr.starts_with("--") {
            let var = expr[2..].trim();
            if is_valid_var_name(var) {
                let value = self.get_var(var)
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0) - 1;
                self.set_var(var, &value.to_string());
                return Ok(value);
            }
        }

        // Handle post-increment/decrement
        if expr.ends_with("++") {
            let var = expr[..expr.len() - 2].trim();
            if is_valid_var_name(var) {
                let value = self.get_var(var)
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                self.set_var(var, &(value + 1).to_string());
                return Ok(value);
            }
        }
        if expr.ends_with("--") {
            let var = expr[..expr.len() - 2].trim();
            if is_valid_var_name(var) {
                let value = self.get_var(var)
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                self.set_var(var, &(value - 1).to_string());
                return Ok(value);
            }
        }

        // Handle parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            return self.eval_arithmetic(&expr[1..expr.len() - 1]);
        }

        // Handle variables
        if expr.starts_with('$') {
            let var = &expr[1..];
            return Ok(self.get_var(var)
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0));
        }

        // Handle bare variable names
        if is_valid_var_name(expr) {
            return Ok(self.get_var(expr)
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0));
        }

        // Handle numeric literals
        if let Some(stripped) = expr.strip_prefix("0x").or_else(|| expr.strip_prefix("0X")) {
            // Hexadecimal
            return i64::from_str_radix(stripped, 16)
                .map_err(|_| JshError::Arithmetic(format!("invalid hex number: {}", expr)));
        }
        if let Some(stripped) = expr.strip_prefix("0b").or_else(|| expr.strip_prefix("0B")) {
            // Binary
            return i64::from_str_radix(stripped, 2)
                .map_err(|_| JshError::Arithmetic(format!("invalid binary number: {}", expr)));
        }
        if expr.starts_with('0') && expr.len() > 1 && expr.chars().all(|c| c.is_ascii_digit()) {
            // Octal
            return i64::from_str_radix(expr, 8)
                .map_err(|_| JshError::Arithmetic(format!("invalid octal number: {}", expr)));
        }

        // Decimal
        expr.parse::<i64>()
            .map_err(|_| JshError::Arithmetic(format!("invalid arithmetic expression: {}", expr)))
    }
}

/// Check if a string is a valid shell variable name
fn is_valid_var_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Find a character at the top level (not inside parentheses)
fn find_top_level_char(expr: &str, target: char) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in expr.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => depth -= 1,
            c if c == target && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Find a string at the top level
fn find_top_level_str(expr: &str, target: &str) -> Option<usize> {
    let mut depth = 0;
    let bytes = expr.as_bytes();
    let target_bytes = target.as_bytes();

    for i in (0..=expr.len().saturating_sub(target.len())).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            _ if depth == 0 && bytes[i..].starts_with(target_bytes) => return Some(i),
            _ => {}
        }
    }
    None
}

/// Find assignment equals (not part of ==, !=, <=, >=)
fn find_assignment_equals(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut depth = 0;

    for i in (0..expr.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'=' if depth == 0 => {
                // Check it's not ==, !=, <=, >=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
                if next != b'=' && !matches!(prev, b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^') {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Find additive operators (+ or -) at top level, scanning right to left
fn find_top_level_additive(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut depth = 0;

    for i in (1..expr.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'+' | b'-' if depth == 0 => {
                // Make sure it's not ++, --, +=, -=, or unary
                let prev = bytes[i - 1];
                let same = bytes[i];
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };

                // Skip if part of ++ or --
                if prev == same || next == same {
                    continue;
                }
                // Skip if part of += or -=
                if next == b'=' {
                    continue;
                }
                // Skip if previous char indicates unary (operator or start)
                if matches!(prev, b'(' | b',' | b'?' | b':' | b'<' | b'>' | b'=' | b'!' | b'&' | b'|' | b'^' | b'~' | b'+' | b'-' | b'*' | b'/' | b'%') {
                    continue;
                }
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

/// Find multiplicative operators (*, /, %) at top level
fn find_top_level_multiplicative(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut depth = 0;

    for i in (1..expr.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => depth -= 1,
            b'*' if depth == 0 => {
                // Make sure it's not ** or *=
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
                if next != b'*' && next != b'=' {
                    return Some(i);
                }
            }
            b'/' | b'%' if depth == 0 => {
                // Make sure it's not /= or %=
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
                if next != b'=' {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

