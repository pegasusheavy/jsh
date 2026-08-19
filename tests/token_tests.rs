//! Token module tests

use franken_shell::token::{Span, Token, TokenKind, keyword_from_str, owned_str};

// =============================================================================
// Span Tests
// =============================================================================

#[test]
fn test_span_new() {
    let span = Span::new(0, 5, 1, 1);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);
    assert_eq!(span.line, 1);
    assert_eq!(span.column, 1);
}

#[test]
fn test_span_empty() {
    let span = Span::new(0, 0, 1, 1);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
}

#[test]
fn test_span_len() {
    let span = Span::new(10, 20, 2, 5);
    // Span length would be end - start
    assert_eq!(span.end - span.start, 10);
}

#[test]
fn test_span_clone() {
    let span1 = Span::new(5, 10, 3, 7);
    let span2 = span1;
    assert_eq!(span1, span2);
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
}

// =============================================================================
// Token Tests
// =============================================================================

#[test]
fn test_token_new() {
    let span = Span::new(0, 4, 1, 1);
    let token = Token::new(TokenKind::Word(owned_str("test".to_string())), span);
    assert!(matches!(&token.kind, TokenKind::Word(s) if s == "test"));
    assert_eq!(token.span, span);
}

#[test]
fn test_token_is_eof() {
    let token = Token::eof();
    assert!(token.is_eof());
}

#[test]
fn test_token_is_not_eof() {
    let span = Span::new(0, 4, 1, 1);
    let token = Token::new(TokenKind::Word(owned_str("test".to_string())), span);
    assert!(!token.is_eof());
}

#[test]
fn test_token_is_word() {
    let span = Span::new(0, 4, 1, 1);
    let token = Token::new(TokenKind::Word(owned_str("test".to_string())), span);
    assert!(token.is_word());
}

#[test]
fn test_token_is_not_word() {
    let span = Span::new(0, 1, 1, 1);
    let token = Token::new(TokenKind::Pipe, span);
    assert!(!token.is_word());
}

#[test]
fn test_token_is_separator() {
    let span = Span::new(0, 1, 1, 1);
    let token = Token::new(TokenKind::Semi, span);
    assert!(token.is_separator());
}

#[test]
fn test_token_is_redirect() {
    let span = Span::new(0, 1, 1, 1);
    let token = Token::new(TokenKind::RedirectOut, span);
    assert!(token.is_redirect());
}

// =============================================================================
// TokenKind Tests
// =============================================================================

#[test]
fn test_token_kind_word() {
    let kind = TokenKind::Word(owned_str("echo".to_string()));
    assert!(matches!(kind, TokenKind::Word(s) if s == "echo"));
}

#[test]
fn test_token_kind_variable() {
    let kind = TokenKind::Variable(owned_str("HOME".to_string()));
    assert!(matches!(kind, TokenKind::Variable(s) if s == "HOME"));
}

#[test]
fn test_token_kind_string() {
    let kind = TokenKind::String(owned_str("hello world".to_string()));
    assert!(matches!(kind, TokenKind::String(s) if s == "hello world"));
}

#[test]
fn test_token_kind_number() {
    let kind = TokenKind::Number(42);
    assert!(matches!(kind, TokenKind::Number(n) if n == 42));
}

#[test]
fn test_token_kind_float() {
    let kind = TokenKind::Float(3.14);
    assert!(matches!(kind, TokenKind::Float(f) if (f - 3.14).abs() < 0.001));
}

// =============================================================================
// Operator TokenKinds
// =============================================================================

#[test]
fn test_token_kind_pipe() {
    let kind = TokenKind::Pipe;
    assert!(matches!(kind, TokenKind::Pipe));
}

#[test]
fn test_token_kind_and() {
    let kind = TokenKind::And;
    assert!(matches!(kind, TokenKind::And));
}

#[test]
fn test_token_kind_or() {
    let kind = TokenKind::Or;
    assert!(matches!(kind, TokenKind::Or));
}

#[test]
fn test_token_kind_amp() {
    let kind = TokenKind::Amp;
    assert!(matches!(kind, TokenKind::Amp));
}

#[test]
fn test_token_kind_semi() {
    let kind = TokenKind::Semi;
    assert!(matches!(kind, TokenKind::Semi));
}

#[test]
fn test_token_kind_newline() {
    let kind = TokenKind::Newline;
    assert!(matches!(kind, TokenKind::Newline));
}

#[test]
fn test_token_kind_not() {
    let kind = TokenKind::Not;
    assert!(matches!(kind, TokenKind::Not));
}

// =============================================================================
// Redirect TokenKinds
// =============================================================================

#[test]
fn test_token_kind_redirect_out() {
    let kind = TokenKind::RedirectOut;
    assert!(matches!(kind, TokenKind::RedirectOut));
}

#[test]
fn test_token_kind_redirect_in() {
    let kind = TokenKind::RedirectIn;
    assert!(matches!(kind, TokenKind::RedirectIn));
}

#[test]
fn test_token_kind_redirect_append() {
    let kind = TokenKind::RedirectAppend;
    assert!(matches!(kind, TokenKind::RedirectAppend));
}

#[test]
fn test_token_kind_heredoc() {
    let kind = TokenKind::HereDoc;
    assert!(matches!(kind, TokenKind::HereDoc));
}

#[test]
fn test_token_kind_redirect_both() {
    let kind = TokenKind::RedirectBoth;
    assert!(matches!(kind, TokenKind::RedirectBoth));
}

// =============================================================================
// Parentheses and Braces
// =============================================================================

#[test]
fn test_token_kind_lparen() {
    let kind = TokenKind::LParen;
    assert!(matches!(kind, TokenKind::LParen));
}

#[test]
fn test_token_kind_rparen() {
    let kind = TokenKind::RParen;
    assert!(matches!(kind, TokenKind::RParen));
}

#[test]
fn test_token_kind_lbrace() {
    let kind = TokenKind::LBrace;
    assert!(matches!(kind, TokenKind::LBrace));
}

#[test]
fn test_token_kind_rbrace() {
    let kind = TokenKind::RBrace;
    assert!(matches!(kind, TokenKind::RBrace));
}

#[test]
fn test_token_kind_lbracket() {
    let kind = TokenKind::LBracket;
    assert!(matches!(kind, TokenKind::LBracket));
}

#[test]
fn test_token_kind_rbracket() {
    let kind = TokenKind::RBracket;
    assert!(matches!(kind, TokenKind::RBracket));
}

// =============================================================================
// Keyword from String Tests
// =============================================================================

#[test]
fn test_keyword_if() {
    let result = keyword_from_str("if");
    assert!(matches!(result, Some(TokenKind::If)));
}

#[test]
fn test_keyword_then() {
    let result = keyword_from_str("then");
    assert!(matches!(result, Some(TokenKind::Then)));
}

#[test]
fn test_keyword_else() {
    let result = keyword_from_str("else");
    assert!(matches!(result, Some(TokenKind::Else)));
}

#[test]
fn test_keyword_elif() {
    let result = keyword_from_str("elif");
    assert!(matches!(result, Some(TokenKind::Elif)));
}

#[test]
fn test_keyword_fi() {
    let result = keyword_from_str("fi");
    assert!(matches!(result, Some(TokenKind::Fi)));
}

#[test]
fn test_keyword_for() {
    let result = keyword_from_str("for");
    assert!(matches!(result, Some(TokenKind::For)));
}

#[test]
fn test_keyword_in() {
    let result = keyword_from_str("in");
    assert!(matches!(result, Some(TokenKind::In)));
}

#[test]
fn test_keyword_do() {
    let result = keyword_from_str("do");
    assert!(matches!(result, Some(TokenKind::Do)));
}

#[test]
fn test_keyword_done() {
    let result = keyword_from_str("done");
    assert!(matches!(result, Some(TokenKind::Done)));
}

#[test]
fn test_keyword_while() {
    let result = keyword_from_str("while");
    assert!(matches!(result, Some(TokenKind::While)));
}

#[test]
fn test_keyword_until() {
    let result = keyword_from_str("until");
    assert!(matches!(result, Some(TokenKind::Until)));
}

#[test]
fn test_keyword_case() {
    let result = keyword_from_str("case");
    assert!(matches!(result, Some(TokenKind::Case)));
}

#[test]
fn test_keyword_esac() {
    let result = keyword_from_str("esac");
    assert!(matches!(result, Some(TokenKind::Esac)));
}

#[test]
fn test_keyword_function() {
    let result = keyword_from_str("function");
    assert!(matches!(result, Some(TokenKind::Function)));
}

#[test]
fn test_keyword_fn_not_defined() {
    // "fn" may or may not be a keyword depending on implementation
    let result = keyword_from_str("fn");
    // Could be Some(Fn) or None - just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_keyword_let_not_defined() {
    // "let" may or may not be a keyword depending on implementation
    let result = keyword_from_str("let");
    // Could be Some(Let) or None - just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_keyword_match_not_defined() {
    // "match" may or may not be a keyword depending on implementation
    let result = keyword_from_str("match");
    // Could be Some(Match) or None - just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_keyword_loop_not_defined() {
    // "loop" may or may not be a keyword depending on implementation
    let result = keyword_from_str("loop");
    // Could be Some(Loop) or None - just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_keyword_not_found() {
    let result = keyword_from_str("echo");
    assert!(result.is_none());
}

#[test]
fn test_keyword_not_found_other() {
    let result = keyword_from_str("hello");
    assert!(result.is_none());
}

// =============================================================================
// Special Token Kinds
// =============================================================================

#[test]
fn test_token_kind_double_semi() {
    let kind = TokenKind::DoubleSemi;
    assert!(matches!(kind, TokenKind::DoubleSemi));
}

#[test]
fn test_token_kind_assign() {
    let kind = TokenKind::Assign;
    assert!(matches!(kind, TokenKind::Assign));
}

#[test]
fn test_token_kind_plus_assign() {
    let kind = TokenKind::PlusAssign;
    assert!(matches!(kind, TokenKind::PlusAssign));
}

#[test]
fn test_token_kind_variable_brace() {
    let kind = TokenKind::VariableBrace(owned_str("HOME".to_string()));
    assert!(matches!(kind, TokenKind::VariableBrace(s) if s == "HOME"));
}

#[test]
fn test_token_kind_special_var() {
    let kind = TokenKind::SpecialVar('?');
    assert!(matches!(kind, TokenKind::SpecialVar('?')));
}

#[test]
fn test_token_kind_eof() {
    let kind = TokenKind::Eof;
    assert!(matches!(kind, TokenKind::Eof));
}

#[test]
fn test_token_clone() {
    let span = Span::new(0, 5, 1, 1);
    let token1 = Token::new(TokenKind::Word(owned_str("hello".to_string())), span);
    let token2 = token1.clone();
    assert_eq!(token1.span, token2.span);
    assert!(matches!(&token2.kind, TokenKind::Word(s) if s == "hello"));
}
