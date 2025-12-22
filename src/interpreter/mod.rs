//! Interpreter/executor for jsh shell

pub mod arithmetic;
pub mod compound;
pub mod execution;
pub mod expansion;
pub mod external;

use crate::ast::FunctionDef;
use crate::error::Result;
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;


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
    /// Disable pathname expansion (-f, noglob)
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

/// Shell interpreter
pub struct Interpreter {
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Local variables
    pub vars: HashMap<String, String>,
    /// Constants (immutable)
    pub consts: HashMap<String, String>,
    /// Exported variables
    pub exports: HashSet<String>,
    /// Functions
    pub functions: HashMap<String, FunctionDef>,
    /// Last exit status
    pub last_status: ExitStatus,
    /// Last background PID
    pub last_bg_pid: Option<u32>,
    /// Current shell PID
    pub shell_pid: u32,
    /// Parent shell PID (POSIX PPID)
    pub parent_pid: u32,
    /// Positional parameters
    pub positional_params: Vec<String>,
    /// Current working directory
    pub cwd: PathBuf,
    /// Loop depth (for break/continue)
    pub(crate) loop_depth: usize,
    /// Shell options (POSIX/Ash compatible)
    pub options: ShellOptions,
    /// Current line number (POSIX LINENO)
    pub lineno: usize,
    /// Last argument of previous command (POSIX $_)
    pub last_arg: String,
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

        // PPID - parent process ID (POSIX required)
        let parent_pid = std::os::unix::process::parent_id();
        env.insert("PPID".to_string(), parent_pid.to_string());
        unsafe { env::set_var("PPID", parent_pid.to_string()) };

        // LINENO - will be updated during execution
        env.insert("LINENO".to_string(), "1".to_string());

        // RANDOM - pseudo-random number (will be updated on each access)
        // Seed from current time
        env.insert("RANDOM".to_string(), "0".to_string());

        // SECONDS - seconds since shell started
        env.insert("SECONDS".to_string(), "0".to_string());

        Self {
            env,
            vars: HashMap::new(),
            consts: HashMap::new(),
            exports: HashSet::new(),
            functions: HashMap::new(),
            last_status: ExitStatus::success(),
            last_bg_pid: None,
            shell_pid,
            parent_pid,
            positional_params: vec![],
            cwd,
            loop_depth: 0,
            options: ShellOptions::default(),
            lineno: 1,
            last_arg: String::new(),
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
    
    /// Update dynamic POSIX variables (call before variable expansion)
    pub fn update_dynamic_vars(&mut self) {
        // Update LINENO
        self.env.insert("LINENO".to_string(), self.lineno.to_string());
        
        // Update _ (last argument)
        if !self.last_arg.is_empty() {
            self.env.insert("_".to_string(), self.last_arg.clone());
        }
        
        // Update RANDOM (pseudo-random 0-32767)
        let random = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u32 ^ self.shell_pid) % 32768;
        self.env.insert("RANDOM".to_string(), random.to_string());
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

