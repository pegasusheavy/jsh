//! Bytecode compiler for Franken Shell
//!
//! Compiles AST nodes to bytecode instructions.

use super::instruction::{Chunk, OpCode};
use crate::ast::*;
use crate::error::Result;
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Bytecode compiler
pub struct Compiler {
    /// Current chunk being compiled
    chunk: Chunk,
    /// Current line number
    current_line: u32,
    /// Local variable names in current scope
    locals: Vec<String>,
    /// Loop break targets (for patching)
    break_targets: Vec<Vec<usize>>,
    /// Loop continue targets
    continue_targets: Vec<usize>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            current_line: 1,
            locals: Vec::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        }
    }

    /// Compile a source string to bytecode
    pub fn compile_string(&mut self, source: &str) -> Result<Chunk> {
        self.chunk = Chunk::new();
        self.chunk.source_name = "<string>".to_string();

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;

        self.compile_program(&program)?;
        self.emit_simple(OpCode::Halt);

        Ok(std::mem::take(&mut self.chunk))
    }

    /// Compile a program (list of statements)
    fn compile_program(&mut self, program: &Program) -> Result<()> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    /// Compile a statement
    fn compile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Pipeline(pipeline) => self.compile_pipeline(pipeline),
            Statement::List(list) => self.compile_list(list),
            Statement::Assignment(assign) => self.compile_assignment(assign),
            Statement::If(if_stmt) => self.compile_if(if_stmt),
            Statement::For(for_loop) => self.compile_for(for_loop),
            Statement::While(while_loop) => self.compile_while(while_loop),
            Statement::Until(until_loop) => self.compile_until(until_loop),
            Statement::Case(case_stmt) => self.compile_case(case_stmt),
            Statement::Function(func_def) => self.compile_function(func_def),
            Statement::Return(value) => self.compile_return(value.as_ref()),
            Statement::Break(_) => {
                let jump = self.emit_op(OpCode::Jump, 0);
                if let Some(targets) = self.break_targets.last_mut() {
                    targets.push(jump);
                }
                Ok(())
            }
            Statement::Continue(_) => {
                if let Some(&target) = self.continue_targets.last() {
                    self.emit_op(OpCode::Jump, target as i32);
                }
                Ok(())
            }
            Statement::BraceGroup(stmts) | Statement::BeginBlock(stmts) => {
                for s in stmts {
                    self.compile_statement(s)?;
                }
                Ok(())
            }
            Statement::Subshell(stmts) => {
                // Compile subshell as a separate chunk (simplified)
                for s in stmts {
                    self.compile_statement(s)?;
                }
                Ok(())
            }
            Statement::Empty => Ok(()),
            // Simplified handling for less common statements
            _ => {
                // For unsupported statements, emit a NOP
                self.emit_simple(OpCode::Nop);
                Ok(())
            }
        }
    }

    /// Compile a pipeline
    fn compile_pipeline(&mut self, pipeline: &Pipeline) -> Result<()> {
        if pipeline.commands.len() == 1 {
            // Single command, no pipeline needed
            self.compile_command(&pipeline.commands[0])?;
        } else {
            // Multi-command pipeline
            self.emit_simple(OpCode::PipelineStart);
            for cmd in &pipeline.commands {
                self.compile_command(cmd)?;
                self.emit_simple(OpCode::PipelineAdd);
            }
            self.emit_simple(OpCode::PipelineExec);
        }

        if pipeline.negated {
            self.emit_simple(OpCode::Not);
        }

        Ok(())
    }

    /// Compile a command
    fn compile_command(&mut self, cmd: &Command) -> Result<()> {
        // Handle redirections later
        match &cmd.kind {
            CommandKind::Simple(simple) => self.compile_simple_command(simple),
            CommandKind::Flat(flat) => self.compile_flat_command(flat),
            CommandKind::Compound(stmt) => self.compile_statement(stmt),
            CommandKind::FunctionCall { name, args } => {
                // Push arguments
                for arg in args {
                    self.compile_word(arg)?;
                }
                // Call function
                let name_idx = self.chunk.add_constant(name.clone());
                self.emit_op(OpCode::Call, name_idx as i32);
                Ok(())
            }
            CommandKind::Coproc { command, .. } => self.compile_command(command),
        }
    }

    /// Compile a simple command
    fn compile_simple_command(&mut self, cmd: &SimpleCommand) -> Result<()> {
        // Handle assignments
        for assign in &cmd.assignments {
            self.compile_assignment(assign)?;
        }

        // Get command name
        let name = self.word_to_string(&cmd.name);

        // Check if it's a builtin
        if crate::builtins::is_builtin(&name) {
            // Push arguments onto stack
            for arg in &cmd.args {
                self.compile_word(arg)?;
            }
            let name_idx = self.chunk.add_constant(name);
            self.emit_op(OpCode::Builtin, name_idx as i32);
        } else {
            // External command
            let name_idx = self.chunk.add_constant(name);
            self.emit_op(OpCode::Const, name_idx as i32);
            for arg in &cmd.args {
                self.compile_word(arg)?;
            }
            self.emit_op(OpCode::External, cmd.args.len() as i32);
        }

        Ok(())
    }

    /// Compile a flat command
    fn compile_flat_command(&mut self, cmd: &FlatCommand) -> Result<()> {
        if cmd.is_empty() {
            return Ok(());
        }

        let name = cmd.name();

        if crate::builtins::is_builtin(name) {
            // Push arguments
            for arg in cmd.args() {
                let idx = self.chunk.add_constant(arg.clone());
                self.emit_op(OpCode::Const, idx as i32);
            }
            let name_idx = self.chunk.add_constant(name.to_string());
            self.emit_op(OpCode::Builtin, name_idx as i32);
        } else {
            // External command
            let name_idx = self.chunk.add_constant(name.to_string());
            self.emit_op(OpCode::Const, name_idx as i32);
            for arg in cmd.args() {
                let idx = self.chunk.add_constant(arg.clone());
                self.emit_op(OpCode::Const, idx as i32);
            }
            self.emit_op(OpCode::External, cmd.args().len() as i32);
        }

        Ok(())
    }

    /// Compile a list (&&, ||)
    fn compile_list(&mut self, list: &List) -> Result<()> {
        self.compile_pipeline(&list.first)?;

        for (op, pipeline) in &list.rest {
            match op {
                ListOp::And => {
                    // Jump over next command if false
                    let jump = self.emit_op(OpCode::JumpIfFalse, 0);
                    self.compile_pipeline(pipeline)?;
                    self.chunk
                        .patch_jump(jump, self.chunk.current_offset() as i32);
                }
                ListOp::Or => {
                    // Jump over next command if true
                    let jump = self.emit_op(OpCode::JumpIfTrue, 0);
                    self.compile_pipeline(pipeline)?;
                    self.chunk
                        .patch_jump(jump, self.chunk.current_offset() as i32);
                }
            }
        }

        Ok(())
    }

    /// Compile an assignment
    fn compile_assignment(&mut self, assign: &Assignment) -> Result<()> {
        // Compile the value
        if let Some(ref value) = assign.value {
            self.compile_word(value)?;
        } else {
            self.emit_simple(OpCode::ConstEmpty);
        }

        // Store in variable
        let name_idx = self.chunk.add_constant(assign.name.clone());

        match assign.op {
            AssignmentOp::Assign => {
                self.emit_op(OpCode::SetVar, name_idx as i32);
            }
            AssignmentOp::Append => {
                // Get current value
                self.emit_op(OpCode::GetVar, name_idx as i32);
                self.emit_simple(OpCode::Swap);
                self.emit_simple(OpCode::Concat);
                self.emit_op(OpCode::SetVar, name_idx as i32);
            }
            AssignmentOp::AssignOnly => {
                // Only set if not already set
                self.emit_op(OpCode::GetVar, name_idx as i32);
                let skip_jump = self.emit_op(OpCode::JumpIfNonZero, 0);
                self.emit_simple(OpCode::Pop);
                self.emit_op(OpCode::SetVar, name_idx as i32);
                self.chunk
                    .patch_jump(skip_jump, self.chunk.current_offset() as i32);
            }
        }

        Ok(())
    }

    /// Compile an if statement
    fn compile_if(&mut self, if_stmt: &IfStatement) -> Result<()> {
        // Compile condition
        for stmt in &if_stmt.condition {
            self.compile_statement(stmt)?;
        }

        // Jump to else if false
        let else_jump = self.emit_op(OpCode::JumpIfFalse, 0);

        // Compile then branch
        for stmt in &if_stmt.then_branch {
            self.compile_statement(stmt)?;
        }

        // Jump over else branch
        let end_jump = self.emit_op(OpCode::Jump, 0);

        // Patch else jump
        self.chunk
            .patch_jump(else_jump, self.chunk.current_offset() as i32);

        // Compile elif branches
        let mut elif_end_jumps = Vec::new();
        for (elif_cond, elif_body) in &if_stmt.elif_branches {
            for stmt in elif_cond {
                self.compile_statement(stmt)?;
            }
            let next_elif_jump = self.emit_op(OpCode::JumpIfFalse, 0);
            for stmt in elif_body {
                self.compile_statement(stmt)?;
            }
            elif_end_jumps.push(self.emit_op(OpCode::Jump, 0));
            self.chunk
                .patch_jump(next_elif_jump, self.chunk.current_offset() as i32);
        }

        // Compile else branch
        if let Some(else_branch) = &if_stmt.else_branch {
            for stmt in else_branch {
                self.compile_statement(stmt)?;
            }
        }

        // Patch end jumps
        let end_offset = self.chunk.current_offset() as i32;
        self.chunk.patch_jump(end_jump, end_offset);
        for jump in elif_end_jumps {
            self.chunk.patch_jump(jump, end_offset);
        }

        Ok(())
    }

    /// Compile a for loop
    fn compile_for(&mut self, for_loop: &ForLoop) -> Result<()> {
        // Get items to iterate
        let items = if let Some(item_words) = &for_loop.items {
            item_words
                .iter()
                .map(|w| self.word_to_string(w))
                .collect::<Vec<_>>()
        } else {
            vec!["$@".to_string()] // Use positional params
        };

        let var_idx = self.chunk.add_constant(for_loop.var.clone());

        // Push break targets
        self.break_targets.push(Vec::new());

        for item in items {
            // Set loop variable
            let item_idx = self.chunk.add_constant(item);
            self.emit_op(OpCode::Const, item_idx as i32);
            self.emit_op(OpCode::SetVar, var_idx as i32);

            // Save continue target
            self.continue_targets.push(self.chunk.current_offset());

            // Compile body
            for stmt in &for_loop.body {
                self.compile_statement(stmt)?;
            }

            self.continue_targets.pop();
        }

        // Patch break jumps
        let end_offset = self.chunk.current_offset() as i32;
        if let Some(breaks) = self.break_targets.pop() {
            for jump in breaks {
                self.chunk.patch_jump(jump, end_offset);
            }
        }

        Ok(())
    }

    /// Compile a while loop
    fn compile_while(&mut self, while_loop: &WhileLoop) -> Result<()> {
        let loop_start = self.chunk.current_offset();

        // Push break/continue targets
        self.break_targets.push(Vec::new());
        self.continue_targets.push(loop_start);

        // Compile condition
        for stmt in &while_loop.condition {
            self.compile_statement(stmt)?;
        }

        // Jump out if false
        let exit_jump = self.emit_op(OpCode::JumpIfFalse, 0);

        // Compile body
        for stmt in &while_loop.body {
            self.compile_statement(stmt)?;
        }

        // Jump back to start
        self.emit_op(OpCode::Jump, loop_start as i32);

        // Patch exit jump
        self.chunk
            .patch_jump(exit_jump, self.chunk.current_offset() as i32);

        // Patch breaks
        let end_offset = self.chunk.current_offset() as i32;
        self.continue_targets.pop();
        if let Some(breaks) = self.break_targets.pop() {
            for jump in breaks {
                self.chunk.patch_jump(jump, end_offset);
            }
        }

        Ok(())
    }

    /// Compile an until loop
    fn compile_until(&mut self, until_loop: &UntilLoop) -> Result<()> {
        let loop_start = self.chunk.current_offset();

        self.break_targets.push(Vec::new());
        self.continue_targets.push(loop_start);

        // Compile condition
        for stmt in &until_loop.condition {
            self.compile_statement(stmt)?;
        }

        // Jump out if true (opposite of while)
        let exit_jump = self.emit_op(OpCode::JumpIfTrue, 0);

        // Compile body
        for stmt in &until_loop.body {
            self.compile_statement(stmt)?;
        }

        // Jump back to start
        self.emit_op(OpCode::Jump, loop_start as i32);

        // Patch jumps
        self.chunk
            .patch_jump(exit_jump, self.chunk.current_offset() as i32);

        let end_offset = self.chunk.current_offset() as i32;
        self.continue_targets.pop();
        if let Some(breaks) = self.break_targets.pop() {
            for jump in breaks {
                self.chunk.patch_jump(jump, end_offset);
            }
        }

        Ok(())
    }

    /// Compile a case statement
    fn compile_case(&mut self, case_stmt: &CaseStatement) -> Result<()> {
        // Compile the test word
        self.compile_word(&case_stmt.word)?;

        let mut end_jumps = Vec::new();

        for arm in &case_stmt.arms {
            // For each pattern, test and jump
            let mut pattern_jumps = Vec::new();

            for pattern in &arm.patterns {
                self.emit_simple(OpCode::Dup); // Duplicate test value
                let pattern_str = self.word_to_string(pattern);
                let idx = self.chunk.add_constant(pattern_str);
                self.emit_op(OpCode::Const, idx as i32);
                self.emit_simple(OpCode::StrEq);
                pattern_jumps.push(self.emit_op(OpCode::JumpIfTrue, 0));
            }

            // None matched, jump to next arm
            let next_arm_jump = self.emit_op(OpCode::Jump, 0);

            // Patch pattern jumps to here
            for jump in pattern_jumps {
                self.chunk
                    .patch_jump(jump, self.chunk.current_offset() as i32);
            }

            // Compile arm body
            for stmt in &arm.body {
                self.compile_statement(stmt)?;
            }

            // Jump to end
            end_jumps.push(self.emit_op(OpCode::Jump, 0));

            // Patch next arm jump
            self.chunk
                .patch_jump(next_arm_jump, self.chunk.current_offset() as i32);
        }

        // Patch end jumps
        let end_offset = self.chunk.current_offset() as i32;
        for jump in end_jumps {
            self.chunk.patch_jump(jump, end_offset);
        }

        // Pop the test value
        self.emit_simple(OpCode::Pop);

        Ok(())
    }

    /// Compile a function definition
    fn compile_function(&mut self, func_def: &FunctionDef) -> Result<()> {
        // Save function start offset
        let start = self.chunk.current_offset();

        // Jump over function body (will be called later)
        let skip_jump = self.emit_op(OpCode::Jump, 0);

        // Record function
        self.chunk
            .functions
            .push((func_def.name.clone(), start + 1));

        // Compile function body
        for stmt in &func_def.body {
            self.compile_statement(stmt)?;
        }
        self.emit_simple(OpCode::Return);

        // Patch skip jump
        self.chunk
            .patch_jump(skip_jump, self.chunk.current_offset() as i32);

        Ok(())
    }

    /// Compile a return statement
    fn compile_return(&mut self, value: Option<&Word>) -> Result<()> {
        if let Some(word) = value {
            self.compile_word(word)?;
        } else {
            self.emit_simple(OpCode::ConstEmpty);
        }
        self.emit_simple(OpCode::Return);
        Ok(())
    }

    /// Compile a word (with expansions)
    fn compile_word(&mut self, word: &Word) -> Result<()> {
        if word.parts.len() == 1 {
            // Single part - simpler compilation
            self.compile_word_part(&word.parts[0])?;
        } else {
            // Multiple parts - concatenate
            self.emit_simple(OpCode::ConstEmpty);
            for part in &word.parts {
                self.compile_word_part(part)?;
                self.emit_simple(OpCode::Concat);
            }
        }
        Ok(())
    }

    /// Compile a word part
    fn compile_word_part(&mut self, part: &WordPart) -> Result<()> {
        match part {
            WordPart::Literal(s) => {
                let idx = self.chunk.add_constant(s.clone());
                self.emit_op(OpCode::Const, idx as i32);
            }
            WordPart::Variable(name) => {
                let idx = self.chunk.add_constant(name.clone());
                self.emit_op(OpCode::GetVar, idx as i32);
            }
            WordPart::SpecialVar(c) => {
                let var_name = format!("{}", c);
                let idx = self.chunk.add_constant(var_name);
                self.emit_op(OpCode::GetVar, idx as i32);
            }
            WordPart::Glob(pattern) => {
                let idx = self.chunk.add_constant(pattern.clone());
                self.emit_op(OpCode::Const, idx as i32);
                self.emit_simple(OpCode::Glob);
            }
            _ => {
                // For complex parts, emit empty string (simplified)
                self.emit_simple(OpCode::ConstEmpty);
            }
        }
        Ok(())
    }

    /// Convert a word to a string (for simple cases)
    fn word_to_string(&self, word: &Word) -> String {
        let mut result = String::new();
        for part in &word.parts {
            match part {
                WordPart::Literal(s) => result.push_str(s),
                WordPart::Variable(name) => {
                    result.push('$');
                    result.push_str(name);
                }
                WordPart::Glob(pattern) => result.push_str(pattern),
                _ => {}
            }
        }
        result
    }

    // Helper methods for emitting instructions

    fn emit_simple(&mut self, op: OpCode) -> usize {
        self.chunk.emit_simple(op, self.current_line)
    }

    fn emit_op(&mut self, op: OpCode, operand: i32) -> usize {
        self.chunk.emit_op(op, operand, self.current_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_assignment() {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile_string("x=hello").unwrap();

        // Should have: Const(hello), SetVar(x), Halt
        assert!(chunk.code.len() >= 3);
        assert_eq!(chunk.code.last().unwrap().op, OpCode::Halt);
    }

    #[test]
    fn test_compile_echo() {
        let mut compiler = Compiler::new();
        let chunk = compiler.compile_string("echo hello").unwrap();

        // Should have Builtin instruction
        assert!(chunk.code.iter().any(|i| i.op == OpCode::Builtin));
    }

    #[test]
    fn test_compile_if() {
        let mut compiler = Compiler::new();
        let chunk = compiler
            .compile_string("if true; then echo yes; fi")
            .unwrap();

        // Should have conditional jumps
        assert!(
            chunk
                .code
                .iter()
                .any(|i| matches!(i.op, OpCode::JumpIfFalse | OpCode::Jump))
        );
    }

    #[test]
    fn test_compile_while() {
        let mut compiler = Compiler::new();
        let chunk = compiler
            .compile_string("while true; do echo loop; done")
            .unwrap();

        // Should have backward jump
        assert!(chunk.code.iter().any(|i| i.op == OpCode::Jump));
    }
}
