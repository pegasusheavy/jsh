//! Abstract Syntax Tree for Franken Shell
//!
//! Includes optimized representations for common command patterns.

use crate::token::Span;
use smallvec::SmallVec;

/// A complete shell program
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A statement in the shell
#[derive(Debug, Clone)]
pub enum Statement {
    /// A pipeline of commands
    Pipeline(Pipeline),
    /// A list of pipelines connected with && or ||
    List(List),
    /// Variable assignment
    Assignment(Assignment),
    /// If-then-else
    If(IfStatement),
    /// For loop
    For(ForLoop),
    /// While loop
    While(WhileLoop),
    /// Until loop
    Until(UntilLoop),
    /// Case statement
    Case(CaseStatement),
    /// Select statement (bash)
    Select(SelectStatement),
    /// Function definition
    Function(FunctionDef),
    /// franken match expression
    Match(MatchExpr),
    /// franken loop (infinite)
    Loop(LoopStatement),
    /// franken let binding
    Let(LetBinding),
    /// franken const binding
    Const(ConstBinding),
    /// franken try-catch-finally
    Try(TryStatement),
    /// Fish-style begin...end block
    BeginBlock(Vec<Statement>),
    /// Fish-style switch statement
    FishSwitch(FishSwitchStatement),
    /// Break statement
    Break(Option<usize>),
    /// Continue statement
    Continue(Option<usize>),
    /// Return statement
    Return(Option<Word>),
    /// Subshell
    Subshell(Vec<Statement>),
    /// Brace group
    BraceGroup(Vec<Statement>),
    /// Empty statement
    Empty,
}

/// A list of pipelines
#[derive(Debug, Clone)]
pub struct List {
    pub first: Pipeline,
    pub rest: Vec<(ListOp, Pipeline)>,
}

/// List operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOp {
    And, // &&
    Or,  // ||
}

/// A pipeline of commands
#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub negated: bool,
    pub background: bool,
}

/// A single command
#[derive(Debug, Clone)]
pub struct Command {
    pub kind: CommandKind,
    pub redirects: Vec<Redirect>,
}

/// Types of commands
#[derive(Debug, Clone)]
pub enum CommandKind {
    /// Simple command (program with arguments)
    Simple(SimpleCommand),
    /// Flat command - optimized for simple commands with no expansions
    /// This is an optimization to avoid Word/WordPart overhead
    Flat(FlatCommand),
    /// Compound command (if, for, while, etc.)
    Compound(Box<Statement>),
    /// Function call
    FunctionCall {
        name: String,
        args: Vec<Word>,
    },
    /// Coprocess
    Coproc {
        name: Option<String>,
        command: Box<Command>,
    },
}

/// A simple command with its arguments
#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub name: Word,
    pub args: Vec<Word>,
    pub assignments: Vec<Assignment>,
}

impl SimpleCommand {
    /// Check if this command can be represented as a flat command
    /// (all arguments are simple literals with no expansions)
    #[inline]
    pub fn is_flat(&self) -> bool {
        self.assignments.is_empty()
            && self.name.is_literal()
            && self.args.iter().all(|w| w.is_literal())
    }

    /// Convert to a flat command if possible
    /// Returns None if the command contains expansions
    pub fn to_flat(&self) -> Option<FlatCommand> {
        if !self.is_flat() {
            return None;
        }

        let mut argv = SmallVec::with_capacity(1 + self.args.len());

        // Add command name
        if let Some(name) = self.name.as_literal() {
            argv.push(name.to_string());
        } else {
            return None;
        }

        // Add arguments
        for arg in &self.args {
            if let Some(lit) = arg.as_literal() {
                argv.push(lit.to_string());
            } else {
                return None;
            }
        }

        Some(FlatCommand { argv })
    }
}

/// A flat representation of a simple command with no variable expansions.
///
/// This is an optimized representation for commands like:
/// - `echo hello world`
/// - `ls -la /tmp`
/// - `grep -r pattern .`
///
/// Most shell commands fall into this category, so having an efficient
/// representation avoids the overhead of the full AST structure.
#[derive(Debug, Clone)]
pub struct FlatCommand {
    /// Command name (index 0) and arguments (indices 1..)
    /// SmallVec avoids heap allocation for commands with ≤8 arguments
    pub argv: SmallVec<[String; 8]>,
}

impl FlatCommand {
    /// Create a new flat command from a command name
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        let mut argv = SmallVec::new();
        argv.push(name.into());
        Self { argv }
    }

    /// Create a flat command with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            argv: SmallVec::with_capacity(capacity),
        }
    }

    /// Get the command name
    #[inline]
    pub fn name(&self) -> &str {
        self.argv.first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Get the arguments (excluding command name)
    #[inline]
    pub fn args(&self) -> &[String] {
        if self.argv.len() > 1 {
            &self.argv[1..]
        } else {
            &[]
        }
    }

    /// Get all argv (command name + arguments)
    #[inline]
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Add an argument
    #[inline]
    pub fn push_arg(&mut self, arg: impl Into<String>) {
        self.argv.push(arg.into());
    }

    /// Number of arguments (excluding command name)
    #[inline]
    pub fn argc(&self) -> usize {
        self.argv.len().saturating_sub(1)
    }

    /// Total length including command name
    #[inline]
    pub fn len(&self) -> usize {
        self.argv.len()
    }

    /// Check if empty (no command name)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.argv.is_empty()
    }

    /// Convert back to a SimpleCommand for cases where full AST is needed
    pub fn to_simple_command(&self, span: Span) -> SimpleCommand {
        let name = if let Some(n) = self.argv.first() {
            Word::literal(n.clone(), span)
        } else {
            Word::literal("", span)
        };

        let args = self.args()
            .iter()
            .map(|a| Word::literal(a.clone(), span))
            .collect();

        SimpleCommand {
            name,
            args,
            assignments: vec![],
        }
    }
}

impl From<Vec<String>> for FlatCommand {
    fn from(argv: Vec<String>) -> Self {
        Self {
            argv: SmallVec::from_vec(argv),
        }
    }
}

impl From<&[&str]> for FlatCommand {
    fn from(argv: &[&str]) -> Self {
        Self {
            argv: argv.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// A word that may contain expansions
#[derive(Debug, Clone)]
pub struct Word {
    pub parts: Vec<WordPart>,
    pub span: Span,
}

impl Word {
    pub fn literal(s: impl Into<String>, span: Span) -> Self {
        Self {
            parts: vec![WordPart::Literal(s.into())],
            span,
        }
    }

    /// Check if this word is a single literal (no expansion needed)
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.parts.len() == 1 && matches!(&self.parts[0], WordPart::Literal(_))
    }

    /// Get the literal value if this is a simple literal word
    #[inline]
    pub fn as_literal(&self) -> Option<&str> {
        if self.parts.len() == 1
            && let WordPart::Literal(s) = &self.parts[0] {
                return Some(s);
            }
        None
    }

    /// Check if this word needs expansion (contains variables, command subs, etc.)
    ///
    /// Returns false if the word is a constant that doesn't depend on runtime state.
    /// Used for caching and lazy expansion optimization.
    #[inline]
    pub fn needs_expansion(&self) -> bool {
        for part in &self.parts {
            match part {
                WordPart::Literal(s) => {
                    // Check for embedded variables in literals (from double-quoted strings)
                    if s.contains('$') || s.starts_with("$__ARITH__") {
                        return true;
                    }
                }
                WordPart::Variable(_) => return true,
                WordPart::SpecialVar(_) => return true,
                WordPart::BraceExpansion(_) => return true,
                WordPart::CommandSub(_) => return true,
                WordPart::BacktickSub(_) => return true,
                WordPart::ArithmeticSub(_) => return true,
                WordPart::ProcessSub { .. } => return true,
                WordPart::Glob(_) => {
                    // Globs don't need variable expansion, but might need glob expansion
                    // We don't consider globs as "needing expansion" for caching purposes
                    // since they're constant patterns
                }
            }
        }
        false
    }

    /// Check if this word is a constant (can be cached)
    ///
    /// A constant word has no runtime-dependent parts.
    #[inline]
    pub fn is_constant(&self) -> bool {
        !self.needs_expansion()
    }
}

/// Parts of a word
#[derive(Debug, Clone)]
pub enum WordPart {
    /// Literal text
    Literal(String),
    /// Variable expansion $var
    Variable(String),
    /// Brace expansion ${...}
    BraceExpansion(BraceExpansion),
    /// Command substitution $(...)
    CommandSub(Vec<Statement>),
    /// Backtick command substitution `...`
    BacktickSub(String),
    /// Arithmetic expansion $((...))
    ArithmeticSub(ArithExpr),
    /// Process substitution <(...) or >(...)
    ProcessSub {
        direction: ProcessSubDir,
        commands: Vec<Statement>,
    },
    /// Glob pattern
    Glob(String),
    /// Special variable $?, $!, etc.
    SpecialVar(char),
}

/// Direction of process substitution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSubDir {
    In,  // <(...)
    Out, // >(...)
}

/// Brace expansion types
#[derive(Debug, Clone)]
pub enum BraceExpansion {
    /// Simple ${var}
    Simple(String),
    /// Default ${var:-default}
    Default {
        var: String,
        default: Box<Word>,
        null_or_unset: bool, // :- vs -
    },
    /// Assign default ${var:=default}
    AssignDefault {
        var: String,
        default: Box<Word>,
        null_or_unset: bool,
    },
    /// Error if unset ${var:?message}
    Error {
        var: String,
        message: Option<Box<Word>>,
        null_or_unset: bool,
    },
    /// Use alternative ${var:+alternative}
    Alternative {
        var: String,
        alternative: Box<Word>,
        null_or_unset: bool,
    },
    /// Substring ${var:offset:length}
    Substring {
        var: String,
        offset: ArithExpr,
        length: Option<ArithExpr>,
    },
    /// Length ${#var}
    Length(String),
    /// Remove prefix ${var#pattern} or ${var##pattern}
    RemovePrefix {
        var: String,
        pattern: String,
        greedy: bool,
    },
    /// Remove suffix ${var%pattern} or ${var%%pattern}
    RemoveSuffix {
        var: String,
        pattern: String,
        greedy: bool,
    },
    /// Replace ${var/pattern/replacement}
    Replace {
        var: String,
        pattern: String,
        replacement: String,
        all: bool,
    },
    /// Case modification ${var^}, ${var^^}, ${var,}, ${var,,}
    CaseModify {
        var: String,
        mode: CaseModifyMode,
    },
    /// Array indexing ${array[index]}
    ArrayIndex {
        var: String,
        index: Box<Word>,
    },
    /// Array slice ${array[@]:offset:length}
    ArraySlice {
        var: String,
        offset: ArithExpr,
        length: Option<ArithExpr>,
    },
}

/// Case modification modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseModifyMode {
    UpperFirst,  // ^
    UpperAll,    // ^^
    LowerFirst,  // ,
    LowerAll,    // ,,
}

/// Arithmetic expression
#[derive(Debug, Clone)]
pub enum ArithExpr {
    Number(i64),
    Float(f64),
    Variable(String),
    UnaryOp {
        op: UnaryOp,
        operand: Box<ArithExpr>,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<ArithExpr>,
        right: Box<ArithExpr>,
    },
    Ternary {
        condition: Box<ArithExpr>,
        then_expr: Box<ArithExpr>,
        else_expr: Box<ArithExpr>,
    },
    Assignment {
        var: String,
        op: AssignOp,
        value: Box<ArithExpr>,
    },
    PreIncrement(String),
    PreDecrement(String),
    PostIncrement(String),
    PostDecrement(String),
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,     // -
    BitwiseNot, // ~
    LogicalNot, // !
    Plus,       // +
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    LogicalAnd,
    LogicalOr,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Assignment operators for arithmetic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

/// Variable assignment
#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub value: Option<Word>,
    pub op: AssignmentOp,
    pub export: bool,
    pub local: bool,
    pub readonly: bool,
    pub span: Span,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,     // =
    Append,     // +=
    AssignOnly, // -=
}

/// Redirection
#[derive(Debug, Clone)]
pub struct Redirect {
    pub kind: RedirectKind,
    pub fd: Option<i32>,
    pub target: RedirectTarget,
    pub span: Span,
}

/// Types of redirections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectKind {
    Input,          // <
    Output,         // >
    Append,         // >>
    InputOutput,    // <>
    DupInput,       // <&
    DupOutput,      // >&
    HereDoc,        // <<
    HereDocStrip,   // <<-
    HereString,     // <<<
    Clobber,        // >|
}

/// Target of a redirection
#[derive(Debug, Clone)]
pub enum RedirectTarget {
    File(Word),
    Fd(i32),
    Close,
    HereDoc {
        delimiter: String,
        content: String,
        quoted: bool,
    },
    HereString(Word),
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Vec<Statement>,
    pub then_branch: Vec<Statement>,
    pub elif_branches: Vec<(Vec<Statement>, Vec<Statement>)>,
    pub else_branch: Option<Vec<Statement>>,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForLoop {
    pub var: String,
    pub items: Option<Vec<Word>>,
    pub body: Vec<Statement>,
    pub span: Span,
    /// Unique ID for JIT hotspot tracking
    pub loop_id: crate::jit::LoopId,
}

/// C-style for loop
#[derive(Debug, Clone)]
pub struct CStyleFor {
    pub init: Option<ArithExpr>,
    pub condition: Option<ArithExpr>,
    pub update: Option<ArithExpr>,
    pub body: Vec<Statement>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Vec<Statement>,
    pub body: Vec<Statement>,
    pub span: Span,
    /// Unique ID for JIT hotspot tracking
    pub loop_id: crate::jit::LoopId,
}

/// Until loop
#[derive(Debug, Clone)]
pub struct UntilLoop {
    pub condition: Vec<Statement>,
    pub body: Vec<Statement>,
    pub span: Span,
    /// Unique ID for JIT hotspot tracking
    pub loop_id: crate::jit::LoopId,
}

/// Case statement
#[derive(Debug, Clone)]
pub struct CaseStatement {
    pub word: Word,
    pub arms: Vec<CaseArm>,
    pub span: Span,
}

/// Case arm
#[derive(Debug, Clone)]
pub struct CaseArm {
    pub patterns: Vec<Word>,
    pub body: Vec<Statement>,
    pub terminator: CaseTerminator,
}

/// Case arm terminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseTerminator {
    Break,       // ;;
    FallThrough, // ;&
    Continue,    // ;;&
}

/// Select statement
#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub var: String,
    pub items: Option<Vec<Word>>,
    pub body: Vec<Statement>,
    pub span: Span,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    /// Named parameters (franken extension): fn greet(name, greeting) { ... }
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub local_vars: Vec<String>,
    pub span: Span,
}

// ============================================================================
// franken-specific enhanced syntax
// ============================================================================

/// franken match expression (similar to Rust match)
#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub value: Word,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Vec<Statement>>,
    pub body: Vec<Statement>,
}

/// Match patterns
#[derive(Debug, Clone)]
pub enum MatchPattern {
    /// Literal value
    Literal(Word),
    /// Glob pattern
    Glob(String),
    /// Regex pattern
    Regex(String),
    /// Range pattern (1..10)
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    /// Multiple patterns (|)
    Or(Vec<MatchPattern>),
    /// Bind pattern (name @ pattern)
    Bind {
        name: String,
        pattern: Box<MatchPattern>,
    },
    /// Wildcard (_)
    Wildcard,
}

/// franken infinite loop
#[derive(Debug, Clone)]
pub struct LoopStatement {
    pub body: Vec<Statement>,
    pub span: Span,
    /// Unique ID for JIT hotspot tracking
    pub loop_id: crate::jit::LoopId,
}

/// franken let binding
#[derive(Debug, Clone)]
pub struct LetBinding {
    pub name: String,
    pub value: Word,
    pub span: Span,
}

/// franken const binding
#[derive(Debug, Clone)]
pub struct ConstBinding {
    pub name: String,
    pub value: Word,
    pub span: Span,
}

/// franken try-catch-finally
#[derive(Debug, Clone)]
pub struct TryStatement {
    pub try_block: Vec<Statement>,
    pub catch_var: Option<String>,
    pub catch_block: Option<Vec<Statement>>,
    pub finally_block: Option<Vec<Statement>>,
    pub span: Span,
}

// ============================================================================
// Fish-compatible syntax
// ============================================================================

/// Fish-style switch statement (switch...case...end)
#[derive(Debug, Clone)]
pub struct FishSwitchStatement {
    pub value: Word,
    pub cases: Vec<FishCase>,
    pub span: Span,
}

/// Fish-style case arm
#[derive(Debug, Clone)]
pub struct FishCase {
    pub patterns: Vec<Word>,
    pub body: Vec<Statement>,
}

/// Test expression for [[ ]]
#[derive(Debug, Clone)]
pub enum TestExpr {
    // File tests
    FileExists(Word),       // -e
    IsFile(Word),           // -f
    IsDir(Word),            // -d
    IsSymlink(Word),        // -L
    IsReadable(Word),       // -r
    IsWritable(Word),       // -w
    IsExecutable(Word),     // -x
    IsOwned(Word),          // -O
    FileSize(Word),         // -s
    IsNewer(Word, Word),    // -nt
    IsOlder(Word, Word),    // -ot
    SameFile(Word, Word),   // -ef

    // String tests
    StringEmpty(Word),      // -z
    StringNotEmpty(Word),   // -n
    StringEqual(Word, Word),    // == or =
    StringNotEqual(Word, Word), // !=
    StringLess(Word, Word),     // <
    StringGreater(Word, Word),  // >
    StringMatch(Word, Word),    // =~

    // Integer tests
    IntEqual(Word, Word),       // -eq
    IntNotEqual(Word, Word),    // -ne
    IntLess(Word, Word),        // -lt
    IntLessEqual(Word, Word),   // -le
    IntGreater(Word, Word),     // -gt
    IntGreaterEqual(Word, Word),// -ge

    // Logical
    And(Box<TestExpr>, Box<TestExpr>),
    Or(Box<TestExpr>, Box<TestExpr>),
    Not(Box<TestExpr>),

    // Grouping
    Group(Box<TestExpr>),

    // Variable tests
    VarSet(String),         // -v
    VarRef(String),         // -R
}

