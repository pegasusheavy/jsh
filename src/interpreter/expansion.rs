//! Word expansion for jsh interpreter

use crate::ast::{BraceExpansion, CaseModifyMode, Word, WordPart};
use crate::error::Result;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use glob::glob;

impl Interpreter {
    /// Expand a word to a string
    pub fn expand_word(&self, word: &Word) -> Result<String> {
        let mut result = String::new();

        for part in &word.parts {
            match part {
                WordPart::Literal(s) => {
                    // Check for arithmetic expansion marker
                    if s.starts_with("$__ARITH__") && s.ends_with("__") {
                        let expr = &s[10..s.len() - 2];
                        match self.eval_arithmetic_const(expr) {
                            Ok(value) => result.push_str(&value.to_string()),
                            Err(_) => result.push_str("0"),
                        }
                    } else {
                        // Expand any embedded variables in the literal (from double-quoted strings)
                        result.push_str(&self.expand_string_variables(s));
                    }
                }
                WordPart::Variable(name) => {
                    if let Some(value) = self.get_var(name) {
                        result.push_str(value);
                    }
                }
                WordPart::BraceExpansion(expansion) => {
                    result.push_str(&self.expand_brace_expansion(expansion)?);
                }
                WordPart::CommandSub(stmts) => {
                    result.push_str(&self.expand_command_sub(stmts)?);
                }
                WordPart::BacktickSub(cmd) => {
                    let mut parser = Parser::from_str(cmd)?;
                    let program = parser.parse_program()?;
                    result.push_str(&self.expand_command_sub(&program.statements)?);
                }
                WordPart::SpecialVar(c) => {
                    result.push_str(&self.expand_special_var(*c));
                }
                WordPart::Glob(pattern) => {
                    // Don't expand globs here, just return the pattern
                    result.push_str(pattern);
                }
                _ => {}
            }
        }

        Ok(result)
    }

    /// Expand a word with glob expansion
    pub fn expand_word_with_glob(&self, word: &Word) -> Result<Vec<String>> {
        let expanded = self.expand_word(word)?;

        // Check if it contains glob characters
        if expanded.contains('*') || expanded.contains('?') || expanded.contains('[') {
            match glob(&expanded) {
                Ok(paths) => {
                    let matches: Vec<String> = paths
                        .filter_map(|r| r.ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();

                    if matches.is_empty() {
                        Ok(vec![expanded])
                    } else {
                        Ok(matches)
                    }
                }
                Err(_) => Ok(vec![expanded]),
            }
        } else {
            Ok(vec![expanded])
        }
    }

    /// Expand brace expansion
    pub(crate) fn expand_brace_expansion(&self, expansion: &BraceExpansion) -> Result<String> {
        match expansion {
            BraceExpansion::Simple(var) => Ok(self.get_var(var).unwrap_or("").to_string()),
            BraceExpansion::Default {
                var,
                default,
                null_or_unset,
            } => {
                let value = self.get_var(var);
                if *null_or_unset {
                    if value.is_none_or(|v| v.is_empty()) {
                        return self.expand_word(default);
                    }
                } else if value.is_none() {
                    return self.expand_word(default);
                }
                Ok(value.unwrap_or("").to_string())
            }
            BraceExpansion::Length(var) => {
                let value = self.get_var(var).unwrap_or("");
                Ok(value.len().to_string())
            }
            BraceExpansion::RemovePrefix {
                var,
                pattern,
                greedy: _,
            } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                // Simple implementation - just remove prefix if it matches
                if let Some(stripped) = value.strip_prefix(pattern) {
                    Ok(stripped.to_string())
                } else {
                    Ok(value)
                }
            }
            BraceExpansion::RemoveSuffix {
                var,
                pattern,
                greedy: _,
            } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                if let Some(stripped) = value.strip_suffix(pattern) {
                    Ok(stripped.to_string())
                } else {
                    Ok(value)
                }
            }
            BraceExpansion::CaseModify { var, mode } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                Ok(match mode {
                    CaseModifyMode::UpperFirst => {
                        let mut chars = value.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().chain(chars).collect(),
                        }
                    }
                    CaseModifyMode::UpperAll => value.to_uppercase(),
                    CaseModifyMode::LowerFirst => {
                        let mut chars = value.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_lowercase().chain(chars).collect(),
                        }
                    }
                    CaseModifyMode::LowerAll => value.to_lowercase(),
                })
            }
            _ => Ok(String::new()),
        }
    }

    /// Expand brace content from within a string (handles ${var:-default} etc)
    fn expand_brace_content(&self, content: &str) -> String {
        let content = content.trim();

        // Length ${#var}
        if let Some(var) = content.strip_prefix('#') {
            return self.get_var(var).unwrap_or("").len().to_string();
        }

        // Check for operators (in order of length to match longer first)
        // Default with null check :-
        if let Some(idx) = content.find(":-") {
            let var = &content[..idx];
            let default = &content[idx + 2..];
            let value = self.get_var(var);
            if value.is_none() || value.is_some_and(|v| v.is_empty()) {
                return self.expand_string_variables(default);
            }
            return value.unwrap_or("").to_string();
        }

        // Default without null check -
        if let Some(idx) = content.find('-') {
            // Make sure it's not :- or ## or %%
            if idx > 0 && !content[..idx].ends_with(':') {
                let var = &content[..idx];
                let default = &content[idx + 1..];
                if self.get_var(var).is_none() {
                    return self.expand_string_variables(default);
                }
                return self.get_var(var).unwrap_or("").to_string();
            }
        }

        // Alternative with null check :+
        if let Some(idx) = content.find(":+") {
            let var = &content[..idx];
            let alt = &content[idx + 2..];
            let value = self.get_var(var);
            if value.is_some_and(|v| !v.is_empty()) {
                return self.expand_string_variables(alt);
            }
            return String::new();
        }

        // Alternative without null check +
        if let Some(idx) = content.find('+') {
            if idx > 0 && !content[..idx].ends_with(':') {
                let var = &content[..idx];
                let alt = &content[idx + 1..];
                if self.get_var(var).is_some() {
                    return self.expand_string_variables(alt);
                }
                return String::new();
            }
        }

        // Greedy prefix removal ##
        if let Some(idx) = content.find("##") {
            let var = &content[..idx];
            let pattern = &content[idx + 2..];
            let value = self.get_var(var).unwrap_or("").to_string();
            // Greedy: remove longest matching prefix
            for i in (0..=value.len()).rev() {
                if self.glob_match(pattern, &value[..i]) {
                    return value[i..].to_string();
                }
            }
            return value;
        }

        // Non-greedy prefix removal #
        if let Some(idx) = content.find('#') {
            if idx > 0 {
                let var = &content[..idx];
                let pattern = &content[idx + 1..];
                let value = self.get_var(var).unwrap_or("").to_string();
                // Non-greedy: remove shortest matching prefix
                for i in 0..=value.len() {
                    if self.glob_match(pattern, &value[..i]) {
                        return value[i..].to_string();
                    }
                }
                return value;
            }
        }

        // Greedy suffix removal %%
        if let Some(idx) = content.find("%%") {
            let var = &content[..idx];
            let pattern = &content[idx + 2..];
            let value = self.get_var(var).unwrap_or("").to_string();
            // Greedy: remove longest matching suffix
            for i in 0..=value.len() {
                if self.glob_match(pattern, &value[i..]) {
                    return value[..i].to_string();
                }
            }
            return value;
        }

        // Non-greedy suffix removal %
        if let Some(idx) = content.find('%') {
            if idx > 0 {
                let var = &content[..idx];
                let pattern = &content[idx + 1..];
                let value = self.get_var(var).unwrap_or("").to_string();
                // Non-greedy: remove shortest matching suffix
                for i in (0..=value.len()).rev() {
                    if self.glob_match(pattern, &value[i..]) {
                        return value[..i].to_string();
                    }
                }
                return value;
            }
        }

        // Simple variable expansion
        self.get_var(content).unwrap_or("").to_string()
    }

    /// Expand command substitution
    pub(crate) fn expand_command_sub(
        &self,
        _stmts: &[crate::ast::Statement],
    ) -> Result<String> {
        // This is tricky because we need a mutable self
        // In a real implementation, we'd fork and capture output
        // For now, just return empty string
        Ok(String::new())
    }

    /// Expand variables embedded in a string (from double-quoted strings)
    pub(crate) fn expand_string_variables(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '(' {
                        chars.next(); // consume first (
                        if chars.peek() == Some(&'(') {
                            // Arithmetic expansion $(())
                            chars.next(); // consume second (
                            let mut expr = String::new();
                            let mut depth = 1; // We're now inside the inner ((, tracking just the expression parens
                            while let Some(c) = chars.next() {
                                if c == '(' {
                                    depth += 1;
                                    expr.push(c);
                                } else if c == ')' {
                                    depth -= 1;
                                    if depth == 0 {
                                        // This is the first ) of ))
                                        // Consume the second )
                                        if chars.peek() == Some(&')') {
                                            chars.next();
                                        }
                                        break;
                                    }
                                    expr.push(c);
                                } else {
                                    expr.push(c);
                                }
                            }
                            // Evaluate the arithmetic expression
                            if let Ok(value) = self.eval_arithmetic_const(&expr) {
                                result.push_str(&value.to_string());
                            }
                        } else {
                            // Command substitution $(...)
                            let mut cmd = String::new();
                            let mut depth = 1;
                            while let Some(c) = chars.next() {
                                if c == '(' {
                                    depth += 1;
                                    cmd.push(c);
                                } else if c == ')' {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                    cmd.push(c);
                                } else {
                                    cmd.push(c);
                                }
                            }
                            // Command substitution would need fork/capture - not implemented in const context
                        }
                    } else if next == '{' {
                        // ${var} or ${var:-default} etc expansion
                        chars.next(); // consume {
                        let mut content = String::new();
                        let mut brace_depth = 1;
                        while let Some(&c) = chars.peek() {
                            if c == '{' {
                                brace_depth += 1;
                                content.push(c);
                            } else if c == '}' {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    chars.next();
                                    break;
                                }
                                content.push(c);
                            } else {
                                content.push(c);
                            }
                            chars.next();
                        }
                        // Handle parameter expansion operators
                        result.push_str(&self.expand_brace_content(&content));
                    } else if next.is_alphabetic() || next == '_' {
                        // $var expansion
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(value) = self.get_var(&var_name) {
                            result.push_str(value);
                        }
                    } else if next == '?'
                        || next == '!'
                        || next == '$'
                        || next == '#'
                        || next == '@'
                        || next == '*'
                        || next == '-'
                        || next.is_ascii_digit()
                    {
                        // Special variable
                        chars.next();
                        result.push_str(&self.expand_special_var(next));
                    } else {
                        result.push('$');
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Evaluate arithmetic without mutation (for use in const contexts)
    fn eval_arithmetic_const(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(0);
        }

        // Simple recursive descent for arithmetic with proper precedence
        self.eval_arith_ternary(expr)
    }

    fn eval_arith_ternary(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        // Find ? at top level for ternary
        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in 0..bytes.len() {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'?' if depth == 0 => {
                    // Find matching :
                    let mut colon_depth = 0;
                    for j in i + 1..bytes.len() {
                        match bytes[j] {
                            b'(' => colon_depth += 1,
                            b')' => colon_depth -= 1,
                            b':' if colon_depth == 0 => {
                                let cond = self.eval_arith_logical_or(&expr[..i])?;
                                if cond != 0 {
                                    return self.eval_arith_ternary(&expr[i + 1..j]);
                                } else {
                                    return self.eval_arith_ternary(&expr[j + 1..]);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        self.eval_arith_logical_or(expr)
    }

    fn eval_arith_logical_or(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();
        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in (1..bytes.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'|' if depth == 0 && i > 0 && bytes[i - 1] == b'|' => {
                    let left = self.eval_arith_logical_or(&expr[..i - 1])?;
                    if left != 0 {
                        return Ok(1);
                    }
                    let right = self.eval_arith_logical_and(&expr[i + 1..])?;
                    return Ok(if right != 0 { 1 } else { 0 });
                }
                _ => {}
            }
        }
        self.eval_arith_logical_and(expr)
    }

    fn eval_arith_logical_and(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();
        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in (1..bytes.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'&' if depth == 0 && i > 0 && bytes[i - 1] == b'&' => {
                    let left = self.eval_arith_logical_and(&expr[..i - 1])?;
                    if left == 0 {
                        return Ok(0);
                    }
                    let right = self.eval_arith_comparison(&expr[i + 1..])?;
                    return Ok(if right != 0 { 1 } else { 0 });
                }
                _ => {}
            }
        }
        self.eval_arith_comparison(expr)
    }

    fn eval_arith_comparison(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();
        let mut depth = 0;
        let bytes = expr.as_bytes();

        // Check for ==, !=, <=, >=, <, >
        for i in (1..bytes.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'=' if depth == 0 && i > 0 => {
                    if bytes[i - 1] == b'=' {
                        let left = self.eval_arith_comparison(&expr[..i - 1])?;
                        let right = self.eval_arith_additive(&expr[i + 1..])?;
                        return Ok(if left == right { 1 } else { 0 });
                    }
                    if bytes[i - 1] == b'!' {
                        let left = self.eval_arith_comparison(&expr[..i - 1])?;
                        let right = self.eval_arith_additive(&expr[i + 1..])?;
                        return Ok(if left != right { 1 } else { 0 });
                    }
                    if bytes[i - 1] == b'<' {
                        let left = self.eval_arith_comparison(&expr[..i - 1])?;
                        let right = self.eval_arith_additive(&expr[i + 1..])?;
                        return Ok(if left <= right { 1 } else { 0 });
                    }
                    if bytes[i - 1] == b'>' {
                        let left = self.eval_arith_comparison(&expr[..i - 1])?;
                        let right = self.eval_arith_additive(&expr[i + 1..])?;
                        return Ok(if left >= right { 1 } else { 0 });
                    }
                }
                b'<' if depth == 0 && (i == 0 || bytes[i - 1] != b'<') && (i + 1 >= bytes.len() || bytes[i + 1] != b'=') => {
                    let left = self.eval_arith_comparison(&expr[..i])?;
                    let right = self.eval_arith_additive(&expr[i + 1..])?;
                    return Ok(if left < right { 1 } else { 0 });
                }
                b'>' if depth == 0 && (i == 0 || bytes[i - 1] != b'>') && (i + 1 >= bytes.len() || bytes[i + 1] != b'=') => {
                    let left = self.eval_arith_comparison(&expr[..i])?;
                    let right = self.eval_arith_additive(&expr[i + 1..])?;
                    return Ok(if left > right { 1 } else { 0 });
                }
                _ => {}
            }
        }

        self.eval_arith_additive(expr)
    }

    fn eval_arith_additive(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        // Find last + or - at top level
        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in (1..bytes.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'+' | b'-' if depth == 0 => {
                    // Check it's not unary
                    let prev = bytes[i - 1];
                    if !matches!(prev, b'(' | b'+' | b'-' | b'*' | b'/' | b'%' | b'<' | b'>' | b'=' | b'!' | b'&' | b'|') {
                        let left = self.eval_arith_additive(&expr[..i])?;
                        let right = self.eval_arith_multiplicative(&expr[i + 1..])?;
                        return Ok(if bytes[i] == b'+' { left + right } else { left - right });
                    }
                }
                _ => {}
            }
        }

        self.eval_arith_multiplicative(expr)
    }

    fn eval_arith_multiplicative(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in (1..bytes.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'*' if depth == 0 => {
                    // Check it's not **
                    if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                        continue;
                    }
                    if i > 0 && bytes[i - 1] == b'*' {
                        // This is the second * of **
                        continue;
                    }
                    let left = self.eval_arith_multiplicative(&expr[..i])?;
                    let right = self.eval_arith_power(&expr[i + 1..])?;
                    return Ok(left * right);
                }
                b'/' if depth == 0 => {
                    let left = self.eval_arith_multiplicative(&expr[..i])?;
                    let right = self.eval_arith_power(&expr[i + 1..])?;
                    return if right != 0 { Ok(left / right) } else { Ok(0) };
                }
                b'%' if depth == 0 => {
                    let left = self.eval_arith_multiplicative(&expr[..i])?;
                    let right = self.eval_arith_power(&expr[i + 1..])?;
                    return if right != 0 { Ok(left % right) } else { Ok(0) };
                }
                _ => {}
            }
        }

        self.eval_arith_power(expr)
    }

    fn eval_arith_power(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        // ** is right-associative, so scan left to right
        let mut depth = 0;
        let bytes = expr.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'*' if depth == 0 && bytes.get(i + 1) == Some(&b'*') => {
                    let base = self.eval_arith_unary(&expr[..i])?;
                    let exp = self.eval_arith_power(&expr[i + 2..])?;
                    return Ok(base.pow(exp as u32));
                }
                _ => {}
            }
        }

        self.eval_arith_unary(expr)
    }

    fn eval_arith_unary(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        if expr.starts_with('-') {
            return Ok(-self.eval_arith_unary(&expr[1..])?);
        }
        if expr.starts_with('+') {
            return self.eval_arith_unary(&expr[1..]);
        }
        if expr.starts_with('!') {
            let val = self.eval_arith_unary(&expr[1..])?;
            return Ok(if val == 0 { 1 } else { 0 });
        }
        if expr.starts_with('~') {
            return Ok(!self.eval_arith_unary(&expr[1..])?);
        }

        self.eval_arith_primary(expr)
    }

    fn eval_arith_primary(&self, expr: &str) -> Result<i64> {
        let expr = expr.trim();

        // Parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            return self.eval_arith_ternary(&expr[1..expr.len() - 1]);
        }

        // Variable
        if expr.starts_with('$') {
            let var = &expr[1..];
            return Ok(self.get_var(var)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0));
        }

        // Bare variable name
        if expr.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
            && expr.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return Ok(self.get_var(expr)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0));
        }

        // Number
        if let Some(hex) = expr.strip_prefix("0x").or_else(|| expr.strip_prefix("0X")) {
            return i64::from_str_radix(hex, 16).map_err(|_| crate::error::JshError::Arithmetic(format!("invalid hex: {}", expr)));
        }

        expr.parse().map_err(|_| crate::error::JshError::Arithmetic(format!("invalid number: {}", expr)))
    }

    /// Expand special variable
    pub(crate) fn expand_special_var(&self, c: char) -> String {
        match c {
            '?' => self.last_status.code.to_string(),
            '!' => self.last_bg_pid.map(|p| p.to_string()).unwrap_or_default(),
            '$' => self.shell_pid.to_string(),
            '#' => self.positional_params.len().to_string(),
            '@' | '*' => self.positional_params.join(" "),
            '-' => {
                // Return current shell option flags (POSIX)
                let mut flags = String::new();
                if self.options.errexit { flags.push('e'); }
                if self.options.nounset { flags.push('u'); }
                if self.options.xtrace { flags.push('x'); }
                if self.options.noexec { flags.push('n'); }
                if self.options.allexport { flags.push('a'); }
                if self.options.noclobber { flags.push('C'); }
                if self.options.notify { flags.push('b'); }
                if self.options.noglob { flags.push('f'); }
                if atty::is(atty::Stream::Stdin) { flags.push('i'); } // interactive
                flags
            }
            '_' => {
                // Last argument of previous command - get from env
                self.get_var("_").unwrap_or("").to_string()
            }
            '0' => "jsh".to_string(),
            c @ '1'..='9' => {
                let idx = (c as usize) - ('1' as usize);
                self.positional_params
                    .get(idx)
                    .cloned()
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    /// Simple glob matching
    pub fn glob_match(&self, pattern: &str, value: &str) -> bool {
        // Convert shell glob to regex
        let mut regex_pattern = String::from("^");
        for c in pattern.chars() {
            match c {
                '*' => regex_pattern.push_str(".*"),
                '?' => regex_pattern.push('.'),
                '.' | '+' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '$' | '|' | '\\' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(c);
                }
                _ => regex_pattern.push(c),
            }
        }
        regex_pattern.push('$');

        regex::Regex::new(&regex_pattern)
            .map(|re| re.is_match(value))
            .unwrap_or(false)
    }
}

