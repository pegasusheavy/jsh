//! Bytecode instruction set for Franken Shell VM
//!
//! This module defines the opcodes and instruction format used by the VM.

use std::fmt;

/// Operation codes for the bytecode VM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // ========================================================================
    // Stack Operations
    // ========================================================================

    /// No operation
    Nop = 0,
    /// Push a constant onto the stack (index in constant pool)
    Const = 1,
    /// Push an integer literal
    ConstInt = 2,
    /// Push empty string
    ConstEmpty = 3,
    /// Duplicate top of stack
    Dup = 4,
    /// Pop top of stack
    Pop = 5,
    /// Swap top two stack elements
    Swap = 6,

    // ========================================================================
    // Variable Operations
    // ========================================================================

    /// Get variable value (name index in constant pool)
    GetVar = 10,
    /// Set variable value (name index in constant pool)
    SetVar = 11,
    /// Get local variable (slot index)
    GetLocal = 12,
    /// Set local variable (slot index)
    SetLocal = 13,
    /// Export variable
    Export = 14,
    /// Unset variable
    Unset = 15,
    /// Declare local variable
    DeclareLocal = 16,

    // ========================================================================
    // Arithmetic Operations
    // ========================================================================

    /// Add two integers
    Add = 20,
    /// Subtract
    Sub = 21,
    /// Multiply
    Mul = 22,
    /// Divide
    Div = 23,
    /// Modulo
    Mod = 24,
    /// Negate
    Neg = 25,
    /// Increment
    Inc = 26,
    /// Decrement
    Dec = 27,
    /// Bitwise AND
    BitAnd = 28,
    /// Bitwise OR
    BitOr = 29,
    /// Bitwise XOR
    BitXor = 30,
    /// Bitwise NOT
    BitNot = 31,
    /// Left shift
    Shl = 32,
    /// Right shift
    Shr = 33,

    // ========================================================================
    // Comparison Operations
    // ========================================================================

    /// Equal
    Eq = 40,
    /// Not equal
    Ne = 41,
    /// Less than
    Lt = 42,
    /// Less than or equal
    Le = 43,
    /// Greater than
    Gt = 44,
    /// Greater than or equal
    Ge = 45,
    /// String equal
    StrEq = 46,
    /// String not equal
    StrNe = 47,
    /// String less than
    StrLt = 48,
    /// String greater than
    StrGt = 49,

    // ========================================================================
    // Logical Operations
    // ========================================================================

    /// Logical AND
    And = 50,
    /// Logical OR
    Or = 51,
    /// Logical NOT
    Not = 52,

    // ========================================================================
    // Control Flow
    // ========================================================================

    /// Unconditional jump (offset)
    Jump = 60,
    /// Jump if top of stack is true
    JumpIfTrue = 61,
    /// Jump if top of stack is false
    JumpIfFalse = 62,
    /// Jump if top of stack is zero
    JumpIfZero = 63,
    /// Jump if top of stack is non-zero
    JumpIfNonZero = 64,
    /// Call function (function index)
    Call = 65,
    /// Return from function
    Return = 66,
    /// Break from loop
    Break = 67,
    /// Continue to next iteration
    Continue = 68,

    // ========================================================================
    // Command Execution
    // ========================================================================

    /// Execute builtin command (builtin index, argc)
    Builtin = 70,
    /// Execute external command
    External = 71,
    /// Start pipeline
    PipelineStart = 72,
    /// Add command to pipeline
    PipelineAdd = 73,
    /// Execute pipeline
    PipelineExec = 74,
    /// Execute command substitution
    CommandSub = 75,
    /// Execute in subshell
    Subshell = 76,

    // ========================================================================
    // I/O Operations
    // ========================================================================

    /// Print string (no newline)
    Print = 80,
    /// Print string with newline
    PrintLn = 81,
    /// Read input
    Read = 82,
    /// Redirect stdout
    RedirectOut = 83,
    /// Redirect stdin
    RedirectIn = 84,
    /// Redirect stderr
    RedirectErr = 85,
    /// Append redirect
    RedirectAppend = 86,

    // ========================================================================
    // String Operations
    // ========================================================================

    /// Concatenate two strings
    Concat = 90,
    /// Get string length
    StrLen = 91,
    /// Get substring
    Substr = 92,
    /// String replace
    Replace = 93,
    /// Glob expand
    Glob = 94,

    // ========================================================================
    // Array/List Operations
    // ========================================================================

    /// Create array
    ArrayNew = 100,
    /// Push element to array
    ArrayPush = 101,
    /// Get array element
    ArrayGet = 102,
    /// Set array element
    ArraySet = 103,
    /// Get array length
    ArrayLen = 104,

    // ========================================================================
    // Test Operations (for [ and [[ )
    // ========================================================================

    /// File exists (-e)
    TestExists = 110,
    /// File is regular (-f)
    TestFile = 111,
    /// File is directory (-d)
    TestDir = 112,
    /// File is readable (-r)
    TestReadable = 113,
    /// File is writable (-w)
    TestWritable = 114,
    /// File is executable (-x)
    TestExecutable = 115,
    /// String is non-empty (-n)
    TestNonEmpty = 116,
    /// String is empty (-z)
    TestEmpty = 117,

    // ========================================================================
    // Special
    // ========================================================================

    /// Halt execution
    Halt = 255,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => OpCode::Nop,
            1 => OpCode::Const,
            2 => OpCode::ConstInt,
            3 => OpCode::ConstEmpty,
            4 => OpCode::Dup,
            5 => OpCode::Pop,
            6 => OpCode::Swap,

            10 => OpCode::GetVar,
            11 => OpCode::SetVar,
            12 => OpCode::GetLocal,
            13 => OpCode::SetLocal,
            14 => OpCode::Export,
            15 => OpCode::Unset,
            16 => OpCode::DeclareLocal,

            20 => OpCode::Add,
            21 => OpCode::Sub,
            22 => OpCode::Mul,
            23 => OpCode::Div,
            24 => OpCode::Mod,
            25 => OpCode::Neg,
            26 => OpCode::Inc,
            27 => OpCode::Dec,
            28 => OpCode::BitAnd,
            29 => OpCode::BitOr,
            30 => OpCode::BitXor,
            31 => OpCode::BitNot,
            32 => OpCode::Shl,
            33 => OpCode::Shr,

            40 => OpCode::Eq,
            41 => OpCode::Ne,
            42 => OpCode::Lt,
            43 => OpCode::Le,
            44 => OpCode::Gt,
            45 => OpCode::Ge,
            46 => OpCode::StrEq,
            47 => OpCode::StrNe,
            48 => OpCode::StrLt,
            49 => OpCode::StrGt,

            50 => OpCode::And,
            51 => OpCode::Or,
            52 => OpCode::Not,

            60 => OpCode::Jump,
            61 => OpCode::JumpIfTrue,
            62 => OpCode::JumpIfFalse,
            63 => OpCode::JumpIfZero,
            64 => OpCode::JumpIfNonZero,
            65 => OpCode::Call,
            66 => OpCode::Return,
            67 => OpCode::Break,
            68 => OpCode::Continue,

            70 => OpCode::Builtin,
            71 => OpCode::External,
            72 => OpCode::PipelineStart,
            73 => OpCode::PipelineAdd,
            74 => OpCode::PipelineExec,
            75 => OpCode::CommandSub,
            76 => OpCode::Subshell,

            80 => OpCode::Print,
            81 => OpCode::PrintLn,
            82 => OpCode::Read,
            83 => OpCode::RedirectOut,
            84 => OpCode::RedirectIn,
            85 => OpCode::RedirectErr,
            86 => OpCode::RedirectAppend,

            90 => OpCode::Concat,
            91 => OpCode::StrLen,
            92 => OpCode::Substr,
            93 => OpCode::Replace,
            94 => OpCode::Glob,

            100 => OpCode::ArrayNew,
            101 => OpCode::ArrayPush,
            102 => OpCode::ArrayGet,
            103 => OpCode::ArraySet,
            104 => OpCode::ArrayLen,

            110 => OpCode::TestExists,
            111 => OpCode::TestFile,
            112 => OpCode::TestDir,
            113 => OpCode::TestReadable,
            114 => OpCode::TestWritable,
            115 => OpCode::TestExecutable,
            116 => OpCode::TestNonEmpty,
            117 => OpCode::TestEmpty,

            255 => OpCode::Halt,
            _ => OpCode::Nop,
        }
    }
}

/// A single bytecode instruction
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Operation code
    pub op: OpCode,
    /// Operand (meaning depends on opcode)
    pub operand: i32,
    /// Source line number (for debugging)
    pub line: u32,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(op: OpCode, operand: i32, line: u32) -> Self {
        Self { op, operand, line }
    }

    /// Create instruction with no operand
    pub fn simple(op: OpCode, line: u32) -> Self {
        Self { op, operand: 0, line }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.op)?;
        if self.operand != 0 {
            write!(f, " {}", self.operand)?;
        }
        Ok(())
    }
}

/// A chunk of bytecode with its constant pool
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Bytecode instructions
    pub code: Vec<Instruction>,
    /// String constants
    pub constants: Vec<String>,
    /// Integer constants
    pub int_constants: Vec<i64>,
    /// Function definitions (name, start offset)
    pub functions: Vec<(String, usize)>,
    /// Source file name (for debugging)
    pub source_name: String,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// Create a new empty chunk
    pub fn new() -> Self {
        Self {
            code: Vec::with_capacity(256),
            constants: Vec::with_capacity(64),
            int_constants: Vec::with_capacity(32),
            functions: Vec::new(),
            source_name: String::new(),
        }
    }

    /// Add an instruction and return its index
    pub fn emit(&mut self, instruction: Instruction) -> usize {
        let idx = self.code.len();
        self.code.push(instruction);
        idx
    }

    /// Emit a simple instruction (no operand)
    pub fn emit_simple(&mut self, op: OpCode, line: u32) -> usize {
        self.emit(Instruction::simple(op, line))
    }

    /// Emit an instruction with operand
    pub fn emit_op(&mut self, op: OpCode, operand: i32, line: u32) -> usize {
        self.emit(Instruction::new(op, operand, line))
    }

    /// Add a string constant and return its index
    pub fn add_constant(&mut self, value: String) -> usize {
        // Check if constant already exists
        if let Some(idx) = self.constants.iter().position(|c| c == &value) {
            return idx;
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }

    /// Add an integer constant and return its index
    pub fn add_int_constant(&mut self, value: i64) -> usize {
        if let Some(idx) = self.int_constants.iter().position(|&c| c == value) {
            return idx;
        }
        let idx = self.int_constants.len();
        self.int_constants.push(value);
        idx
    }

    /// Get the current code offset
    pub fn current_offset(&self) -> usize {
        self.code.len()
    }

    /// Patch a jump instruction's operand
    pub fn patch_jump(&mut self, offset: usize, target: i32) {
        if let Some(inst) = self.code.get_mut(offset) {
            inst.operand = target;
        }
    }

    /// Disassemble the chunk for debugging
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("== {} ==\n", self.source_name));

        for (i, inst) in self.code.iter().enumerate() {
            output.push_str(&format!("{:04} {:4} {}\n", i, inst.line, inst));
        }

        if !self.constants.is_empty() {
            output.push_str("\n-- Constants --\n");
            for (i, c) in self.constants.iter().enumerate() {
                output.push_str(&format!("{:04} \"{}\"\n", i, c));
            }
        }

        if !self.int_constants.is_empty() {
            output.push_str("\n-- Int Constants --\n");
            for (i, c) in self.int_constants.iter().enumerate() {
                output.push_str(&format!("{:04} {}\n", i, c));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversion() {
        assert_eq!(OpCode::from(0), OpCode::Nop);
        assert_eq!(OpCode::from(1), OpCode::Const);
        assert_eq!(OpCode::from(255), OpCode::Halt);
    }

    #[test]
    fn test_chunk_emit() {
        let mut chunk = Chunk::new();
        chunk.emit_simple(OpCode::Nop, 1);
        chunk.emit_op(OpCode::Const, 0, 2);

        assert_eq!(chunk.code.len(), 2);
        assert_eq!(chunk.code[0].op, OpCode::Nop);
        assert_eq!(chunk.code[1].op, OpCode::Const);
        assert_eq!(chunk.code[1].operand, 0);
    }

    #[test]
    fn test_chunk_constants() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant("hello".to_string());
        let idx2 = chunk.add_constant("world".to_string());
        let idx3 = chunk.add_constant("hello".to_string()); // Duplicate

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0); // Should reuse existing
        assert_eq!(chunk.constants.len(), 2);
    }

    #[test]
    fn test_chunk_patch_jump() {
        let mut chunk = Chunk::new();
        let jump_idx = chunk.emit_op(OpCode::Jump, 0, 1);
        chunk.emit_simple(OpCode::Nop, 2);
        chunk.emit_simple(OpCode::Nop, 3);

        let target = chunk.current_offset() as i32;
        chunk.patch_jump(jump_idx, target);

        assert_eq!(chunk.code[jump_idx].operand, 3);
    }
}

