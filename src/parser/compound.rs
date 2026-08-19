//! Compound statement parsing: if, for, while, case, select, function, etc.
//!
//! Optimized to use references for token access.

use crate::ast::*;
use crate::error::{JshError, Result};
use crate::token::TokenKind;

use super::Parser;

impl Parser {
    // ========================================================================
    // Keyword detection helpers (optimized to use peek_kind)
    // ========================================================================

    /// Check if current token is the `in` keyword
    #[inline]
    pub(crate) fn is_in_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::In)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "in")
    }

    /// Check if current token is the `do` keyword
    #[inline]
    pub(crate) fn is_do_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Do)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "do")
    }

    /// Check if current token is the `done` keyword
    #[inline]
    pub(crate) fn is_done_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Done)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "done")
    }

    /// Check if current token is the `then` keyword
    #[inline]
    pub(crate) fn is_then_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Then)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "then")
    }

    /// Check if current token is the `fi` keyword
    #[inline]
    pub(crate) fn is_fi_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Fi)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "fi")
    }

    /// Check if current token is the `else` keyword
    #[inline]
    pub(crate) fn is_else_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Else)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "else")
    }

    /// Check if current token is the `elif` keyword
    #[inline]
    pub(crate) fn is_elif_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Elif)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "elif")
    }

    /// Check if current token is the `esac` keyword
    #[inline]
    pub(crate) fn is_esac_keyword(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Esac)
            || matches!(self.peek_kind(), TokenKind::Word(s) if s == "esac")
    }

    // ========================================================================
    // Compound list and terminators
    // ========================================================================

    /// Check if current token is a compound list terminator
    pub(crate) fn is_compound_list_terminator(&self) -> bool {
        if self.is_at_end() {
            return true;
        }

        // Check TokenKind first (uses peek_kind for no cloning)
        if matches!(
            self.peek_kind(),
            TokenKind::Then
                | TokenKind::Else
                | TokenKind::Elif
                | TokenKind::Fi
                | TokenKind::Do
                | TokenKind::Done
                | TokenKind::Esac
                | TokenKind::RBrace
                | TokenKind::RParen
                | TokenKind::DoubleSemi
                | TokenKind::When
                // Fish-compatible
                | TokenKind::End // Note: Case is NOT a terminator - it starts a new case statement
        ) {
            return true;
        }

        // Also check for keyword words
        if let TokenKind::Word(s) = self.peek_kind() {
            matches!(
                s.as_ref(),
                "then" | "else" | "elif" | "fi" | "do" | "done" | "esac" | "in"
            )
        } else {
            false
        }
    }

    /// Parse compound list (multiple statements)
    pub(crate) fn parse_compound_list(&mut self) -> Result<Vec<Statement>> {
        // Most compound lists have 3-8 statements
        let mut statements = Vec::with_capacity(8);
        self.skip_newlines();

        loop {
            // Check for terminators
            if self.is_compound_list_terminator() {
                break;
            }

            let stmt = self.parse_list()?;
            if !matches!(stmt, Statement::Empty) {
                statements.push(stmt);
            }
            self.skip_newlines();
        }

        Ok(statements)
    }

    // ========================================================================
    // Control flow statements
    // ========================================================================

    /// Parse if statement
    pub fn parse_if(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::If)?;
        self.skip_newlines();

        let condition = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_then_keyword() {
            return Err(JshError::syntax("Expected 'then' after if condition"));
        }
        self.advance();
        self.skip_newlines();

        let then_branch = self.parse_compound_list()?;

        let mut elif_branches = Vec::new();
        while self.is_elif_keyword() {
            self.advance();
            self.skip_newlines();
            let elif_cond = self.parse_compound_list()?;
            self.skip_newlines();
            if !self.is_then_keyword() {
                return Err(JshError::syntax("Expected 'then' after elif condition"));
            }
            self.advance();
            self.skip_newlines();
            let elif_body = self.parse_compound_list()?;
            elif_branches.push((elif_cond, elif_body));
        }

        let else_branch = if self.is_else_keyword() {
            self.advance();
            self.skip_newlines();
            Some(self.parse_compound_list()?)
        } else {
            None
        };

        self.skip_newlines();
        if !self.is_fi_keyword() {
            return Err(JshError::syntax("Expected 'fi' to end if statement"));
        }
        self.advance();

        Ok(Statement::If(IfStatement {
            condition,
            then_branch,
            elif_branches,
            else_branch,
            span,
        }))
    }

    /// Parse for loop
    pub fn parse_for(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::For)?;
        self.skip_newlines();

        let var = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected variable name after 'for'")),
        };
        self.advance();
        self.skip_newlines();

        let items = if self.is_in_keyword() {
            self.advance();
            let mut items = Vec::new();
            while self.is_word_start() {
                items.push(self.parse_word()?);
            }
            Some(items)
        } else {
            None
        };

        // Skip optional semicolon/newline before do
        while matches!(self.peek().kind, TokenKind::Semi | TokenKind::Newline) {
            self.advance();
        }

        if !self.is_do_keyword() {
            return Err(JshError::syntax("Expected 'do' in for loop"));
        }
        self.advance();
        self.skip_newlines();

        let body = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_done_keyword() {
            return Err(JshError::syntax("Expected 'done' to end for loop"));
        }
        self.advance();

        Ok(Statement::For(ForLoop {
            var,
            items,
            body,
            span,
            loop_id: crate::jit::LoopId::new(),
        }))
    }

    /// Parse while loop
    pub fn parse_while(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::While)?;
        self.skip_newlines();

        let condition = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_do_keyword() {
            return Err(JshError::syntax("Expected 'do' in while loop"));
        }
        self.advance();
        self.skip_newlines();

        let body = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_done_keyword() {
            return Err(JshError::syntax("Expected 'done' to end while loop"));
        }
        self.advance();

        Ok(Statement::While(WhileLoop {
            condition,
            body,
            span,
            loop_id: crate::jit::LoopId::new(),
        }))
    }

    /// Parse until loop
    pub fn parse_until(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Until)?;
        self.skip_newlines();

        let condition = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_do_keyword() {
            return Err(JshError::syntax("Expected 'do' in until loop"));
        }
        self.advance();
        self.skip_newlines();

        let body = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_done_keyword() {
            return Err(JshError::syntax("Expected 'done' to end until loop"));
        }
        self.advance();

        Ok(Statement::Until(UntilLoop {
            condition,
            body,
            span,
            loop_id: crate::jit::LoopId::new(),
        }))
    }

    /// Parse case statement
    pub fn parse_case(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Case)?;
        self.skip_newlines();

        let word = self.parse_word()?;
        self.skip_newlines();

        if !self.is_in_keyword() {
            return Err(JshError::syntax("Expected 'in' in case statement"));
        }
        self.advance();
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.is_esac_keyword() {
            // Parse patterns
            let mut patterns = Vec::new();

            // Skip optional (
            if matches!(self.peek().kind, TokenKind::LParen) {
                self.advance();
            }

            patterns.push(self.parse_word()?);
            while matches!(self.peek().kind, TokenKind::Pipe) {
                self.advance();
                patterns.push(self.parse_word()?);
            }

            self.expect(&TokenKind::RParen)?;
            self.skip_newlines();

            let body = self.parse_compound_list()?;

            let terminator = match self.peek().kind {
                TokenKind::DoubleSemi => {
                    self.advance();
                    CaseTerminator::Break
                }
                _ => CaseTerminator::Break,
            };

            self.skip_newlines();
            arms.push(CaseArm {
                patterns,
                body,
                terminator,
            });
        }

        if !self.is_esac_keyword() {
            return Err(JshError::syntax("Expected 'esac' to end case statement"));
        }
        self.advance();

        Ok(Statement::Case(CaseStatement { word, arms, span }))
    }

    /// Parse select statement
    pub fn parse_select(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Select)?;
        self.skip_newlines();

        let var = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected variable name after 'select'")),
        };
        self.advance();
        self.skip_newlines();

        let items = if self.is_in_keyword() {
            self.advance();
            let mut items = Vec::new();
            while self.is_word_start() {
                items.push(self.parse_word()?);
            }
            Some(items)
        } else {
            None
        };

        while matches!(self.peek().kind, TokenKind::Semi | TokenKind::Newline) {
            self.advance();
        }

        if !self.is_do_keyword() {
            return Err(JshError::syntax("Expected 'do' in select statement"));
        }
        self.advance();
        self.skip_newlines();

        let body = self.parse_compound_list()?;
        self.skip_newlines();

        if !self.is_done_keyword() {
            return Err(JshError::syntax("Expected 'done' to end select statement"));
        }
        self.advance();

        Ok(Statement::Select(SelectStatement {
            var,
            items,
            body,
            span,
        }))
    }

    // ========================================================================
    // Function definitions
    // ========================================================================

    /// Parse function definition
    pub fn parse_function(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Function)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected function name")),
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

        // Function body
        let body = if matches!(self.peek().kind, TokenKind::LBrace) {
            let stmt = self.parse_brace_group()?;
            vec![stmt]
        } else {
            self.parse_compound_list()?
        };

        Ok(Statement::Function(FunctionDef {
            name,
            params,
            body,
            local_vars: vec![],
            span,
        }))
    }

    /// Parse function shorthand: name() { ... } or name(param1, param2) { ... }
    pub fn parse_function_shorthand(&mut self) -> Result<Statement> {
        let span = self.current_span();
        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected function name")),
        };
        self.advance();
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_function_params()?;
        self.skip_newlines();

        let body = if matches!(self.peek().kind, TokenKind::LBrace) {
            let stmt = self.parse_brace_group()?;
            vec![stmt]
        } else {
            self.parse_compound_list()?
        };

        Ok(Statement::Function(FunctionDef {
            name,
            params,
            body,
            local_vars: vec![],
            span,
        }))
    }

    /// Parse function parameters: param1, param2, param3)
    /// Called after consuming the opening (
    pub fn parse_function_params(&mut self) -> Result<Vec<String>> {
        let mut params = Vec::new();

        // Handle empty params ()
        if matches!(self.peek().kind, TokenKind::RParen) {
            self.advance();
            return Ok(params);
        }

        loop {
            self.skip_newlines();

            // Parse parameter name
            let param = match self.peek().kind.clone() {
                TokenKind::Word(s) => s.into_owned(),
                TokenKind::RParen => break,
                _ => return Err(JshError::syntax("Expected parameter name or ')'")),
            };
            self.advance();
            params.push(param);

            self.skip_newlines();

            // Check for comma or end
            match self.peek().kind {
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::RParen => break,
                _ => return Err(JshError::syntax("Expected ',' or ')' after parameter")),
            }
        }

        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }

    // ========================================================================
    // Grouping constructs
    // ========================================================================

    /// Parse brace group { ... }
    pub fn parse_brace_group(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let body = self.parse_compound_list()?;
        self.skip_newlines();
        self.expect(&TokenKind::RBrace)?;
        Ok(Statement::BraceGroup(body))
    }

    /// Parse subshell ( ... )
    pub fn parse_subshell(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::LParen)?;
        self.skip_newlines();
        let body = self.parse_compound_list()?;
        self.skip_newlines();
        self.expect(&TokenKind::RParen)?;
        Ok(Statement::Subshell(body))
    }
}
