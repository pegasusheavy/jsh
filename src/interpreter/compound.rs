//! Compound statement execution for Franken Shell interpreter

use crate::ast::{
    CaseStatement, ConstBinding, FishSwitchStatement, ForLoop, IfStatement, LetBinding,
    LoopStatement, MatchExpr, MatchPattern, SelectStatement, TryStatement, UntilLoop, WhileLoop,
};
use crate::error::{JshError, Result};
use crate::interpreter::{ExitStatus, Interpreter};
use std::io::{self, Write};

impl Interpreter {
    /// Execute if statement
    pub(crate) fn execute_if(&mut self, if_stmt: &IfStatement) -> Result<ExitStatus> {
        let cond_status = self.execute_statements(&if_stmt.condition)?;

        if cond_status.is_success() {
            self.execute_statements(&if_stmt.then_branch)
        } else {
            // Try elif branches
            for elif in &if_stmt.elif_branches {
                let elif_cond = self.execute_statements(&elif.0)?;
                if elif_cond.is_success() {
                    return self.execute_statements(&elif.1);
                }
            }
            // Else branch
            if let Some(else_branch) = &if_stmt.else_branch {
                self.execute_statements(else_branch)
            } else {
                Ok(ExitStatus::success())
            }
        }
    }

    /// Execute for loop
    pub(crate) fn execute_for(&mut self, for_loop: &ForLoop) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        // Expand items
        let items: Vec<String> = if let Some(item_words) = &for_loop.items {
            item_words
                .iter()
                .flat_map(|w| self.expand_word_with_glob(w).unwrap_or_default())
                .collect()
        } else {
            // If no items specified, use positional parameters
            self.positional_params.to_vec()
        };

        for item in items {
            self.set_var(&for_loop.var, &item);

            match self.execute_statements(&for_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute while loop
    pub(crate) fn execute_while(&mut self, while_loop: &WhileLoop) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            let cond = self.execute_statements(&while_loop.condition)?;
            if !cond.is_success() {
                break;
            }

            match self.execute_statements(&while_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute until loop
    pub(crate) fn execute_until(&mut self, until_loop: &UntilLoop) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            let cond = self.execute_statements(&until_loop.condition)?;
            if cond.is_success() {
                break;
            }

            match self.execute_statements(&until_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute case statement
    pub(crate) fn execute_case(&mut self, case_stmt: &CaseStatement) -> Result<ExitStatus> {
        let word = self.expand_word(&case_stmt.word)?;

        for arm in &case_stmt.arms {
            for pattern in &arm.patterns {
                let pattern_str = self.expand_word(pattern)?;
                if self.glob_match(&pattern_str, &word) || pattern_str == word {
                    return self.execute_statements(&arm.body);
                }
            }
        }

        Ok(ExitStatus::success())
    }

    /// Execute select statement
    pub(crate) fn execute_select(&mut self, select_stmt: &SelectStatement) -> Result<ExitStatus> {
        self.loop_depth += 1;

        // Expand items
        let items: Vec<String> = if let Some(item_words) = &select_stmt.items {
            item_words
                .iter()
                .flat_map(|w| self.expand_word_with_glob(w).unwrap_or_default())
                .collect()
        } else {
            // If no items specified, use positional parameters
            self.positional_params.to_vec()
        };

        loop {
            // Print menu
            for (i, item) in items.iter().enumerate() {
                eprintln!("{}) {}", i + 1, item);
            }

            // Print prompt and read input
            let prompt = self.get_var("PS3").unwrap_or("#? ");
            eprint!("{}", prompt);
            io::stderr().flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input)? == 0 {
                break; // EOF
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            if let Ok(num) = input.parse::<usize>() {
                if num > 0 && num <= items.len() {
                    self.set_var(&select_stmt.var, &items[num - 1]);
                    self.set_var("REPLY", input);

                    match self.execute_statements(&select_stmt.body) {
                        Ok(_) => {}
                        Err(JshError::Break) => break,
                        Err(JshError::Continue) => continue,
                        Err(e) => {
                            self.loop_depth -= 1;
                            return Err(e);
                        }
                    }
                }
            }
        }

        self.loop_depth -= 1;
        Ok(ExitStatus::success())
    }

    /// Execute Fish-style switch statement
    pub(crate) fn execute_fish_switch(
        &mut self,
        switch_stmt: &FishSwitchStatement,
    ) -> Result<ExitStatus> {
        let value = self.expand_word(&switch_stmt.value)?;

        for case in &switch_stmt.cases {
            for pattern in &case.patterns {
                let pattern_str = self.expand_word(pattern)?;
                // Fish uses glob matching
                if self.glob_match(&pattern_str, &value)
                    || pattern_str == "*"
                    || pattern_str == value
                {
                    return self.execute_statements(&case.body);
                }
            }
        }

        Ok(ExitStatus::success())
    }

    /// Execute franken match expression
    pub(crate) fn execute_match(&mut self, match_expr: &MatchExpr) -> Result<ExitStatus> {
        let value = self.expand_word(&match_expr.value)?;

        for arm in &match_expr.arms {
            // Check if pattern matches
            if self.match_pattern(&arm.pattern, &value)? {
                // Check guard if present
                if let Some(guard) = &arm.guard {
                    let guard_status = self.execute_statements(guard)?;
                    if !guard_status.is_success() {
                        continue;
                    }
                }

                return self.execute_statements(&arm.body);
            }
        }

        Ok(ExitStatus::success())
    }

    /// Check if a value matches a pattern
    pub(crate) fn match_pattern(&self, pattern: &MatchPattern, value: &str) -> Result<bool> {
        match pattern {
            MatchPattern::Wildcard => Ok(true),
            MatchPattern::Literal(word) => {
                let pattern_str = self.expand_word(word)?;
                Ok(pattern_str == value)
            }
            MatchPattern::Glob(glob_pattern) => Ok(self.glob_match(glob_pattern, value)),
            MatchPattern::Regex(regex_pattern) => {
                let re = regex::Regex::new(regex_pattern)?;
                Ok(re.is_match(value))
            }
            MatchPattern::Range {
                start,
                end,
                inclusive,
            } => {
                if let Ok(v) = value.parse::<i64>() {
                    if *inclusive {
                        Ok(v >= *start && v <= *end)
                    } else {
                        Ok(v >= *start && v < *end)
                    }
                } else {
                    Ok(false)
                }
            }
            MatchPattern::Or(patterns) => {
                for p in patterns {
                    if self.match_pattern(p, value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            MatchPattern::Bind { name: _, pattern } => self.match_pattern(pattern, value),
        }
    }

    /// Execute franken infinite loop
    pub(crate) fn execute_loop(&mut self, loop_stmt: &LoopStatement) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            match self.execute_statements(&loop_stmt.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute franken let binding
    pub(crate) fn execute_let(&mut self, let_binding: &LetBinding) -> Result<ExitStatus> {
        let value = self.expand_word(&let_binding.value)?;
        self.set_var(&let_binding.name, &value);
        Ok(ExitStatus::success())
    }

    /// Execute franken const binding
    pub(crate) fn execute_const(&mut self, const_binding: &ConstBinding) -> Result<ExitStatus> {
        let value = self.expand_word(&const_binding.value)?;
        self.consts.insert(const_binding.name.clone(), value);
        Ok(ExitStatus::success())
    }

    /// Execute franken try-catch-finally
    pub(crate) fn execute_try(&mut self, try_stmt: &TryStatement) -> Result<ExitStatus> {
        let result = self.execute_statements(&try_stmt.try_block);

        let status = match result {
            Ok(s) => s,
            Err(e) => {
                // Execute catch block
                if let Some(catch_block) = &try_stmt.catch_block {
                    if let Some(var) = &try_stmt.catch_var {
                        self.set_var(var, &e.to_string());
                    }
                    self.execute_statements(catch_block)?
                } else {
                    ExitStatus::failure(1)
                }
            }
        };

        // Execute finally block
        if let Some(finally_block) = &try_stmt.finally_block {
            self.execute_statements(finally_block)?;
        }

        Ok(status)
    }
}
