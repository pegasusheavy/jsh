//! Tmux pane management

use std::time::Instant;

/// A tmux pane
#[derive(Debug)]
pub struct Pane {
    pub id: usize,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub title: String,
    pub current_command: Option<String>,
    pub shell: String,
    pub pid: Option<u32>,
    pub created: Instant,
    pub activity: Instant,
    pub mode: PaneMode,
    pub options: PaneOptions,
    pub scroll_position: usize,
    pub history: Vec<String>,
    pub search_string: Option<String>,
    pub marked: bool,
    pub synchronized: bool,
}

impl Pane {
    pub fn new(id: usize, shell: Option<&str>) -> Self {
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        
        Self {
            id,
            x: 0,
            y: 0,
            width: 80,
            height: 24,
            title: String::new(),
            current_command: None,
            shell: shell.map(|s| s.to_string()).unwrap_or(default_shell),
            pid: None,
            created: Instant::now(),
            activity: Instant::now(),
            mode: PaneMode::Normal,
            options: PaneOptions::default(),
            scroll_position: 0,
            history: Vec::new(),
            search_string: None,
            marked: false,
            synchronized: false,
        }
    }

    /// Send keys to the pane
    pub fn send_keys(&mut self, keys: &str, _literal: bool) {
        // In a real implementation, this would send keys to the PTY
        self.activity = Instant::now();
        
        // For now, just update the title if it looks like a command
        if !keys.is_empty() && !keys.starts_with('\x1b') {
            self.current_command = Some(keys.to_string());
        }
    }

    /// Enter copy mode
    pub fn enter_copy_mode(&mut self, vi_mode: bool) {
        self.mode = if vi_mode {
            PaneMode::CopyVi
        } else {
            PaneMode::CopyEmacs
        };
        self.scroll_position = self.history.len().saturating_sub(1);
    }

    /// Exit copy mode
    pub fn exit_copy_mode(&mut self) {
        self.mode = PaneMode::Normal;
        self.scroll_position = 0;
    }

    /// Enter view mode (output-only)
    pub fn enter_view_mode(&mut self) {
        self.mode = PaneMode::View;
    }

    /// Check if pane is in copy/view mode
    pub fn is_in_mode(&self) -> bool {
        !matches!(self.mode, PaneMode::Normal)
    }

    /// Scroll up
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, lines: usize) {
        let max_pos = self.history.len().saturating_sub(1);
        self.scroll_position = (self.scroll_position + lines).min(max_pos);
    }

    /// Scroll to top
    pub fn scroll_top(&mut self) {
        self.scroll_position = 0;
    }

    /// Scroll to bottom
    pub fn scroll_bottom(&mut self) {
        self.scroll_position = self.history.len().saturating_sub(1);
    }

    /// Page up
    pub fn page_up(&mut self) {
        self.scroll_up(self.height as usize);
    }

    /// Page down
    pub fn page_down(&mut self) {
        self.scroll_down(self.height as usize);
    }

    /// Half page up
    pub fn half_page_up(&mut self) {
        self.scroll_up(self.height as usize / 2);
    }

    /// Half page down
    pub fn half_page_down(&mut self) {
        self.scroll_down(self.height as usize / 2);
    }

    /// Search forward
    pub fn search_forward(&mut self, pattern: &str) -> Option<usize> {
        self.search_string = Some(pattern.to_string());
        
        for (i, line) in self.history.iter().enumerate().skip(self.scroll_position + 1) {
            if line.contains(pattern) {
                self.scroll_position = i;
                return Some(i);
            }
        }
        None
    }

    /// Search backward
    pub fn search_backward(&mut self, pattern: &str) -> Option<usize> {
        self.search_string = Some(pattern.to_string());
        
        for i in (0..self.scroll_position).rev() {
            if let Some(line) = self.history.get(i) {
                if line.contains(pattern) {
                    self.scroll_position = i;
                    return Some(i);
                }
            }
        }
        None
    }

    /// Search next
    pub fn search_next(&mut self) -> Option<usize> {
        if let Some(ref pattern) = self.search_string.clone() {
            self.search_forward(pattern)
        } else {
            None
        }
    }

    /// Search previous
    pub fn search_previous(&mut self) -> Option<usize> {
        if let Some(ref pattern) = self.search_string.clone() {
            self.search_backward(pattern)
        } else {
            None
        }
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.scroll_position = 0;
    }

    /// Get visible lines
    pub fn visible_lines(&self) -> Vec<&str> {
        let start = self.scroll_position;
        let end = (start + self.height as usize).min(self.history.len());
        
        self.history[start..end]
            .iter()
            .map(|s| s.as_str())
            .collect()
    }

    /// Capture pane contents
    pub fn capture(&self, start: Option<i32>, end: Option<i32>) -> String {
        let total = self.history.len() as i32;
        
        let start_line = match start {
            Some(s) if s < 0 => (total + s).max(0) as usize,
            Some(s) => s as usize,
            None => 0,
        };
        
        let end_line = match end {
            Some(e) if e < 0 => (total + e).max(0) as usize,
            Some(e) => e as usize,
            None => self.history.len(),
        };

        self.history[start_line..end_line.min(self.history.len())].join("\n")
    }

    /// Respawn pane (restart shell)
    pub fn respawn(&mut self, command: Option<&str>) {
        self.current_command = command.map(|s| s.to_string());
        self.history.clear();
        self.scroll_position = 0;
        self.mode = PaneMode::Normal;
        self.activity = Instant::now();
    }

    /// Get pane mode string
    pub fn mode_string(&self) -> &str {
        match self.mode {
            PaneMode::Normal => "",
            PaneMode::CopyVi => "[copy-mode-vi]",
            PaneMode::CopyEmacs => "[copy-mode]",
            PaneMode::View => "[view]",
        }
    }

    /// Set pane title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Get display title
    pub fn display_title(&self) -> &str {
        if self.title.is_empty() {
            self.current_command.as_deref().unwrap_or(&self.shell)
        } else {
            &self.title
        }
    }
}

/// Pane mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneMode {
    Normal,
    CopyVi,
    CopyEmacs,
    View,
}

/// Pane-specific options
#[derive(Debug, Clone)]
pub struct PaneOptions {
    pub allow_rename: bool,
    pub alternate_screen: bool,
    pub remain_on_exit: RemainOnExit,
    pub synchronize_panes: bool,
}

impl Default for PaneOptions {
    fn default() -> Self {
        Self {
            allow_rename: true,
            alternate_screen: true,
            remain_on_exit: RemainOnExit::Off,
            synchronize_panes: false,
        }
    }
}

/// Behavior when pane command exits
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RemainOnExit {
    Off,
    On,
    Failed,
}

impl RemainOnExit {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "on" | "1" | "true" | "yes" => RemainOnExit::On,
            "failed" => RemainOnExit::Failed,
            _ => RemainOnExit::Off,
        }
    }
}

