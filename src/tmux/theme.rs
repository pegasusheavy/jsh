//! Tmux status bar theming

use super::{TmuxStyle, TmuxColor};
use std::collections::HashMap;

/// Status bar theme
#[derive(Debug, Clone)]
pub struct StatusBarTheme {
    pub name: String,

    // Status bar
    pub status_style: TmuxStyle,
    pub status_left: String,
    pub status_right: String,
    pub status_left_length: usize,
    pub status_right_length: usize,

    // Window status
    pub window_status_format: String,
    pub window_status_current_format: String,
    pub window_status_style: TmuxStyle,
    pub window_status_current_style: TmuxStyle,
    pub window_status_activity_style: TmuxStyle,
    pub window_status_separator: String,

    // Pane borders
    pub pane_border_style: TmuxStyle,
    pub pane_active_border_style: TmuxStyle,

    // Message/command line
    pub message_style: TmuxStyle,
    pub message_command_style: TmuxStyle,

    // Mode indicator
    pub mode_style: TmuxStyle,

    // Clock
    pub clock_mode_colour: TmuxColor,

    // Custom variables
    pub variables: HashMap<String, String>,
}

impl Default for StatusBarTheme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            status_style: TmuxStyle::parse("fg=black,bg=green"),
            status_left: "[#S] ".to_string(),
            status_right: " \"#H\" %H:%M %d-%b-%y".to_string(),
            status_left_length: 10,
            status_right_length: 40,
            window_status_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_current_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::parse("fg=black,bg=yellow"),
            window_status_activity_style: TmuxStyle::parse("reverse"),
            window_status_separator: " ".to_string(),
            pane_border_style: TmuxStyle::default(),
            pane_active_border_style: TmuxStyle::parse("fg=green"),
            message_style: TmuxStyle::parse("fg=black,bg=yellow"),
            message_command_style: TmuxStyle::parse("fg=yellow,bg=black"),
            mode_style: TmuxStyle::parse("fg=black,bg=yellow"),
            clock_mode_colour: TmuxColor::Blue,
            variables: HashMap::new(),
        }
    }
}

impl StatusBarTheme {
    /// Powerline theme (popular tmux theme)
    pub fn powerline() -> Self {
        Self {
            name: "powerline".to_string(),
            status_style: TmuxStyle::parse("fg=colour231,bg=colour234"),
            status_left: "#[fg=colour16,bg=colour254,bold] #S #[fg=colour254,bg=colour234,nobold]".to_string(),
            status_right: "#[fg=colour236,bg=colour234]#[fg=colour247,bg=colour236] %Y-%m-%d #[fg=colour252,bg=colour236]#[fg=colour235,bg=colour252] %H:%M ".to_string(),
            status_left_length: 20,
            status_right_length: 150,
            window_status_format: "  #I#F  #W  ".to_string(),
            window_status_current_format: "#[fg=colour234,bg=colour31]#[fg=colour117,bg=colour31] #I#F  #W #[fg=colour31,bg=colour234]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=colour250,bg=colour234"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=colour238"),
            pane_active_border_style: TmuxStyle::parse("fg=colour31"),
            message_style: TmuxStyle::parse("fg=colour231,bg=colour31"),
            message_command_style: TmuxStyle::parse("fg=colour231,bg=colour31"),
            mode_style: TmuxStyle::parse("fg=colour231,bg=colour31"),
            clock_mode_colour: TmuxColor::Color256(31),
            variables: HashMap::new(),
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            status_style: TmuxStyle::parse("fg=#f8f8f2,bg=#282a36"),
            status_left: "#[fg=#282a36,bg=#bd93f9,bold] #S #[fg=#bd93f9,bg=#282a36]".to_string(),
            status_right: "#[fg=#44475a,bg=#282a36]#[fg=#f8f8f2,bg=#44475a] %Y-%m-%d  %H:%M #[fg=#50fa7b,bg=#44475a]#[fg=#282a36,bg=#50fa7b,bold] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#f8f8f2,bg=#44475a] #I #W ".to_string(),
            window_status_current_format: "#[fg=#282a36,bg=#50fa7b,bold] #I #W #[fg=#50fa7b,bg=#282a36]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#ff79c6"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#44475a"),
            pane_active_border_style: TmuxStyle::parse("fg=#bd93f9"),
            message_style: TmuxStyle::parse("fg=#282a36,bg=#50fa7b"),
            message_command_style: TmuxStyle::parse("fg=#282a36,bg=#bd93f9"),
            mode_style: TmuxStyle::parse("fg=#282a36,bg=#ff79c6"),
            clock_mode_colour: TmuxColor::Rgb(189, 147, 249),
            variables: HashMap::new(),
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            status_style: TmuxStyle::parse("fg=#d8dee9,bg=#2e3440"),
            status_left: "#[fg=#2e3440,bg=#88c0d0] #S #[fg=#88c0d0,bg=#2e3440]".to_string(),
            status_right: "#[fg=#4c566a,bg=#2e3440]#[fg=#d8dee9,bg=#4c566a] %Y-%m-%d  %H:%M #[fg=#88c0d0,bg=#4c566a]#[fg=#2e3440,bg=#88c0d0] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#d8dee9,bg=#4c566a] #I #W ".to_string(),
            window_status_current_format: "#[fg=#2e3440,bg=#81a1c1,bold] #I #W #[fg=#81a1c1,bg=#2e3440]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#bf616a"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#4c566a"),
            pane_active_border_style: TmuxStyle::parse("fg=#88c0d0"),
            message_style: TmuxStyle::parse("fg=#2e3440,bg=#88c0d0"),
            message_command_style: TmuxStyle::parse("fg=#2e3440,bg=#81a1c1"),
            mode_style: TmuxStyle::parse("fg=#2e3440,bg=#81a1c1"),
            clock_mode_colour: TmuxColor::Rgb(136, 192, 208),
            variables: HashMap::new(),
        }
    }

    /// Gruvbox theme
    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".to_string(),
            status_style: TmuxStyle::parse("fg=#ebdbb2,bg=#282828"),
            status_left: "#[fg=#282828,bg=#fabd2f,bold] #S #[fg=#fabd2f,bg=#282828]".to_string(),
            status_right: "#[fg=#3c3836,bg=#282828]#[fg=#ebdbb2,bg=#3c3836] %Y-%m-%d  %H:%M #[fg=#b8bb26,bg=#3c3836]#[fg=#282828,bg=#b8bb26,bold] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#ebdbb2,bg=#3c3836] #I #W ".to_string(),
            window_status_current_format: "#[fg=#282828,bg=#83a598,bold] #I #W #[fg=#83a598,bg=#282828]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#fb4934"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#3c3836"),
            pane_active_border_style: TmuxStyle::parse("fg=#fabd2f"),
            message_style: TmuxStyle::parse("fg=#282828,bg=#b8bb26"),
            message_command_style: TmuxStyle::parse("fg=#282828,bg=#fabd2f"),
            mode_style: TmuxStyle::parse("fg=#282828,bg=#83a598"),
            clock_mode_colour: TmuxColor::Rgb(250, 189, 47),
            variables: HashMap::new(),
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha".to_string(),
            status_style: TmuxStyle::parse("fg=#cdd6f4,bg=#1e1e2e"),
            status_left: "#[fg=#1e1e2e,bg=#cba6f7,bold] #S #[fg=#cba6f7,bg=#1e1e2e]".to_string(),
            status_right: "#[fg=#313244,bg=#1e1e2e]#[fg=#cdd6f4,bg=#313244] %Y-%m-%d  %H:%M #[fg=#a6e3a1,bg=#313244]#[fg=#1e1e2e,bg=#a6e3a1,bold] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#cdd6f4,bg=#313244] #I #W ".to_string(),
            window_status_current_format: "#[fg=#1e1e2e,bg=#89b4fa,bold] #I #W #[fg=#89b4fa,bg=#1e1e2e]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#f38ba8"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#313244"),
            pane_active_border_style: TmuxStyle::parse("fg=#cba6f7"),
            message_style: TmuxStyle::parse("fg=#1e1e2e,bg=#a6e3a1"),
            message_command_style: TmuxStyle::parse("fg=#1e1e2e,bg=#cba6f7"),
            mode_style: TmuxStyle::parse("fg=#1e1e2e,bg=#89b4fa"),
            clock_mode_colour: TmuxColor::Rgb(203, 166, 247),
            variables: HashMap::new(),
        }
    }

    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night".to_string(),
            status_style: TmuxStyle::parse("fg=#a9b1d6,bg=#1a1b26"),
            status_left: "#[fg=#1a1b26,bg=#7aa2f7,bold] #S #[fg=#7aa2f7,bg=#1a1b26]".to_string(),
            status_right: "#[fg=#24283b,bg=#1a1b26]#[fg=#a9b1d6,bg=#24283b] %Y-%m-%d  %H:%M #[fg=#9ece6a,bg=#24283b]#[fg=#1a1b26,bg=#9ece6a,bold] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#a9b1d6,bg=#24283b] #I #W ".to_string(),
            window_status_current_format: "#[fg=#1a1b26,bg=#7aa2f7,bold] #I #W #[fg=#7aa2f7,bg=#1a1b26]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#f7768e"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#24283b"),
            pane_active_border_style: TmuxStyle::parse("fg=#7aa2f7"),
            message_style: TmuxStyle::parse("fg=#1a1b26,bg=#9ece6a"),
            message_command_style: TmuxStyle::parse("fg=#1a1b26,bg=#7aa2f7"),
            mode_style: TmuxStyle::parse("fg=#1a1b26,bg=#bb9af7"),
            clock_mode_colour: TmuxColor::Rgb(122, 162, 247),
            variables: HashMap::new(),
        }
    }

    /// One Dark theme
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark".to_string(),
            status_style: TmuxStyle::parse("fg=#abb2bf,bg=#282c34"),
            status_left: "#[fg=#282c34,bg=#61afef,bold] #S #[fg=#61afef,bg=#282c34]".to_string(),
            status_right: "#[fg=#3e4452,bg=#282c34]#[fg=#abb2bf,bg=#3e4452] %Y-%m-%d  %H:%M #[fg=#98c379,bg=#3e4452]#[fg=#282c34,bg=#98c379,bold] #H ".to_string(),
            status_left_length: 20,
            status_right_length: 60,
            window_status_format: "#[fg=#abb2bf,bg=#3e4452] #I #W ".to_string(),
            window_status_current_format: "#[fg=#282c34,bg=#61afef,bold] #I #W #[fg=#61afef,bg=#282c34]".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::default(),
            window_status_activity_style: TmuxStyle::parse("fg=#e06c75"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=#3e4452"),
            pane_active_border_style: TmuxStyle::parse("fg=#61afef"),
            message_style: TmuxStyle::parse("fg=#282c34,bg=#98c379"),
            message_command_style: TmuxStyle::parse("fg=#282c34,bg=#61afef"),
            mode_style: TmuxStyle::parse("fg=#282c34,bg=#c678dd"),
            clock_mode_colour: TmuxColor::Rgb(97, 175, 239),
            variables: HashMap::new(),
        }
    }

    /// Minimal theme
    pub fn minimal() -> Self {
        Self {
            name: "minimal".to_string(),
            status_style: TmuxStyle::parse("fg=white,bg=black"),
            status_left: "#S ".to_string(),
            status_right: " %H:%M".to_string(),
            status_left_length: 20,
            status_right_length: 20,
            window_status_format: " #I:#W ".to_string(),
            window_status_current_format: " [#I:#W] ".to_string(),
            window_status_style: TmuxStyle::default(),
            window_status_current_style: TmuxStyle::parse("bold"),
            window_status_activity_style: TmuxStyle::parse("underscore"),
            window_status_separator: "".to_string(),
            pane_border_style: TmuxStyle::parse("fg=white"),
            pane_active_border_style: TmuxStyle::parse("fg=cyan"),
            message_style: TmuxStyle::parse("fg=black,bg=white"),
            message_command_style: TmuxStyle::parse("fg=white,bg=black"),
            mode_style: TmuxStyle::parse("reverse"),
            clock_mode_colour: TmuxColor::Cyan,
            variables: HashMap::new(),
        }
    }

    /// Get theme by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default()),
            "powerline" => Some(Self::powerline()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "gruvbox" => Some(Self::gruvbox()),
            "catppuccin" | "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
            "tokyo-night" | "tokyonight" => Some(Self::tokyo_night()),
            "one-dark" | "onedark" => Some(Self::one_dark()),
            "minimal" => Some(Self::minimal()),
            _ => None,
        }
    }

    /// List available themes
    pub fn list_themes() -> Vec<&'static str> {
        vec![
            "default",
            "powerline",
            "dracula",
            "nord",
            "gruvbox",
            "catppuccin-mocha",
            "tokyo-night",
            "one-dark",
            "minimal",
        ]
    }
}

/// Format a status string with variable expansion
pub fn format_status(format: &str, vars: &HashMap<String, String>) -> String {
    let mut result = format.to_string();

    // Replace simple variables like #S, #I, #W, #H, etc.
    result = result.replace("#S", vars.get("session_name").map(|s| s.as_str()).unwrap_or(""));
    result = result.replace("#I", vars.get("window_index").map(|s| s.as_str()).unwrap_or(""));
    result = result.replace("#W", vars.get("window_name").map(|s| s.as_str()).unwrap_or(""));
    result = result.replace("#P", vars.get("pane_index").map(|s| s.as_str()).unwrap_or(""));
    result = result.replace("#T", vars.get("pane_title").map(|s| s.as_str()).unwrap_or(""));
    result = result.replace("#H", vars.get("host").map(|s| s.as_str()).unwrap_or(&hostname()));
    result = result.replace("#h", vars.get("host_short").map(|s| s.as_str()).unwrap_or(&hostname_short()));

    // Handle #{variable} syntax - simple implementation without regex
    while let Some(start) = result.find("#{") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            // Skip conditional syntax
            if !var_name.starts_with('?') {
                let replacement = vars.get(var_name).cloned().unwrap_or_default();
                result = format!("{}{}{}", &result[..start], replacement, &result[start + end + 1..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Handle conditionals #{?condition,true,false} - simple implementation
    while let Some(start) = result.find("#{?") {
        if let Some(end) = result[start..].find('}') {
            let inner = &result[start + 3..start + end];
            let parts: Vec<&str> = inner.splitn(3, ',').collect();
            if parts.len() == 3 {
                let condition = parts[0];
                let true_val = parts[1];
                let false_val = parts[2];

                let cond_result = vars.get(condition)
                    .map(|v| !v.is_empty() && v != "0" && v != "false")
                    .unwrap_or(false);

                let replacement = if cond_result { true_val } else { false_val };
                result = format!("{}{}{}", &result[..start], replacement, &result[start + end + 1..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string())
}

fn hostname_short() -> String {
    hostname().split('.').next().unwrap_or("localhost").to_string()
}

