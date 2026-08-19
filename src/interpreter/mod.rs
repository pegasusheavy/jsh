//! Interpreter/executor for Franken Shell
//!
//! Uses FxHashMap for faster variable lookups and includes caching
//! for frequently accessed variables. Variable scoping is handled
//! by a scope stack for proper function-local variables.

pub mod arithmetic;
pub mod async_io;
pub mod compound;
pub mod execution;
pub mod expansion;
pub mod external;
pub mod parallel;
pub mod scope;

pub use scope::{Scope, ScopeGuard, ScopeStack};

use crate::ast::FunctionDef;
use crate::error::Result;
use crate::intern;
use crate::parser::Parser;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Type alias for small argument vectors (most commands have < 8 args)
pub type SmallArgs = SmallVec<[String; 8]>;

/// Exit status of a command
#[derive(Debug, Clone, Copy, Default)]
pub struct ExitStatus {
    pub code: i32,
}

impl ExitStatus {
    #[inline]
    pub fn success() -> Self {
        Self { code: 0 }
    }

    #[inline]
    pub fn failure(code: i32) -> Self {
        Self { code }
    }

    #[inline]
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

/// Cached values for frequently accessed variables
#[derive(Debug, Clone, Default)]
pub struct VarCache {
    /// Cached PATH value
    pub path: Option<String>,
    /// Cached HOME value
    pub home: Option<String>,
    /// Cached USER value
    pub user: Option<String>,
    /// Cached PWD value
    pub pwd: Option<String>,
    /// Cached IFS value
    pub ifs: Option<String>,
    /// Cache validity flag
    valid: bool,
}

impl VarCache {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Shell interpreter with optimized data structures
pub struct Interpreter {
    /// Environment variables (FxHashMap for faster hashing)
    pub env: FxHashMap<String, String>,
    /// Local variables (FxHashMap for faster hashing) - legacy, prefer scope_stack
    pub vars: FxHashMap<String, String>,
    /// Variable scope stack for proper function-local scoping
    pub scope_stack: ScopeStack,
    /// Constants (immutable)
    pub consts: FxHashMap<String, String>,
    /// Exported variables (FxHashSet for faster lookups)
    pub exports: FxHashSet<String>,
    /// Functions
    pub functions: FxHashMap<String, FunctionDef>,
    /// Last exit status
    pub last_status: ExitStatus,
    /// Last background PID
    pub last_bg_pid: Option<u32>,
    /// Current shell PID
    pub shell_pid: u32,
    /// Parent shell PID (POSIX PPID)
    pub parent_pid: u32,
    /// Positional parameters (SmallVec for efficiency)
    pub positional_params: SmallVec<[String; 16]>,
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
    /// Variable cache for frequently accessed values
    var_cache: VarCache,
    /// Cache for constant word expansions (words without variables)
    word_cache: FxHashMap<u64, String>,
    /// JIT runtime for hot loop compilation
    pub jit: crate::jit::JitRuntime,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        // Pre-warm the string interner with common strings
        intern::prewarm_interner();

        // Pre-allocate environment map with reasonable capacity
        let mut env = FxHashMap::with_capacity_and_hasher(64, Default::default());

        // Import all existing environment variables
        for (key, value) in env::vars() {
            env.insert(key, value);
        }

        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let shell_pid = std::process::id();

        // Set shell-specific environment variables
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/bin/fsh".to_string());

        // SHELL - always set to franken path (we are franken!)
        if let Some(parent_shell) = env.get("SHELL").cloned() {
            env.insert("_PARENT_SHELL".to_string(), parent_shell);
        }
        env.insert("SHELL".to_string(), exe_path.clone());
        unsafe { env::set_var("SHELL", &exe_path) };

        // FRANKEN - set to 1 to indicate we're running in franken
        env.insert("JSH".to_string(), "1".to_string());
        unsafe { env::set_var("JSH", "1") };

        // FSH_VERSION - shell version
        env.insert("FSH_VERSION".to_string(), crate::VERSION.to_string());
        unsafe { env::set_var("FSH_VERSION", crate::VERSION) };

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
        let user_val = if !env.contains_key("USER") {
            if let Ok(user) = env::var("LOGNAME") {
                env.insert("USER".to_string(), user.clone());
                unsafe { env::set_var("USER", &user) };
                Some(user)
            } else {
                None
            }
        } else {
            env.get("USER").cloned()
        };

        // HOME - user's home directory
        let home_val = if !env.contains_key("HOME") {
            if let Some(home) = dirs::home_dir() {
                let home_str = home.to_string_lossy().to_string();
                env.insert("HOME".to_string(), home_str.clone());
                unsafe { env::set_var("HOME", &home_str) };
                Some(home_str)
            } else {
                None
            }
        } else {
            env.get("HOME").cloned()
        };

        // HOSTNAME
        if !env.contains_key("HOSTNAME")
            && let Ok(hostname) = hostname::get()
        {
            let hostname_str = hostname.to_string_lossy().to_string();
            env.insert("HOSTNAME".to_string(), hostname_str.clone());
            unsafe { env::set_var("HOSTNAME", &hostname_str) };
        }

        // TERM - ensure a default terminal type
        if !env.contains_key("TERM") {
            env.insert("TERM".to_string(), "xterm-256color".to_string());
            unsafe { env::set_var("TERM", "xterm-256color") };
        }

        // PATH - ensure a sensible default if not set
        let path_val = if !env.contains_key("PATH") {
            let default_path = "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin";
            env.insert("PATH".to_string(), default_path.to_string());
            unsafe { env::set_var("PATH", default_path) };
            Some(default_path.to_string())
        } else {
            env.get("PATH").cloned()
        };

        // IFS - Internal Field Separator
        let ifs_val = if !env.contains_key("IFS") {
            let ifs = " \t\n".to_string();
            env.insert("IFS".to_string(), ifs.clone());
            Some(ifs)
        } else {
            env.get("IFS").cloned()
        };

        // PS1/PS2 - prompt strings (will be overridden by theme)
        if !env.contains_key("PS1") {
            env.insert("PS1".to_string(), "\\u@\\h:\\w\\$ ".to_string());
        }
        if !env.contains_key("PS2") {
            env.insert("PS2".to_string(), "> ".to_string());
        }

        // HISTFILE - history file location
        if !env.contains_key("HISTFILE")
            && let Some(home) = dirs::home_dir()
        {
            let histfile = home.join(".fsh_history").to_string_lossy().to_string();
            env.insert("HISTFILE".to_string(), histfile);
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

        // Initialize variable cache with frequently accessed values
        let var_cache = VarCache {
            path: path_val,
            home: home_val,
            user: user_val,
            pwd: Some(pwd),
            ifs: ifs_val,
            valid: true,
        };

        Self {
            env,
            vars: FxHashMap::default(),
            scope_stack: ScopeStack::new(),
            consts: FxHashMap::default(),
            exports: FxHashSet::default(),
            functions: FxHashMap::default(),
            last_status: ExitStatus::success(),
            last_bg_pid: None,
            shell_pid,
            parent_pid,
            positional_params: SmallVec::new(),
            cwd,
            loop_depth: 0,
            options: ShellOptions::default(),
            lineno: 1,
            last_arg: String::new(),
            var_cache,
            // Word expansion cache: 256 slots for constant word expansions
            word_cache: FxHashMap::with_capacity_and_hasher(256, Default::default()),
            // JIT runtime for hot loop compilation
            jit: crate::jit::JitRuntime::new(),
        }
    }

    /// Get a variable value with caching for frequently accessed vars
    #[inline]
    pub fn get_var(&self, name: &str) -> Option<&str> {
        // Fast path: check cache for commonly accessed variables
        if self.var_cache.is_valid() {
            match name {
                "PATH" => {
                    if let Some(ref v) = self.var_cache.path {
                        return Some(v.as_str());
                    }
                }
                "HOME" => {
                    if let Some(ref v) = self.var_cache.home {
                        return Some(v.as_str());
                    }
                }
                "USER" => {
                    if let Some(ref v) = self.var_cache.user {
                        return Some(v.as_str());
                    }
                }
                "PWD" => {
                    if let Some(ref v) = self.var_cache.pwd {
                        return Some(v.as_str());
                    }
                }
                "IFS" => {
                    if let Some(ref v) = self.var_cache.ifs {
                        return Some(v.as_str());
                    }
                }
                _ => {}
            }
        }

        // Check scope stack first (for function-local variables)
        if let Some(value) = self.scope_stack.get(name) {
            return Some(value);
        }

        // Then legacy vars -> env -> consts
        self.vars
            .get(name)
            .or_else(|| self.env.get(name))
            .or_else(|| self.consts.get(name))
            .map(|s| s.as_str())
    }

    /// Get a variable value without cache (for internal use)
    #[inline]
    pub fn get_var_uncached(&self, name: &str) -> Option<&str> {
        // Check scope stack first
        if let Some(value) = self.scope_stack.get(name) {
            return Some(value);
        }

        self.vars
            .get(name)
            .or_else(|| self.env.get(name))
            .or_else(|| self.consts.get(name))
            .map(|s| s.as_str())
    }

    /// Update dynamic POSIX variables (call before variable expansion)
    #[inline]
    pub fn update_dynamic_vars(&mut self) {
        // Update LINENO
        self.env
            .insert("LINENO".to_string(), self.lineno.to_string());

        // Update _ (last argument)
        if !self.last_arg.is_empty() {
            self.env.insert("_".to_string(), self.last_arg.clone());
        }

        // Update RANDOM (pseudo-random 0-32767)
        let random = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u32
            ^ self.shell_pid)
            % 32768;
        self.env.insert("RANDOM".to_string(), random.to_string());
    }

    /// Set a variable value with cache invalidation
    #[inline]
    pub fn set_var(&mut self, name: &str, value: &str) {
        if self.consts.contains_key(name) {
            eprintln!("fsh: {}: readonly variable", name);
            return;
        }

        // Invalidate cache for commonly modified variables
        self.update_var_cache(name, value);

        if self.exports.contains(name) {
            self.env.insert(name.to_string(), value.to_string());
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, value) };
        } else if !self.scope_stack.is_global() {
            // In a function scope, set in scope stack
            self.scope_stack.set(name, value.to_string());
        } else {
            // Global scope, use legacy vars
            self.vars.insert(name.to_string(), value.to_string());
        }
    }

    /// Set a local variable (only visible in current scope)
    #[inline]
    pub fn set_local_var(&mut self, name: &str, value: &str) {
        if self.consts.contains_key(name) {
            eprintln!("fsh: {}: readonly variable", name);
            return;
        }

        self.update_var_cache(name, value);
        self.scope_stack.set_local(name, value.to_string());
    }

    /// Push a new variable scope (call when entering a function)
    #[inline]
    pub fn push_scope(&mut self) {
        self.scope_stack.push_scope();
    }

    /// Pop the current variable scope (call when exiting a function)
    #[inline]
    pub fn pop_scope(&mut self) {
        self.scope_stack.pop_scope();
        // Invalidate cache since scope changed
        self.var_cache.invalidate();
    }

    /// Get the current scope depth (0 = global)
    #[inline]
    pub fn scope_depth(&self) -> usize {
        self.scope_stack.depth()
    }

    /// Check if we're in the global scope
    #[inline]
    pub fn is_global_scope(&self) -> bool {
        self.scope_stack.is_global()
    }

    /// Update the variable cache when a cached variable changes
    #[inline]
    fn update_var_cache(&mut self, name: &str, value: &str) {
        match name {
            "PATH" => self.var_cache.path = Some(value.to_string()),
            "HOME" => self.var_cache.home = Some(value.to_string()),
            "USER" => self.var_cache.user = Some(value.to_string()),
            "PWD" => self.var_cache.pwd = Some(value.to_string()),
            "IFS" => self.var_cache.ifs = Some(value.to_string()),
            _ => {}
        }
    }

    /// Invalidate the entire variable cache (call after bulk operations)
    #[inline]
    pub fn invalidate_var_cache(&mut self) {
        self.var_cache.invalidate();
    }

    /// Refresh the variable cache from current values
    pub fn refresh_var_cache(&mut self) {
        self.var_cache.path = self.get_var_uncached("PATH").map(|s| s.to_string());
        self.var_cache.home = self.get_var_uncached("HOME").map(|s| s.to_string());
        self.var_cache.user = self.get_var_uncached("USER").map(|s| s.to_string());
        self.var_cache.pwd = self.get_var_uncached("PWD").map(|s| s.to_string());
        self.var_cache.ifs = self.get_var_uncached("IFS").map(|s| s.to_string());
        self.var_cache.valid = true;
    }

    /// Export a variable
    #[inline]
    pub fn export_var(&mut self, name: &str, value: Option<&str>) {
        if let Some(val) = value {
            self.env.insert(name.to_string(), val.to_string());
            self.update_var_cache(name, val);
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, val) };
        } else if let Some(val) = self.vars.remove(name) {
            self.env.insert(name.to_string(), val.clone());
            self.update_var_cache(name, &val);
            // SAFETY: We control the environment and this is single-threaded
            unsafe { env::set_var(name, &val) };
        }
        self.exports.insert(name.to_string());
    }

    /// Execute a string as shell code
    #[inline]
    pub fn execute_string(&mut self, input: &str) -> Result<ExitStatus> {
        let mut parser = Parser::from_str(input)?;
        let program = parser.parse_program()?;
        self.execute(&program)
    }

    /// Run a script file
    pub fn run_script(&mut self, path: &str) -> Result<ExitStatus> {
        let mut file = File::open(path)?;
        // Pre-allocate string with file size hint
        let metadata = file.metadata().ok();
        let mut contents = if let Some(m) = metadata {
            String::with_capacity(m.len() as usize)
        } else {
            String::new()
        };
        file.read_to_string(&mut contents)?;
        self.execute_string(&contents)
    }
}
