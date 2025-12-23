//! Built-in commands for jsh shell
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
use std::collections::HashMap;

/// Type for builtin command functions
pub type BuiltinFn = fn(&[String], &mut Interpreter) -> Result<ExitStatus>;

/// Built-in command handler
pub struct Builtins {
    commands: HashMap<String, BuiltinFn>,
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}

impl Builtins {
    pub fn new() -> Self {
        let mut commands: HashMap<String, BuiltinFn> = HashMap::new();

        // Navigation
        commands.insert("cd".to_string(), navigation::builtin_cd);
        commands.insert("pwd".to_string(), navigation::builtin_pwd);
        commands.insert("pushd".to_string(), navigation::builtin_pushd);
        commands.insert("popd".to_string(), navigation::builtin_popd);
        commands.insert("dirs".to_string(), navigation::builtin_dirs);

        // Output
        commands.insert("echo".to_string(), output::builtin_echo);
        commands.insert("printf".to_string(), output::builtin_printf);

        // Variables
        commands.insert("export".to_string(), variables::builtin_export);
        commands.insert("unset".to_string(), variables::builtin_unset);
        commands.insert("set".to_string(), variables::builtin_set);
        commands.insert("local".to_string(), variables::builtin_local);
        commands.insert("readonly".to_string(), variables::builtin_readonly);
        commands.insert("declare".to_string(), variables::builtin_declare);
        commands.insert("typeset".to_string(), variables::builtin_declare); // alias

        // Control flow
        commands.insert("exit".to_string(), control::builtin_exit);
        commands.insert("return".to_string(), control::builtin_return);
        commands.insert("true".to_string(), control::builtin_true);
        commands.insert("false".to_string(), control::builtin_false);
        commands.insert(":".to_string(), control::builtin_true); // colon is true

        // Job control
        commands.insert("jobs".to_string(), jobs::builtin_jobs);
        commands.insert("fg".to_string(), jobs::builtin_fg);
        commands.insert("bg".to_string(), jobs::builtin_bg);
        commands.insert("wait".to_string(), jobs::builtin_wait);

        // Source and eval
        commands.insert("source".to_string(), scripting::builtin_source);
        commands.insert(".".to_string(), scripting::builtin_source);
        commands.insert("eval".to_string(), scripting::builtin_eval);

        // Test
        commands.insert("test".to_string(), test::builtin_test);
        commands.insert("[".to_string(), test::builtin_test);

        // Information
        commands.insert("type".to_string(), info::builtin_type);
        commands.insert("which".to_string(), info::builtin_which);
        commands.insert("command".to_string(), info::builtin_command);
        commands.insert("builtin".to_string(), info::builtin_builtin);
        commands.insert("help".to_string(), info::builtin_help);

        // Misc
        commands.insert("alias".to_string(), misc::builtin_alias);
        commands.insert("unalias".to_string(), misc::builtin_unalias);
        commands.insert("hash".to_string(), misc::builtin_hash);
        commands.insert("read".to_string(), scripting::builtin_read);
        commands.insert("shift".to_string(), scripting::builtin_shift);
        commands.insert("exec".to_string(), scripting::builtin_exec);
        commands.insert("trap".to_string(), scripting::builtin_trap);
        commands.insert("umask".to_string(), misc::builtin_umask);
        commands.insert("ulimit".to_string(), misc::builtin_ulimit);
        commands.insert("times".to_string(), posix::builtin_times_posix);
        commands.insert("getopts".to_string(), scripting::builtin_getopts);
        commands.insert("enable".to_string(), misc::builtin_enable);
        commands.insert("shopt".to_string(), misc::builtin_shopt);
        commands.insert("let".to_string(), misc::builtin_let_arith);
        commands.insert("history".to_string(), misc::builtin_history);

        // Fish-compatible builtins
        commands.insert("string".to_string(), fish::builtin_string);
        commands.insert("contains".to_string(), fish::builtin_contains);
        commands.insert("status".to_string(), fish::builtin_status);
        commands.insert("functions".to_string(), fish::builtin_functions);
        commands.insert("abbr".to_string(), fish::builtin_abbr);
        commands.insert("math".to_string(), fish::builtin_math);

        // Git integration
        commands.insert("git_branch".to_string(), git::builtin_git_branch);
        commands.insert("git_status".to_string(), git::builtin_git_status);
        commands.insert("git_info".to_string(), git::builtin_git_info);
        commands.insert("git_prompt".to_string(), git::builtin_git_prompt);
        commands.insert("in_git_repo".to_string(), git::builtin_in_git_repo);

        // POSIX required builtins
        commands.insert("kill".to_string(), posix::builtin_kill);
        commands.insert("fc".to_string(), posix::builtin_fc);
        commands.insert("newgrp".to_string(), posix::builtin_newgrp);

        // SSH agent integration
        commands.insert("ssh_agent".to_string(), misc::builtin_ssh_agent);

        // FZF integration
        commands.insert("fzf".to_string(), fzf::builtin_fzf);
        commands.insert("fzf_history".to_string(), fzf::builtin_fzf_history);
        commands.insert("fzf_file".to_string(), fzf::builtin_fzf_file);
        commands.insert("fzf_dir".to_string(), fzf::builtin_fzf_dir);
        commands.insert("fzf_cd".to_string(), fzf::builtin_fzf_cd);
        commands.insert("fzf_git_branch".to_string(), fzf::builtin_fzf_git_branch);
        commands.insert("fzf_git_log".to_string(), fzf::builtin_fzf_git_log);
        commands.insert("fzf_process".to_string(), fzf::builtin_fzf_process);
        commands.insert("fzf_kill".to_string(), fzf::builtin_fzf_kill);

        // Plugin manager
        commands.insert("plug".to_string(), plugins::builtin_plug);
        commands.insert("plug_install".to_string(), plugins::builtin_plug_install);
        commands.insert("plug_update".to_string(), plugins::builtin_plug_update);
        commands.insert("plug_clean".to_string(), plugins::builtin_plug_clean);
        commands.insert("plug_list".to_string(), plugins::builtin_plug_list);
        commands.insert("plug_load".to_string(), plugins::builtin_plug_load);
        commands.insert("plug_source".to_string(), plugins::builtin_plug_source);
        commands.insert("plug_info".to_string(), plugins::builtin_plug_info);

        // Tmux integration
        commands.insert("tmux".to_string(), tmux::builtin_tmux);
        commands.insert("tmux-new".to_string(), tmux::builtin_tmux_new);
        commands.insert("tmux-ls".to_string(), tmux::builtin_tmux_ls);
        commands.insert("tmux-attach".to_string(), tmux::builtin_tmux_attach);
        commands.insert("tmux-kill".to_string(), tmux::builtin_tmux_kill);
        commands.insert("tmux-split".to_string(), tmux::builtin_tmux_split);
        commands.insert("tmux-theme".to_string(), tmux::builtin_tmux_theme);
        commands.insert("tmux-plugins".to_string(), tmux::builtin_tmux_plugins);

        Self { commands }
    }

    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Execute a builtin command
    pub fn execute(
        &self,
        name: &str,
        args: &[String],
        interp: &mut Interpreter,
    ) -> Result<Option<ExitStatus>> {
        if let Some(func) = self.commands.get(name) {
            Ok(Some(func(args, interp)?))
        } else {
            Ok(None)
        }
    }

    /// List all builtins
    pub fn list(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}


