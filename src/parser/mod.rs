//! Parser for Franken Shell syntax
//!
//! This module parses shell commands, control flow, and franken-specific extensions.
//!
//! ## Submodules
//! - `word` - Word and brace expansion parsing
//! - `compound` - Control flow statements (if, for, while, case, etc.)
//! - `extensions` - franken-specific and Fish-compatible syntax
//!
//! ## Optimizations
//! - Uses references for token access to avoid cloning
//! - Look-ahead buffer for repeated peeks
//! - Pre-allocated vectors with capacity hints

mod compound;
mod extensions;
mod word;

use crate::ast::*;
use crate::error::{JshError, Result};
use crate::lexer::Lexer;
use crate::token::{Span, Token, TokenKind};

/// Static EOF token to avoid repeated allocation
static EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span {
        start: 0,
        end: 0,
        line: 0,
        column: 0,
    },
};

/// Parser for shell syntax
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self::new(tokens))
    }

    // ========================================================================
    // Core token manipulation methods (optimized)
    // ========================================================================

    /// Get a reference to the current token (no cloning)
    #[inline]
    pub(crate) fn peek_ref(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&EOF_TOKEN)
    }

    /// Get a reference to the token at offset n (no cloning)
    #[inline]
    pub(crate) fn peek_nth_ref(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or(&EOF_TOKEN)
    }

    /// Get a reference to the current token kind (most common operation)
    #[inline]
    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.peek_ref().kind
    }

    /// Legacy peek that clones - use peek_ref() for performance when possible
    pub(crate) fn peek(&self) -> Token {
        self.peek_ref().clone()
    }

    /// Legacy peek_nth that clones - use peek_nth_ref() for performance
    pub(crate) fn peek_nth(&self, n: usize) -> Token {
        self.peek_nth_ref(n).clone()
    }

    /// Advance and return the consumed token
    pub(crate) fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let token = std::mem::replace(&mut self.tokens[self.pos], EOF_TOKEN.clone());
            self.pos += 1;
            token
        } else {
            EOF_TOKEN.clone()
        }
    }

    /// Advance without returning the token (faster when token not needed)
    #[inline]
    pub(crate) fn advance_skip(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    #[inline]
    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Result<Token> {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            Ok(self.advance())
        } else {
            Err(JshError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.peek_kind()),
            })
        }
    }

    /// Check if current token matches kind without cloning
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    /// Check if current token is one of the given kinds
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn check_any(&self, kinds: &[TokenKind]) -> bool {
        let current = self.peek_kind();
        kinds
            .iter()
            .any(|k| std::mem::discriminant(current) == std::mem::discriminant(k))
    }

    pub(crate) fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) {
            self.advance_skip();
        }
    }

    #[inline]
    pub(crate) fn current_span(&self) -> Span {
        self.peek_ref().span
    }

    // ========================================================================
    // Program and statement parsing
    // ========================================================================

    /// Parse a complete program
    pub fn parse_program(&mut self) -> Result<Program> {
        // Most scripts have ~10 top-level statements
        let mut statements = Vec::with_capacity(10);
        self.skip_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_complete_statement()?;
            if !matches!(stmt, Statement::Empty) {
                statements.push(stmt);
            }
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    /// Parse a complete statement (list or compound)
    fn parse_complete_statement(&mut self) -> Result<Statement> {
        self.parse_list()
    }

    /// Parse a list of pipelines (cmd && cmd || cmd)
    fn parse_list(&mut self) -> Result<Statement> {
        let first = self.parse_pipeline()?;

        // Most lists have 1-2 additional pipelines (command && command)
        let mut rest = Vec::with_capacity(2);
        while matches!(self.peek_kind(), TokenKind::And | TokenKind::Or) {
            let op = match self.advance().kind {
                TokenKind::And => ListOp::And,
                TokenKind::Or => ListOp::Or,
                _ => unreachable!(),
            };
            self.skip_newlines();
            let pipeline = self.parse_pipeline()?;
            rest.push((op, pipeline));
        }

        if rest.is_empty() {
            if first.commands.is_empty() {
                return Ok(Statement::Empty);
            }
            // If there's a single compound command without pipes, return it directly
            if first.commands.len() == 1
                && !first.negated
                && !first.background
                && first.commands[0].redirects.is_empty()
                && let CommandKind::Compound(stmt) = &first.commands[0].kind
            {
                return Ok(stmt.as_ref().clone());
            }
            Ok(Statement::Pipeline(first))
        } else {
            Ok(Statement::List(List { first, rest }))
        }
    }

    /// Parse a pipeline (cmd | cmd | cmd)
    fn parse_pipeline(&mut self) -> Result<Pipeline> {
        let negated = if matches!(self.peek_kind(), TokenKind::Not) {
            self.advance_skip();
            true
        } else {
            false
        };

        // Most pipelines have 1-3 commands
        let mut commands = Vec::with_capacity(3);

        // Check for compound commands first
        if let Some(stmt) = self.try_parse_compound()? {
            let cmd = Command {
                kind: CommandKind::Compound(Box::new(stmt)),
                redirects: self.parse_redirects()?,
            };
            commands.push(cmd);
        } else {
            // Parse simple command
            if let Some(cmd) = self.parse_command()? {
                commands.push(cmd);
            }
        }

        // Parse rest of pipeline
        while matches!(self.peek_kind(), TokenKind::Pipe | TokenKind::PipeErr) {
            self.advance_skip();
            self.skip_newlines();

            if let Some(stmt) = self.try_parse_compound()? {
                let cmd = Command {
                    kind: CommandKind::Compound(Box::new(stmt)),
                    redirects: self.parse_redirects()?,
                };
                commands.push(cmd);
            } else if let Some(cmd) = self.parse_command()? {
                commands.push(cmd);
            }
        }

        let background = if matches!(self.peek_kind(), TokenKind::Amp) {
            self.advance_skip();
            true
        } else {
            false
        };

        // Skip optional semicolon
        if matches!(self.peek().kind, TokenKind::Semi) {
            self.advance();
        }

        Ok(Pipeline {
            commands,
            negated,
            background,
        })
    }

    /// Try to parse a compound command
    fn try_parse_compound(&mut self) -> Result<Option<Statement>> {
        match self.peek().kind {
            TokenKind::If => Ok(Some(self.parse_if()?)),
            TokenKind::For => Ok(Some(self.parse_for()?)),
            TokenKind::While => Ok(Some(self.parse_while()?)),
            TokenKind::Until => Ok(Some(self.parse_until()?)),
            TokenKind::Case => Ok(Some(self.parse_case()?)),
            TokenKind::Select => Ok(Some(self.parse_select()?)),
            TokenKind::Function => Ok(Some(self.parse_function()?)),
            TokenKind::LBrace => Ok(Some(self.parse_brace_group()?)),
            TokenKind::LParen => Ok(Some(self.parse_subshell()?)),
            // franken-specific
            TokenKind::Match_ => Ok(Some(self.parse_match()?)),
            TokenKind::Loop => Ok(Some(self.parse_loop()?)),
            TokenKind::Let => Ok(Some(self.parse_let()?)),
            TokenKind::Const => Ok(Some(self.parse_const()?)),
            TokenKind::Try => Ok(Some(self.parse_try()?)),
            TokenKind::Fn => Ok(Some(self.parse_fn()?)),
            // Fish-compatible
            TokenKind::Begin => Ok(Some(self.parse_begin_block()?)),
            TokenKind::Switch => Ok(Some(self.parse_fish_switch()?)),
            TokenKind::Break => {
                self.advance();
                let n = if let TokenKind::Number(n) = self.peek().kind {
                    self.advance();
                    Some(n as usize)
                } else {
                    None
                };
                Ok(Some(Statement::Break(n)))
            }
            TokenKind::Continue => {
                self.advance();
                let n = if let TokenKind::Number(n) = self.peek().kind {
                    self.advance();
                    Some(n as usize)
                } else {
                    None
                };
                Ok(Some(Statement::Continue(n)))
            }
            TokenKind::Return => {
                self.advance();
                let value = if self.peek().is_word()
                    || matches!(
                        self.peek().kind,
                        TokenKind::String(_) | TokenKind::Number(_)
                    ) {
                    Some(self.parse_word()?)
                } else {
                    None
                };
                Ok(Some(Statement::Return(value)))
            }
            _ => Ok(None),
        }
    }

    // ========================================================================
    // Command parsing
    // ========================================================================

    /// Parse a simple command
    fn parse_command(&mut self) -> Result<Option<Command>> {
        // Most commands have 0-2 assignments and 0-2 redirects
        let mut assignments = Vec::with_capacity(2);
        let mut redirects = Vec::with_capacity(2);

        // Parse leading assignments and redirects
        loop {
            if self.peek().is_redirect() {
                redirects.push(self.parse_redirect()?);
            } else if self.is_assignment() {
                assignments.push(self.parse_assignment()?);
            } else {
                break;
            }
        }

        // Check if we have a command name
        if !self.is_word_start() {
            if assignments.is_empty() && redirects.is_empty() {
                return Ok(None);
            }
            // Assignment-only command
            return Ok(Some(Command {
                kind: CommandKind::Simple(SimpleCommand {
                    name: Word {
                        parts: vec![],
                        span: self.current_span(),
                    },
                    args: vec![],
                    assignments,
                }),
                redirects,
            }));
        }

        // Check for function definition: name() { ... } or name(param1, param2) { ... }
        if self.looks_like_function_def() {
            return Ok(Some(Command {
                kind: CommandKind::Compound(Box::new(self.parse_function_shorthand()?)),
                redirects,
            }));
        }

        let name = self.parse_word()?;
        // Most commands have 2-6 arguments
        let mut args = Vec::with_capacity(6);

        // Parse arguments
        loop {
            if self.peek().is_redirect() {
                redirects.push(self.parse_redirect()?);
            } else if self.is_word_start() {
                args.push(self.parse_word()?);
            } else {
                break;
            }
        }

        Ok(Some(Command {
            kind: CommandKind::Simple(SimpleCommand {
                name,
                args,
                assignments,
            }),
            redirects,
        }))
    }

    /// Check if current position looks like a function definition
    fn looks_like_function_def(&self) -> bool {
        // Must start with Word followed by (
        if !matches!(self.peek().kind, TokenKind::Word(_))
            || !matches!(self.peek_nth(1).kind, TokenKind::LParen)
        {
            return false;
        }

        // Scan forward to find the matching ) and check if followed by {
        let mut n = 2;
        let mut paren_depth = 1;

        loop {
            let token = self.peek_nth(n);
            match token.kind {
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        // Found matching ), check what follows (skip newlines)
                        let mut m = n + 1;
                        while matches!(self.peek_nth(m).kind, TokenKind::Newline) {
                            m += 1;
                        }
                        return matches!(self.peek_nth(m).kind, TokenKind::LBrace);
                    }
                }
                TokenKind::Eof | TokenKind::Newline | TokenKind::Semi => {
                    // Definitely not a function definition
                    return false;
                }
                _ => {}
            }
            n += 1;
            if n > 100 {
                // Safety limit
                return false;
            }
        }
    }

    /// Check if current position looks like an assignment
    pub(crate) fn is_assignment(&self) -> bool {
        if let TokenKind::Word(name) = &self.peek().kind {
            // Valid variable name followed by =
            if name
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
                && matches!(
                    self.peek_nth(1).kind,
                    TokenKind::Assign | TokenKind::PlusAssign
                )
            {
                return true;
            }
        }
        false
    }

    /// Check if current position is a word start
    pub(crate) fn is_word_start(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Word(_)
                | TokenKind::String(_)
                | TokenKind::RawString(_)
                | TokenKind::Number(_)
                | TokenKind::Float(_)
                | TokenKind::Variable(_)
                | TokenKind::VariableBrace(_)
                | TokenKind::SpecialVar(_)
                | TokenKind::Glob(_)
                | TokenKind::LBracket  // [ as command (test builtin alias)
                | TokenKind::RBracket  // ] as word (end of [ command)
                | TokenKind::Assign    // = as word (for test string comparison)
                | TokenKind::Not       // ! as word (for test negation)
                | TokenKind::Ne        // != as word (for test)
                | TokenKind::Eq // == as word (for test)
        )
    }

    /// Parse an assignment
    fn parse_assignment(&mut self) -> Result<Assignment> {
        let span = self.current_span();
        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s.into_owned(),
            _ => return Err(JshError::syntax("Expected variable name")),
        };
        self.advance();

        let op = match self.peek().kind {
            TokenKind::Assign => AssignmentOp::Assign,
            TokenKind::PlusAssign => AssignmentOp::Append,
            _ => return Err(JshError::syntax("Expected assignment operator")),
        };
        self.advance();

        let value = if self.is_word_start() {
            Some(self.parse_word()?)
        } else {
            None
        };

        Ok(Assignment {
            name,
            value,
            op,
            export: false,
            local: false,
            readonly: false,
            span,
        })
    }

    /// Parse redirections
    pub(crate) fn parse_redirects(&mut self) -> Result<Vec<Redirect>> {
        let mut redirects = Vec::new();
        while self.peek().is_redirect() {
            redirects.push(self.parse_redirect()?);
        }
        Ok(redirects)
    }

    /// Parse a single redirect
    fn parse_redirect(&mut self) -> Result<Redirect> {
        let span = self.current_span();
        let (kind, fd) = match self.peek().kind.clone() {
            TokenKind::RedirectIn => (RedirectKind::Input, Some(0)),
            TokenKind::RedirectOut => (RedirectKind::Output, Some(1)),
            TokenKind::RedirectAppend => (RedirectKind::Append, Some(1)),
            TokenKind::RedirectErr => (RedirectKind::Output, Some(2)),
            TokenKind::RedirectErrAppend => (RedirectKind::Append, Some(2)),
            TokenKind::RedirectBoth => (RedirectKind::Output, None),
            TokenKind::RedirectBothAppend => (RedirectKind::Append, None),
            TokenKind::HereDoc => (RedirectKind::HereDoc, Some(0)),
            TokenKind::HereString => (RedirectKind::HereString, Some(0)),
            TokenKind::RedirectFd(n, m) => {
                self.advance();
                // Determine direction based on source fd convention:
                // n>&m is output duplication (fd n points to where fd m points)
                // n<&m is input duplication
                // For simplicity, we use DupOutput for all fd duplications
                // since the actual direction is determined by n
                let kind = if m == -1 {
                    // Close fd
                    RedirectKind::DupOutput
                } else {
                    RedirectKind::DupOutput
                };
                return Ok(Redirect {
                    kind,
                    fd: Some(n),
                    target: if m == -1 {
                        RedirectTarget::Close
                    } else {
                        RedirectTarget::Fd(m)
                    },
                    span,
                });
            }
            _ => return Err(JshError::syntax("Expected redirect")),
        };
        self.advance();

        let target = if kind == RedirectKind::HereString {
            RedirectTarget::HereString(self.parse_word()?)
        } else if kind == RedirectKind::HereDoc {
            // TODO: Handle heredocs properly
            let delimiter = self.parse_word()?;
            RedirectTarget::HereDoc {
                delimiter: delimiter.as_literal().unwrap_or("EOF").to_string(),
                content: String::new(),
                quoted: false,
            }
        } else {
            RedirectTarget::File(self.parse_word()?)
        };

        Ok(Redirect {
            kind,
            fd,
            target,
            span,
        })
    }
}
