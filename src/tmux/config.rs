//! Tmux configuration file parser
//!
//! Parses .tmux.conf files with full compatibility

use crate::error::{JshError, Result};
use super::{TmuxColor, TmuxStyle, FormatSpec};
use super::keybind::{KeyBinding, KeyTable};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Tmux configuration
#[derive(Debug, Clone)]
pub struct TmuxConfig {
    // General options
    pub default_shell: Option<String>,
    pub default_command: Option<String>,
    pub default_terminal: String,
    pub history_limit: usize,
    pub escape_time: Duration,
    pub repeat_time: Duration,
    pub display_time: Duration,
    pub display_panes_time: Duration,
    pub focus_events: bool,
    pub mouse: bool,
    pub set_clipboard: ClipboardMode,
    pub base_index: usize,
    pub pane_base_index: usize,
    pub renumber_windows: bool,
    pub automatic_rename: bool,
    pub automatic_rename_format: String,
    pub allow_rename: bool,
    pub set_titles: bool,
    pub set_titles_string: String,

    // Prefix key
    pub prefix: String,
    pub prefix2: Option<String>,

    // Status bar
    pub status: StatusPosition,
    pub status_interval: Duration,
    pub status_justify: StatusJustify,
    pub status_left: FormatSpec,
    pub status_left_length: usize,
    pub status_right: FormatSpec,
    pub status_right_length: usize,
    pub status_style: TmuxStyle,
    pub status_left_style: TmuxStyle,
    pub status_right_style: TmuxStyle,

    // Window status
    pub window_status_format: String,
    pub window_status_current_format: String,
    pub window_status_separator: String,
    pub window_status_style: TmuxStyle,
    pub window_status_current_style: TmuxStyle,
    pub window_status_activity_style: TmuxStyle,
    pub window_status_bell_style: TmuxStyle,

    // Pane options
    pub pane_border_style: TmuxStyle,
    pub pane_active_border_style: TmuxStyle,
    pub pane_border_format: String,
    pub pane_border_status: PaneBorderStatus,
    pub pane_border_lines: PaneBorderLines,

    // Message/mode styles
    pub message_style: TmuxStyle,
    pub message_command_style: TmuxStyle,
    pub mode_style: TmuxStyle,

    // Clock
    pub clock_mode_colour: TmuxColor,
    pub clock_mode_style: ClockStyle,

    // Activity
    pub activity_action: ActivityAction,
    pub bell_action: BellAction,
    pub silence_action: ActivityAction,
    pub visual_activity: VisualMode,
    pub visual_bell: VisualMode,
    pub visual_silence: VisualMode,
    pub monitor_activity: bool,
    pub monitor_bell: bool,
    pub monitor_silence: Duration,

    // Key bindings
    pub key_tables: HashMap<String, KeyTable>,

    // Hooks
    pub hooks: HashMap<String, Vec<String>>,

    // Custom options (user-defined)
    pub user_options: HashMap<String, String>,

    // Environment variables to update
    pub update_environment: Vec<String>,

    // Plugin settings
    pub tpm_plugins: Vec<String>,
}

impl Default for TmuxConfig {
    fn default() -> Self {
        let mut key_tables = HashMap::new();
        key_tables.insert("prefix".to_string(), KeyTable::default_prefix());
        key_tables.insert("root".to_string(), KeyTable::default_root());
        key_tables.insert("copy-mode".to_string(), KeyTable::default_copy_mode());
        key_tables.insert("copy-mode-vi".to_string(), KeyTable::default_copy_mode_vi());

        Self {
            default_shell: None,
            default_command: None,
            default_terminal: "screen-256color".to_string(),
            history_limit: 2000,
            escape_time: Duration::from_millis(500),
            repeat_time: Duration::from_millis(500),
            display_time: Duration::from_millis(750),
            display_panes_time: Duration::from_millis(1000),
            focus_events: false,
            mouse: false,
            set_clipboard: ClipboardMode::External,
            base_index: 0,
            pane_base_index: 0,
            renumber_windows: false,
            automatic_rename: true,
            automatic_rename_format: "#{?pane_in_mode,[tmux],#{pane_current_command}}#{?pane_dead,[dead],}".to_string(),
            allow_rename: true,
            set_titles: false,
            set_titles_string: "#S:#I:#W - \"#T\" #{session_alerts}".to_string(),

            prefix: "C-b".to_string(),
            prefix2: None,

            status: StatusPosition::Bottom,
            status_interval: Duration::from_secs(15),
            status_justify: StatusJustify::Left,
            status_left: FormatSpec::new("[#S] "),
            status_left_length: 10,
            status_right: FormatSpec::new(" \"#H\" %H:%M %d-%b-%y"),
            status_right_length: 40,
            status_style: TmuxStyle::parse("fg=black,bg=green"),
            status_left_style: TmuxStyle::default(),
            status_right_style: TmuxStyle::default(),

            window_status_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_current_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_separator: " ".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::parse("fg=black,bg=yellow"),
            window_status_activity_style: TmuxStyle::parse("reverse"),
            window_status_bell_style: TmuxStyle::parse("reverse"),

            pane_border_style: TmuxStyle::parse("fg=default"),
            pane_active_border_style: TmuxStyle::parse("fg=green"),
            pane_border_format: "#{?pane_active,#[reverse],}#{pane_index}#[default] \"#{pane_title}\"".to_string(),
            pane_border_status: PaneBorderStatus::Off,
            pane_border_lines: PaneBorderLines::Single,

            message_style: TmuxStyle::parse("fg=black,bg=yellow"),
            message_command_style: TmuxStyle::parse("fg=yellow,bg=black"),
            mode_style: TmuxStyle::parse("fg=black,bg=yellow"),

            clock_mode_colour: TmuxColor::Blue,
            clock_mode_style: ClockStyle::H24,

            activity_action: ActivityAction::Other,
            bell_action: BellAction::Any,
            silence_action: ActivityAction::Other,
            visual_activity: VisualMode::Off,
            visual_bell: VisualMode::Off,
            visual_silence: VisualMode::Off,
            monitor_activity: false,
            monitor_bell: true,
            monitor_silence: Duration::from_secs(0),

            key_tables,

            hooks: HashMap::new(),
            user_options: HashMap::new(),
            update_environment: vec![
                "DISPLAY".to_string(),
                "SSH_ASKPASS".to_string(),
                "SSH_AUTH_SOCK".to_string(),
                "SSH_AGENT_PID".to_string(),
                "SSH_CONNECTION".to_string(),
                "WINDOWID".to_string(),
                "XAUTHORITY".to_string(),
            ],
            tpm_plugins: vec![],
        }
    }
}

impl TmuxConfig {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| JshError::runtime(format!("Failed to read config: {}", e)))?;

        let mut config = TmuxConfig::default();
        config.parse(&content)?;
        Ok(config)
    }

    /// Parse configuration content
    pub fn parse(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Handle line continuations
            let line = if let Some(stripped) = line.strip_suffix('\\') {
                // TODO: Handle multi-line commands
                stripped
            } else {
                line
            };

            self.parse_command(line)?;
        }

        Ok(())
    }

    /// Parse a single tmux command
    pub fn parse_command(&mut self, cmd: &str) -> Result<()> {
        let parts: Vec<&str> = shell_split(cmd);
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "set" | "set-option" => self.handle_set(&parts[1..])?,
            "setw" | "set-window-option" => self.handle_setw(&parts[1..])?,
            "bind" | "bind-key" => self.handle_bind(&parts[1..])?,
            "unbind" | "unbind-key" => self.handle_unbind(&parts[1..])?,
            "set-hook" => self.handle_set_hook(&parts[1..])?,
            "run" | "run-shell" => { /* Executed at load time */ }
            "source" | "source-file" => {
                if parts.len() > 1 {
                    let path = expand_path(parts[1]);
                    if let Ok(content) = fs::read_to_string(&path) {
                        self.parse(&content)?;
                    }
                }
            }
            "if" | "if-shell" => { /* TODO: Conditional */ }
            _ => {
                // Unknown command - store for later or ignore
            }
        }

        Ok(())
    }

    fn handle_set(&mut self, args: &[&str]) -> Result<()> {
        let mut args = args.iter().peekable();
        let mut _global = false;
        let mut _append = false;
        let mut _quiet = false;

        // Parse flags
        while let Some(arg) = args.peek() {
            match **arg {
                "-g" => { _global = true; args.next(); }
                "-a" => { _append = true; args.next(); }
                "-q" => { _quiet = true; args.next(); }
                "-s" | "-w" | "-u" | "-o" | "-F" | "-U" => { args.next(); }
                _ if arg.starts_with('-') => { args.next(); }
                _ => break,
            }
        }

        let option = match args.next() {
            Some(o) => *o,
            None => return Ok(()),
        };

        let value = args.copied().collect::<Vec<_>>().join(" ");
        let value = value.trim_matches(|c| c == '"' || c == '\'');

        self.set_option(option, value)
    }

    fn handle_setw(&mut self, args: &[&str]) -> Result<()> {
        // setw is an alias for set-window-option, handle similarly
        self.handle_set(args)
    }

    fn handle_bind(&mut self, args: &[&str]) -> Result<()> {
        let mut args = args.iter().peekable();
        let mut table = "prefix".to_string();
        let mut repeat = false;

        // Parse flags
        while let Some(arg) = args.peek() {
            match **arg {
                "-T" => {
                    args.next();
                    if let Some(t) = args.next() {
                        table = t.to_string();
                    }
                }
                "-n" => {
                    args.next();
                    table = "root".to_string();
                }
                "-r" => {
                    args.next();
                    repeat = true;
                }
                _ if arg.starts_with('-') => { args.next(); }
                _ => break,
            }
        }

        let key = match args.next() {
            Some(k) => k.to_string(),
            None => return Ok(()),
        };

        let command: String = args.copied().collect::<Vec<_>>().join(" ");

        let binding = KeyBinding {
            key: key.clone(),
            command,
            repeat,
        };

        self.key_tables
            .entry(table)
            .or_default()
            .bindings
            .insert(key, binding);

        Ok(())
    }

    fn handle_unbind(&mut self, args: &[&str]) -> Result<()> {
        let mut args = args.iter().peekable();
        let mut table = "prefix".to_string();

        while let Some(arg) = args.peek() {
            match **arg {
                "-T" => {
                    args.next();
                    if let Some(t) = args.next() {
                        table = t.to_string();
                    }
                }
                "-n" => {
                    args.next();
                    table = "root".to_string();
                }
                "-a" => {
                    args.next();
                    // Unbind all
                    if let Some(kt) = self.key_tables.get_mut(&table) {
                        kt.bindings.clear();
                    }
                    return Ok(());
                }
                _ if arg.starts_with('-') => { args.next(); }
                _ => break,
            }
        }

        if let Some(key) = args.next()
            && let Some(kt) = self.key_tables.get_mut(&table) {
                kt.bindings.remove(*key);
            }

        Ok(())
    }

    fn handle_set_hook(&mut self, args: &[&str]) -> Result<()> {
        let mut args = args.iter().peekable();
        let mut _append = false;

        while let Some(arg) = args.peek() {
            match **arg {
                "-a" => { _append = true; args.next(); }
                "-g" | "-R" | "-u" => { args.next(); }
                _ if arg.starts_with('-') => { args.next(); }
                _ => break,
            }
        }

        let hook_name = match args.next() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        let command: String = args.copied().collect::<Vec<_>>().join(" ");

        self.hooks
            .entry(hook_name)
            .or_default()
            .push(command);

        Ok(())
    }

    /// Set an option value
    pub fn set_option(&mut self, option: &str, value: &str) -> Result<()> {
        match option {
            "default-shell" => self.default_shell = Some(value.to_string()),
            "default-command" => self.default_command = Some(value.to_string()),
            "default-terminal" => self.default_terminal = value.to_string(),
            "history-limit" => self.history_limit = value.parse().unwrap_or(2000),
            "escape-time" => self.escape_time = Duration::from_millis(value.parse().unwrap_or(500)),
            "repeat-time" => self.repeat_time = Duration::from_millis(value.parse().unwrap_or(500)),
            "display-time" => self.display_time = Duration::from_millis(value.parse().unwrap_or(750)),
            "display-panes-time" => self.display_panes_time = Duration::from_millis(value.parse().unwrap_or(1000)),
            "focus-events" => self.focus_events = parse_bool(value),
            "mouse" => self.mouse = parse_bool(value),
            "set-clipboard" => self.set_clipboard = ClipboardMode::parse(value),
            "base-index" => self.base_index = value.parse().unwrap_or(0),
            "pane-base-index" => self.pane_base_index = value.parse().unwrap_or(0),
            "renumber-windows" => self.renumber_windows = parse_bool(value),
            "automatic-rename" => self.automatic_rename = parse_bool(value),
            "automatic-rename-format" => self.automatic_rename_format = value.to_string(),
            "allow-rename" => self.allow_rename = parse_bool(value),
            "set-titles" => self.set_titles = parse_bool(value),
            "set-titles-string" => self.set_titles_string = value.to_string(),

            "prefix" => self.prefix = value.to_string(),
            "prefix2" => self.prefix2 = Some(value.to_string()),

            "status" => self.status = StatusPosition::parse(value),
            "status-interval" => self.status_interval = Duration::from_secs(value.parse().unwrap_or(15)),
            "status-justify" => self.status_justify = StatusJustify::parse(value),
            "status-left" => self.status_left = FormatSpec::new(value),
            "status-left-length" => self.status_left_length = value.parse().unwrap_or(10),
            "status-right" => self.status_right = FormatSpec::new(value),
            "status-right-length" => self.status_right_length = value.parse().unwrap_or(40),
            "status-style" => self.status_style = TmuxStyle::parse(value),
            "status-left-style" => self.status_left_style = TmuxStyle::parse(value),
            "status-right-style" => self.status_right_style = TmuxStyle::parse(value),

            "window-status-format" => self.window_status_format = value.to_string(),
            "window-status-current-format" => self.window_status_current_format = value.to_string(),
            "window-status-separator" => self.window_status_separator = value.to_string(),
            "window-status-style" => self.window_status_style = TmuxStyle::parse(value),
            "window-status-current-style" => self.window_status_current_style = TmuxStyle::parse(value),
            "window-status-activity-style" => self.window_status_activity_style = TmuxStyle::parse(value),
            "window-status-bell-style" => self.window_status_bell_style = TmuxStyle::parse(value),

            "pane-border-style" => self.pane_border_style = TmuxStyle::parse(value),
            "pane-active-border-style" => self.pane_active_border_style = TmuxStyle::parse(value),
            "pane-border-format" => self.pane_border_format = value.to_string(),
            "pane-border-status" => self.pane_border_status = PaneBorderStatus::parse(value),
            "pane-border-lines" => self.pane_border_lines = PaneBorderLines::parse(value),

            "message-style" => self.message_style = TmuxStyle::parse(value),
            "message-command-style" => self.message_command_style = TmuxStyle::parse(value),
            "mode-style" => self.mode_style = TmuxStyle::parse(value),

            "clock-mode-colour" | "clock-mode-color" => self.clock_mode_colour = TmuxColor::parse(value),
            "clock-mode-style" => self.clock_mode_style = ClockStyle::parse(value),

            "activity-action" => self.activity_action = ActivityAction::parse(value),
            "bell-action" => self.bell_action = BellAction::parse(value),
            "silence-action" => self.silence_action = ActivityAction::parse(value),
            "visual-activity" => self.visual_activity = VisualMode::parse(value),
            "visual-bell" => self.visual_bell = VisualMode::parse(value),
            "visual-silence" => self.visual_silence = VisualMode::parse(value),
            "monitor-activity" => self.monitor_activity = parse_bool(value),
            "monitor-bell" => self.monitor_bell = parse_bool(value),
            "monitor-silence" => self.monitor_silence = Duration::from_secs(value.parse().unwrap_or(0)),

            // User options (start with @)
            opt if opt.starts_with('@') => {
                // Check for TPM plugin
                if opt == "@plugin" {
                    self.tpm_plugins.push(value.to_string());
                } else {
                    self.user_options.insert(opt.to_string(), value.to_string());
                }
            }

            _ => {
                // Store unknown options for compatibility
                self.user_options.insert(option.to_string(), value.to_string());
            }
        }

        Ok(())
    }

    /// Get an option value
    pub fn get_option(&self, option: &str) -> Option<String> {
        match option {
            "default-shell" => self.default_shell.clone(),
            "default-terminal" => Some(self.default_terminal.clone()),
            "history-limit" => Some(self.history_limit.to_string()),
            "prefix" => Some(self.prefix.clone()),
            "prefix2" => self.prefix2.clone(),
            "mouse" => Some(if self.mouse { "on" } else { "off" }.to_string()),
            "base-index" => Some(self.base_index.to_string()),
            "status" => Some(self.status.to_string()),
            opt if opt.starts_with('@') => self.user_options.get(opt).cloned(),
            _ => self.user_options.get(option).cloned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusPosition {
    Top,
    Bottom,
    Off,
}

impl StatusPosition {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "top" => StatusPosition::Top,
            "bottom" => StatusPosition::Bottom,
            "off" | "0" | "false" | "no" => StatusPosition::Off,
            _ => StatusPosition::Bottom,
        }
    }
}

impl std::fmt::Display for StatusPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusPosition::Top => write!(f, "top"),
            StatusPosition::Bottom => write!(f, "bottom"),
            StatusPosition::Off => write!(f, "off"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusJustify {
    Left,
    Centre,
    Right,
    Absolute,
}

impl StatusJustify {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "left" => StatusJustify::Left,
            "centre" | "center" => StatusJustify::Centre,
            "right" => StatusJustify::Right,
            "absolute-centre" | "absolute-center" => StatusJustify::Absolute,
            _ => StatusJustify::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipboardMode {
    Off,
    External,
    On,
}

impl ClipboardMode {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "off" | "0" | "false" | "no" => ClipboardMode::Off,
            "external" => ClipboardMode::External,
            "on" | "1" | "true" | "yes" => ClipboardMode::On,
            _ => ClipboardMode::External,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneBorderStatus {
    Off,
    Top,
    Bottom,
}

impl PaneBorderStatus {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "top" => PaneBorderStatus::Top,
            "bottom" => PaneBorderStatus::Bottom,
            _ => PaneBorderStatus::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneBorderLines {
    Single,
    Double,
    Heavy,
    Simple,
    Number,
}

impl PaneBorderLines {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "double" => PaneBorderLines::Double,
            "heavy" => PaneBorderLines::Heavy,
            "simple" => PaneBorderLines::Simple,
            "number" => PaneBorderLines::Number,
            _ => PaneBorderLines::Single,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockStyle {
    H12,
    H24,
}

impl ClockStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "12" => ClockStyle::H12,
            _ => ClockStyle::H24,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityAction {
    Any,
    None,
    Current,
    Other,
}

impl ActivityAction {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "any" => ActivityAction::Any,
            "none" => ActivityAction::None,
            "current" => ActivityAction::Current,
            "other" => ActivityAction::Other,
            _ => ActivityAction::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BellAction {
    Any,
    None,
    Current,
    Other,
}

impl BellAction {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "any" => BellAction::Any,
            "none" => BellAction::None,
            "current" => BellAction::Current,
            "other" => BellAction::Other,
            _ => BellAction::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualMode {
    Off,
    On,
    Both,
}

impl VisualMode {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "on" | "1" | "true" | "yes" => VisualMode::On,
            "both" => VisualMode::Both,
            _ => VisualMode::Off,
        }
    }
}

fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "on" | "1" | "true" | "yes")
}

fn expand_path(path: &str) -> String {
    if path.starts_with('~')
        && let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    path.to_string()
}

/// Simple shell-like string splitting (handles quotes)
fn shell_split(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut start = 0;
    let chars: Vec<char> = s.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if !in_quote {
            if c == '"' || c == '\'' {
                in_quote = true;
                quote_char = c;
                start = i + 1;
            } else if c.is_whitespace() {
                if start < i {
                    let part = &s[start..i];
                    if !part.trim().is_empty() {
                        result.push(part.trim());
                    }
                }
                start = i + 1;
            }
        } else if c == quote_char {
            result.push(&s[start..i]);
            in_quote = false;
            start = i + 1;
        }

        i += 1;
    }

    if start < s.len() {
        let part = &s[start..];
        if !part.trim().is_empty() {
            result.push(part.trim());
        }
    }

    result
}

