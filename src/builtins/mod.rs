//! Built-in commands for Franken Shell
//!
//! This module provides all built-in shell commands organized into submodules:
//! - `navigation` - Directory navigation (cd, pwd, pushd, popd, dirs)
//! - `output` - Output commands (echo, printf)
//! - `variables` - Variable management (export, unset, set, local, readonly, declare)
//! - `control` - Control flow (exit, return, true, false)
//! - `jobs` - Job control (jobs, fg, bg, wait)
//! - `scripting` - Script execution (source, eval, exec, trap, shift, read)
//! - `test` - Conditional tests (test, [)
//! - `info` - Information commands (type, which, command, builtin, help)
//! - `misc` - Miscellaneous (alias, hash, umask, ulimit, let, history)
//! - `fish` - Fish-compatible (string, math, contains, status, functions, abbr)
//! - `git` - Git integration (git_branch, git_status, git_info, git_prompt, in_git_repo)
//! - `fzf` - FZF integration (fzf_history, fzf_file, fzf_dir, fzf_git_branch, fzf_cd, fzf_kill)
//! - `plugins` - Plugin manager (plug, plug_install, plug_update, plug_list, plug_load)
//! - `tmux` - Terminal multiplexer (tmux, tmux-new, tmux-ls, tmux-attach, tmux-theme)
//!
//! ## Performance
//! Uses PHF (Perfect Hash Function) for O(1) builtin lookup with zero runtime overhead.

mod control;
mod fish;
mod fzf;
mod git;
mod info;
mod jobs;
mod misc;
mod navigation;
mod output;
mod plugins;
mod posix;
mod scripting;
mod test;
mod tmux;
mod variables;

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use phf::phf_map;

/// Type for builtin command functions
pub type BuiltinFn = fn(&[String], &mut Interpreter) -> Result<ExitStatus>;

// =============================================================================
// Perfect Hash Map for O(1) Builtin Lookup
// =============================================================================

/// Static perfect hash map of all builtin commands
/// Generated at compile time for zero-cost lookup
static BUILTINS: phf::Map<&'static str, BuiltinFn> = phf_map! {
    // Navigation
    "cd" => navigation::builtin_cd,
    "pwd" => navigation::builtin_pwd,
    "pushd" => navigation::builtin_pushd,
    "popd" => navigation::builtin_popd,
    "dirs" => navigation::builtin_dirs,

    // Output
    "echo" => output::builtin_echo,
    "printf" => output::builtin_printf,

    // Variables
    "export" => variables::builtin_export,
    "unset" => variables::builtin_unset,
    "set" => variables::builtin_set,
    "local" => variables::builtin_local,
    "readonly" => variables::builtin_readonly,
    "declare" => variables::builtin_declare,
    "typeset" => variables::builtin_declare,

    // Control flow
    "exit" => control::builtin_exit,
    "return" => control::builtin_return,
    "true" => control::builtin_true,
    "false" => control::builtin_false,
    ":" => control::builtin_true,

    // Job control
    "jobs" => jobs::builtin_jobs,
    "fg" => jobs::builtin_fg,
    "bg" => jobs::builtin_bg,
    "wait" => jobs::builtin_wait,

    // Scripting
    "source" => scripting::builtin_source,
    "." => scripting::builtin_source,
    "eval" => scripting::builtin_eval,
    "exec" => scripting::builtin_exec,
    "trap" => scripting::builtin_trap,
    "shift" => scripting::builtin_shift,
    "read" => scripting::builtin_read,
    "getopts" => scripting::builtin_getopts,

    // Test
    "test" => test::builtin_test,
    "[" => test::builtin_test,

    // Information
    "type" => info::builtin_type,
    "which" => info::builtin_which,
    "command" => info::builtin_command,
    "builtin" => info::builtin_builtin,
    "help" => info::builtin_help,

    // Miscellaneous
    "alias" => misc::builtin_alias,
    "unalias" => misc::builtin_unalias,
    "hash" => misc::builtin_hash,
    "umask" => misc::builtin_umask,
    "ulimit" => misc::builtin_ulimit,
    "let" => misc::builtin_let_arith,
    "history" => misc::builtin_history,
    "times" => misc::builtin_times,
    "enable" => misc::builtin_enable,
    "shopt" => misc::builtin_shopt,
    "ssh-agent" => misc::builtin_ssh_agent,

    // POSIX
    "kill" => posix::builtin_kill,
    "fc" => posix::builtin_fc,
    "newgrp" => posix::builtin_newgrp,

    // Fish-compatible
    "string" => fish::builtin_string,
    "math" => fish::builtin_math,
    "contains" => fish::builtin_contains,
    "status" => fish::builtin_status,
    "functions" => fish::builtin_functions,
    "abbr" => fish::builtin_abbr,

    // Git integration
    "git_branch" => git::builtin_git_branch,
    "git_status" => git::builtin_git_status,
    "git_info" => git::builtin_git_info,
    "git_prompt" => git::builtin_git_prompt,
    "in_git_repo" => git::builtin_in_git_repo,

    // FZF integration
    "fzf" => fzf::builtin_fzf,
    "fzf_history" => fzf::builtin_fzf_history,
    "fzf_file" => fzf::builtin_fzf_file,
    "fzf_dir" => fzf::builtin_fzf_dir,
    "fzf_git_branch" => fzf::builtin_fzf_git_branch,
    "fzf_git_log" => fzf::builtin_fzf_git_log,
    "fzf_process" => fzf::builtin_fzf_process,
    "fzf_cd" => fzf::builtin_fzf_cd,
    "fzf_kill" => fzf::builtin_fzf_kill,

    // Plugin manager
    "plug" => plugins::builtin_plug,
    "plug_install" => plugins::builtin_plug_install,
    "plug_update" => plugins::builtin_plug_update,
    "plug_clean" => plugins::builtin_plug_clean,
    "plug_list" => plugins::builtin_plug_list,
    "plug_load" => plugins::builtin_plug_load,
    "plug_source" => plugins::builtin_plug_source,
    "plug_info" => plugins::builtin_plug_info,

    // Tmux
    "tmux" => tmux::builtin_tmux,
    "tmux-new" => tmux::builtin_tmux_new,
    "tmux-ls" => tmux::builtin_tmux_ls,
    "tmux-attach" => tmux::builtin_tmux_attach,
    "tmux-kill" => tmux::builtin_tmux_kill,
    "tmux-split" => tmux::builtin_tmux_split,
    "tmux-theme" => tmux::builtin_tmux_theme,
    "tmux-plugins" => tmux::builtin_tmux_plugins,
};

// =============================================================================
// Fast Builtin Lookup Functions
// =============================================================================

/// Check if a command is a builtin (O(1) lookup)
#[inline]
pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains_key(name)
}

/// Get a builtin function by name (O(1) lookup)
#[inline]
pub fn get_builtin(name: &str) -> Option<BuiltinFn> {
    BUILTINS.get(name).copied()
}

/// Execute a builtin command directly (fastest path)
#[inline]
pub fn execute_builtin(
    name: &str,
    args: &[String],
    interp: &mut Interpreter,
) -> Result<Option<ExitStatus>> {
    if let Some(func) = BUILTINS.get(name) {
        Ok(Some(func(args, interp)?))
    } else {
        Ok(None)
    }
}

/// List all builtin names
pub fn list_builtins() -> Vec<&'static str> {
    BUILTINS.keys().copied().collect()
}

// =============================================================================
// Legacy Builtins Struct (for backward compatibility)
// =============================================================================

/// Built-in command handler
///
/// This is a lightweight wrapper around the static PHF map.
/// Use the module-level functions (`is_builtin`, `execute_builtin`, `list_builtins`)
/// directly for best performance.
#[derive(Default)]
pub struct Builtins;

impl Builtins {
    /// Create a new Builtins instance (zero cost - uses static PHF map)
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Check if a command is a builtin (O(1) PHF lookup)
    #[inline]
    pub fn is_builtin(&self, name: &str) -> bool {
        is_builtin(name)
    }

    /// Execute a builtin command (O(1) PHF lookup)
    #[inline]
    pub fn execute(
        &self,
        name: &str,
        args: &[String],
        interp: &mut Interpreter,
    ) -> Result<Option<ExitStatus>> {
        execute_builtin(name, args, interp)
    }

    /// List all builtins
    pub fn list(&self) -> Vec<&'static str> {
        list_builtins()
    }
}
