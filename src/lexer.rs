//! Lexer for jsh shell syntax

use crate::error::{JshError, Result};
use crate::token::{keyword_from_str, Span, Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

/// Lexer for tokenizing shell input
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    column: usize,
    /// Track if we're at the start of a command (for keyword detection)
    at_command_start: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            line: 1,
            column: 1,
            at_command_start: true,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(n)
    }

    fn span_from(&self, start: usize, start_line: usize, start_col: usize) -> Span {
        Span::new(start, self.pos, start_line, start_col)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else if c == '\\' && self.peek_nth(1) == Some('\n') {
                // Line continuation
                self.advance();
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_comment(&mut self) -> Token {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume #
        let mut comment = String::new();

        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            comment.push(c);
            self.advance();
        }

        Token::new(
            TokenKind::Comment(comment),
            self.span_from(start, start_line, start_col),
        )
    }

    fn read_string(&mut self, quote: char) -> Result<Token> {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume opening quote
        let mut string = String::new();

        loop {
            match self.peek() {
                None => {
                    return Err(JshError::parse(
                        format!("Unterminated string starting with {}", quote),
                        start_line,
                        start_col,
                    ));
                }
                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some('\\') if quote == '"' => {
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        let ch = match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            '"' => '"',
                            '$' => '$',
                            '`' => '`',
                            '\n' => {
                                self.advance();
                                continue;
                            }
                            _ => {
                                string.push('\\');
                                escaped
                            }
                        };
                        string.push(ch);
                        self.advance();
                    }
                }
                Some('$') if quote == '"' => {
                    // Variable expansion inside double quotes - keep the $ for later expansion
                    string.push('$');
                    self.advance();
                    // Collect variable name, brace content, or subshell/arithmetic
                    if let Some('{') = self.peek() {
                        string.push('{');
                        self.advance();
                        let mut brace_depth = 1;
                        while let Some(c) = self.peek() {
                            if c == '{' {
                                brace_depth += 1;
                            } else if c == '}' {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    string.push('}');
                                    self.advance();
                                    break;
                                }
                            }
                            string.push(c);
                            self.advance();
                        }
                    } else if let Some('(') = self.peek() {
                        // Command substitution $(  ...) or arithmetic $((...))
                        string.push('(');
                        self.advance();
                        let mut paren_depth = 1;
                        while let Some(c) = self.peek() {
                            if c == '(' {
                                paren_depth += 1;
                            } else if c == ')' {
                                paren_depth -= 1;
                                if paren_depth == 0 {
                                    string.push(')');
                                    self.advance();
                                    break;
                                }
                            }
                            string.push(c);
                            self.advance();
                        }
                    } else {
                        // Simple variable name or special character
                        while let Some(c) = self.peek() {
                            if c.is_alphanumeric() || c == '_' || c == '?' || c == '!' || c == '#' || c == '@' || c == '*' || c == '-' {
                                string.push(c);
                                self.advance();
                                // Special characters are single-char variables
                                if matches!(c, '?' | '!' | '#' | '@' | '*' | '-') {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                Some(c) => {
                    string.push(c);
                    self.advance();
                }
            }
        }

        Ok(Token::new(
            TokenKind::String(string),
            self.span_from(start, start_line, start_col),
        ))
    }

    fn read_raw_string(&mut self) -> Result<Token> {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume $
        self.advance(); // consume '
        let mut string = String::new();

        loop {
            match self.peek() {
                None => {
                    return Err(JshError::parse(
                        "Unterminated $'...' string",
                        start_line,
                        start_col,
                    ));
                }
                Some('\'') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        let ch = match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            'a' => '\x07',
                            'b' => '\x08',
                            'e' | 'E' => '\x1b',
                            'f' => '\x0c',
                            'v' => '\x0b',
                            '\\' => '\\',
                            '\'' => '\'',
                            '"' => '"',
                            '?' => '?',
                            '0'..='7' => {
                                // Octal escape
                                let mut val = (escaped as u8 - b'0') as u32;
                                self.advance();
                                for _ in 0..2 {
                                    if let Some(c @ '0'..='7') = self.peek() {
                                        val = val * 8 + (c as u8 - b'0') as u32;
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                char::from_u32(val).unwrap_or('?')
                            }
                            'x' => {
                                // Hex escape
                                self.advance();
                                let mut val = 0u32;
                                for _ in 0..2 {
                                    if let Some(c) = self.peek() {
                                        if let Some(digit) = c.to_digit(16) {
                                            val = val * 16 + digit;
                                            self.advance();
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                char::from_u32(val).unwrap_or('?')
                            }
                            _ => {
                                string.push('\\');
                                escaped
                            }
                        };
                        if !matches!(escaped, '0'..='7' | 'x') {
                            string.push(ch);
                            self.advance();
                        } else {
                            string.push(ch);
                        }
                    }
                }
                Some(c) => {
                    string.push(c);
                    self.advance();
                }
            }
        }

        Ok(Token::new(
            TokenKind::RawString(string),
            self.span_from(start, start_line, start_col),
        ))
    }

    fn read_variable(&mut self) -> Token {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.advance(); // consume $

        match self.peek() {
            Some('{') => {
                self.advance();
                let mut name = String::new();
                let mut brace_depth = 1;

                while let Some(c) = self.peek() {
                    if c == '{' {
                        brace_depth += 1;
                        name.push(c);
                    } else if c == '}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            self.advance();
                            break;
                        }
                        name.push(c);
                    } else {
                        name.push(c);
                    }
                    self.advance();
                }

                Token::new(
                    TokenKind::VariableBrace(name),
                    self.span_from(start, start_line, start_col),
                )
            }
            Some('(') => {
                // Command substitution $(...)
                self.advance();
                let mut content = String::new();
                let mut paren_depth = 1;

                while let Some(c) = self.peek() {
                    if c == '(' {
                        paren_depth += 1;
                    } else if c == ')' {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            self.advance();
                            break;
                        }
                    }
                    content.push(c);
                    self.advance();
                }

                Token::new(
                    TokenKind::VariableBrace(format!("$({})", content)),
                    self.span_from(start, start_line, start_col),
                )
            }
            Some(c @ ('?' | '!' | '$' | '#' | '@' | '*' | '-' | '_')) => {
                self.advance();
                Token::new(
                    TokenKind::SpecialVar(c),
                    self.span_from(start, start_line, start_col),
                )
            }
            Some(c @ '0'..='9') => {
                self.advance();
                Token::new(
                    TokenKind::SpecialVar(c),
                    self.span_from(start, start_line, start_col),
                )
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut name = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        name.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token::new(
                    TokenKind::Variable(name),
                    self.span_from(start, start_line, start_col),
                )
            }
            _ => Token::new(
                TokenKind::Word("$".to_string()),
                self.span_from(start, start_line, start_col),
            ),
        }
    }

    fn read_word(&mut self) -> Token {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;
        let mut word = String::new();

        while let Some(c) = self.peek() {
            match c {
                // Word terminators
                ' ' | '\t' | '\n' | '\r' | ';' | '|' | '&' | '<' | '>' | '(' | ')' | '{' | '}'
                | '[' | ']' | '#' | '"' | '\'' | '`' | '$' | ',' => break,
                '\\' => {
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        word.push(escaped);
                        self.advance();
                    }
                }
                '=' if word.is_empty() || self.at_command_start => {
                    // Could be assignment
                    break;
                }
                _ => {
                    word.push(c);
                    self.advance();
                }
            }
        }

        let span = self.span_from(start, start_line, start_col);

        // Check if it's a keyword (only at command position)
        if self.at_command_start {
            if let Some(kw) = keyword_from_str(&word) {
                return Token::new(kw, span);
            }
        }

        // Check if it's a number
        if let Ok(n) = word.parse::<i64>() {
            return Token::new(TokenKind::Number(n), span);
        }
        if let Ok(n) = word.parse::<f64>() {
            return Token::new(TokenKind::Float(n), span);
        }

        Token::new(TokenKind::Word(word), span)
    }

    fn read_operator(&mut self) -> Token {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let c = self.advance().unwrap();
        let next = self.peek();

        let kind = match (c, next) {
            ('|', Some('|')) => {
                self.advance();
                TokenKind::Or
            }
            ('|', Some('&')) => {
                self.advance();
                TokenKind::PipeErr
            }
            ('|', _) => TokenKind::Pipe,

            ('&', Some('&')) => {
                self.advance();
                TokenKind::And
            }
            ('&', Some('>')) => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::RedirectBothAppend
                } else {
                    TokenKind::RedirectBoth
                }
            }
            ('&', _) => TokenKind::Amp,

            (';', Some(';')) => {
                self.advance();
                TokenKind::DoubleSemi
            }
            (';', _) => TokenKind::Semi,

            ('<', Some('<')) => {
                self.advance();
                if self.peek() == Some('<') {
                    self.advance();
                    TokenKind::HereString
                } else {
                    TokenKind::HereDoc
                }
            }
            ('<', Some('=')) => {
                self.advance();
                TokenKind::Le
            }
            ('<', _) => TokenKind::RedirectIn,

            ('>', Some('>')) => {
                self.advance();
                TokenKind::RedirectAppend
            }
            ('>', Some('=')) => {
                self.advance();
                TokenKind::Ge
            }
            ('>', _) => TokenKind::RedirectOut,

            ('=', Some('=')) => {
                self.advance();
                TokenKind::Eq
            }
            ('=', Some('~')) => {
                self.advance();
                TokenKind::Match
            }
            ('=', _) => TokenKind::Assign,

            ('!', Some('=')) => {
                self.advance();
                TokenKind::Ne
            }
            ('!', _) => TokenKind::Not,

            ('+', Some('+')) => {
                self.advance();
                TokenKind::PlusPlus
            }
            ('+', Some('=')) => {
                self.advance();
                TokenKind::PlusAssign
            }
            ('+', _) => TokenKind::Plus,

            ('-', Some('-')) => {
                self.advance();
                TokenKind::MinusMinus
            }
            ('-', Some('=')) => {
                self.advance();
                TokenKind::MinusAssign
            }
            ('-', _) => TokenKind::Minus,

            ('*', Some('*')) => {
                self.advance();
                TokenKind::DoubleStar
            }
            ('*', _) => TokenKind::Star,

            ('/', _) => TokenKind::Slash,
            ('%', _) => TokenKind::Percent,

            ('(', _) => TokenKind::LParen,
            (')', _) => TokenKind::RParen,

            ('{', _) => TokenKind::LBrace,
            ('}', _) => TokenKind::RBrace,

            ('[', Some('[')) => {
                self.advance();
                TokenKind::DoubleLBracket
            }
            ('[', _) => TokenKind::LBracket,

            (']', Some(']')) => {
                self.advance();
                TokenKind::DoubleRBracket
            }
            (']', _) => TokenKind::RBracket,

            ('\n', _) => TokenKind::Newline,

            (',', _) => TokenKind::Comma,

            _ => TokenKind::Word(c.to_string()),
        };

        Token::new(kind, self.span_from(start, start_line, start_col))
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let Some(c) = self.peek() else {
            return Ok(Token::eof());
        };

        let token = match c {
            '#' => self.read_comment(),
            '"' | '\'' => self.read_string(c)?,
            '$' => {
                if self.peek_nth(1) == Some('\'') {
                    self.read_raw_string()?
                } else {
                    self.read_variable()
                }
            }
            '|' | '&' | ';' | '<' | '>' | '=' | '!' | '(' | ')'
            | '{' | '}' | '[' | ']' | ',' | '\n' => self.read_operator(),
            // Note: '*', '/', '%' are handled in read_word to allow them in paths and globs
            // Handle '-' and '+' specially: if followed by alphanumeric, it's part of a word (like -e, +o)
            '-' | '+' => {
                let next = self.peek_nth(1);
                if matches!(next, Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '+') {
                    self.read_word()
                } else {
                    self.read_operator()
                }
            }
            '2' if self.peek_nth(1) == Some('>') => {
                let start = self.pos;
                let start_line = self.line;
                let start_col = self.column;
                self.advance();
                self.advance();
                let kind = if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::RedirectErrAppend
                } else {
                    TokenKind::RedirectErr
                };
                Token::new(kind, self.span_from(start, start_line, start_col))
            }
            _ => self.read_word(),
        };

        // Update command start state
        self.at_command_start = matches!(
            token.kind,
            TokenKind::Semi
                | TokenKind::Newline
                | TokenKind::Pipe
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::Do
                | TokenKind::Then
                | TokenKind::Else
                | TokenKind::Elif
                | TokenKind::If      // After if, we expect a command (condition)
                | TokenKind::While   // After while, we expect a command (condition)
                | TokenKind::Until   // After until, we expect a command (condition)
                | TokenKind::For     // After for, we expect variable name (word)
                | TokenKind::Case    // After case, we expect a word
                | TokenKind::Fi      // After fi, new command can start
                | TokenKind::Done    // After done, new command can start
                | TokenKind::Esac    // After esac, new command can start
                // Fish-compatible
                | TokenKind::Begin
                | TokenKind::And_
                | TokenKind::Or_
        );

        Ok(token)
    }

    /// Tokenize all input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.is_eof();

            // Skip comments for the token stream
            if !matches!(token.kind, TokenKind::Comment(_)) {
                tokens.push(token);
            }

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let mut lexer = Lexer::new("echo hello world");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // echo, hello, world, EOF
        assert!(matches!(&tokens[0].kind, TokenKind::Word(s) if s == "echo"));
        assert!(matches!(&tokens[1].kind, TokenKind::Word(s) if s == "hello"));
        assert!(matches!(&tokens[2].kind, TokenKind::Word(s) if s == "world"));
    }

    #[test]
    fn test_variable() {
        let mut lexer = Lexer::new("$HOME ${PATH}");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Variable(s) if s == "HOME"));
        assert!(matches!(&tokens[1].kind, TokenKind::VariableBrace(s) if s == "PATH"));
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new(r#""hello world" 'single'"#);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
        assert!(matches!(&tokens[1].kind, TokenKind::String(s) if s == "single"));
    }

    #[test]
    fn test_pipe() {
        let mut lexer = Lexer::new("ls | grep test");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Word(s) if s == "ls"));
        assert!(matches!(&tokens[1].kind, TokenKind::Pipe));
        assert!(matches!(&tokens[2].kind, TokenKind::Word(s) if s == "grep"));
    }

    #[test]
    fn test_keywords() {
        // Keywords are recognized at command start positions
        // Test individual keywords at start position
        let mut lexer = Lexer::new("if");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::If));
        
        let mut lexer = Lexer::new("for");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::For));
        
        let mut lexer = Lexer::new("while");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::While));
        
        // Test keywords after semicolon (command start)
        let mut lexer = Lexer::new("; if");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Semi));
        assert!(matches!(tokens[1].kind, TokenKind::If));
        
        // Test keywords in proper syntactic positions
        let mut lexer = Lexer::new("if x; then echo; fi");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::If));
        // After if, "x" is a command name (Word)
        assert!(matches!(tokens[1].kind, TokenKind::Word(_)));
        assert!(matches!(tokens[2].kind, TokenKind::Semi));
        assert!(matches!(tokens[3].kind, TokenKind::Then));
    }
}

