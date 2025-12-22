//! Word and brace expansion parsing

use crate::ast::*;
use crate::error::{JshError, Result};
use crate::token::{Span, TokenKind};

use super::Parser;

impl Parser {
    /// Parse a word (may contain multiple parts when adjacent without whitespace)
    pub fn parse_word(&mut self) -> Result<Word> {
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
            TokenKind::LBracket => {
                // [ as word (for test builtin)
                self.advance();
                WordPart::Literal("[".to_string())
            }
            TokenKind::RBracket => {
                // ] as word (for test builtin)
                self.advance();
                WordPart::Literal("]".to_string())
            }
            TokenKind::Assign => {
                // = as word (for test builtin string comparison)
                self.advance();
                WordPart::Literal("=".to_string())
            }
            TokenKind::Not => {
                // ! as word (for test negation)
                self.advance();
                WordPart::Literal("!".to_string())
            }
            TokenKind::Ne => {
                // != as word (for test string comparison)
                self.advance();
                WordPart::Literal("!=".to_string())
            }
            TokenKind::Eq => {
                // == as word (for test string comparison)
                self.advance();
                WordPart::Literal("==".to_string())
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
    pub fn parse_brace_expansion(&self, content: &str) -> Result<WordPart> {
        // Check for arithmetic $(()) FIRST (before command substitution)
        if content.starts_with("$((") && content.ends_with("))") {
            // Extract the expression between $(( and ))
            let expr = &content[3..content.len() - 2];
            return Ok(WordPart::Literal(format!("$__ARITH__{}__", expr)));
        }
        
        // Check for command substitution $(...)
        if content.starts_with("$(") && content.ends_with(')') {
            let inner = &content[2..content.len() - 1];
            let mut parser = Parser::from_str(inner)?;
            let program = parser.parse_program()?;
            return Ok(WordPart::CommandSub(program.statements));
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
}

