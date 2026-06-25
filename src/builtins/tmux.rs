//! Tmux built-in commands

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use crate::tmux::{TmuxServer, commands::execute_command};
use std::sync::Mutex;

// Global tmux server instance
static TMUX_SERVER: Mutex<Option<TmuxServer>> = Mutex::new(None);

fn get_server() -> std::sync::MutexGuard<'static, Option<TmuxServer>> {
    let mut guard = TMUX_SERVER.lock().unwrap();
    if guard.is_none() {
        let mut server = TmuxServer::new();
        // Load default config
        let _ = server.load_config(None);
        *guard = Some(server);
    }
    guard
}

/// tmux - Main tmux command
/// Usage: tmux [command] [args...]
///
/// Session commands:
///   new-session [-d] [-s name]     Create a new session
///   kill-session -t name           Kill a session
///   list-sessions                  List all sessions
///   attach [-t name]               Attach to a session
///   detach                         Detach from session
///
/// Window commands:
///   new-window [-n name]           Create a new window
///   kill-window                    Kill current window
///   list-windows                   List windows
///   select-window -t index         Select a window
///   next-window / previous-window  Navigate windows
///
/// Pane commands:
///   split-window [-h|-v]           Split pane
///   kill-pane                      Kill current pane
///   select-pane -U/-D/-L/-R        Navigate panes
///   resize-pane -U/-D/-L/-R [n]    Resize pane
///
/// Other:
///   set -g option value            Set option
///   bind key command               Bind key
///   source-file file               Load config
///   set-theme name                 Apply theme
pub fn builtin_tmux(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // No command - show help
        print_tmux_help();
        return Ok(ExitStatus::success());
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    let mut server = get_server();
    if let Some(ref mut srv) = *server {
        match execute_command(srv, cmd, &cmd_args) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
                Ok(ExitStatus::success())
            }
            Err(e) => {
                eprintln!("tmux: {}", e);
                Ok(ExitStatus::failure(1))
            }
        }
    } else {
        eprintln!("tmux: failed to initialize server");
        Ok(ExitStatus::failure(1))
    }
}

/// tmux-new - Create new session (shorthand)
pub fn builtin_tmux_new(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut new_args = vec!["new-session".to_string()];
    new_args.extend(args.iter().cloned());
    builtin_tmux(&new_args, interp)
}

/// tmux-ls - List sessions (shorthand)
pub fn builtin_tmux_ls(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    builtin_tmux(&["list-sessions".to_string()], interp)
}

/// tmux-attach - Attach to session (shorthand)
pub fn builtin_tmux_attach(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut new_args = vec!["attach-session".to_string()];
    new_args.extend(args.iter().cloned());
    builtin_tmux(&new_args, interp)
}

/// tmux-kill - Kill session (shorthand)
pub fn builtin_tmux_kill(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut new_args = vec!["kill-session".to_string()];
    new_args.extend(args.iter().cloned());
    builtin_tmux(&new_args, interp)
}

/// tmux-split - Split window (shorthand)
pub fn builtin_tmux_split(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let mut new_args = vec!["split-window".to_string()];
    new_args.extend(args.iter().cloned());
    builtin_tmux(&new_args, interp)
}

/// tmux-theme - Apply a theme
pub fn builtin_tmux_theme(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // List themes
        builtin_tmux(&["list-themes".to_string()], interp)
    } else {
        let mut new_args = vec!["set-theme".to_string()];
        new_args.extend(args.iter().cloned());
        builtin_tmux(&new_args, interp)
    }
}

/// tmux-plugins - Plugin management
pub fn builtin_tmux_plugins(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        builtin_tmux(&["tpm-list".to_string()], interp)
    } else {
        let cmd = match args[0].as_str() {
            "install" => "tpm-install",
            "update" => "tpm-update",
            "clean" => "tpm-clean",
            "list" => "tpm-list",
            _ => {
                eprintln!("tmux-plugins: unknown command: {}", args[0]);
                eprintln!("Commands: install, update, clean, list");
                return Ok(ExitStatus::failure(1));
            }
        };
        builtin_tmux(&[cmd.to_string()], interp)
    }
}

fn print_tmux_help() {
    println!("franken tmux - Terminal multiplexer (tmux-compatible)");
    println!();
    println!("Usage: tmux [command] [options]");
    println!();
    println!("Session Commands:");
    println!("  new-session [-d] [-s name]     Create new session");
    println!("  attach-session [-t name]       Attach to session");
    println!("  detach-client                  Detach from session");
    println!("  kill-session -t name           Kill session");
    println!("  list-sessions                  List sessions");
    println!("  rename-session new-name        Rename session");
    println!("  switch-client -n/-p/-t name    Switch session");
    println!();
    println!("Window Commands:");
    println!("  new-window [-n name]           Create window");
    println!("  kill-window                    Kill window");
    println!("  rename-window name             Rename window");
    println!("  list-windows                   List windows");
    println!("  select-window -t index         Select window");
    println!("  next-window                    Next window");
    println!("  previous-window                Previous window");
    println!("  last-window                    Last window");
    println!();
    println!("Pane Commands:");
    println!("  split-window [-h|-v]           Split pane");
    println!("  kill-pane                      Kill pane");
    println!("  select-pane -U/-D/-L/-R        Select pane");
    println!("  resize-pane -U/-D/-L/-R [n]    Resize pane");
    println!("  display-panes                  Show pane numbers");
    println!("  select-layout layout           Set layout");
    println!();
    println!("Configuration:");
    println!("  set [-g] option value          Set option");
    println!("  setw option value              Set window option");
    println!("  show-options                   Show options");
    println!("  source-file file               Load config file");
    println!("  bind key command               Bind key");
    println!("  unbind key                     Unbind key");
    println!("  list-keys                      List key bindings");
    println!();
    println!("Themes:");
    println!("  set-theme name                 Apply theme");
    println!("  list-themes                    List themes");
    println!();
    println!("Plugins (TPM Compatible):");
    println!("  tpm-install                    Install plugins");
    println!("  tpm-update                     Update plugins");
    println!("  tpm-clean                      Remove unused plugins");
    println!("  tpm-list                       List plugins");
    println!();
    println!("Shortcut Commands:");
    println!("  tmux-new [-s name]             Create session");
    println!("  tmux-ls                        List sessions");
    println!("  tmux-attach [-t name]          Attach to session");
    println!("  tmux-kill -t name              Kill session");
    println!("  tmux-split [-h|-v]             Split pane");
    println!("  tmux-theme [name]              Apply/list themes");
    println!("  tmux-plugins [cmd]             Plugin management");
    println!();
    println!("Layouts: even-horizontal, even-vertical, main-horizontal,");
    println!("         main-vertical, tiled");
    println!();
    println!("Themes: default, powerline, dracula, nord, gruvbox,");
    println!("        catppuccin-mocha, tokyo-night, one-dark, minimal");
}
