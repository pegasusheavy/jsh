//! Tmux-compatible terminal multiplexer module for jsh
//!
//! Provides full tmux compatibility including:
//! - Session, window, and pane management
//! - .tmux.conf configuration parsing
//! - TPM (Tmux Plugin Manager) compatible plugin system
//! - Status bar theming
//! - Key bindings
//! - Copy mode

pub mod config;
pub mod session;
pub mod window;
pub mod pane;
pub mod keybind;
pub mod theme;
pub mod plugin;
pub mod commands;

use crate::error::{JshError, Result};
use crate::shell::jsh_config_dir;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use config::TmuxConfig;
pub use session::Session;
pub use window::Window;
pub use pane::Pane;
pub use keybind::{KeyBinding, KeyTable};
pub use theme::StatusBarTheme;
pub use plugin::TmuxPluginManager;

/// Global tmux server state
pub struct TmuxServer {
    /// All sessions
    pub sessions: HashMap<String, Arc<Mutex<Session>>>,
    /// Current session name
    pub current_session: Option<String>,
    /// Configuration
    pub config: TmuxConfig,
    /// Plugin manager
    pub plugins: TmuxPluginManager,
    /// Server socket path
    pub socket_path: PathBuf,
    /// Server started time
    pub start_time: std::time::Instant,
}

impl Default for TmuxServer {
    fn default() -> Self {
        Self::new()
    }
}

impl TmuxServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_session: None,
            config: TmuxConfig::default(),
            plugins: TmuxPluginManager::new(),
            socket_path: default_socket_path(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Load configuration from file
    pub fn load_config(&mut self, path: Option<&PathBuf>) -> Result<()> {
        let config_path = path.cloned().unwrap_or_else(default_config_path);
        if config_path.exists() {
            self.config = TmuxConfig::load(&config_path)?;
        }
        Ok(())
    }

    /// Create a new session
    pub fn new_session(&mut self, name: &str) -> Result<Arc<Mutex<Session>>> {
        if self.sessions.contains_key(name) {
            return Err(JshError::runtime(format!("Session '{}' already exists", name)));
        }

        let session = Arc::new(Mutex::new(Session::new(name)));
        self.sessions.insert(name.to_string(), session.clone());
        
        if self.current_session.is_none() {
            self.current_session = Some(name.to_string());
        }

        Ok(session)
    }

    /// Get a session by name
    pub fn get_session(&self, name: &str) -> Option<Arc<Mutex<Session>>> {
        self.sessions.get(name).cloned()
    }

    /// Get current session
    pub fn current(&self) -> Option<Arc<Mutex<Session>>> {
        self.current_session.as_ref().and_then(|n| self.sessions.get(n).cloned())
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Kill a session
    pub fn kill_session(&mut self, name: &str) -> Result<()> {
        if !self.sessions.contains_key(name) {
            return Err(JshError::runtime(format!("Session '{}' not found", name)));
        }

        self.sessions.remove(name);
        
        if self.current_session.as_ref() == Some(&name.to_string()) {
            self.current_session = self.sessions.keys().next().cloned();
        }

        Ok(())
    }

    /// Attach to a session
    pub fn attach_session(&mut self, name: &str) -> Result<()> {
        if !self.sessions.contains_key(name) {
            return Err(JshError::runtime(format!("Session '{}' not found", name)));
        }
        self.current_session = Some(name.to_string());
        Ok(())
    }

    /// Rename a session
    pub fn rename_session(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        if let Some(session) = self.sessions.remove(old_name) {
            {
                let mut s = session.lock().unwrap();
                s.name = new_name.to_string();
            }
            self.sessions.insert(new_name.to_string(), session);
            
            if self.current_session.as_ref() == Some(&old_name.to_string()) {
                self.current_session = Some(new_name.to_string());
            }
            Ok(())
        } else {
            Err(JshError::runtime(format!("Session '{}' not found", old_name)))
        }
    }

    /// Check if server has any sessions
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Get server uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Get default socket path
pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"));
    runtime_dir.join(format!("jsh-tmux-{}", std::process::id()))
}

/// Get default config path
pub fn default_config_path() -> PathBuf {
    // Check multiple locations in order
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    
    // XDG config first
    let xdg_config = jsh_config_dir().join("tmux.conf");
    if xdg_config.exists() {
        return xdg_config;
    }

    // Then ~/.tmux.conf for compatibility
    let home_config = PathBuf::from(&home).join(".tmux.conf");
    if home_config.exists() {
        return home_config;
    }

    // Then ~/.config/tmux/tmux.conf
    let config_tmux = PathBuf::from(&home).join(".config/tmux/tmux.conf");
    if config_tmux.exists() {
        return config_tmux;
    }

    // Default to XDG location
    xdg_config
}

/// Format specification for status line
#[derive(Debug, Clone)]
pub struct FormatSpec {
    pub format: String,
    pub style: Option<String>,
}

impl FormatSpec {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
            style: None,
        }
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }
}

/// Tmux color
#[derive(Debug, Clone, PartialEq)]
pub enum TmuxColor {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Color256(u8),
    Rgb(u8, u8, u8),
    Terminal,
}

impl TmuxColor {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "default" => TmuxColor::Default,
            "black" => TmuxColor::Black,
            "red" => TmuxColor::Red,
            "green" => TmuxColor::Green,
            "yellow" => TmuxColor::Yellow,
            "blue" => TmuxColor::Blue,
            "magenta" => TmuxColor::Magenta,
            "cyan" => TmuxColor::Cyan,
            "white" => TmuxColor::White,
            "terminal" => TmuxColor::Terminal,
            s if s.starts_with("colour") || s.starts_with("color") => {
                let num = s.trim_start_matches("colour").trim_start_matches("color");
                num.parse().map(TmuxColor::Color256).unwrap_or(TmuxColor::Default)
            }
            s if s.starts_with('#') && s.len() == 7 => {
                let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
                TmuxColor::Rgb(r, g, b)
            }
            _ => TmuxColor::Default,
        }
    }

    pub fn to_ansi(&self, fg: bool) -> String {
        let base = if fg { 30 } else { 40 };
        match self {
            TmuxColor::Default => if fg { "\x1b[39m".to_string() } else { "\x1b[49m".to_string() },
            TmuxColor::Black => format!("\x1b[{}m", base),
            TmuxColor::Red => format!("\x1b[{}m", base + 1),
            TmuxColor::Green => format!("\x1b[{}m", base + 2),
            TmuxColor::Yellow => format!("\x1b[{}m", base + 3),
            TmuxColor::Blue => format!("\x1b[{}m", base + 4),
            TmuxColor::Magenta => format!("\x1b[{}m", base + 5),
            TmuxColor::Cyan => format!("\x1b[{}m", base + 6),
            TmuxColor::White => format!("\x1b[{}m", base + 7),
            TmuxColor::Color256(n) => {
                if fg {
                    format!("\x1b[38;5;{}m", n)
                } else {
                    format!("\x1b[48;5;{}m", n)
                }
            }
            TmuxColor::Rgb(r, g, b) => {
                if fg {
                    format!("\x1b[38;2;{};{};{}m", r, g, b)
                } else {
                    format!("\x1b[48;2;{};{};{}m", r, g, b)
                }
            }
            TmuxColor::Terminal => String::new(),
        }
    }
}

/// Tmux style attributes
#[derive(Debug, Clone, Default)]
pub struct TmuxStyle {
    pub fg: Option<TmuxColor>,
    pub bg: Option<TmuxColor>,
    pub bold: bool,
    pub dim: bool,
    pub underscore: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub italics: bool,
    pub strikethrough: bool,
}

impl TmuxStyle {
    pub fn parse(s: &str) -> Self {
        let mut style = TmuxStyle::default();
        
        for part in s.split(',') {
            let part = part.trim();
            if let Some(color) = part.strip_prefix("fg=") {
                style.fg = Some(TmuxColor::parse(color));
            } else if let Some(color) = part.strip_prefix("bg=") {
                style.bg = Some(TmuxColor::parse(color));
            } else {
                match part {
                    "bold" => style.bold = true,
                    "dim" => style.dim = true,
                    "underscore" | "underline" => style.underscore = true,
                    "blink" => style.blink = true,
                    "reverse" => style.reverse = true,
                    "hidden" => style.hidden = true,
                    "italics" | "italic" => style.italics = true,
                    "strikethrough" => style.strikethrough = true,
                    "default" | "none" => {
                        style = TmuxStyle::default();
                    }
                    "nobold" => style.bold = false,
                    "nodim" => style.dim = false,
                    "nounderscore" | "nounderline" => style.underscore = false,
                    "noblink" => style.blink = false,
                    "noreverse" => style.reverse = false,
                    "nohidden" => style.hidden = false,
                    "noitalics" | "noitalic" => style.italics = false,
                    "nostrikethrough" => style.strikethrough = false,
                    _ => {}
                }
            }
        }

        style
    }

    pub fn to_ansi(&self) -> String {
        let mut codes = vec![];

        if let Some(ref fg) = self.fg {
            codes.push(fg.to_ansi(true));
        }
        if let Some(ref bg) = self.bg {
            codes.push(bg.to_ansi(false));
        }
        if self.bold { codes.push("\x1b[1m".to_string()); }
        if self.dim { codes.push("\x1b[2m".to_string()); }
        if self.italics { codes.push("\x1b[3m".to_string()); }
        if self.underscore { codes.push("\x1b[4m".to_string()); }
        if self.blink { codes.push("\x1b[5m".to_string()); }
        if self.reverse { codes.push("\x1b[7m".to_string()); }
        if self.hidden { codes.push("\x1b[8m".to_string()); }
        if self.strikethrough { codes.push("\x1b[9m".to_string()); }

        codes.join("")
    }

    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}

