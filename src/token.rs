//! Token definitions for jsh lexer

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
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// Token types for shell syntax
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Word(String),           // Regular word/identifier
    String(String),         // "double quoted" or 'single quoted'
    RawString(String),      // $'...' ANSI-C style
    Number(i64),            // Integer literal
    Float(f64),             // Float literal

    // Variables
    Variable(String),       // $var
    VariableBrace(String),  // ${var}
    SpecialVar(char),       // $?, $!, $$, $#, $@, $*, $0-$9

    // Operators
    Pipe,                   // |
    PipeErr,                // |&
    And,                    // &&
    Or,                     // ||
    Not,                    // !
    Semi,                   // ;
    DoubleSemi,             // ;;
    Amp,                    // &
    Comma,                  // ,
    Newline,                // \n

    // Redirections
    RedirectIn,             // <
    RedirectOut,            // >
    RedirectAppend,         // >>
    RedirectErr,            // 2>
    RedirectErrAppend,      // 2>>
    RedirectBoth,           // &>
    RedirectBothAppend,     // &>>
    HereDoc,                // <<
    HereString,             // <<<
    RedirectFd(i32, i32),   // n>&m or n<&m

    // Grouping
    LParen,                 // (
    RParen,                 // )
    LBrace,                 // {
    RBrace,                 // }
    LBracket,               // [
    RBracket,               // ]
    DoubleLBracket,         // [[
    DoubleRBracket,         // ]]

    // Assignment
    Assign,                 // =
    PlusAssign,             // +=
    MinusAssign,            // -=

    // Arithmetic
    Plus,                   // +
    Minus,                  // -
    Star,                   // *
    Slash,                  // /
    Percent,                // %
    DoubleStar,             // **
    PlusPlus,               // ++
    MinusMinus,             // --

    // Comparison
    Eq,                     // ==
    Ne,                     // !=
    Lt,                     // <
    Le,                     // <=
    Gt,                     // >
    Ge,                     // >=
    Match,                  // =~

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
    Local,
    Export,
    Readonly,
    Declare,
    Typeset,
    Unset,
    Shift,
    Time,
    Coproc,

    // jsh-specific keywords (enhanced syntax)
    Match_,                 // match keyword (different from =~ operator)
    When,                   // when (for match arms)
    Loop,                   // loop (infinite loop)
    Fn,                     // fn (function shorthand)
    Let,                    // let (variable binding)
    Const,                  // const (immutable binding)
    Try,                    // try
    Catch,                  // catch
    Finally,                // finally
    Throw,                  // throw

    // Fish-compatible keywords
    End,                    // end (Fish-style block terminator)
    Begin,                  // begin (Fish-style block start)
    Switch,                 // switch (Fish-style switch)
    And_,                   // and (Fish-style logical and)
    Or_,                    // or (Fish-style logical or)
    Not_,                   // not (Fish-style logical not)
    Set,                    // set (Fish-style variable assignment)
    Contains,               // contains (Fish-style list contains)

    // Special
    Glob(String),           // *, ?, [...]
    Comment(String),        // # comment
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
            TokenKind::Local => write!(f, "local"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Readonly => write!(f, "readonly"),
            TokenKind::Declare => write!(f, "declare"),
            TokenKind::Typeset => write!(f, "typeset"),
            TokenKind::Unset => write!(f, "unset"),
            TokenKind::Shift => write!(f, "shift"),
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

/// Check if a word is a keyword
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "if" => Some(TokenKind::If),
        "then" => Some(TokenKind::Then),
        "else" => Some(TokenKind::Else),
        "elif" => Some(TokenKind::Elif),
        "fi" => Some(TokenKind::Fi),
        "case" => Some(TokenKind::Case),
        "esac" => Some(TokenKind::Esac),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "do" => Some(TokenKind::Do),
        "done" => Some(TokenKind::Done),
        "while" => Some(TokenKind::While),
        "until" => Some(TokenKind::Until),
        "select" => Some(TokenKind::Select),
        "function" => Some(TokenKind::Function),
        "return" => Some(TokenKind::Return),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        // "local" is handled as builtin, not keyword
        // "local" => Some(TokenKind::Local),
        // "export" is handled as builtin, not keyword
        // "export" => Some(TokenKind::Export),
        // "readonly" is handled as builtin, not keyword
        // "readonly" => Some(TokenKind::Readonly),
        // "declare" is handled as builtin, not keyword
        // "declare" => Some(TokenKind::Declare),
        // "typeset" is handled as builtin, not keyword
        // "typeset" => Some(TokenKind::Typeset),
        "unset" => Some(TokenKind::Unset),
        "shift" => Some(TokenKind::Shift),
        "time" => Some(TokenKind::Time),
        "coproc" => Some(TokenKind::Coproc),
        // jsh-specific
        "match" => Some(TokenKind::Match_),
        "when" => Some(TokenKind::When),
        "loop" => Some(TokenKind::Loop),
        "fn" => Some(TokenKind::Fn),
        "let" => Some(TokenKind::Let),
        "const" => Some(TokenKind::Const),
        "try" => Some(TokenKind::Try),
        "catch" => Some(TokenKind::Catch),
        "finally" => Some(TokenKind::Finally),
        "throw" => Some(TokenKind::Throw),
        // Fish-compatible keywords
        "end" => Some(TokenKind::End),
        "begin" => Some(TokenKind::Begin),
        "switch" => Some(TokenKind::Switch),
        "and" => Some(TokenKind::And_),
        "or" => Some(TokenKind::Or_),
        "not" => Some(TokenKind::Not_),
        // "set" is handled as builtin, not keyword
        // "set" => Some(TokenKind::Set),
        "contains" => Some(TokenKind::Contains),
        _ => None,
    }
}

