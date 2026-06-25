//! Statement execution for Franken Shell interpreter

use crate::ast::{
    Assignment, AssignmentOp, Command as AstCommand, CommandKind, FlatCommand, List, ListOp,
    Pipeline, Program, Redirect, SimpleCommand, Statement, Word,
};
use crate::builtins::Builtins;
use crate::error::{JshError, Result};
use crate::interpreter::{ExitStatus, Interpreter};
use crate::token::Span;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Child, Stdio};

impl Interpreter {
    /// Execute a program
    pub fn execute(&mut self, program: &Program) -> Result<ExitStatus> {
        let mut status = ExitStatus::success();

        for stmt in &program.statements {
            status = self.execute_statement(stmt)?;
        }

        Ok(status)
    }

    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: &Statement) -> Result<ExitStatus> {
        match stmt {
            Statement::Pipeline(pipeline) => self.execute_pipeline(pipeline),
            Statement::List(list) => self.execute_list(list),
            Statement::Assignment(assign) => self.execute_assignment(assign),
            Statement::If(if_stmt) => self.execute_if(if_stmt),
            Statement::For(for_loop) => self.execute_for(for_loop),
            Statement::While(while_loop) => self.execute_while(while_loop),
            Statement::Until(until_loop) => self.execute_until(until_loop),
            Statement::Case(case_stmt) => self.execute_case(case_stmt),
            Statement::Select(select_stmt) => self.execute_select(select_stmt),
            Statement::Function(func_def) => {
                self.functions.insert(func_def.name.clone(), func_def.clone());
                Ok(ExitStatus::success())
            }
            Statement::Match(match_expr) => self.execute_match(match_expr),
            Statement::Loop(loop_stmt) => self.execute_loop(loop_stmt),
            Statement::Let(let_binding) => self.execute_let(let_binding),
            Statement::Const(const_binding) => self.execute_const(const_binding),
            Statement::Try(try_stmt) => self.execute_try(try_stmt),
            // Fish-compatible
            Statement::BeginBlock(stmts) => self.execute_statements(stmts),
            Statement::FishSwitch(switch_stmt) => self.execute_fish_switch(switch_stmt),
            Statement::Break(_n) => Err(JshError::Break),
            Statement::Continue(_n) => Err(JshError::Continue),
            Statement::Return(value) => {
                let val = value.as_ref().map(|w| self.expand_word(w)).transpose()?;
                Err(JshError::Return(val))
            }
            Statement::Subshell(stmts) => self.execute_subshell(stmts),
            Statement::BraceGroup(stmts) => self.execute_statements(stmts),
            Statement::Empty => Ok(ExitStatus::success()),
        }
    }

    /// Execute a list (&&, ||)
    pub(crate) fn execute_list(&mut self, list: &List) -> Result<ExitStatus> {
        let mut status = self.execute_pipeline(&list.first)?;

        for (op, pipeline) in &list.rest {
            match op {
                ListOp::And if status.is_success() => {
                    status = self.execute_pipeline(pipeline)?;
                }
                ListOp::Or if !status.is_success() => {
                    status = self.execute_pipeline(pipeline)?;
                }
                _ => {}
            }
        }

        Ok(status)
    }

    /// Execute a pipeline
    pub(crate) fn execute_pipeline(&mut self, pipeline: &Pipeline) -> Result<ExitStatus> {
        if pipeline.commands.is_empty() {
            return Ok(ExitStatus::success());
        }

        let commands = &pipeline.commands;

        if commands.len() == 1 {
            // Single command, no pipes needed
            let status = self.execute_ast_command(&commands[0])?;
            self.last_status = if pipeline.negated {
                ExitStatus::failure(if status.is_success() { 1 } else { 0 })
            } else {
                status
            };
            return Ok(self.last_status);
        }

        // Multi-command pipeline
        let mut children: Vec<Child> = Vec::new();
        let mut prev_stdout: Option<Stdio> = None;

        for (i, cmd) in commands.iter().enumerate() {
            let is_last = i == commands.len() - 1;

            let stdin = prev_stdout.take().unwrap_or(Stdio::inherit());
            let stdout = if is_last {
                Stdio::inherit()
            } else {
                Stdio::piped()
            };

            if let Some(mut child) = self.spawn_ast_command(cmd, stdin, stdout)? {
                if let Some(stdout) = child.stdout.take() {
                    prev_stdout = Some(unsafe { Stdio::from_raw_fd(stdout.as_raw_fd()) });
                    std::mem::forget(stdout); // Don't close the fd
                }
                children.push(child);
            }
        }

        // Wait for all children
        let mut final_status = ExitStatus::success();
        for mut child in children {
            match child.wait() {
                Ok(status) => {
                    final_status = ExitStatus::failure(status.code().unwrap_or(1));
                }
                Err(e) => {
                    eprintln!("franken: error waiting for process: {}", e);
                    final_status = ExitStatus::failure(1);
                }
            }
        }

        self.last_status = if pipeline.negated {
            ExitStatus::failure(if final_status.is_success() { 1 } else { 0 })
        } else {
            final_status
        };

        Ok(self.last_status)
    }

    /// Execute a single AST command
    pub(crate) fn execute_ast_command(&mut self, cmd: &AstCommand) -> Result<ExitStatus> {
        match &cmd.kind {
            CommandKind::Simple(simple) => self.execute_simple_command(simple, &cmd.redirects),
            CommandKind::Flat(flat) => self.execute_flat_command(flat, &cmd.redirects),
            CommandKind::Compound(stmt) => self.execute_statement(stmt),
            CommandKind::FunctionCall { name, args } => self.call_function(name, args),
            CommandKind::Coproc { name: _, command } => self.execute_ast_command(command),
        }
    }

    /// Execute a flat command (optimized path for simple commands)
    ///
    /// Flat commands have no variable expansions or glob patterns,
    /// so we can skip the expansion phase entirely.
    #[inline]
    pub(crate) fn execute_flat_command(
        &mut self,
        cmd: &FlatCommand,
        redirects: &[Redirect],
    ) -> Result<ExitStatus> {
        if cmd.is_empty() {
            return Ok(ExitStatus::success());
        }

        let name = cmd.name();
        let args = cmd.args();

        // Check for functions first
        if self.functions.contains_key(name) {
            // Convert to Word args for function call
            let word_args: Vec<Word> = args
                .iter()
                .map(|a| Word::literal(a.clone(), Span::default()))
                .collect();
            return self.call_function(name, &word_args);
        }

        // Check for builtins (fast path - most commands are builtins or external)
        let builtins = Builtins::new();
        let args_vec: Vec<String> = args.to_vec();
        if let Some(status) = builtins.execute(name, &args_vec, self)? {
            self.last_status = status;
            return Ok(status);
        }

        // External command
        let status = self.execute_external_flat(cmd, redirects)?;
        self.last_status = status;
        Ok(status)
    }

    /// Execute a simple command
    pub(crate) fn execute_simple_command(
        &mut self,
        cmd: &SimpleCommand,
        redirects: &[Redirect],
    ) -> Result<ExitStatus> {
        // Handle assignments
        for assign in &cmd.assignments {
            self.execute_assignment(assign)?;
        }

        // If no command name, we're done (assignment only)
        if cmd.name.parts.is_empty() {
            return Ok(ExitStatus::success());
        }

        let name = self.expand_word(&cmd.name)?;
        let mut args: Vec<String> = Vec::new();

        for arg in &cmd.args {
            let expanded = self.expand_word_with_glob(arg)?;
            args.extend(expanded);
        }

        // Check for functions first
        if self.functions.contains_key(&name) {
            return self.call_function(&name, &cmd.args);
        }

        // Check for builtins
        let builtins = Builtins::new();
        if let Some(status) = builtins.execute(&name, &args, self)? {
            self.last_status = status;
            return Ok(status);
        }

        // Execute external command
        self.execute_external(&name, &args, redirects)
    }

    /// Execute an assignment
    pub(crate) fn execute_assignment(&mut self, assign: &Assignment) -> Result<ExitStatus> {
        let value = if let Some(word) = &assign.value {
            self.expand_word(word)?
        } else {
            String::new()
        };

        match assign.op {
            AssignmentOp::Assign => {
                if assign.export {
                    self.export_var(&assign.name, Some(&value));
                } else {
                    self.set_var(&assign.name, &value);
                }
            }
            AssignmentOp::Append => {
                let current = self.get_var(&assign.name).unwrap_or("").to_string();
                self.set_var(&assign.name, &format!("{}{}", current, value));
            }
            AssignmentOp::AssignOnly => {
                if self.get_var(&assign.name).is_none() {
                    self.set_var(&assign.name, &value);
                }
            }
        }

        if assign.readonly
            && let Some(val) = self.vars.remove(&assign.name) {
                self.consts.insert(assign.name.clone(), val);
            }

        Ok(ExitStatus::success())
    }

    /// Execute a list of statements
    pub fn execute_statements(&mut self, stmts: &[Statement]) -> Result<ExitStatus> {
        let mut status = ExitStatus::success();
        for stmt in stmts {
            status = self.execute_statement(stmt)?;
        }
        Ok(status)
    }

    /// Call a function
    pub fn call_function(&mut self, name: &str, args: &[Word]) -> Result<ExitStatus> {
        let func = self.functions.get(name).cloned();

        if let Some(func) = func {
            let old_params = std::mem::take(&mut self.positional_params);

            // Expand all arguments
            let mut expanded_args = smallvec::SmallVec::<[String; 16]>::new();
            for arg in args {
                expanded_args.push(self.expand_word(arg)?);
            }

            // If function has named parameters, bind them as local variables
            let mut old_vars: Vec<(String, Option<String>)> = Vec::new();
            if !func.params.is_empty() {
                for (i, param_name) in func.params.iter().enumerate() {
                    // Save old value if exists
                    old_vars.push((param_name.clone(), self.vars.get(param_name).cloned()));

                    // Bind the argument to the parameter name
                    let value = expanded_args.get(i).cloned().unwrap_or_default();
                    // SAFETY: Setting local variable for function scope
                    unsafe { std::env::set_var(param_name, &value) };
                    self.vars.insert(param_name.clone(), value);
                }
            }

            // Set positional parameters for $1, $2, etc. (traditional shell behavior)
            self.positional_params = expanded_args;

            let result = self.execute_statements(&func.body);

            // Restore named parameter variables
            for (var_name, old_val) in old_vars {
                if let Some(val) = old_val {
                    unsafe { std::env::set_var(&var_name, &val) };
                    self.vars.insert(var_name, val);
                } else {
                    unsafe { std::env::remove_var(&var_name) };
                    self.vars.remove(&var_name);
                }
            }

            // Restore positional params
            self.positional_params = old_params;

            match result {
                Ok(status) => Ok(status),
                Err(JshError::Return(val)) => {
                    if let Some(v) = val
                        && let Ok(code) = v.parse::<i32>() {
                            return Ok(ExitStatus::failure(code));
                        }
                    Ok(ExitStatus::success())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(JshError::CommandNotFound(name.to_string()))
        }
    }

    /// Execute a subshell
    pub(crate) fn execute_subshell(&mut self, stmts: &[Statement]) -> Result<ExitStatus> {
        // In a real implementation, we'd fork here
        // For now, just execute in a "pseudo-subshell"
        let old_vars = self.vars.clone();
        let result = self.execute_statements(stmts);
        self.vars = old_vars;
        result
    }
}

