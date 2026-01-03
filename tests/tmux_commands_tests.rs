//! Unit tests for tmux integration via shell commands

use franken_shell::Shell;

#[test]
fn test_tmux_command_exists() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("type tmux");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_help() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux help");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_version() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux version");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_new_session() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux new-session -d -s test_session");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_list_sessions() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s list_test");
    let result = shell.run_command("tmux list-sessions");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_has_session() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s has_test");
    let result = shell.run_command("tmux has-session -t has_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_kill_session() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s kill_test");
    let result = shell.run_command("tmux kill-session -t kill_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_new_window() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s win_test");
    let result = shell.run_command("tmux new-window -t win_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_list_windows() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s listwin_test");
    let result = shell.run_command("tmux list-windows -t listwin_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_split_window() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s split_test");
    let result = shell.run_command("tmux split-window -t split_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_list_panes() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s pane_test");
    let result = shell.run_command("tmux list-panes -t pane_test");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_send_keys() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // Create a session first
    let _ = shell.run_command("tmux new-session -d -s keys_test");
    let result = shell.run_command("tmux send-keys -t keys_test 'echo hello' Enter");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_set_option() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux set-option -g prefix C-a");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_show_options() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux show-options -g");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_bind_key() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux bind-key c new-window");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_unbind_key() {
    let mut shell = Shell::with_options(false, false).unwrap();
    // First bind a key
    let _ = shell.run_command("tmux bind-key x kill-pane");
    let result = shell.run_command("tmux unbind-key x");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_list_keys() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux list-keys");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_display_message() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux display-message 'Hello World'");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_run_shell() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux run-shell 'echo test'");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_refresh_client() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux refresh-client");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_theme_list() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux theme list");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_theme_set() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux theme set dracula");
    assert!(result.is_ok());
}

#[test]
fn test_tmux_list_plugins() {
    let mut shell = Shell::with_options(false, false).unwrap();
    let result = shell.run_command("tmux list-plugins");
    assert!(result.is_ok());
}
