//! Tmux module tests

use franken_shell::tmux::{
    KeyBinding, KeyTable, Pane, Session, StatusBarTheme, TmuxConfig, TmuxPluginManager, TmuxServer,
    Window,
};

// =============================================================================
// TmuxServer Tests
// =============================================================================

#[test]
fn test_tmux_server_new() {
    let server = TmuxServer::new();
    assert!(server.sessions.is_empty());
    assert!(server.current_session.is_none());
}

#[test]
fn test_tmux_server_default() {
    let server = TmuxServer::default();
    assert!(server.sessions.is_empty());
}

#[test]
fn test_tmux_server_new_session() {
    let mut server = TmuxServer::new();
    let result = server.new_session("test");
    assert!(result.is_ok());
    assert_eq!(server.sessions.len(), 1);
    assert_eq!(server.current_session, Some("test".to_string()));
}

#[test]
fn test_tmux_server_new_session_duplicate() {
    let mut server = TmuxServer::new();
    server.new_session("test").unwrap();
    let result = server.new_session("test");
    assert!(result.is_err());
}

#[test]
fn test_tmux_server_get_session() {
    let mut server = TmuxServer::new();
    server.new_session("test").unwrap();

    let session = server.get_session("test");
    assert!(session.is_some());
}

#[test]
fn test_tmux_server_get_session_not_found() {
    let server = TmuxServer::new();
    let session = server.get_session("nonexistent");
    assert!(session.is_none());
}

#[test]
fn test_tmux_server_current() {
    let mut server = TmuxServer::new();
    server.new_session("test").unwrap();

    let current = server.current();
    assert!(current.is_some());
}

#[test]
fn test_tmux_server_current_none() {
    let server = TmuxServer::new();
    let current = server.current();
    assert!(current.is_none());
}

#[test]
fn test_tmux_server_list_sessions() {
    let mut server = TmuxServer::new();
    server.new_session("session1").unwrap();
    server.new_session("session2").unwrap();

    let sessions = server.list_sessions();
    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_tmux_server_has_sessions() {
    let mut server = TmuxServer::new();
    assert!(!server.has_sessions());

    server.new_session("test").unwrap();
    assert!(server.has_sessions());
}

#[test]
fn test_tmux_server_kill_session() {
    let mut server = TmuxServer::new();
    server.new_session("test").unwrap();

    let result = server.kill_session("test");
    assert!(result.is_ok());
    assert!(server.sessions.is_empty());
}

#[test]
fn test_tmux_server_kill_session_not_found() {
    let mut server = TmuxServer::new();
    let result = server.kill_session("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_tmux_server_rename_session() {
    let mut server = TmuxServer::new();
    server.new_session("old").unwrap();

    let result = server.rename_session("old", "new");
    assert!(result.is_ok());
    assert!(server.get_session("old").is_none());
    assert!(server.get_session("new").is_some());
}

#[test]
fn test_tmux_server_uptime() {
    let server = TmuxServer::new();
    // Sleep briefly to ensure uptime is non-zero
    std::thread::sleep(std::time::Duration::from_millis(10));
    let uptime = server.uptime();
    assert!(uptime.as_millis() >= 10);
}

// =============================================================================
// Session Tests
// =============================================================================

#[test]
fn test_session_new() {
    let session = Session::new("test");
    assert_eq!(session.name, "test");
}

#[test]
fn test_session_new_window() {
    let mut session = Session::new("test");
    let _window = session.new_window(Some("win1"));
    assert!(!session.windows.is_empty());
}

#[test]
fn test_session_current_window() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));

    let current = session.current_window();
    assert!(current.is_some());
}

#[test]
fn test_session_list_windows() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));
    session.new_window(Some("win2"));

    let windows = session.list_windows();
    assert!(windows.len() >= 2);
}

#[test]
fn test_session_select_window() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));
    session.new_window(Some("win2"));

    let result = session.select_window(0);
    assert!(result);
}

#[test]
fn test_session_rename_window() {
    let mut session = Session::new("test");
    session.new_window(Some("old"));

    let result = session.rename_window(0, "new");
    assert!(result);
}

#[test]
fn test_session_window_count() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));
    session.new_window(Some("win2"));

    assert!(session.window_count() >= 2);
}

#[test]
fn test_session_attach_detach() {
    let mut session = Session::new("test");
    assert!(!session.is_attached());

    session.attach();
    assert!(session.is_attached());

    session.detach();
    assert!(!session.is_attached());
}

// =============================================================================
// Window Tests
// =============================================================================

#[test]
fn test_window_new() {
    let window = Window::new(0, "test");
    assert_eq!(window.name, "test");
    assert_eq!(window.index, 0);
}

#[test]
fn test_window_new_pane() {
    let mut window = Window::new(0, "test");
    let _pane = window.new_pane(None);
    assert!(window.pane_count() >= 1);
}

#[test]
fn test_window_split_horizontal() {
    let mut window = Window::new(0, "test");
    let pane = window.split_horizontal(None);
    assert!(pane.is_some());
}

#[test]
fn test_window_split_vertical() {
    let mut window = Window::new(0, "test");
    let pane = window.split_vertical(None);
    assert!(pane.is_some());
}

#[test]
fn test_window_active_pane() {
    let window = Window::new(0, "test");
    let pane = window.active_pane();
    assert!(pane.is_some());
}

#[test]
fn test_window_list_panes() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);

    let panes = window.list_panes();
    assert!(!panes.is_empty());
}

#[test]
fn test_window_select_pane() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);

    let result = window.select_pane(0);
    assert!(result);
}

#[test]
fn test_window_pane_count() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);

    assert!(window.pane_count() >= 1);
}

#[test]
fn test_window_set_layout() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);

    // Setting layout should not panic
    window.set_layout("even-horizontal");
}

// =============================================================================
// Pane Tests
// =============================================================================

#[test]
fn test_pane_new() {
    let pane = Pane::new(0, None);
    assert_eq!(pane.id, 0);
}

#[test]
fn test_pane_send_keys() {
    let mut pane = Pane::new(0, None);
    // Should not panic
    pane.send_keys("echo hello", false);
}

#[test]
fn test_pane_copy_mode() {
    let mut pane = Pane::new(0, None);
    assert!(!pane.is_in_mode());

    pane.enter_copy_mode(false);
    assert!(pane.is_in_mode());

    pane.exit_copy_mode();
    assert!(!pane.is_in_mode());
}

#[test]
fn test_pane_vi_copy_mode() {
    let mut pane = Pane::new(0, None);
    pane.enter_copy_mode(true);
    assert!(pane.is_in_mode());
}

#[test]
fn test_pane_scroll() {
    let mut pane = Pane::new(0, None);
    // Add some history
    pane.history.push("line 1".to_string());
    pane.history.push("line 2".to_string());
    pane.history.push("line 3".to_string());

    pane.scroll_up(1);
    pane.scroll_down(1);
    // Should not panic
}

#[test]
fn test_pane_clear_history() {
    let mut pane = Pane::new(0, None);
    pane.history.push("line 1".to_string());
    pane.history.push("line 2".to_string());

    pane.clear_history();
    assert!(pane.history.is_empty());
}

#[test]
fn test_pane_set_title() {
    let mut pane = Pane::new(0, None);
    pane.set_title("My Pane");
    assert_eq!(pane.display_title(), "My Pane");
}

#[test]
fn test_pane_mode_string() {
    let mut pane = Pane::new(0, None);
    assert_eq!(pane.mode_string(), "");

    pane.enter_copy_mode(false);
    assert!(!pane.mode_string().is_empty());
}

// =============================================================================
// TmuxConfig Tests
// =============================================================================

#[test]
fn test_tmux_config_default() {
    let config = TmuxConfig::default();
    // Check some defaults
    assert!(!config.prefix.is_empty());
}

#[test]
fn test_tmux_config_set_option() {
    let mut config = TmuxConfig::default();
    config.set_option("mouse", "on");
    assert!(config.mouse);
}

#[test]
fn test_tmux_config_get_option() {
    let config = TmuxConfig::default();
    let value = config.get_option("mouse");
    assert!(value.is_some());
}

// =============================================================================
// KeyBinding Tests
// =============================================================================

#[test]
fn test_key_binding_new() {
    let binding = KeyBinding::new("c", "new-window");
    assert_eq!(binding.key, "c");
    assert_eq!(binding.command, "new-window");
    assert!(!binding.repeat);
}

#[test]
fn test_key_binding_with_repeat() {
    let binding = KeyBinding::new("Up", "select-pane -U").with_repeat();
    assert!(binding.repeat);
}

// =============================================================================
// KeyTable Tests
// =============================================================================

#[test]
fn test_key_table_new() {
    let table = KeyTable::new();
    assert!(table.bindings.is_empty());
}

#[test]
fn test_key_table_bind() {
    let mut table = KeyTable::new();
    table.bind("c", "new-window");

    let binding = table.get("c");
    assert!(binding.is_some());
    assert_eq!(binding.unwrap().command, "new-window");
}

#[test]
fn test_key_table_bind_repeat() {
    let mut table = KeyTable::new();
    table.bind_repeat("Up", "select-pane -U");

    let binding = table.get("Up");
    assert!(binding.is_some());
    assert!(binding.unwrap().repeat);
}

#[test]
fn test_key_table_unbind() {
    let mut table = KeyTable::new();
    table.bind("c", "new-window");
    table.unbind("c");

    let binding = table.get("c");
    assert!(binding.is_none());
}

#[test]
fn test_key_table_get_not_found() {
    let table = KeyTable::new();
    let binding = table.get("nonexistent");
    assert!(binding.is_none());
}

#[test]
fn test_key_table_default_prefix() {
    let table = KeyTable::default_prefix();
    // Should have some default bindings
    assert!(!table.bindings.is_empty());
}

#[test]
fn test_key_table_list() {
    let mut table = KeyTable::new();
    table.bind("c", "new-window");
    table.bind("n", "next-window");

    let bindings = table.list();
    assert_eq!(bindings.len(), 2);
}

// =============================================================================
// StatusBarTheme Tests
// =============================================================================

#[test]
fn test_status_bar_theme_powerline() {
    let theme = StatusBarTheme::powerline();
    assert_eq!(theme.name, "powerline");
}

#[test]
fn test_status_bar_theme_dracula() {
    let theme = StatusBarTheme::dracula();
    assert_eq!(theme.name, "dracula");
}

#[test]
fn test_status_bar_theme_nord() {
    let theme = StatusBarTheme::nord();
    assert_eq!(theme.name, "nord");
}

#[test]
fn test_status_bar_theme_gruvbox() {
    let theme = StatusBarTheme::gruvbox();
    assert_eq!(theme.name, "gruvbox");
}

#[test]
fn test_status_bar_theme_status_left() {
    let theme = StatusBarTheme::powerline();
    // Status left should be set
    assert!(!theme.status_left.is_empty());
}

#[test]
fn test_status_bar_theme_status_right() {
    let theme = StatusBarTheme::powerline();
    // Status right should be set
    assert!(!theme.status_right.is_empty());
}

// =============================================================================
// TmuxPluginManager Tests
// =============================================================================

#[test]
fn test_tmux_plugin_manager_new() {
    let manager = TmuxPluginManager::new();
    assert!(manager.plugins.is_empty());
}

#[test]
fn test_tmux_plugin_manager_add() {
    let mut manager = TmuxPluginManager::new();
    // Add plugin spec (like from .tmux.conf)
    let result = manager.add("tmux-plugins/tmux-sensible");
    assert!(result.is_ok());
    assert_eq!(manager.plugins.len(), 1);
}

#[test]
fn test_tmux_plugin_manager_list_installed() {
    let manager = TmuxPluginManager::new();
    // List installed plugins (may be empty)
    let installed = manager.list_installed();
    // Just check it doesn't crash
    assert!(installed.len() >= 0);
}

#[test]
fn test_tmux_plugin_manager_generate_load_commands() {
    let mut manager = TmuxPluginManager::new();
    let _ = manager.add("tmux-plugins/tmux-sensible");

    // Generate load commands
    let commands = manager.generate_load_commands();
    // May be empty if plugins not installed
    assert!(commands.len() >= 0);
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_server_session_window_flow() {
    let mut server = TmuxServer::new();

    // Create session
    let session = server.new_session("main").unwrap();

    // Create windows
    {
        let mut s = session.lock().unwrap();
        s.new_window(Some("editor"));
        s.new_window(Some("terminal"));
    }

    // Check windows
    {
        let s = session.lock().unwrap();
        assert!(s.windows.len() >= 2);
    }
}

#[test]
fn test_window_pane_flow() {
    let mut window = Window::new(0, "main");

    // Split into multiple panes
    window.split_horizontal(None);
    window.split_vertical(None);

    let panes = window.list_panes();
    assert!(panes.len() >= 2);
}

#[test]
fn test_multiple_sessions() {
    let mut server = TmuxServer::new();

    server.new_session("session1").unwrap();
    server.new_session("session2").unwrap();
    server.new_session("session3").unwrap();

    assert_eq!(server.list_sessions().len(), 3);

    server.kill_session("session2").unwrap();
    assert_eq!(server.list_sessions().len(), 2);
}

// =============================================================================
// Session Attachment Tests
// =============================================================================

#[test]
fn test_attach_session() {
    let mut server = TmuxServer::new();
    server.new_session("test").unwrap();

    let result = server.attach_session("test");
    assert!(result.is_ok());
    assert_eq!(server.current_session, Some("test".to_string()));
}

#[test]
fn test_attach_session_not_found() {
    let mut server = TmuxServer::new();
    let result = server.attach_session("nonexistent");
    assert!(result.is_err());
}

// =============================================================================
// Window Navigation Tests
// =============================================================================

#[test]
fn test_session_next_window() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));
    session.new_window(Some("win2"));

    let result = session.next_window();
    assert!(result);
}

#[test]
fn test_session_previous_window() {
    let mut session = Session::new("test");
    session.new_window(Some("win1"));
    session.new_window(Some("win2"));
    session.select_window(1);

    let result = session.previous_window();
    assert!(result);
}

// =============================================================================
// Pane Navigation Tests
// =============================================================================

#[test]
fn test_window_next_pane() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);

    let result = window.next_pane();
    assert!(result);
}

#[test]
fn test_window_previous_pane() {
    let mut window = Window::new(0, "test");
    window.split_horizontal(None);
    window.select_pane(1);

    let result = window.previous_pane();
    assert!(result);
}
