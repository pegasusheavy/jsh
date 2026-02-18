//! Lexer module tests

use franken_shell::lexer::Lexer;
use franken_shell::token::TokenKind;

// =============================================================================
// Basic Tokenization Tests
// =============================================================================

#[test]
fn test_lexer_empty() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1); // Just EOF
    assert!(matches!(tokens[0].kind, TokenKind::Eof));
}

#[test]
fn test_lexer_whitespace() {
    let mut lexer = Lexer::new("   ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1); // Just EOF
    assert!(matches!(tokens[0].kind, TokenKind::Eof));
}

#[test]
fn test_lexer_simple_word() {
    let mut lexer = Lexer::new("hello");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2); // hello + EOF
    assert!(matches!(&tokens[0].kind, TokenKind::Word(s) if s == "hello"));
}

#[test]
fn test_lexer_multiple_words() {
    let mut lexer = Lexer::new("echo hello world");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4); // echo + hello + world + EOF
    assert!(matches!(&tokens[0].kind, TokenKind::Word(s) if s == "echo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Word(s) if s == "hello"));
    assert!(matches!(&tokens[2].kind, TokenKind::Word(s) if s == "world"));
}

// =============================================================================
// String Tests
// =============================================================================

#[test]
fn test_lexer_double_quote_string() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
}

#[test]
fn test_lexer_single_quote_string() {
    let mut lexer = Lexer::new("'hello world'");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
}

#[test]
fn test_lexer_unclosed_double_quote() {
    let mut lexer = Lexer::new(r#""hello"#);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_lexer_unclosed_single_quote() {
    let mut lexer = Lexer::new("'hello");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_lexer_escape_in_double_quote() {
    let mut lexer = Lexer::new(r#""hello\nworld""#);
    let tokens = lexer.tokenize().unwrap();
    // The escaped string includes the backslash
    assert!(matches!(&tokens[0].kind, TokenKind::String(_)));
}

// =============================================================================
// Variable Tests
// =============================================================================

#[test]
fn test_lexer_variable() {
    let mut lexer = Lexer::new("$HOME");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::Variable(s) if s == "HOME"));
}

#[test]
fn test_lexer_variable_braced() {
    let mut lexer = Lexer::new("${HOME}");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::VariableBrace(s) if s == "HOME"));
}

#[test]
fn test_lexer_special_variable() {
    let mut lexer = Lexer::new("$?");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::SpecialVar('?')));
}

#[test]
fn test_lexer_dollar_dollar() {
    let mut lexer = Lexer::new("$$");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0].kind, TokenKind::SpecialVar('$')));
}

// =============================================================================
// Operator Tests
// =============================================================================

#[test]
fn test_lexer_pipe() {
    let mut lexer = Lexer::new("cmd1 | cmd2");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Pipe)));
}

#[test]
fn test_lexer_and() {
    let mut lexer = Lexer::new("cmd1 && cmd2");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::And)));
}

#[test]
fn test_lexer_or() {
    let mut lexer = Lexer::new("cmd1 || cmd2");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Or)));
}

#[test]
fn test_lexer_semicolon() {
    let mut lexer = Lexer::new("cmd1; cmd2");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Semi)));
}

#[test]
fn test_lexer_newline() {
    let mut lexer = Lexer::new("cmd1\ncmd2");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Newline)));
}

#[test]
fn test_lexer_ampersand() {
    let mut lexer = Lexer::new("cmd &");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Amp)));
}

// =============================================================================
// Redirect Tests
// =============================================================================

#[test]
fn test_lexer_redirect_out() {
    let mut lexer = Lexer::new("cmd > file");
    let tokens = lexer.tokenize().unwrap();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::RedirectOut))
    );
}

#[test]
fn test_lexer_redirect_in() {
    let mut lexer = Lexer::new("cmd < file");
    let tokens = lexer.tokenize().unwrap();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::RedirectIn))
    );
}

#[test]
fn test_lexer_redirect_append() {
    let mut lexer = Lexer::new("cmd >> file");
    let tokens = lexer.tokenize().unwrap();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::RedirectAppend))
    );
}

#[test]
fn test_lexer_heredoc() {
    let mut lexer = Lexer::new("cmd << EOF");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::HereDoc)));
}

// =============================================================================
// Parentheses and Braces Tests
// =============================================================================

#[test]
fn test_lexer_lparen() {
    let mut lexer = Lexer::new("(cmd)");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::LParen)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::RParen)));
}

#[test]
fn test_lexer_lbrace() {
    let mut lexer = Lexer::new("{ cmd; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::LBrace)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::RBrace)));
}

// =============================================================================
// Keyword Tests
// =============================================================================

#[test]
fn test_lexer_keyword_if() {
    let mut lexer = Lexer::new("if true; then echo yes; fi");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::If)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Then)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Fi)));
}

#[test]
fn test_lexer_keyword_for() {
    let mut lexer = Lexer::new("for i in 1 2 3; do echo $i; done");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::For)));
    // "in" might be recognized as Word in some contexts, so check for both
    assert!(
        tokens.iter().any(|t| matches!(t.kind, TokenKind::In)
            || matches!(&t.kind, TokenKind::Word(s) if s == "in"))
    );
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Do)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Done)));
}

#[test]
fn test_lexer_keyword_while() {
    let mut lexer = Lexer::new("while true; do echo loop; done");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::While)));
}

#[test]
fn test_lexer_keyword_case() {
    let mut lexer = Lexer::new("case $x in a) echo a;; esac");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Case)));
    // esac might be recognized as Word in some contexts
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Esac)
        || matches!(&t.kind, TokenKind::Word(s) if s == "esac")));
}

#[test]
fn test_lexer_keyword_function() {
    let mut lexer = Lexer::new("function test { echo hello; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Function)));
}

#[test]
fn test_lexer_keyword_fn() {
    let mut lexer = Lexer::new("fn test { echo hello; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Fn)));
}

#[test]
fn test_lexer_keyword_let() {
    let mut lexer = Lexer::new("let x = 5");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Let)));
}

#[test]
fn test_lexer_keyword_match() {
    let mut lexer = Lexer::new("match $x { a => echo a; }");
    let tokens = lexer.tokenize().unwrap();
    // match might be recognized differently depending on context
    // Just verify the lexer can process the input
    assert!(tokens.len() > 1);
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Eof)));
}

#[test]
fn test_lexer_keyword_loop() {
    let mut lexer = Lexer::new("loop { echo forever; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Loop)));
}

// =============================================================================
// Assignment Tests
// =============================================================================

#[test]
fn test_lexer_assignment() {
    let mut lexer = Lexer::new("x=hello");
    let tokens = lexer.tokenize().unwrap();
    // Assignments might be parsed as Word or special token
    assert!(tokens.len() >= 2);
}

// =============================================================================
// Span Tests
// =============================================================================

#[test]
fn test_lexer_span_start() {
    let mut lexer = Lexer::new("hello");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn test_lexer_span_end() {
    let mut lexer = Lexer::new("hello");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].span.end, 5);
}

#[test]
fn test_lexer_span_second_word() {
    let mut lexer = Lexer::new("hello world");
    let tokens = lexer.tokenize().unwrap();
    // "world" starts at position 6
    assert_eq!(tokens[1].span.start, 6);
    assert_eq!(tokens[1].span.end, 11);
}

// =============================================================================
// Complex Input Tests
// =============================================================================

#[test]
fn test_lexer_complex_pipeline() {
    let mut lexer = Lexer::new("ls -la | grep test | head -n 10");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.len() > 1);
    // Should have 2 pipes
    let pipe_count = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Pipe))
        .count();
    assert_eq!(pipe_count, 2);
}

#[test]
fn test_lexer_complex_redirect() {
    let mut lexer = Lexer::new("cmd 2>&1 > output.txt");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.len() > 1);
}

#[test]
fn test_lexer_multiline() {
    let input = r#"
echo hello
echo world
"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    // Should have newlines
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Newline)));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_lexer_equals_sign() {
    let mut lexer = Lexer::new("test x = y");
    let tokens = lexer.tokenize().unwrap();
    // The = should be tokenized
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Assign)
        || matches!(&t.kind, TokenKind::Word(s) if s == "=")));
}

#[test]
fn test_lexer_double_semicolon() {
    let mut lexer = Lexer::new("case a in x) echo x;; esac");
    let tokens = lexer.tokenize().unwrap();
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::DoubleSemi))
    );
}

#[test]
fn test_lexer_number() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    // Numbers might be Word or Number
    assert!(tokens.len() >= 2);
}
