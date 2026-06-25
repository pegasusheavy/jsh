//! External command execution for Franken Shell interpreter

use crate::ast::{
    Command as AstCommand, CommandKind, FlatCommand, Redirect, RedirectKind, RedirectTarget,
};
use crate::error::{JshError, Result};
use crate::interpreter::{ExitStatus, Interpreter};
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Child, Command as ProcessCommand, Stdio};

impl Interpreter {
    /// Execute an external command (flat version - no expansion needed)
    ///
    /// This is an optimized path for commands with no variable expansions
    /// or glob patterns, which is the common case for most commands.
    #[inline]
    pub fn execute_external_flat(
        &mut self,
        flat: &FlatCommand,
        redirects: &[Redirect],
    ) -> Result<ExitStatus> {
        let name = flat.name();
        let args = flat.args();

        let mut cmd = ProcessCommand::new(name);
        cmd.args(args);
        cmd.current_dir(&self.cwd);

        // Set up environment
        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        // Set up redirections
        self.setup_redirects(&mut cmd, redirects)?;

        // Execute
        match cmd.status() {
            Ok(status) => {
                let code = status.code().unwrap_or(128);
                self.last_arg = args.last().cloned().unwrap_or_default();
                Ok(ExitStatus::failure(code))
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("fsh: {}: command not found", name);
                    Ok(ExitStatus::failure(127))
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!("fsh: {}: permission denied", name);
                    Ok(ExitStatus::failure(126))
                } else {
                    eprintln!("fsh: {}: {}", name, e);
                    Ok(ExitStatus::failure(1))
                }
            }
        }
    }

    /// Execute an external command
    pub fn execute_external(
        &mut self,
        name: &str,
        args: &[String],
        redirects: &[Redirect],
    ) -> Result<ExitStatus> {
        let mut cmd = ProcessCommand::new(name);
        cmd.args(args);
        cmd.current_dir(&self.cwd);

        // Set up environment
        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        // Handle redirections
        self.setup_redirects(&mut cmd, redirects)?;

        match cmd.spawn() {
            Ok(mut child) => {
                let status = child.wait()?;
                self.last_status = ExitStatus::failure(status.code().unwrap_or(1));
                Ok(self.last_status)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("franken: {}: command not found", name);
                    Err(JshError::CommandNotFound(name.to_string()))
                } else {
                    eprintln!("franken: {}: {}", name, e);
                    Err(JshError::Io(e))
                }
            }
        }
    }

    /// Spawn an AST command for pipeline
    pub(crate) fn spawn_ast_command(
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
                    args.extend(self.expand_word_with_glob(arg)?);
                }

                // Check for functions (can't be piped as a child process)
                if self.functions.contains_key(&name) {
                    // Execute function inline (not ideal for pipes)
                    self.call_function(&name, &simple.args)?;
                    return Ok(None);
                }

                // Check for builtins (some can't be forked)
                // For now, just spawn external command
                let mut process_cmd = ProcessCommand::new(&name);
                process_cmd.args(&args);
                process_cmd.current_dir(&self.cwd);
                process_cmd.stdin(stdin);
                process_cmd.stdout(stdout);

                // Set up environment
                for (key, value) in &self.env {
                    process_cmd.env(key, value);
                }

                // Handle redirections
                self.setup_redirects(&mut process_cmd, &cmd.redirects)?;

                match process_cmd.spawn() {
                    Ok(child) => Ok(Some(child)),
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            eprintln!("franken: {}: command not found", name);
                        } else {
                            eprintln!("franken: {}: {}", name, e);
                        }
                        Ok(None)
                    }
                }
            }
            CommandKind::Flat(flat) => {
                // Fast path for flat commands (no expansions needed)
                if flat.is_empty() {
                    return Ok(None);
                }

                let name = flat.name();
                let args = flat.args();

                // Check for functions
                if self.functions.contains_key(name) {
                    let word_args: Vec<crate::ast::Word> = args
                        .iter()
                        .map(|a| crate::ast::Word::literal(a.clone(), crate::token::Span::default()))
                        .collect();
                    self.call_function(name, &word_args)?;
                    return Ok(None);
                }

                let mut process_cmd = ProcessCommand::new(name);
                process_cmd.args(args);
                process_cmd.current_dir(&self.cwd);
                process_cmd.stdin(stdin);
                process_cmd.stdout(stdout);

                for (key, value) in &self.env {
                    process_cmd.env(key, value);
                }

                self.setup_redirects(&mut process_cmd, &cmd.redirects)?;

                match process_cmd.spawn() {
                    Ok(child) => Ok(Some(child)),
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            eprintln!("fsh: {}: command not found", name);
                        } else {
                            eprintln!("fsh: {}: {}", name, e);
                        }
                        Ok(None)
                    }
                }
            }
            CommandKind::Compound(stmt) => {
                // Execute compound command inline
                self.execute_statement(stmt)?;
                Ok(None)
            }
            CommandKind::FunctionCall { name, args } => {
                self.call_function(name, args)?;
                Ok(None)
            }
            CommandKind::Coproc { name: _, command } => self.spawn_ast_command(command, stdin, stdout),
        }
    }

    /// Set up redirections for a command
    fn setup_redirects(
        &self,
        cmd: &mut ProcessCommand,
        redirects: &[Redirect],
    ) -> Result<()> {
        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::Output | RedirectKind::Clobber => {
                    if let RedirectTarget::File(path) = &redirect.target {
                        let path = self.expand_word(path)?;
                        let file = File::create(&path)?;
                        let fd = redirect.fd.unwrap_or(1);
                        match fd {
                            1 => cmd.stdout(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                            2 => cmd.stderr(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                            _ => cmd.stdout(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                        };
                        std::mem::forget(file);
                    }
                }
                RedirectKind::Append => {
                    if let RedirectTarget::File(path) = &redirect.target {
                        let path = self.expand_word(path)?;
                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&path)?;
                        let fd = redirect.fd.unwrap_or(1);
                        match fd {
                            1 => cmd.stdout(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                            2 => cmd.stderr(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                            _ => cmd.stdout(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) }),
                        };
                        std::mem::forget(file);
                    }
                }
                RedirectKind::Input => {
                    if let RedirectTarget::File(path) = &redirect.target {
                        let path = self.expand_word(path)?;
                        let file = File::open(&path)?;
                        cmd.stdin(unsafe { Stdio::from_raw_fd(file.as_raw_fd()) });
                        std::mem::forget(file);
                    }
                }
                RedirectKind::InputOutput => {
                    if let RedirectTarget::File(path) = &redirect.target {
                        let path = self.expand_word(path)?;
                        // `<>` opens for read+write and must NOT truncate the
                        // existing file contents; state that explicitly.
                        let file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .truncate(false)
                            .open(&path)?;
                        let fd = file.as_raw_fd();
                        cmd.stdin(unsafe { Stdio::from_raw_fd(fd) });
                        cmd.stdout(unsafe { Stdio::from_raw_fd(fd) });
                        std::mem::forget(file);
                    }
                }
                RedirectKind::DupOutput => {
                    if let RedirectTarget::Fd(target_fd) = &redirect.target
                        && *target_fd == 1 {
                            // 2>&1 - stderr to stdout
                            cmd.stderr(Stdio::inherit());
                        }
                }
                RedirectKind::DupInput => {
                    // Handle input duplication
                }
                RedirectKind::HereDoc | RedirectKind::HereDocStrip => {
                    // Handle heredoc
                    if let RedirectTarget::HereDoc { content, .. } = &redirect.target {
                        // Create a pipe for heredoc content
                        if let Ok((read_fd, write_fd)) = nix::unistd::pipe() {
                            let read_raw = read_fd.as_raw_fd();
                            let write_raw = write_fd.as_raw_fd();
                            // Prevent fd from being closed when OwnedFd drops
                            std::mem::forget(read_fd);
                            std::mem::forget(write_fd);

                            // Write content to pipe
                            let content = content.clone();
                            std::thread::spawn(move || {
                                use std::io::Write;
                                let mut writer = unsafe { File::from_raw_fd(write_raw) };
                                let _ = writer.write_all(content.as_bytes());
                            });

                            cmd.stdin(unsafe { Stdio::from_raw_fd(read_raw) });
                        }
                    }
                }
                RedirectKind::HereString => {
                    // Handle herestring
                    if let RedirectTarget::HereString(word) = &redirect.target {
                        let content = self.expand_word(word)?;
                        if let Ok((read_fd, write_fd)) = nix::unistd::pipe() {
                            let read_raw = read_fd.as_raw_fd();
                            let write_raw = write_fd.as_raw_fd();
                            // Prevent fd from being closed when OwnedFd drops
                            std::mem::forget(read_fd);
                            std::mem::forget(write_fd);

                            let content = content + "\n";
                            std::thread::spawn(move || {
                                use std::io::Write;
                                let mut writer = unsafe { File::from_raw_fd(write_raw) };
                                let _ = writer.write_all(content.as_bytes());
                            });

                            cmd.stdin(unsafe { Stdio::from_raw_fd(read_raw) });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

