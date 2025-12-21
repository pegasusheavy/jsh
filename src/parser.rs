//! Parser for jsh shell syntax

use crate::ast::*;
use crate::error::{JshError, Result};
use crate::lexer::Lexer;
use crate::token::{Span, Token, TokenKind};

/// Parser for shell syntax
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn from_str(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self::new(tokens))
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or_else(Token::eof)
    }

    fn peek_nth(&self, n: usize) -> Token {
        self.tokens.get(self.pos + n).cloned().unwrap_or_else(Token::eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek();
        if !token.is_eof() {
            self.pos += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_eof()
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind) {
            Ok(self.advance())
        } else {
            Err(JshError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.peek().kind),
            })
        }
    }

    #[allow(dead_code)]
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }
    }

    fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Parse a complete program
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
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

        let mut rest = Vec::new();
        while matches!(self.peek().kind, TokenKind::And | TokenKind::Or) {
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
            Ok(Statement::Pipeline(first))
        } else {
            Ok(Statement::List(List { first, rest }))
        }
    }

    /// Parse a pipeline (cmd | cmd | cmd)
    fn parse_pipeline(&mut self) -> Result<Pipeline> {
        let negated = if matches!(self.peek().kind, TokenKind::Not) {
            self.advance();
            true
        } else {
            false
        };

        let mut commands = Vec::new();

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
        while matches!(self.peek().kind, TokenKind::Pipe | TokenKind::PipeErr) {
            self.advance();
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

        let background = if matches!(self.peek().kind, TokenKind::Amp) {
            self.advance();
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
            // jsh-specific
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
                let value = if self.peek().is_word() || matches!(self.peek().kind, TokenKind::String(_) | TokenKind::Number(_)) {
                    Some(self.parse_word()?)
                } else {
                    None
                };
                Ok(Some(Statement::Return(value)))
            }
            _ => Ok(None),
        }
    }

    /// Parse a simple command
    fn parse_command(&mut self) -> Result<Option<Command>> {
        let mut assignments = Vec::new();
        let mut redirects = Vec::new();

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
        let mut args = Vec::new();

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

    /// Check if current position looks like a function definition: name() { or name(params) {
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
    fn is_assignment(&self) -> bool {
        if let TokenKind::Word(name) = &self.peek().kind {
            // Valid variable name followed by =
            if name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                if matches!(
                    self.peek_nth(1).kind,
                    TokenKind::Assign | TokenKind::PlusAssign
                ) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if current position is a word start
    fn is_word_start(&self) -> bool {
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
        )
    }

    /// Parse a word (may contain multiple parts when adjacent without whitespace)
    fn parse_word(&mut self) -> Result<Word> {
        let span = self.current_span();
        let mut parts = Vec::new();

        // First, get the initial part based on the current token
        let first_part = match self.peek().kind.clone() {
            TokenKind::Word(s) => {
                self.advance();
                WordPart::Literal(s)
            }
            TokenKind::String(s) => {
                self.advance();
                WordPart::Literal(s)
            }
            TokenKind::RawString(s) => {
                self.advance();
                WordPart::Literal(s)
            }
            TokenKind::Number(n) => {
                self.advance();
                WordPart::Literal(n.to_string())
            }
            TokenKind::Float(n) => {
                self.advance();
                WordPart::Literal(n.to_string())
            }
            TokenKind::Variable(name) => {
                self.advance();
                WordPart::Variable(name)
            }
            TokenKind::VariableBrace(content) => {
                self.advance();
                self.parse_brace_expansion(&content)?
            }
            TokenKind::SpecialVar(c) => {
                self.advance();
                WordPart::SpecialVar(c)
            }
            TokenKind::Glob(pattern) => {
                self.advance();
                WordPart::Glob(pattern)
            }
            _ => {
                return Err(JshError::syntax("Expected word"));
            }
        };
        parts.push(first_part);

        // For shell words, each token separated by whitespace is a distinct word.
        // We don't concatenate tokens in the parser - the lexer already handles
        // tokenization properly. If adjacent tokens need to be combined (like $VAR"text"),
        // that's done at the lexer level or during expansion.

        Ok(Word { parts, span })
    }

    /// Parse brace expansion content
    fn parse_brace_expansion(&self, content: &str) -> Result<WordPart> {
        // Check for command substitution $()
        if content.starts_with("$(") && content.ends_with(')') {
            let inner = &content[2..content.len() - 1];
            let mut parser = Parser::from_str(inner)?;
            let program = parser.parse_program()?;
            return Ok(WordPart::CommandSub(program.statements));
        }

        // Check for arithmetic $(())
        if content.starts_with("$((") && content.ends_with("))") {
            // For now, just return as literal
            return Ok(WordPart::Literal(content.to_string()));
        }

        // Parse variable expansion
        let content = content.trim();

        // Length ${#var}
        if content.starts_with('#') {
            return Ok(WordPart::BraceExpansion(BraceExpansion::Length(
                content[1..].to_string(),
            )));
        }

        // Check for operators in order of specificity (longer operators first)
        // Default with null check
        if let Some(idx) = content.find(":-") {
            let var = &content[..idx];
            let rest = &content[idx + 2..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::Default {
                var: var.to_string(),
                default: Box::new(Word::literal(rest, Span::default())),
                null_or_unset: true,
            }));
        }

        // Assign default with null check
        if let Some(idx) = content.find(":=") {
            let var = &content[..idx];
            let rest = &content[idx + 2..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::AssignDefault {
                var: var.to_string(),
                default: Box::new(Word::literal(rest, Span::default())),
                null_or_unset: true,
            }));
        }

        // Alternative with null check
        if let Some(idx) = content.find(":+") {
            let var = &content[..idx];
            let rest = &content[idx + 2..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::Alternative {
                var: var.to_string(),
                alternative: Box::new(Word::literal(rest, Span::default())),
                null_or_unset: true,
            }));
        }

        // Greedy prefix removal
        if let Some(idx) = content.find("##") {
            let var = &content[..idx];
            let rest = &content[idx + 2..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::RemovePrefix {
                var: var.to_string(),
                pattern: rest.to_string(),
                greedy: true,
            }));
        }

        // Greedy suffix removal
        if let Some(idx) = content.find("%%") {
            let var = &content[..idx];
            let rest = &content[idx + 2..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::RemoveSuffix {
                var: var.to_string(),
                pattern: rest.to_string(),
                greedy: true,
            }));
        }

        // Uppercase all
        if let Some(idx) = content.find("^^") {
            let var = &content[..idx];
            return Ok(WordPart::BraceExpansion(BraceExpansion::CaseModify {
                var: var.to_string(),
                mode: CaseModifyMode::UpperAll,
            }));
        }

        // Lowercase all
        if let Some(idx) = content.find(",,") {
            let var = &content[..idx];
            return Ok(WordPart::BraceExpansion(BraceExpansion::CaseModify {
                var: var.to_string(),
                mode: CaseModifyMode::LowerAll,
            }));
        }

        // Default without null check (must check after :-)
        if let Some(idx) = content.find('-') {
            if !content[..idx].contains(':') {
                let var = &content[..idx];
                let rest = &content[idx + 1..];
                return Ok(WordPart::BraceExpansion(BraceExpansion::Default {
                    var: var.to_string(),
                    default: Box::new(Word::literal(rest, Span::default())),
                    null_or_unset: false,
                }));
            }
        }

        // Assign default without null check
        if let Some(idx) = content.find('=') {
            if !content[..idx].contains(':') {
                let var = &content[..idx];
                let rest = &content[idx + 1..];
                return Ok(WordPart::BraceExpansion(BraceExpansion::AssignDefault {
                    var: var.to_string(),
                    default: Box::new(Word::literal(rest, Span::default())),
                    null_or_unset: false,
                }));
            }
        }

        // Alternative without null check
        if let Some(idx) = content.find('+') {
            if !content[..idx].contains(':') {
                let var = &content[..idx];
                let rest = &content[idx + 1..];
                return Ok(WordPart::BraceExpansion(BraceExpansion::Alternative {
                    var: var.to_string(),
                    alternative: Box::new(Word::literal(rest, Span::default())),
                    null_or_unset: false,
                }));
            }
        }

        // Non-greedy prefix removal
        if let Some(idx) = content.find('#') {
            let var = &content[..idx];
            let rest = &content[idx + 1..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::RemovePrefix {
                var: var.to_string(),
                pattern: rest.to_string(),
                greedy: false,
            }));
        }

        // Non-greedy suffix removal
        if let Some(idx) = content.find('%') {
            let var = &content[..idx];
            let rest = &content[idx + 1..];
            return Ok(WordPart::BraceExpansion(BraceExpansion::RemoveSuffix {
                var: var.to_string(),
                pattern: rest.to_string(),
                greedy: false,
            }));
        }

        // Uppercase first
        if let Some(idx) = content.find('^') {
            let var = &content[..idx];
            return Ok(WordPart::BraceExpansion(BraceExpansion::CaseModify {
                var: var.to_string(),
                mode: CaseModifyMode::UpperFirst,
            }));
        }

        // Lowercase first
        if let Some(idx) = content.find(',') {
            let var = &content[..idx];
            return Ok(WordPart::BraceExpansion(BraceExpansion::CaseModify {
                var: var.to_string(),
                mode: CaseModifyMode::LowerFirst,
            }));
        }

        // Simple variable
        Ok(WordPart::BraceExpansion(BraceExpansion::Simple(
            content.to_string(),
        )))
    }

    /// Parse an assignment
    fn parse_assignment(&mut self) -> Result<Assignment> {
        let span = self.current_span();
        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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
    fn parse_redirects(&mut self) -> Result<Vec<Redirect>> {
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
                return Ok(Redirect {
                    kind: RedirectKind::DupOutput,
                    fd: Some(n),
                    target: RedirectTarget::Fd(m),
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

    /// Parse if statement
    fn parse_if(&mut self) -> Result<Statement> {
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

    /// Check if current token is the `in` keyword (either as TokenKind::In or Word("in"))
    fn is_in_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::In)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "in")
    }

    /// Check if current token is the `do` keyword
    fn is_do_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Do)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "do")
    }

    /// Check if current token is the `done` keyword
    fn is_done_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Done)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "done")
    }

    /// Check if current token is the `then` keyword
    fn is_then_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Then)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "then")
    }

    /// Check if current token is the `fi` keyword
    fn is_fi_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Fi)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "fi")
    }

    /// Check if current token is the `else` keyword
    fn is_else_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Else)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "else")
    }

    /// Check if current token is the `elif` keyword
    fn is_elif_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Elif)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "elif")
    }

    /// Check if current token is the `esac` keyword
    fn is_esac_keyword(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Esac)
            || matches!(&self.peek().kind, TokenKind::Word(s) if s == "esac")
    }

    /// Parse for loop
    fn parse_for(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::For)?;
        self.skip_newlines();

        let var = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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
        }))
    }

    /// Parse while loop
    fn parse_while(&mut self) -> Result<Statement> {
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
        }))
    }

    /// Parse until loop
    fn parse_until(&mut self) -> Result<Statement> {
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
        }))
    }

    /// Parse case statement
    fn parse_case(&mut self) -> Result<Statement> {
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
    fn parse_select(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Select)?;
        self.skip_newlines();

        let var = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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

    /// Parse function definition
    fn parse_function(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Function)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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
    fn parse_function_shorthand(&mut self) -> Result<Statement> {
        let span = self.current_span();
        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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

    /// Parse brace group { ... }
    fn parse_brace_group(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::LBrace)?;
        self.skip_newlines();
        let body = self.parse_compound_list()?;
        self.skip_newlines();
        self.expect(&TokenKind::RBrace)?;
        Ok(Statement::BraceGroup(body))
    }

    /// Parse subshell ( ... )
    fn parse_subshell(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::LParen)?;
        self.skip_newlines();
        let body = self.parse_compound_list()?;
        self.skip_newlines();
        self.expect(&TokenKind::RParen)?;
        Ok(Statement::Subshell(body))
    }

    /// Check if current token is a compound list terminator
    fn is_compound_list_terminator(&self) -> bool {
        if self.is_at_end() {
            return true;
        }

        // Check TokenKind first
        if matches!(
            self.peek().kind,
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
                | TokenKind::End
                | TokenKind::Case
        ) {
            return true;
        }

        // Also check for keyword words
        if let TokenKind::Word(s) = &self.peek().kind {
            matches!(s.as_str(), "then" | "else" | "elif" | "fi" | "do" | "done" | "esac" | "in")
        } else {
            false
        }
    }

    /// Parse compound list (multiple statements)
    fn parse_compound_list(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        self.skip_newlines();

        loop {
            // Check for terminators
            if self.is_compound_list_terminator() {
                break;
            }

            let stmt = self.parse_complete_statement()?;
            if !matches!(stmt, Statement::Empty) {
                statements.push(stmt);
            }
            self.skip_newlines();
        }

        Ok(statements)
    }

    // ========================================================================
    // jsh-specific enhanced syntax
    // ========================================================================

    /// Parse jsh match expression
    fn parse_match(&mut self) -> Result<Statement> {
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
            vec![self.parse_complete_statement()?]
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
        if let TokenKind::Word(s) = &self.peek().kind {
            if s == "_" {
                self.advance();
                return Ok(MatchPattern::Wildcard);
            }
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
        if let TokenKind::Word(s) = &self.peek().kind {
            if s.starts_with('/') && s.ends_with('/') && s.len() > 2 {
                let regex = s[1..s.len() - 1].to_string();
                self.advance();
                return Ok(MatchPattern::Regex(regex));
            }
        }

        // Check for range: start..end or start..=end
        if let TokenKind::Number(start) = self.peek().kind {
            let next = self.peek_nth(1);
            if let TokenKind::Word(s) = &next.kind {
                if s.starts_with("..") {
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
        }

        // Literal or glob pattern
        let word = self.parse_word()?;

        // Check if it's a glob
        for part in &word.parts {
            if let WordPart::Glob(pattern) = part {
                return Ok(MatchPattern::Glob(pattern.clone()));
            }
            if let WordPart::Literal(s) = part {
                if s.contains('*') || s.contains('?') || s.contains('[') {
                    return Ok(MatchPattern::Glob(s.clone()));
                }
            }
        }

        Ok(MatchPattern::Literal(word))
    }

    /// Parse jsh infinite loop
    fn parse_loop(&mut self) -> Result<Statement> {
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

        Ok(Statement::Loop(LoopStatement { body, span }))
    }

    /// Parse jsh let binding
    fn parse_let(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Let)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
            _ => return Err(JshError::syntax("Expected variable name after 'let'")),
        };
        self.advance();

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_word()?;

        Ok(Statement::Let(LetBinding { name, value, span }))
    }

    /// Parse jsh const binding
    fn parse_const(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Const)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
            _ => return Err(JshError::syntax("Expected variable name after 'const'")),
        };
        self.advance();

        self.expect(&TokenKind::Assign)?;
        let value = self.parse_word()?;

        Ok(Statement::Const(ConstBinding { name, value, span }))
    }

    /// Parse jsh try-catch-finally
    fn parse_try(&mut self) -> Result<Statement> {
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
            vec![self.parse_complete_statement()?]
        };
        self.skip_newlines();

        // Optional catch block
        let (catch_var, catch_block) = if matches!(self.peek().kind, TokenKind::Catch) {
            self.advance();
            self.skip_newlines();

            let var = if let TokenKind::Word(s) = &self.peek().kind {
                let v = s.clone();
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
                vec![self.parse_complete_statement()?]
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
                vec![self.parse_complete_statement()?]
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

    /// Parse jsh fn (function shorthand)
    fn parse_fn(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Fn)?;
        self.skip_newlines();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
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

    /// Parse function parameters: param1, param2, param3)
    /// Called after consuming the opening (
    fn parse_function_params(&mut self) -> Result<Vec<String>> {
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
                TokenKind::Word(s) => s,
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
    // Fish-compatible parsing
    // ========================================================================

    /// Parse Fish-style begin...end block
    fn parse_begin_block(&mut self) -> Result<Statement> {
        self.expect(&TokenKind::Begin)?;
        self.skip_newlines();

        let body = self.parse_fish_body()?;

        self.skip_newlines();
        self.expect(&TokenKind::End)?;

        Ok(Statement::BeginBlock(body))
    }

    /// Parse Fish-style switch statement (switch...case...end)
    fn parse_fish_switch(&mut self) -> Result<Statement> {
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

    /// Parse Fish function definition (function name ... end)
    #[allow(dead_code)]
    fn parse_fish_function(&mut self) -> Result<Statement> {
        let span = self.current_span();
        self.expect(&TokenKind::Function)?;
        self.skip_whitespace();

        let name = match self.peek().kind.clone() {
            TokenKind::Word(s) => s,
            _ => return Err(JshError::syntax("Expected function name")),
        };
        self.advance();

        // Parse optional argument names (Fish style --argument-names or jsh style (params))
        let mut params = Vec::new();
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance();
            params = self.parse_function_params()?;
        } else {
            // Check for Fish's --argument-names flag
            while let TokenKind::Word(ref word) = self.peek().kind {
                if word == "--argument-names" || word == "-a" {
                    self.advance();
                    // Collect argument names until newline or end
                    while let TokenKind::Word(param) = self.peek().kind.clone() {
                        if param.starts_with('-') || param == "end" {
                            break;
                        }
                        params.push(param);
                        self.advance();
                    }
                } else if word.starts_with('-') {
                    // Skip other Fish function flags like --description
                    self.advance();
                    // Skip the flag value if it's a word
                    if matches!(self.peek().kind, TokenKind::Word(_) | TokenKind::String(_)) {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
        }
        self.skip_newlines();

        // Check for 'end' keyword terminator (Fish style)
        // or brace/compound list terminator (Bash style)
        let body = if matches!(self.peek().kind, TokenKind::LBrace) {
            // Bash style: function name { ... }
            let stmt = self.parse_brace_group()?;
            vec![stmt]
        } else if self.is_fish_block_terminator() {
            // Empty function
            vec![]
        } else {
            // Fish style: function name ... end
            self.parse_fish_body()?
        };

        // If Fish style, expect 'end'
        if matches!(self.peek().kind, TokenKind::End) {
            self.advance();
        }

        Ok(Statement::Function(FunctionDef {
            name,
            params,
            body,
            local_vars: vec![],
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

            stmts.push(self.parse_complete_statement()?);
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

            stmts.push(self.parse_complete_statement()?);
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
    fn skip_whitespace(&mut self) {
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
