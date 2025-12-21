//! Interpreter/executor for jsh shell

use crate::ast::{
    Assignment, AssignmentOp, BraceExpansion, CaseModifyMode, CaseStatement,
    CommandKind, ConstBinding, FishSwitchStatement, ForLoop, FunctionDef, IfStatement,
    LetBinding, List, ListOp, LoopStatement, MatchExpr, MatchPattern, Pipeline, Program,
    Redirect, RedirectKind, RedirectTarget, SelectStatement, SimpleCommand, Statement,
    TryStatement, UntilLoop, WhileLoop, Word, WordPart,
};
use crate::ast::Command as AstCommand;
use crate::error::{JshError, Result};
use crate::parser::Parser;
use glob::glob;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Child, Command as ProcessCommand, Stdio};

/// Exit status of a command
#[derive(Debug, Clone, Copy, Default)]
pub struct ExitStatus {
    pub code: i32,
}

impl ExitStatus {
    pub fn success() -> Self {
        Self { code: 0 }
    }

    pub fn failure(code: i32) -> Self {
        Self { code }
    }

    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}

/// Shell interpreter
pub struct Interpreter {
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Local variables
    pub vars: HashMap<String, String>,
    /// Constants (immutable)
    pub consts: HashMap<String, String>,
    /// Exported variables
    pub exports: std::collections::HashSet<String>,
    /// Functions
    pub functions: HashMap<String, FunctionDef>,
    /// Last exit status
    pub last_status: ExitStatus,
    /// Last background PID
    pub last_bg_pid: Option<u32>,
    /// Current shell PID
    pub shell_pid: u32,
    /// Positional parameters
    pub positional_params: Vec<String>,
    /// Current working directory
    pub cwd: PathBuf,
    /// Loop depth (for break/continue)
    loop_depth: usize,
    /// Shell options (POSIX/Ash compatible)
    pub options: ShellOptions,
}

/// Shell options (POSIX/Ash compatible)
#[derive(Debug, Clone, Default)]
pub struct ShellOptions {
    /// Exit immediately if command fails (-e, errexit)
    pub errexit: bool,
    /// Treat unset variables as error (-u, nounset)
    pub nounset: bool,
    /// Print commands before execution (-x, xtrace)
    pub xtrace: bool,
    /// Do not execute, just parse (-n, noexec)
    pub noexec: bool,
    /// Read commands from stdin (-s)
    pub stdin: bool,
    /// Export all variables (-a, allexport)
    pub allexport: bool,
    /// Do not overwrite files with > (-C, noclobber)
    pub noclobber: bool,
    /// Notify of job completion immediately (-b, notify)
    pub notify: bool,
    /// Exit on unset variable in substitution (-u, nounset)
    pub noglob: bool,
    /// Enable vi mode
    pub vi: bool,
    /// Enable emacs mode (default)
    pub emacs: bool,
    /// Ignore EOF (Ctrl-D)
    pub ignoreeof: bool,
    /// Enable POSIX mode
    pub posix: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = HashMap::new();

        // Import all existing environment variables
        for (key, value) in env::vars() {
            env.insert(key, value);
        }

        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let shell_pid = std::process::id();

        // Set shell-specific environment variables
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/bin/jsh".to_string());

        // SHELL - always set to jsh path (we are jsh!)
        // Note: Some scripts check $SHELL to determine which shell is running
        // We keep the parent's SHELL in _PARENT_SHELL for reference
        if let Some(parent_shell) = env.get("SHELL").cloned() {
            env.insert("_PARENT_SHELL".to_string(), parent_shell);
        }
        env.insert("SHELL".to_string(), exe_path.clone());
        unsafe { env::set_var("SHELL", &exe_path) };

        // JSH - set to 1 to indicate we're running in jsh
        env.insert("JSH".to_string(), "1".to_string());
        unsafe { env::set_var("JSH", "1") };

        // JSH_VERSION - shell version
        env.insert("JSH_VERSION".to_string(), crate::VERSION.to_string());
        unsafe { env::set_var("JSH_VERSION", crate::VERSION) };

        // SHLVL - shell nesting level
        let shlvl = env
            .get("SHLVL")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
            + 1;
        env.insert("SHLVL".to_string(), shlvl.to_string());
        unsafe { env::set_var("SHLVL", shlvl.to_string()) };

        // PWD - current working directory
        let pwd = cwd.to_string_lossy().to_string();
        env.insert("PWD".to_string(), pwd.clone());
        unsafe { env::set_var("PWD", &pwd) };

        // OLDPWD - previous directory (initialize to current)
        if !env.contains_key("OLDPWD") {
            env.insert("OLDPWD".to_string(), pwd.clone());
        }

        // USER - current user (should already be set, but ensure it)
        if !env.contains_key("USER") {
            if let Ok(user) = env::var("LOGNAME") {
                env.insert("USER".to_string(), user.clone());
                unsafe { env::set_var("USER", &user) };
            }
        }

        // HOME - user's home directory
        if !env.contains_key("HOME") {
            if let Some(home) = dirs::home_dir() {
                let home_str = home.to_string_lossy().to_string();
                env.insert("HOME".to_string(), home_str.clone());
                unsafe { env::set_var("HOME", &home_str) };
            }
        }

        // HOSTNAME
        if !env.contains_key("HOSTNAME") {
            if let Ok(hostname) = hostname::get() {
                let hostname_str = hostname.to_string_lossy().to_string();
                env.insert("HOSTNAME".to_string(), hostname_str.clone());
                unsafe { env::set_var("HOSTNAME", &hostname_str) };
            }
        }

        // TERM - ensure a default terminal type
        if !env.contains_key("TERM") {
            env.insert("TERM".to_string(), "xterm-256color".to_string());
            unsafe { env::set_var("TERM", "xterm-256color") };
        }

        // PATH - ensure a sensible default if not set
        if !env.contains_key("PATH") {
            let default_path = "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin";
            env.insert("PATH".to_string(), default_path.to_string());
            unsafe { env::set_var("PATH", default_path) };
        }

        // IFS - Internal Field Separator
        if !env.contains_key("IFS") {
            env.insert("IFS".to_string(), " \t\n".to_string());
        }

        // PS1/PS2 - prompt strings (will be overridden by theme)
        if !env.contains_key("PS1") {
            env.insert("PS1".to_string(), "\\u@\\h:\\w\\$ ".to_string());
        }
        if !env.contains_key("PS2") {
            env.insert("PS2".to_string(), "> ".to_string());
        }

        // HISTFILE - history file location
        if !env.contains_key("HISTFILE") {
            if let Some(home) = dirs::home_dir() {
                let histfile = home.join(".jsh_history").to_string_lossy().to_string();
                env.insert("HISTFILE".to_string(), histfile);
            }
        }

        // HISTSIZE - number of history entries
        if !env.contains_key("HISTSIZE") {
            env.insert("HISTSIZE".to_string(), "10000".to_string());
        }

        // $$ - current shell PID (stored as special var)
        // $0 - shell name
        // These are handled specially in variable expansion

        Self {
            env,
            vars: HashMap::new(),
            consts: HashMap::new(),
            exports: std::collections::HashSet::new(),
            functions: HashMap::new(),
            last_status: ExitStatus::success(),
            last_bg_pid: None,
            shell_pid,
            positional_params: vec![],
            cwd,
            loop_depth: 0,
            options: ShellOptions::default(),
        }
    }

    /// Get a variable value
    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.vars
            .get(name)
            .or_else(|| self.env.get(name))
            .or_else(|| self.consts.get(name))
            .map(|s| s.as_str())
    }

    /// Set a variable value
    pub fn set_var(&mut self, name: &str, value: &str) {
        if self.consts.contains_key(name) {
            eprintln!("jsh: {}: readonly variable", name);
            return;
        }

        if self.exports.contains(name) {
            self.env.insert(name.to_string(), value.to_string());
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, value) };
        } else {
            self.vars.insert(name.to_string(), value.to_string());
        }
    }

    /// Export a variable
    pub fn export_var(&mut self, name: &str, value: Option<&str>) {
        if let Some(val) = value {
            self.env.insert(name.to_string(), val.to_string());
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, val) };
        } else if let Some(val) = self.vars.remove(name) {
            self.env.insert(name.to_string(), val.clone());
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, &val) };
        }
        self.exports.insert(name.to_string());
    }

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
    fn execute_list(&mut self, list: &List) -> Result<ExitStatus> {
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
    fn execute_pipeline(&mut self, pipeline: &Pipeline) -> Result<ExitStatus> {
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
                    eprintln!("jsh: error waiting for process: {}", e);
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
    fn execute_ast_command(&mut self, cmd: &AstCommand) -> Result<ExitStatus> {
        match &cmd.kind {
            CommandKind::Simple(simple) => self.execute_simple_command(simple, &cmd.redirects),
            CommandKind::Compound(stmt) => self.execute_statement(stmt),
            CommandKind::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            CommandKind::Coproc { name: _, command } => {
                self.execute_ast_command(command)
            }
        }
    }

    /// Execute a simple command
    fn execute_simple_command(
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

        // Check for function
        if let Some(func) = self.functions.get(&name).cloned() {
            // Save old positional params
            let old_params = std::mem::take(&mut self.positional_params);

            // If function has named parameters, bind them as local variables
            let mut old_vars: Vec<(String, Option<String>)> = Vec::new();
            if !func.params.is_empty() {
                for (i, param_name) in func.params.iter().enumerate() {
                    // Save old value if exists
                    old_vars.push((param_name.clone(), self.vars.get(param_name).cloned()));

                    // Bind the argument to the parameter name
                    let value = args.get(i).cloned().unwrap_or_default();
                    // SAFETY: Setting local variable for function scope
                    unsafe { env::set_var(param_name, &value) };
                    self.vars.insert(param_name.clone(), value);
                }
            }

            // Set positional parameters for $1, $2, etc. (traditional shell behavior)
            self.positional_params = args;

            let result = self.execute_statements(&func.body);

            // Restore named parameter variables
            for (var_name, old_val) in old_vars {
                if let Some(val) = old_val {
                    unsafe { env::set_var(&var_name, &val) };
                    self.vars.insert(var_name, val);
                } else {
                    unsafe { env::remove_var(&var_name) };
                    self.vars.remove(&var_name);
                }
            }

            // Restore positional params
            self.positional_params = old_params;

            return match result {
                Ok(status) => Ok(status),
                Err(JshError::Return(val)) => {
                    if let Some(v) = val {
                        if let Ok(code) = v.parse::<i32>() {
                            return Ok(ExitStatus::failure(code));
                        }
                    }
                    Ok(ExitStatus::success())
                }
                Err(e) => Err(e),
            };
        }

        // Check for builtin - create a temporary builtins to avoid borrow issue
        let builtins = crate::builtins::Builtins::new();
        if let Some(status) = builtins.execute(&name, &args, self)? {
            return Ok(status);
        }

        // External command
        self.execute_external(&name, &args, redirects)
    }

    /// Spawn a command (for pipelines)
    fn spawn_ast_command(
        &mut self,
        cmd: &AstCommand,
        stdin: Stdio,
        stdout: Stdio,
    ) -> Result<Option<Child>> {
        match &cmd.kind {
            CommandKind::Simple(simple) => {
                if simple.name.parts.is_empty() {
                    return Ok(None);
                }

                let name = self.expand_word(&simple.name)?;
                let mut args: Vec<String> = Vec::new();

                for arg in &simple.args {
                    let expanded = self.expand_word_with_glob(arg)?;
                    args.extend(expanded);
                }

                // For builtins in a pipeline, we need to fork
                // For now, treat them as external
                let mut command = ProcessCommand::new(&name);
                command.args(&args);
                command.stdin(stdin);
                command.stdout(stdout);

                // Apply redirects
                for redirect in &cmd.redirects {
                    self.apply_redirect_to_command(&mut command, redirect)?;
                }

                match command.spawn() {
                    Ok(child) => Ok(Some(child)),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        eprintln!("jsh: {}: command not found", name);
                        Ok(None)
                    }
                    Err(e) => Err(JshError::Io(e)),
                }
            }
            _ => {
                // For compound commands in pipelines, we'd need to fork
                // For now, just execute directly
                self.execute_ast_command(cmd)?;
                Ok(None)
            }
        }
    }

    /// Execute an external command
    fn execute_external(
        &mut self,
        name: &str,
        args: &[String],
        redirects: &[Redirect],
    ) -> Result<ExitStatus> {
        let mut command = ProcessCommand::new(name);
        command.args(args);
        command.current_dir(&self.cwd);

        // Set environment
        for (key, value) in &self.env {
            command.env(key, value);
        }

        // Apply redirects
        for redirect in redirects {
            self.apply_redirect_to_command(&mut command, redirect)?;
        }

        match command.spawn() {
            Ok(mut child) => {
                match child.wait() {
                    Ok(status) => {
                        let code = status.code().unwrap_or(1);
                        Ok(ExitStatus::failure(code))
                    }
                    Err(e) => {
                        eprintln!("jsh: error waiting for {}: {}", name, e);
                        Ok(ExitStatus::failure(1))
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                eprintln!("jsh: {}: command not found", name);
                Ok(ExitStatus::failure(127))
            }
            Err(e) => {
                eprintln!("jsh: {}: {}", name, e);
                Ok(ExitStatus::failure(126))
            }
        }
    }

    /// Apply redirect to a Command
    fn apply_redirect_to_command(
        &self,
        command: &mut ProcessCommand,
        redirect: &Redirect,
    ) -> Result<()> {
        match &redirect.target {
            RedirectTarget::File(word) => {
                let path = self.expand_word(word)?;
                match redirect.kind {
                    RedirectKind::Input => {
                        let file = File::open(&path)?;
                        command.stdin(Stdio::from(file));
                    }
                    RedirectKind::Output => {
                        let file = File::create(&path)?;
                        if redirect.fd == Some(2) {
                            command.stderr(Stdio::from(file));
                        } else {
                            command.stdout(Stdio::from(file));
                        }
                    }
                    RedirectKind::Append => {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&path)?;
                        if redirect.fd == Some(2) {
                            command.stderr(Stdio::from(file));
                        } else {
                            command.stdout(Stdio::from(file));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute an assignment
    fn execute_assignment(&mut self, assign: &Assignment) -> Result<ExitStatus> {
        let value = if let Some(word) = &assign.value {
            self.expand_word(word)?
        } else {
            String::new()
        };

        match assign.op {
            AssignmentOp::Assign => {
                if assign.export {
                    self.export_var(&assign.name, Some(&value));
                } else if assign.readonly {
                    self.consts.insert(assign.name.clone(), value);
                } else {
                    self.set_var(&assign.name, &value);
                }
            }
            AssignmentOp::Append => {
                let current = self.get_var(&assign.name).unwrap_or("").to_string();
                let new_value = format!("{}{}", current, value);
                self.set_var(&assign.name, &new_value);
            }
            _ => {}
        }

        Ok(ExitStatus::success())
    }

    /// Execute if statement
    fn execute_if(&mut self, if_stmt: &IfStatement) -> Result<ExitStatus> {
        // Evaluate condition
        let cond_status = self.execute_statements(&if_stmt.condition)?;

        if cond_status.is_success() {
            return self.execute_statements(&if_stmt.then_branch);
        }

        // Check elif branches
        for (elif_cond, elif_body) in &if_stmt.elif_branches {
            let elif_status = self.execute_statements(elif_cond)?;
            if elif_status.is_success() {
                return self.execute_statements(elif_body);
            }
        }

        // Execute else branch
        if let Some(else_body) = &if_stmt.else_branch {
            return self.execute_statements(else_body);
        }

        Ok(ExitStatus::success())
    }

    /// Execute for loop
    fn execute_for(&mut self, for_loop: &ForLoop) -> Result<ExitStatus> {
        let items: Vec<String> = if let Some(words) = &for_loop.items {
            let mut items = Vec::new();
            for word in words {
                items.extend(self.expand_word_with_glob(word)?);
            }
            items
        } else {
            self.positional_params.clone()
        };

        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        for item in items {
            self.set_var(&for_loop.var, &item);

            match self.execute_statements(&for_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute while loop
    fn execute_while(&mut self, while_loop: &WhileLoop) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            let cond_status = self.execute_statements(&while_loop.condition)?;
            if !cond_status.is_success() {
                break;
            }

            match self.execute_statements(&while_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute until loop
    fn execute_until(&mut self, until_loop: &UntilLoop) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            let cond_status = self.execute_statements(&until_loop.condition)?;
            if cond_status.is_success() {
                break;
            }

            match self.execute_statements(&until_loop.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute case statement
    fn execute_case(&mut self, case_stmt: &CaseStatement) -> Result<ExitStatus> {
        let value = self.expand_word(&case_stmt.word)?;

        for arm in &case_stmt.arms {
            for pattern in &arm.patterns {
                let pattern_str = self.expand_word(pattern)?;

                // Check for glob match
                if self.glob_match(&pattern_str, &value) {
                    return self.execute_statements(&arm.body);
                }
            }
        }

        Ok(ExitStatus::success())
    }

    /// Execute Fish-style switch statement
    fn execute_fish_switch(&mut self, switch_stmt: &FishSwitchStatement) -> Result<ExitStatus> {
        let value = self.expand_word(&switch_stmt.value)?;

        for case in &switch_stmt.cases {
            for pattern in &case.patterns {
                let pattern_str = self.expand_word(pattern)?;

                // Fish uses glob matching for patterns
                // '*' matches anything, '?' matches single char
                if self.glob_match(&pattern_str, &value) || pattern_str == "*" {
                    return self.execute_statements(&case.body);
                }
            }
        }

        Ok(ExitStatus::success())
    }

    /// Execute select statement
    fn execute_select(&mut self, select_stmt: &SelectStatement) -> Result<ExitStatus> {
        let items: Vec<String> = if let Some(words) = &select_stmt.items {
            let mut items = Vec::new();
            for word in words {
                items.extend(self.expand_word_with_glob(word)?);
            }
            items
        } else {
            self.positional_params.clone()
        };

        if items.is_empty() {
            return Ok(ExitStatus::success());
        }

        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            // Print menu
            for (i, item) in items.iter().enumerate() {
                println!("{}) {}", i + 1, item);
            }
            print!("#? ");
            io::stdout().flush()?;

            // Read selection
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            if let Ok(n) = input.parse::<usize>() {
                if n > 0 && n <= items.len() {
                    self.set_var(&select_stmt.var, &items[n - 1]);
                    self.set_var("REPLY", input);

                    match self.execute_statements(&select_stmt.body) {
                        Ok(s) => status = s,
                        Err(JshError::Break) => break,
                        Err(JshError::Continue) => continue,
                        Err(e) => {
                            self.loop_depth -= 1;
                            return Err(e);
                        }
                    }
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    // ========================================================================
    // jsh-specific
    // ========================================================================

    /// Execute jsh match expression
    fn execute_match(&mut self, match_expr: &MatchExpr) -> Result<ExitStatus> {
        let value = self.expand_word(&match_expr.value)?;

        for arm in &match_expr.arms {
            if self.match_pattern(&arm.pattern, &value)? {
                // Check guard if present
                if let Some(guard) = &arm.guard {
                    let guard_status = self.execute_statements(guard)?;
                    if !guard_status.is_success() {
                        continue;
                    }
                }

                return self.execute_statements(&arm.body);
            }
        }

        Ok(ExitStatus::success())
    }

    /// Check if a value matches a pattern
    fn match_pattern(&self, pattern: &MatchPattern, value: &str) -> Result<bool> {
        match pattern {
            MatchPattern::Wildcard => Ok(true),
            MatchPattern::Literal(word) => {
                let pattern_str = self.expand_word(word)?;
                Ok(pattern_str == value)
            }
            MatchPattern::Glob(glob_pattern) => {
                Ok(self.glob_match(glob_pattern, value))
            }
            MatchPattern::Regex(regex_pattern) => {
                let re = regex::Regex::new(regex_pattern)?;
                Ok(re.is_match(value))
            }
            MatchPattern::Range { start, end, inclusive } => {
                if let Ok(v) = value.parse::<i64>() {
                    if *inclusive {
                        Ok(v >= *start && v <= *end)
                    } else {
                        Ok(v >= *start && v < *end)
                    }
                } else {
                    Ok(false)
                }
            }
            MatchPattern::Or(patterns) => {
                for p in patterns {
                    if self.match_pattern(p, value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            MatchPattern::Bind { name: _, pattern } => {
                self.match_pattern(pattern, value)
            }
        }
    }

    /// Execute jsh infinite loop
    fn execute_loop(&mut self, loop_stmt: &LoopStatement) -> Result<ExitStatus> {
        self.loop_depth += 1;
        let mut status = ExitStatus::success();

        loop {
            match self.execute_statements(&loop_stmt.body) {
                Ok(s) => status = s,
                Err(JshError::Break) => break,
                Err(JshError::Continue) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.loop_depth -= 1;
        Ok(status)
    }

    /// Execute jsh let binding
    fn execute_let(&mut self, let_binding: &LetBinding) -> Result<ExitStatus> {
        let value = self.expand_word(&let_binding.value)?;
        self.set_var(&let_binding.name, &value);
        Ok(ExitStatus::success())
    }

    /// Execute jsh const binding
    fn execute_const(&mut self, const_binding: &ConstBinding) -> Result<ExitStatus> {
        let value = self.expand_word(&const_binding.value)?;
        self.consts.insert(const_binding.name.clone(), value);
        Ok(ExitStatus::success())
    }

    /// Execute jsh try-catch-finally
    fn execute_try(&mut self, try_stmt: &TryStatement) -> Result<ExitStatus> {
        let result = self.execute_statements(&try_stmt.try_block);

        let status = match result {
            Ok(s) => s,
            Err(e) => {
                // Execute catch block
                if let Some(catch_block) = &try_stmt.catch_block {
                    if let Some(var) = &try_stmt.catch_var {
                        self.set_var(var, &e.to_string());
                    }
                    self.execute_statements(catch_block)?
                } else {
                    ExitStatus::failure(1)
                }
            }
        };

        // Execute finally block
        if let Some(finally_block) = &try_stmt.finally_block {
            self.execute_statements(finally_block)?;
        }

        Ok(status)
    }

    /// Execute a subshell
    fn execute_subshell(&mut self, stmts: &[Statement]) -> Result<ExitStatus> {
        // In a real implementation, we'd fork here
        // For now, just execute in a "pseudo-subshell"
        let old_vars = self.vars.clone();
        let result = self.execute_statements(stmts);
        self.vars = old_vars;
        result
    }

    /// Execute a list of statements
    fn execute_statements(&mut self, stmts: &[Statement]) -> Result<ExitStatus> {
        let mut status = ExitStatus::success();
        for stmt in stmts {
            status = self.execute_statement(stmt)?;
        }
        Ok(status)
    }

    /// Call a function
    fn call_function(&mut self, name: &str, args: &[Word]) -> Result<ExitStatus> {
        let func = self.functions.get(name).cloned();

        if let Some(func) = func {
            let old_params = std::mem::take(&mut self.positional_params);

            // Expand all arguments
            let mut expanded_args = Vec::new();
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
                    unsafe { env::set_var(param_name, &value) };
                    self.vars.insert(param_name.clone(), value);
                }
            }

            // Set positional parameters for $1, $2, etc. (traditional shell behavior)
            self.positional_params = expanded_args;

            let result = self.execute_statements(&func.body);

            // Restore named parameter variables
            for (var_name, old_val) in old_vars {
                if let Some(val) = old_val {
                    unsafe { env::set_var(&var_name, &val) };
                    self.vars.insert(var_name, val);
                } else {
                    unsafe { env::remove_var(&var_name) };
                    self.vars.remove(&var_name);
                }
            }

            // Restore positional params
            self.positional_params = old_params;

            match result {
                Ok(status) => Ok(status),
                Err(JshError::Return(val)) => {
                    if let Some(v) = val {
                        if let Ok(code) = v.parse::<i32>() {
                            return Ok(ExitStatus::failure(code));
                        }
                    }
                    Ok(ExitStatus::success())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(JshError::CommandNotFound(name.to_string()))
        }
    }

    /// Expand a word to a string
    pub fn expand_word(&self, word: &Word) -> Result<String> {
        let mut result = String::new();

        for part in &word.parts {
            match part {
                WordPart::Literal(s) => {
                    // Expand any embedded variables in the literal (from double-quoted strings)
                    result.push_str(&self.expand_string_variables(s));
                }
                WordPart::Variable(name) => {
                    if let Some(value) = self.get_var(name) {
                        result.push_str(value);
                    }
                }
                WordPart::BraceExpansion(expansion) => {
                    result.push_str(&self.expand_brace_expansion(expansion)?);
                }
                WordPart::CommandSub(stmts) => {
                    result.push_str(&self.expand_command_sub(stmts)?);
                }
                WordPart::BacktickSub(cmd) => {
                    let mut parser = Parser::from_str(cmd)?;
                    let program = parser.parse_program()?;
                    result.push_str(&self.expand_command_sub(&program.statements)?);
                }
                WordPart::SpecialVar(c) => {
                    result.push_str(&self.expand_special_var(*c));
                }
                WordPart::Glob(pattern) => {
                    // Don't expand globs here, just return the pattern
                    result.push_str(pattern);
                }
                _ => {}
            }
        }

        Ok(result)
    }

    /// Expand a word with glob expansion
    fn expand_word_with_glob(&self, word: &Word) -> Result<Vec<String>> {
        let expanded = self.expand_word(word)?;

        // Check if it contains glob characters
        if expanded.contains('*') || expanded.contains('?') || expanded.contains('[') {
            match glob(&expanded) {
                Ok(paths) => {
                    let matches: Vec<String> = paths
                        .filter_map(|r| r.ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();

                    if matches.is_empty() {
                        Ok(vec![expanded])
                    } else {
                        Ok(matches)
                    }
                }
                Err(_) => Ok(vec![expanded]),
            }
        } else {
            Ok(vec![expanded])
        }
    }

    /// Expand brace expansion
    fn expand_brace_expansion(&self, expansion: &BraceExpansion) -> Result<String> {
        match expansion {
            BraceExpansion::Simple(var) => {
                Ok(self.get_var(var).unwrap_or("").to_string())
            }
            BraceExpansion::Default { var, default, null_or_unset } => {
                let value = self.get_var(var);
                if *null_or_unset {
                    if value.is_none_or(|v| v.is_empty()) {
                        return self.expand_word(default);
                    }
                } else if value.is_none() {
                    return self.expand_word(default);
                }
                Ok(value.unwrap_or("").to_string())
            }
            BraceExpansion::Length(var) => {
                let value = self.get_var(var).unwrap_or("");
                Ok(value.len().to_string())
            }
            BraceExpansion::RemovePrefix { var, pattern, greedy: _ } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                // Simple implementation - just remove prefix if it matches
                if let Some(stripped) = value.strip_prefix(pattern) {
                    Ok(stripped.to_string())
                } else {
                    Ok(value)
                }
            }
            BraceExpansion::RemoveSuffix { var, pattern, greedy: _ } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                if let Some(stripped) = value.strip_suffix(pattern) {
                    Ok(stripped.to_string())
                } else {
                    Ok(value)
                }
            }
            BraceExpansion::CaseModify { var, mode } => {
                let value = self.get_var(var).unwrap_or("").to_string();
                Ok(match mode {
                    CaseModifyMode::UpperFirst => {
                        let mut chars = value.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().chain(chars).collect(),
                        }
                    }
                    CaseModifyMode::UpperAll => value.to_uppercase(),
                    CaseModifyMode::LowerFirst => {
                        let mut chars = value.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_lowercase().chain(chars).collect(),
                        }
                    }
                    CaseModifyMode::LowerAll => value.to_lowercase(),
                })
            }
            _ => Ok(String::new()),
        }
    }

    /// Expand command substitution
    fn expand_command_sub(&self, _stmts: &[Statement]) -> Result<String> {
        // This is tricky because we need a mutable self
        // In a real implementation, we'd fork and capture output
        // For now, just return empty string
        Ok(String::new())
    }

    /// Expand variables embedded in a string (from double-quoted strings)
    fn expand_string_variables(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        // ${var} expansion
                        chars.next(); // consume {
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' {
                                chars.next();
                                break;
                            }
                            var_name.push(c);
                            chars.next();
                        }
                        if let Some(value) = self.get_var(&var_name) {
                            result.push_str(value);
                        }
                    } else if next.is_alphabetic() || next == '_' {
                        // $var expansion
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(value) = self.get_var(&var_name) {
                            result.push_str(value);
                        }
                    } else if next == '?' || next == '!' || next == '$' || next == '#' || next == '@' || next == '*' || next.is_ascii_digit() {
                        // Special variable
                        chars.next();
                        result.push_str(&self.expand_special_var(next));
                    } else {
                        result.push('$');
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Expand special variable
    fn expand_special_var(&self, c: char) -> String {
        match c {
            '?' => self.last_status.code.to_string(),
            '!' => self.last_bg_pid.map(|p| p.to_string()).unwrap_or_default(),
            '$' => self.shell_pid.to_string(),
            '#' => self.positional_params.len().to_string(),
            '@' | '*' => self.positional_params.join(" "),
            '-' => String::new(), // Shell options - not implemented
            '_' => String::new(), // Last argument - not implemented
            '0' => "jsh".to_string(),
            c @ '1'..='9' => {
                let idx = (c as usize) - ('1' as usize);
                self.positional_params
                    .get(idx)
                    .cloned()
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    /// Simple glob matching
    fn glob_match(&self, pattern: &str, value: &str) -> bool {
        // Convert shell glob to regex
        let mut regex_pattern = String::from("^");
        for c in pattern.chars() {
            match c {
                '*' => regex_pattern.push_str(".*"),
                '?' => regex_pattern.push('.'),
                '.' | '+' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '$' | '|' | '\\' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(c);
                }
                _ => regex_pattern.push(c),
            }
        }
        regex_pattern.push('$');

        regex::Regex::new(&regex_pattern)
            .map(|re| re.is_match(value))
            .unwrap_or(false)
    }

    /// Execute a string as shell code
    pub fn execute_string(&mut self, input: &str) -> Result<ExitStatus> {
        let mut parser = Parser::from_str(input)?;
        let program = parser.parse_program()?;
        self.execute(&program)
    }

    /// Run a script file
    pub fn run_script(&mut self, path: &str) -> Result<ExitStatus> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.execute_string(&contents)
    }
}
