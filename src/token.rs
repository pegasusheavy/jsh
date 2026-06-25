//! Token definitions for Franken Shell lexer
//!
//! Uses `Cow<'static, str>` with string interning for efficient string storage.
//! Common strings (keywords, variable names) are interned and shared.
//! Keyword lookup uses perfect hashing (PHF) for O(1) lookup.

use phf::phf_map;
use std::borrow::Cow;
use std::fmt;

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// A string that may be borrowed (interned) or owned.
/// Using Cow<'static, str> allows us to:
/// - Borrow from static strings (keywords, common variable names)
/// - Own strings that need to be allocated (escaped strings, unique identifiers)
pub type TokenStr = Cow<'static, str>;

/// Create a borrowed token string from a static str
#[inline]
pub fn borrowed_str(s: &'static str) -> TokenStr {
    Cow::Borrowed(s)
}

/// Create an owned token string
#[inline]
pub fn owned_str(s: String) -> TokenStr {
    Cow::Owned(s)
}

/// Convert a TokenStr to an owned String
#[inline]
pub fn into_string(s: TokenStr) -> String {
    s.into_owned()
}

/// Get a &str reference from TokenStr
#[inline]
pub fn as_str(s: &TokenStr) -> &str {
    s.as_ref()
}

/// Token types for shell syntax
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Word(TokenStr),      // Regular word/identifier
    String(TokenStr),    // "double quoted" or 'single quoted'
    RawString(TokenStr), // $'...' ANSI-C style
    Number(i64),         // Integer literal
    Float(f64),          // Float literal

    // Variables
    Variable(TokenStr),      // $var
    VariableBrace(TokenStr), // ${var}
    SpecialVar(char),        // $?, $!, $$, $#, $@, $*, $0-$9

    // Operators
    Pipe,       // |
    PipeErr,    // |&
    And,        // &&
    Or,         // ||
    Not,        // !
    Semi,       // ;
    DoubleSemi, // ;;
    Amp,        // &
    Comma,      // ,
    Newline,    // \n

    // Redirections
    RedirectIn,           // <
    RedirectOut,          // >
    RedirectAppend,       // >>
    RedirectErr,          // 2>
    RedirectErrAppend,    // 2>>
    RedirectBoth,         // &>
    RedirectBothAppend,   // &>>
    HereDoc,              // <<
    HereString,           // <<<
    RedirectFd(i32, i32), // n>&m or n<&m

    // Grouping
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    DoubleLBracket, // [[
    DoubleRBracket, // ]]

    // Assignment
    Assign,      // =
    PlusAssign,  // +=
    MinusAssign, // -=

    // Arithmetic
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    DoubleStar, // **
    PlusPlus,   // ++
    MinusMinus, // --

    // Comparison
    Eq,    // ==
    Ne,    // !=
    Lt,    // <
    Le,    // <=
    Gt,    // >
    Ge,    // >=
    Match, // =~

    // Keywords
    If,
    Then,
    Else,
    Elif,
    Fi,
    Case,
    Esac,
    For,
    In,
    Do,
    Done,
    While,
    Until,
    Select,
    Function,
    Return,
    Break,
    Continue,
    Time,
    Coproc,
    // Note: local, export, readonly, declare, typeset, unset, shift are builtins, not keywords

    // franken-specific keywords (enhanced syntax)
    Match_,  // match keyword (different from =~ operator)
    When,    // when (for match arms)
    Loop,    // loop (infinite loop)
    Fn,      // fn (function shorthand)
    Let,     // let (variable binding)
    Const,   // const (immutable binding)
    Try,     // try
    Catch,   // catch
    Finally, // finally
    Throw,   // throw

    // Fish-compatible keywords
    End,      // end (Fish-style block terminator)
    Begin,    // begin (Fish-style block start)
    Switch,   // switch (Fish-style switch)
    And_,     // and (Fish-style logical and)
    Or_,      // or (Fish-style logical or)
    Not_,     // not (Fish-style logical not)
    Set,      // set (Fish-style variable assignment)
    Contains, // contains (Fish-style list contains)

    // Special
    Glob(TokenStr),    // *, ?, [...]
    Comment(TokenStr), // # comment
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Word(s) => write!(f, "{}", s),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::RawString(s) => write!(f, "$'{}'", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Variable(s) => write!(f, "${}", s),
            TokenKind::VariableBrace(s) => write!(f, "${{{}}}", s),
            TokenKind::SpecialVar(c) => write!(f, "${}", c),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::PipeErr => write!(f, "|&"),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::DoubleSemi => write!(f, ";;"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::RedirectIn => write!(f, "<"),
            TokenKind::RedirectOut => write!(f, ">"),
            TokenKind::RedirectAppend => write!(f, ">>"),
            TokenKind::RedirectErr => write!(f, "2>"),
            TokenKind::RedirectErrAppend => write!(f, "2>>"),
            TokenKind::RedirectBoth => write!(f, "&>"),
            TokenKind::RedirectBothAppend => write!(f, "&>>"),
            TokenKind::HereDoc => write!(f, "<<"),
            TokenKind::HereString => write!(f, "<<<"),
            TokenKind::RedirectFd(n, m) => write!(f, "{}>& {}", n, m),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::DoubleLBracket => write!(f, "[["),
            TokenKind::DoubleRBracket => write!(f, "]]"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::PlusAssign => write!(f, "+="),
            TokenKind::MinusAssign => write!(f, "-="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::DoubleStar => write!(f, "**"),
            TokenKind::PlusPlus => write!(f, "++"),
            TokenKind::MinusMinus => write!(f, "--"),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::Match => write!(f, "=~"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::Fi => write!(f, "fi"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Esac => write!(f, "esac"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Done => write!(f, "done"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Until => write!(f, "until"),
            TokenKind::Select => write!(f, "select"),
            TokenKind::Function => write!(f, "function"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Time => write!(f, "time"),
            TokenKind::Coproc => write!(f, "coproc"),
            TokenKind::Match_ => write!(f, "match"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Begin => write!(f, "begin"),
            TokenKind::Switch => write!(f, "switch"),
            TokenKind::And_ => write!(f, "and"),
            TokenKind::Or_ => write!(f, "or"),
            TokenKind::Not_ => write!(f, "not"),
            TokenKind::Set => write!(f, "set"),
            TokenKind::Contains => write!(f, "contains"),
            TokenKind::Glob(s) => write!(f, "{}", s),
            TokenKind::Comment(s) => write!(f, "#{}", s),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its position
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            span: Span::default(),
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    pub fn is_word(&self) -> bool {
        matches!(self.kind, TokenKind::Word(_))
    }

    pub fn is_separator(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Semi | TokenKind::Newline | TokenKind::Amp | TokenKind::Eof
        )
    }

    pub fn is_redirect(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::RedirectIn
                | TokenKind::RedirectOut
                | TokenKind::RedirectAppend
                | TokenKind::RedirectErr
                | TokenKind::RedirectErrAppend
                | TokenKind::RedirectBoth
                | TokenKind::RedirectBothAppend
                | TokenKind::HereDoc
                | TokenKind::HereString
                | TokenKind::RedirectFd(_, _)
        )
    }
}

/// Keyword ID for perfect hash lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum KeywordId {
    If,
    Then,
    Else,
    Elif,
    Fi,
    Case,
    Esac,
    For,
    In,
    Do,
    Done,
    While,
    Until,
    Select,
    Function,
    Return,
    Break,
    Continue,
    Time,
    Coproc,
    // Franken-specific
    Match,
    When,
    Loop,
    Fn,
    Let,
    Const,
    Try,
    Catch,
    Finally,
    Throw,
    // Fish-compatible
    End,
    Begin,
    Switch,
    And,
    Or,
    Not,
    // Note: there is no `Contains` keyword id on purpose: "contains" is a Fish
    // builtin command, not a keyword (see the KEYWORDS map below).
}

/// Perfect hash map for keyword lookup (O(1) lookup, generated at compile time)
static KEYWORDS: phf::Map<&'static str, KeywordId> = phf_map! {
    // Shell keywords
    "if" => KeywordId::If,
    "then" => KeywordId::Then,
    "else" => KeywordId::Else,
    "elif" => KeywordId::Elif,
    "fi" => KeywordId::Fi,
    "case" => KeywordId::Case,
    "esac" => KeywordId::Esac,
    "for" => KeywordId::For,
    "in" => KeywordId::In,
    "do" => KeywordId::Do,
    "done" => KeywordId::Done,
    "while" => KeywordId::While,
    "until" => KeywordId::Until,
    "select" => KeywordId::Select,
    "function" => KeywordId::Function,
    "return" => KeywordId::Return,
    "break" => KeywordId::Break,
    "continue" => KeywordId::Continue,
    "time" => KeywordId::Time,
    "coproc" => KeywordId::Coproc,
    // Franken-specific keywords
    "match" => KeywordId::Match,
    "when" => KeywordId::When,
    "loop" => KeywordId::Loop,
    "fn" => KeywordId::Fn,
    "let" => KeywordId::Let,
    "const" => KeywordId::Const,
    "try" => KeywordId::Try,
    "catch" => KeywordId::Catch,
    "finally" => KeywordId::Finally,
    "throw" => KeywordId::Throw,
    // Fish-compatible keywords
    "end" => KeywordId::End,
    "begin" => KeywordId::Begin,
    "switch" => KeywordId::Switch,
    "and" => KeywordId::And,
    "or" => KeywordId::Or,
    "not" => KeywordId::Not,
    // Note: "contains" is NOT a keyword - it's a Fish builtin command
    // If it were a keyword, it would break "contains foo bar" as a command
};

/// Check if a word is a keyword using perfect hash lookup
///
/// Uses compile-time generated perfect hash function for O(1) lookup.
#[inline]
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    KEYWORDS.get(s).map(|id| match id {
        KeywordId::If => TokenKind::If,
        KeywordId::Then => TokenKind::Then,
        KeywordId::Else => TokenKind::Else,
        KeywordId::Elif => TokenKind::Elif,
        KeywordId::Fi => TokenKind::Fi,
        KeywordId::Case => TokenKind::Case,
        KeywordId::Esac => TokenKind::Esac,
        KeywordId::For => TokenKind::For,
        KeywordId::In => TokenKind::In,
        KeywordId::Do => TokenKind::Do,
        KeywordId::Done => TokenKind::Done,
        KeywordId::While => TokenKind::While,
        KeywordId::Until => TokenKind::Until,
        KeywordId::Select => TokenKind::Select,
        KeywordId::Function => TokenKind::Function,
        KeywordId::Return => TokenKind::Return,
        KeywordId::Break => TokenKind::Break,
        KeywordId::Continue => TokenKind::Continue,
        KeywordId::Time => TokenKind::Time,
        KeywordId::Coproc => TokenKind::Coproc,
        KeywordId::Match => TokenKind::Match_,
        KeywordId::When => TokenKind::When,
        KeywordId::Loop => TokenKind::Loop,
        KeywordId::Fn => TokenKind::Fn,
        KeywordId::Let => TokenKind::Let,
        KeywordId::Const => TokenKind::Const,
        KeywordId::Try => TokenKind::Try,
        KeywordId::Catch => TokenKind::Catch,
        KeywordId::Finally => TokenKind::Finally,
        KeywordId::Throw => TokenKind::Throw,
        KeywordId::End => TokenKind::End,
        KeywordId::Begin => TokenKind::Begin,
        KeywordId::Switch => TokenKind::Switch,
        KeywordId::And => TokenKind::And_,
        KeywordId::Or => TokenKind::Or_,
        KeywordId::Not => TokenKind::Not_,
    })
}

/// Check if a string is a keyword (without allocating TokenKind)
#[inline]
pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains_key(s)
}
