//! Franken-specific and Fish-compatible syntax extensions

use crate::ast::*;
use crate::error::{JshError, Result};
use crate::token::TokenKind;

use super::Parser;

impl Parser {
    // ========================================================================
    // Franken-specific enhanced syntax
    // ========================================================================

    /// Parse franken match expression
    pub fn parse_match(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Match_)?;
        self.skip_newlines();

        let value = self.parse_word()?;
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            let arm = self.parse_match_arm()?;
            arms.push(arm);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;

        Ok(Statement::Match(MatchExpr { value, arms, span }))
    }

    /// Parse a match arm
    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pattern = self.parse_match_pattern()?;
        self.skip_newlines();

        // Optional guard: when condition
        let guard = if matches!(self.peek().kind, TokenKind::When) {
            self.advance();
            self.skip_newlines();
            Some(self.parse_compound_list()?)
        } else {
            None
        };

        // Arrow =>
        if matches!(self.peek().kind, TokenKind::Assign) {
            self.advance();
            if matches!(self.peek().kind, TokenKind::RedirectOut) {
                self.advance(); // consume >
            }
        }
        self.skip_newlines();

        // Body: either { ... } or single command
        let body = if matches!(self.peek().kind, TokenKind::LBrace) {
            self.advance();
            self.skip_newlines();
            let stmts = self.parse_compound_list()?;
            self.skip_newlines();
            self.expect(&TokenKind::RBrace)?;
            stmts
        } else {
            vec![self.parse_list()?]
        };

        Ok(MatchArm {
            pattern,
            guard,
            body,
        })
    }

    /// Parse a match pattern
    fn parse_match_pattern(&mut self) -> Result<MatchPattern> {
        // Check for wildcard
        if matches!(self.peek().kind, TokenKind::Star) {
            self.advance();
            return Ok(MatchPattern::Wildcard);
        }

        // Check for _ wildcard
        if let TokenKind::Word(s) = &self.peek().kind
            && s == "_"
        {
            self.advance();
            return Ok(MatchPattern::Wildcard);
        }

        // Parse first pattern
        let mut pattern = self.parse_single_match_pattern()?;

        // Check for | (or patterns)
        if matches!(self.peek().kind, TokenKind::Pipe) {
            let mut patterns = vec![pattern];
            while matches!(self.peek().kind, TokenKind::Pipe) {
                self.advance();
                self.skip_newlines();
                patterns.push(self.parse_single_match_pattern()?);
            }
            pattern = MatchPattern::Or(patterns);
        }

        Ok(pattern)
    }

    /// Parse a single match pattern
    fn parse_single_match_pattern(&mut self) -> Result<MatchPattern> {
        // Check for regex pattern /.../
        if let TokenKind::Word(s) = &self.peek().kind
            && s.starts_with('/')
            && s.ends_with('/')
            && s.len() > 2
        {
            let regex = s[1..s.len() - 1].to_string();
            self.advance();
            return Ok(MatchPattern::Regex(regex));
        }

        // Check for range: start..end or start..=end
        if let TokenKind::Number(start) = self.peek().kind {
            let next = self.peek_nth(1);
            if let TokenKind::Word(s) = &next.kind
                && s.starts_with("..")
            {
                self.advance(); // consume start
                let inclusive = s.starts_with("..=");
                self.advance(); // consume ..

                if let TokenKind::Number(end) = self.peek().kind {
                    self.advance();
                    return Ok(MatchPattern::Range {
                        start,
                        end,
                        inclusive,
                    });
                }
            }
        }

        // Literal or glob pattern
        let word = self.parse_word()?;

        // Check if it's a glob
        for part in &word.parts {
            if let WordPart::Glob(pattern) = part {
                return Ok(MatchPattern::Glob(pattern.clone()));
            }
            if let WordPart::Literal(s) = part
                && (s.contains('*') || s.contains('?') || s.contains('['))
            {
                return Ok(MatchPattern::Glob(s.clone()));
            }
        }

        Ok(MatchPattern::Literal(word))
    }

    /// Parse franken infinite loop
    pub fn parse_loop(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Loop)?;
        self.skip_newlines();

        let body = if matches!(self.peek().kind, TokenKind::LBrace) {
            self.advance();
            self.skip_newlines();
            let stmts = self.parse_compound_list()?;
            self.skip_newlines();
            self.expect(&TokenKind::RBrace)?;
            stmts
        } else {
            self.expect(&TokenKind::Do)?;
            self.skip_newlines();
            let stmts = self.parse_compound_list()?;
            self.skip_newlines();
            self.expect(&TokenKind::Done)?;
            stmts
        };

        Ok(Statement::Loop(LoopStatement {
            body,
            span,
            loop_id: crate::jit::LoopId::new(),
        }))
    }

    /// Parse franken let binding
    pub fn parse_let(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Let)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected variable name after 'let'")),
        };
        self.advance();

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_word()?;

        Ok(Statement::Let(LetBinding { name, value, span }))
    }

    /// Parse franken const binding
    pub fn parse_const(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Const)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected variable name after 'const'")),
        };
        self.advance();

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_word()?;

        Ok(Statement::Const(ConstBinding { name, value, span }))
    }

    /// Parse franken try-catch-finally
    pub fn parse_try(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Try)?;
        self.skip_newlines();

        // Try block
        let try_block = if matches!(self.peek().kind, TokenKind::LBrace) {
            self.advance();
            self.skip_newlines();
            let stmts = self.parse_compound_list()?;
            self.skip_newlines();
            self.expect(&TokenKind::RBrace)?;
            stmts
        } else {
            vec![self.parse_list()?]
        };
        self.skip_newlines();

        // Optional catch block
        let (catch_var, catch_block) = if matches!(self.peek().kind, TokenKind::Catch) {
            self.advance();
            self.skip_newlines();

            let var = if let TokenKind::Word(s) = &self.peek().kind {
                let v = s.clone().into_owned();
                self.advance();
                self.skip_newlines();
                Some(v)
            } else {
                None
            };

            let block = if matches!(self.peek().kind, TokenKind::LBrace) {
                self.advance();
                self.skip_newlines();
                let stmts = self.parse_compound_list()?;
                self.skip_newlines();
                self.expect(&TokenKind::RBrace)?;
                stmts
            } else {
                vec![self.parse_list()?]
            };

            (var, Some(block))
        } else {
            (None, None)
        };
        self.skip_newlines();

        // Optional finally block
        let finally_block = if matches!(self.peek().kind, TokenKind::Finally) {
            self.advance();
            self.skip_newlines();

            let block = if matches!(self.peek().kind, TokenKind::LBrace) {
                self.advance();
                self.skip_newlines();
                let stmts = self.parse_compound_list()?;
                self.skip_newlines();
                self.expect(&TokenKind::RBrace)?;
                stmts
            } else {
                vec![self.parse_list()?]
            };

            Some(block)
        } else {
            None
        };

        Ok(Statement::Try(TryStatement {
            try_block,
            catch_var,
            catch_block,
            finally_block,
            span,
        }))
    }

    /// Parse franken fn (function shorthand)
    pub fn parse_fn(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Fn)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected function name after 'fn'")),
        };
        self.advance();

        // Optional (param1, param2, ...) or ()
        let params = if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance();
            self.parse_function_params()?
        } else {
            vec![]
        };
        self.skip_newlines();

        // Function body with { }
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let body = self.parse_compound_list()?;
        self.skip_newlines();
        self.expect(&TokenKind::RBrace)?;

        Ok(Statement::Function(FunctionDef {
            name,
            params,
            body,
            local_vars: vec![],
            span,
        }))
    }

    // ========================================================================
    // Fish-compatible parsing
    // ========================================================================

    /// Parse Fish-style begin...end block
    pub fn parse_begin_block(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::Begin)?;
        self.skip_newlines();

        let body = self.parse_fish_body()?;

        self.skip_newlines();
        self.expect(&TokenKind::End)?;

        Ok(Statement::BeginBlock(body))
    }

    /// Parse Fish-style switch statement (switch...case...end)
    pub fn parse_fish_switch(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Switch)?;
        self.skip_whitespace();

        let value = self.parse_word()?;
        self.skip_newlines();

        let mut cases = Vec::new();

        while matches!(self.peek().kind, TokenKind::Case) {
            self.advance(); // consume 'case'
            self.skip_whitespace();

            // Parse patterns
            let mut patterns = Vec::new();
            loop {
                if self.peek().is_separator() || matches!(self.peek().kind, TokenKind::Newline) {
                    break;
                }
                patterns.push(self.parse_word()?);
                self.skip_whitespace();
            }
            self.skip_newlines();

            // Parse body until next case or end
            let body = self.parse_fish_case_body()?;

            cases.push(FishCase { patterns, body });
        }

        self.skip_newlines();
        self.expect(&TokenKind::End)?;

        Ok(Statement::FishSwitch(FishSwitchStatement {
            value,
            cases,
            span,
        }))
    }

    /// Parse statements until Fish block terminator (end, case)
    fn parse_fish_body(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();

        while !self.is_fish_block_terminator() && !self.is_at_end() {
            self.skip_newlines();
            if self.is_fish_block_terminator() || self.is_at_end() {
                break;
            }

            stmts.push(self.parse_list()?);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    /// Parse Fish case body (until next case or end)
    fn parse_fish_case_body(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();

        while !matches!(self.peek().kind, TokenKind::Case | TokenKind::End) && !self.is_at_end() {
            self.skip_newlines();
            if matches!(self.peek().kind, TokenKind::Case | TokenKind::End) || self.is_at_end() {
                break;
            }

            stmts.push(self.parse_list()?);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    /// Check if current token is a Fish block terminator
    fn is_fish_block_terminator(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::End | TokenKind::Case | TokenKind::Else | TokenKind::Eof
        )
    }

    /// Skip whitespace (but not newlines)
    pub(crate) fn skip_whitespace(&mut self) {
        // The lexer already handles whitespace, so we just need to handle continuation
        while matches!(self.peek().kind, TokenKind::Newline) && self.pos + 1 < self.tokens.len() {
            if matches!(self.tokens[self.pos + 1].kind, TokenKind::Newline) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple_command() {
        let mut parser = Parser::from_str("echo hello world").unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_pipeline() {
        let mut parser = Parser::from_str("ls | grep test | wc -l").unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_if_statement() {
        let mut parser = Parser::from_str("if true; then echo yes; fi").unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Statement::If(_)));
    }

    #[test]
    fn test_for_loop() {
        let mut parser = Parser::from_str("for i in 1 2 3; do echo $i; done").unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Statement::For(_)));
    }

    #[test]
    fn test_match_expression() {
        let mut parser = Parser::from_str(
            r#"match $x {
                1 => echo one
                2 | 3 => echo two_or_three
                * => echo other
            }"#,
        )
        .unwrap();
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Statement::Match(_)));
    }
}
