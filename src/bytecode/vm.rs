//! Stack-based virtual machine for executing bytecode
//!
//! The VM executes bytecode instructions using a value stack and call stack.

use super::instruction::{Chunk, OpCode};
use rustc_hash::FxHashMap;
use std::io::{self, Write};

/// Maximum stack size
const MAX_STACK_SIZE: usize = 1024;

/// Maximum call depth
const MAX_CALL_DEPTH: usize = 256;

/// VM error types
#[derive(Debug)]
pub enum VMError {
    /// Stack overflow
    StackOverflow,
    /// Stack underflow
    StackUnderflow,
    /// Call stack overflow
    CallStackOverflow,
    /// Invalid opcode
    InvalidOpcode(u8),
    /// Division by zero
    DivisionByZero,
    /// Invalid operand
    InvalidOperand(String),
    /// Undefined variable
    UndefinedVariable(String),
    /// Type error
    TypeError(String),
    /// Break outside loop
    BreakOutsideLoop,
    /// Continue outside loop
    ContinueOutsideLoop,
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::StackOverflow => write!(f, "stack overflow"),
            VMError::StackUnderflow => write!(f, "stack underflow"),
            VMError::CallStackOverflow => write!(f, "call stack overflow"),
            VMError::InvalidOpcode(op) => write!(f, "invalid opcode: {}", op),
            VMError::DivisionByZero => write!(f, "division by zero"),
            VMError::InvalidOperand(msg) => write!(f, "invalid operand: {}", msg),
            VMError::UndefinedVariable(name) => write!(f, "undefined variable: {}", name),
            VMError::TypeError(msg) => write!(f, "type error: {}", msg),
            VMError::BreakOutsideLoop => write!(f, "break outside loop"),
            VMError::ContinueOutsideLoop => write!(f, "continue outside loop"),
            VMError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for VMError {}

/// VM result type
pub type VMResult<T> = std::result::Result<T, VMError>;

/// Value on the VM stack
#[derive(Debug, Clone, Default)]
pub enum Value {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Boolean value
    Bool(bool),
    /// Null/empty
    #[default]
    Null,
    /// Array value
    Array(Vec<Value>),
}

impl Value {
    /// Convert to string
    pub fn to_string_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            Value::Null => String::new(),
            Value::Array(arr) => arr
                .iter()
                .map(|v| v.to_string_value())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// Convert to integer
    pub fn to_int(&self) -> i64 {
        match self {
            Value::String(s) => s.parse().unwrap_or(0),
            Value::Int(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Value::Null => 0,
            Value::Array(arr) => arr.len() as i64,
        }
    }

    /// Convert to boolean
    pub fn to_bool(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Int(n) => *n != 0,
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Array(arr) => !arr.is_empty(),
        }
    }

    /// Check if value is truthy (non-zero exit status)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => *n == 0, // Shell convention: 0 = success = true
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::Array(arr) => !arr.is_empty(),
        }
    }
}

/// Call frame for function calls
#[derive(Debug)]
struct CallFrame {
    /// Return address (instruction pointer)
    return_ip: usize,
    /// Stack base pointer
    base_ptr: usize,
    /// Local variables
    #[allow(dead_code)]
    locals: Vec<Value>,
}

/// Bytecode virtual machine
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
    /// Instruction pointer
    ip: usize,
    /// Variables
    vars: FxHashMap<String, Value>,
    /// Call stack
    call_stack: Vec<CallFrame>,
    /// Last exit status
    pub last_status: i32,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new VM
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            vars: FxHashMap::default(),
            call_stack: Vec::with_capacity(64),
            last_status: 0,
        }
    }

    /// Run a chunk of bytecode
    pub fn run(&mut self, chunk: &Chunk) -> VMResult<i32> {
        self.ip = 0;
        self.stack.clear();

        while self.ip < chunk.code.len() {
            let inst = &chunk.code[self.ip];
            self.ip += 1;

            match inst.op {
                OpCode::Nop => {}

                OpCode::Halt => {
                    return Ok(self.last_status);
                }

                // Stack operations
                OpCode::Const => {
                    let idx = inst.operand as usize;
                    let value = chunk
                        .constants
                        .get(idx)
                        .map(|s| Value::String(s.clone()))
                        .unwrap_or(Value::Null);
                    self.push(value)?;
                }

                OpCode::ConstInt => {
                    let idx = inst.operand as usize;
                    let value = chunk
                        .int_constants
                        .get(idx)
                        .map(|&n| Value::Int(n))
                        .unwrap_or(Value::Int(0));
                    self.push(value)?;
                }

                OpCode::ConstEmpty => {
                    self.push(Value::String(String::new()))?;
                }

                OpCode::Dup => {
                    let value = self.peek()?.clone();
                    self.push(value)?;
                }

                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::Swap => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a)?;
                    self.push(b)?;
                }

                // Variable operations
                OpCode::GetVar => {
                    let idx = inst.operand as usize;
                    let name = chunk
                        .constants
                        .get(idx)
                        .ok_or_else(|| VMError::InvalidOperand("invalid constant index".into()))?;
                    let value = self.vars.get(name).cloned().unwrap_or(Value::Null);
                    self.push(value)?;
                }

                OpCode::SetVar => {
                    let idx = inst.operand as usize;
                    let name = chunk
                        .constants
                        .get(idx)
                        .ok_or_else(|| VMError::InvalidOperand("invalid constant index".into()))?
                        .clone();
                    let value = self.pop()?;
                    self.vars.insert(name, value);
                }

                // Arithmetic
                OpCode::Add => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Int(a + b))?;
                }

                OpCode::Sub => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Int(a - b))?;
                }

                OpCode::Mul => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Int(a * b))?;
                }

                OpCode::Div => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.push(Value::Int(a / b))?;
                }

                OpCode::Mod => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.push(Value::Int(a % b))?;
                }

                OpCode::Neg => {
                    let a = self.pop()?.to_int();
                    self.push(Value::Int(-a))?;
                }

                // Comparison
                OpCode::Eq => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a == b))?;
                }

                OpCode::Ne => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a != b))?;
                }

                OpCode::Lt => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a < b))?;
                }

                OpCode::Le => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a <= b))?;
                }

                OpCode::Gt => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a > b))?;
                }

                OpCode::Ge => {
                    let b = self.pop()?.to_int();
                    let a = self.pop()?.to_int();
                    self.push(Value::Bool(a >= b))?;
                }

                OpCode::StrEq => {
                    let b = self.pop()?.to_string_value();
                    let a = self.pop()?.to_string_value();
                    self.push(Value::Bool(a == b))?;
                }

                OpCode::StrNe => {
                    let b = self.pop()?.to_string_value();
                    let a = self.pop()?.to_string_value();
                    self.push(Value::Bool(a != b))?;
                }

                // Logical
                OpCode::And => {
                    let b = self.pop()?.to_bool();
                    let a = self.pop()?.to_bool();
                    self.push(Value::Bool(a && b))?;
                }

                OpCode::Or => {
                    let b = self.pop()?.to_bool();
                    let a = self.pop()?.to_bool();
                    self.push(Value::Bool(a || b))?;
                }

                OpCode::Not => {
                    let a = self.pop()?.to_bool();
                    self.push(Value::Bool(!a))?;
                }

                // Control flow
                OpCode::Jump => {
                    self.ip = inst.operand as usize;
                }

                OpCode::JumpIfTrue => {
                    let cond = self.peek()?.is_truthy();
                    if cond {
                        self.ip = inst.operand as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    let cond = self.peek()?.is_truthy();
                    if !cond {
                        self.ip = inst.operand as usize;
                    }
                }

                OpCode::JumpIfZero => {
                    let val = self.peek()?.to_int();
                    if val == 0 {
                        self.ip = inst.operand as usize;
                    }
                }

                OpCode::JumpIfNonZero => {
                    let val = self.peek()?.to_int();
                    if val != 0 {
                        self.ip = inst.operand as usize;
                    }
                }

                OpCode::Call => {
                    let idx = inst.operand as usize;
                    let name = chunk
                        .constants
                        .get(idx)
                        .ok_or_else(|| VMError::InvalidOperand("invalid function".into()))?;

                    // Find function
                    if let Some(&(_, start)) = chunk.functions.iter().find(|(n, _)| n == name) {
                        if self.call_stack.len() >= MAX_CALL_DEPTH {
                            return Err(VMError::CallStackOverflow);
                        }

                        self.call_stack.push(CallFrame {
                            return_ip: self.ip,
                            base_ptr: self.stack.len(),
                            locals: Vec::new(),
                        });
                        self.ip = start;
                    }
                }

                OpCode::Return => {
                    if let Some(frame) = self.call_stack.pop() {
                        // Get return value if any
                        let return_val = if self.stack.len() > frame.base_ptr {
                            self.pop()?
                        } else {
                            Value::Int(0)
                        };

                        // Restore stack
                        self.stack.truncate(frame.base_ptr);
                        self.push(return_val)?;

                        self.ip = frame.return_ip;
                    }
                }

                // I/O
                OpCode::Print => {
                    let value = self.pop()?.to_string_value();
                    print!("{}", value);
                    io::stdout()
                        .flush()
                        .map_err(|e| VMError::IoError(e.to_string()))?;
                }

                OpCode::PrintLn => {
                    let value = self.pop()?.to_string_value();
                    println!("{}", value);
                }

                // String operations
                OpCode::Concat => {
                    let b = self.pop()?.to_string_value();
                    let a = self.pop()?.to_string_value();
                    self.push(Value::String(a + &b))?;
                }

                OpCode::StrLen => {
                    let s = self.pop()?.to_string_value();
                    self.push(Value::Int(s.len() as i64))?;
                }

                // Builtin execution (simplified)
                OpCode::Builtin => {
                    let idx = inst.operand as usize;
                    let name = chunk
                        .constants
                        .get(idx)
                        .ok_or_else(|| VMError::InvalidOperand("invalid builtin".into()))?;

                    // Simple builtin handling
                    match name.as_str() {
                        "echo" => {
                            // Pop all args and print
                            let args: Vec<String> =
                                self.stack.drain(..).map(|v| v.to_string_value()).collect();
                            println!("{}", args.join(" "));
                            self.last_status = 0;
                        }
                        "true" => {
                            self.last_status = 0;
                        }
                        "false" => {
                            self.last_status = 1;
                        }
                        _ => {
                            // Clear stack and return success
                            self.stack.clear();
                            self.last_status = 0;
                        }
                    }
                    self.push(Value::Int(self.last_status as i64))?;
                }

                OpCode::External => {
                    // External command execution (simplified)
                    let argc = inst.operand as usize;
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop()?.to_string_value());
                    }
                    let cmd = self.pop()?.to_string_value();

                    // Execute command
                    match std::process::Command::new(&cmd)
                        .args(args.iter().rev())
                        .status()
                    {
                        Ok(status) => {
                            self.last_status = status.code().unwrap_or(1);
                        }
                        Err(_) => {
                            self.last_status = 127; // Command not found
                        }
                    }
                    self.push(Value::Int(self.last_status as i64))?;
                }

                // Pipeline (simplified - executes sequentially)
                OpCode::PipelineStart | OpCode::PipelineAdd | OpCode::PipelineExec => {
                    // Simplified: just continue
                }

                // Test operations
                OpCode::TestNonEmpty => {
                    let s = self.pop()?.to_string_value();
                    self.push(Value::Bool(!s.is_empty()))?;
                }

                OpCode::TestEmpty => {
                    let s = self.pop()?.to_string_value();
                    self.push(Value::Bool(s.is_empty()))?;
                }

                // Unhandled opcodes
                _ => {
                    // Skip unimplemented opcodes
                }
            }
        }

        Ok(self.last_status)
    }

    // Stack operations

    fn push(&mut self, value: Value) -> VMResult<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VMError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> VMResult<Value> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn peek(&self) -> VMResult<&Value> {
        self.stack.last().ok_or(VMError::StackUnderflow)
    }

    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    /// Set a variable value
    pub fn set_var(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::super::compiler::Compiler;
    use super::*;

    #[test]
    fn test_vm_stack_operations() {
        let mut vm = VM::new();

        vm.push(Value::Int(42)).unwrap();
        assert_eq!(vm.pop().unwrap().to_int(), 42);
    }

    #[test]
    fn test_vm_arithmetic() {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile_string("x=$((2 + 3))").unwrap();

        let mut vm = VM::new();
        let result = vm.run(&chunk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_variables() {
        let mut vm = VM::new();
        vm.set_var("x", Value::Int(42));

        let val = vm.get_var("x").unwrap();
        assert_eq!(val.to_int(), 42);
    }

    #[test]
    fn test_value_conversions() {
        let s = Value::String("42".to_string());
        assert_eq!(s.to_int(), 42);
        assert!(s.to_bool());

        let n = Value::Int(0);
        assert_eq!(n.to_string_value(), "0");
        assert!(!n.to_bool());

        let b = Value::Bool(true);
        assert_eq!(b.to_int(), 1);
    }
}
