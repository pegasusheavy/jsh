//! Tmux key bindings

use std::collections::HashMap;

/// A key binding
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: String,
    pub command: String,
    pub repeat: bool,
}

impl KeyBinding {
    pub fn new(key: &str, command: &str) -> Self {
        Self {
            key: key.to_string(),
            command: command.to_string(),
            repeat: false,
        }
    }

    pub fn with_repeat(mut self) -> Self {
        self.repeat = true;
        self
    }
}

/// Key table (prefix, root, copy-mode, etc.)
#[derive(Debug, Clone)]
pub struct KeyTable {
    pub name: String,
    pub bindings: HashMap<String, KeyBinding>,
}

impl KeyTable {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            bindings: HashMap::new(),
        }
    }

    /// Default prefix key table
    pub fn default_prefix() -> Self {
        let mut table = Self {
            name: "prefix".to_string(),
            bindings: HashMap::new(),
        };

        // Window management
        table.bind("c", "new-window");
        table.bind("&", "kill-window");
        table.bind(",", "command-prompt -I \"#W\" \"rename-window -- '%%'\"");
        table.bind("w", "choose-tree -Zw");
        table.bind("n", "next-window");
        table.bind("p", "previous-window");
        table.bind("l", "last-window");

        // Window selection by number
        for i in 0..=9 {
            table.bind(&i.to_string(), &format!("select-window -t :{}", i));
        }

        // Pane management
        table.bind("\"", "split-window");
        table.bind("%", "split-window -h");
        table.bind("x", "confirm-before -p \"kill-pane #P? (y/n)\" kill-pane");
        table.bind("z", "resize-pane -Z");
        table.bind("!", "break-pane");
        table.bind("q", "display-panes");
        table.bind("o", "select-pane -t :.+");
        table.bind(";", "last-pane");
        table.bind("{", "swap-pane -U");
        table.bind("}", "swap-pane -D");
        table.bind("Space", "next-layout");
        table.bind("M-1", "select-layout even-horizontal");
        table.bind("M-2", "select-layout even-vertical");
        table.bind("M-3", "select-layout main-horizontal");
        table.bind("M-4", "select-layout main-vertical");
        table.bind("M-5", "select-layout tiled");

        // Pane navigation
        table.bind("Up", "select-pane -U");
        table.bind("Down", "select-pane -D");
        table.bind("Left", "select-pane -L");
        table.bind("Right", "select-pane -R");

        // Pane resizing
        table.bind("C-Up", "resize-pane -U");
        table.bind("C-Down", "resize-pane -D");
        table.bind("C-Left", "resize-pane -L");
        table.bind("C-Right", "resize-pane -R");
        table.bind("M-Up", "resize-pane -U 5");
        table.bind("M-Down", "resize-pane -D 5");
        table.bind("M-Left", "resize-pane -L 5");
        table.bind("M-Right", "resize-pane -R 5");

        // Session management
        table.bind("$", "command-prompt -I \"#S\" \"rename-session -- '%%'\"");
        table.bind("d", "detach-client");
        table.bind("s", "choose-tree -Zs");
        table.bind("(", "switch-client -p");
        table.bind(")", "switch-client -n");
        table.bind("L", "switch-client -l");

        // Copy mode
        table.bind("[", "copy-mode");
        table.bind("]", "paste-buffer -p");
        table.bind("=", "choose-buffer -Z");
        table.bind("#", "list-buffers");
        table.bind("-", "delete-buffer");

        // Misc
        table.bind(":", "command-prompt");
        table.bind("?", "list-keys -N");
        table.bind("t", "clock-mode");
        table.bind("~", "show-messages");
        table.bind("i", "display-message");
        table.bind(
            "r",
            "source-file ~/.tmux.conf \\; display-message \"Reloaded tmux.conf\"",
        );
        table.bind("C-z", "suspend-client");
        table.bind("f", "command-prompt \"find-window -Z -- '%%'\"");

        table
    }

    /// Default root key table (no prefix)
    pub fn default_root() -> Self {
        let mut table = Self {
            name: "root".to_string(),
            bindings: HashMap::new(),
        };

        // Mouse bindings would go here
        table.bind("MouseDown1Pane", "select-pane -t = ; send-keys -M");
        table.bind(
            "MouseDrag1Pane",
            "if-shell -F \"#{mouse_any_flag}\" \"send-keys -M\" \"copy-mode -M\"",
        );
        table.bind("WheelUpPane", "if-shell -F \"#{mouse_any_flag}\" \"send-keys -M\" \"if -Ft= '#{alternate_on}' 'send-keys -M' 'copy-mode -e'\"");

        table
    }

    /// Default copy-mode key table (emacs style)
    pub fn default_copy_mode() -> Self {
        let mut table = Self {
            name: "copy-mode".to_string(),
            bindings: HashMap::new(),
        };

        // Navigation
        table.bind("Up", "send-keys -X cursor-up");
        table.bind("Down", "send-keys -X cursor-down");
        table.bind("Left", "send-keys -X cursor-left");
        table.bind("Right", "send-keys -X cursor-right");
        table.bind("C-a", "send-keys -X start-of-line");
        table.bind("C-e", "send-keys -X end-of-line");
        table.bind("M-f", "send-keys -X next-word");
        table.bind("M-b", "send-keys -X previous-word");
        table.bind("C-b", "send-keys -X cursor-left");
        table.bind("C-f", "send-keys -X cursor-right");
        table.bind("C-n", "send-keys -X cursor-down");
        table.bind("C-p", "send-keys -X cursor-up");

        // Page navigation
        table.bind("PageUp", "send-keys -X page-up");
        table.bind("PageDown", "send-keys -X page-down");
        table.bind("C-v", "send-keys -X page-down");
        table.bind("M-v", "send-keys -X page-up");
        table.bind("M-<", "send-keys -X history-top");
        table.bind("M->", "send-keys -X history-bottom");

        // Selection
        table.bind("C-Space", "send-keys -X begin-selection");
        table.bind("M-w", "send-keys -X copy-selection-and-cancel");
        table.bind("C-w", "send-keys -X copy-selection-and-cancel");
        table.bind("C-g", "send-keys -X clear-selection");

        // Search
        table.bind("C-r", "command-prompt -i -I \"#{pane_search_string}\" -p \"(search up)\" \"send-keys -X search-backward-incremental \\\"%%%\\\"\"");
        table.bind("C-s", "command-prompt -i -I \"#{pane_search_string}\" -p \"(search down)\" \"send-keys -X search-forward-incremental \\\"%%%\\\"\"");

        // Exit
        table.bind("q", "send-keys -X cancel");
        table.bind("Escape", "send-keys -X cancel");

        table
    }

    /// Default copy-mode-vi key table (vi style)
    pub fn default_copy_mode_vi() -> Self {
        let mut table = Self {
            name: "copy-mode-vi".to_string(),
            bindings: HashMap::new(),
        };

        // Navigation
        table.bind("h", "send-keys -X cursor-left");
        table.bind("j", "send-keys -X cursor-down");
        table.bind("k", "send-keys -X cursor-up");
        table.bind("l", "send-keys -X cursor-right");
        table.bind("w", "send-keys -X next-word");
        table.bind("b", "send-keys -X previous-word");
        table.bind("e", "send-keys -X next-word-end");
        table.bind("0", "send-keys -X start-of-line");
        table.bind("$", "send-keys -X end-of-line");
        table.bind("^", "send-keys -X back-to-indentation");
        table.bind("g", "send-keys -X history-top");
        table.bind("G", "send-keys -X history-bottom");
        table.bind("H", "send-keys -X top-line");
        table.bind("L", "send-keys -X bottom-line");
        table.bind("M", "send-keys -X middle-line");

        // Page navigation
        table.bind("C-b", "send-keys -X page-up");
        table.bind("C-f", "send-keys -X page-down");
        table.bind("C-d", "send-keys -X halfpage-down");
        table.bind("C-u", "send-keys -X halfpage-up");

        // Selection
        table.bind("v", "send-keys -X begin-selection");
        table.bind("V", "send-keys -X select-line");
        table.bind("C-v", "send-keys -X rectangle-toggle");
        table.bind("y", "send-keys -X copy-selection-and-cancel");
        table.bind("Enter", "send-keys -X copy-selection-and-cancel");
        table.bind("Escape", "send-keys -X clear-selection");

        // Search
        table.bind(
            "/",
            "command-prompt -p \"(search down)\" \"send-keys -X search-forward \\\"%%%\\\"\"",
        );
        table.bind(
            "?",
            "command-prompt -p \"(search up)\" \"send-keys -X search-backward \\\"%%%\\\"\"",
        );
        table.bind("n", "send-keys -X search-again");
        table.bind("N", "send-keys -X search-reverse");

        // Exit
        table.bind("q", "send-keys -X cancel");

        table
    }

    /// Bind a key
    pub fn bind(&mut self, key: &str, command: &str) {
        self.bindings
            .insert(key.to_string(), KeyBinding::new(key, command));
    }

    /// Bind a key with repeat
    pub fn bind_repeat(&mut self, key: &str, command: &str) {
        self.bindings
            .insert(key.to_string(), KeyBinding::new(key, command).with_repeat());
    }

    /// Unbind a key
    pub fn unbind(&mut self, key: &str) {
        self.bindings.remove(key);
    }

    /// Get binding for a key
    pub fn get(&self, key: &str) -> Option<&KeyBinding> {
        self.bindings.get(key)
    }

    /// List all bindings
    pub fn list(&self) -> Vec<&KeyBinding> {
        let mut bindings: Vec<_> = self.bindings.values().collect();
        bindings.sort_by(|a, b| a.key.cmp(&b.key));
        bindings
    }
}

impl Default for KeyTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse key string to key code
pub fn parse_key(key: &str) -> Option<KeyCode> {
    let key = key.trim();

    // Check for modifiers
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;

    let parts: Vec<&str> = key.split('-').collect();
    let base_key = if parts.len() > 1 {
        for part in &parts[..parts.len() - 1] {
            match *part {
                "C" | "Ctrl" | "Control" => ctrl = true,
                "M" | "Alt" | "Meta" => alt = true,
                "S" | "Shift" => shift = true,
                _ => {}
            }
        }
        parts.last().unwrap_or(&key)
    } else {
        key
    };

    let base = match base_key.to_lowercase().as_str() {
        "enter" | "return" | "cr" => BaseKey::Enter,
        "escape" | "esc" => BaseKey::Escape,
        "space" => BaseKey::Space,
        "tab" => BaseKey::Tab,
        "backspace" | "bs" => BaseKey::Backspace,
        "delete" | "del" => BaseKey::Delete,
        "insert" | "ins" => BaseKey::Insert,
        "home" => BaseKey::Home,
        "end" => BaseKey::End,
        "pageup" | "pgup" => BaseKey::PageUp,
        "pagedown" | "pgdn" => BaseKey::PageDown,
        "up" => BaseKey::Up,
        "down" => BaseKey::Down,
        "left" => BaseKey::Left,
        "right" => BaseKey::Right,
        "f1" => BaseKey::F(1),
        "f2" => BaseKey::F(2),
        "f3" => BaseKey::F(3),
        "f4" => BaseKey::F(4),
        "f5" => BaseKey::F(5),
        "f6" => BaseKey::F(6),
        "f7" => BaseKey::F(7),
        "f8" => BaseKey::F(8),
        "f9" => BaseKey::F(9),
        "f10" => BaseKey::F(10),
        "f11" => BaseKey::F(11),
        "f12" => BaseKey::F(12),
        s if s.len() == 1 => BaseKey::Char(s.chars().next().unwrap()),
        _ => return None,
    };

    Some(KeyCode {
        base,
        ctrl,
        alt,
        shift,
    })
}

/// Key code
#[derive(Debug, Clone, PartialEq)]
pub struct KeyCode {
    pub base: BaseKey,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// Base key
#[derive(Debug, Clone, PartialEq)]
pub enum BaseKey {
    Char(char),
    F(u8),
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![];
        if self.ctrl {
            parts.push("C");
        }
        if self.alt {
            parts.push("M");
        }
        if self.shift {
            parts.push("S");
        }

        let base = match &self.base {
            BaseKey::Char(c) => c.to_string(),
            BaseKey::F(n) => format!("F{}", n),
            BaseKey::Enter => "Enter".to_string(),
            BaseKey::Escape => "Escape".to_string(),
            BaseKey::Space => "Space".to_string(),
            BaseKey::Tab => "Tab".to_string(),
            BaseKey::Backspace => "BSpace".to_string(),
            BaseKey::Delete => "DC".to_string(),
            BaseKey::Insert => "IC".to_string(),
            BaseKey::Home => "Home".to_string(),
            BaseKey::End => "End".to_string(),
            BaseKey::PageUp => "PPage".to_string(),
            BaseKey::PageDown => "NPage".to_string(),
            BaseKey::Up => "Up".to_string(),
            BaseKey::Down => "Down".to_string(),
            BaseKey::Left => "Left".to_string(),
            BaseKey::Right => "Right".to_string(),
        };

        parts.push(&base);
        write!(f, "{}", parts.join("-"))
    }
}
