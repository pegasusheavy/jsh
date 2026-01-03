//! Tmux session management

use super::window::Window;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A tmux session
#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub id: usize,
    pub windows: HashMap<usize, Arc<Mutex<Window>>>,
    pub current_window: usize,
    pub last_window: Option<usize>,
    pub created: Instant,
    pub last_attached: Option<Instant>,
    pub attached: bool,
    pub activity: Instant,
    pub alerts: SessionAlerts,
    pub options: SessionOptions,
    next_window_id: usize,
}

impl Session {
    pub fn new(name: &str) -> Self {
        static SESSION_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = SESSION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut session = Self {
            name: name.to_string(),
            id,
            windows: HashMap::new(),
            current_window: 0,
            last_window: None,
            created: Instant::now(),
            last_attached: None,
            attached: false,
            activity: Instant::now(),
            alerts: SessionAlerts::default(),
            options: SessionOptions::default(),
            next_window_id: 0,
        };

        // Create default window
        session.new_window(Some("0"));
        session
    }

    /// Create a new window
    pub fn new_window(&mut self, name: Option<&str>) -> Arc<Mutex<Window>> {
        let id = self.next_window_id;
        self.next_window_id += 1;

        let window_name = name.unwrap_or(&format!("{}", id)).to_string();
        let window = Arc::new(Mutex::new(Window::new(id, &window_name)));

        self.windows.insert(id, window.clone());

        if self.windows.len() == 1 {
            self.current_window = id;
        }

        window
    }

    /// Get current window
    pub fn current_window(&self) -> Option<Arc<Mutex<Window>>> {
        self.windows.get(&self.current_window).cloned()
    }

    /// Select a window by index
    pub fn select_window(&mut self, index: usize) -> bool {
        if self.windows.contains_key(&index) {
            self.last_window = Some(self.current_window);
            self.current_window = index;
            self.activity = Instant::now();
            true
        } else {
            false
        }
    }

    /// Select the last window
    pub fn select_last_window(&mut self) -> bool {
        if let Some(last) = self.last_window {
            self.select_window(last)
        } else {
            false
        }
    }

    /// Select next window
    pub fn next_window(&mut self) -> bool {
        let mut indices: Vec<usize> = self.windows.keys().cloned().collect();
        indices.sort();

        if let Some(pos) = indices.iter().position(|&i| i == self.current_window) {
            let next = if pos + 1 < indices.len() {
                indices[pos + 1]
            } else {
                indices[0]
            };
            self.select_window(next)
        } else {
            false
        }
    }

    /// Select previous window
    pub fn previous_window(&mut self) -> bool {
        let mut indices: Vec<usize> = self.windows.keys().cloned().collect();
        indices.sort();

        if let Some(pos) = indices.iter().position(|&i| i == self.current_window) {
            let prev = if pos > 0 {
                indices[pos - 1]
            } else {
                indices[indices.len() - 1]
            };
            self.select_window(prev)
        } else {
            false
        }
    }

    /// Kill a window
    pub fn kill_window(&mut self, index: usize) -> bool {
        if self.windows.len() <= 1 {
            return false; // Can't kill last window
        }

        if self.windows.remove(&index).is_some() {
            if self.current_window == index {
                // Select another window
                self.current_window = *self.windows.keys().next().unwrap_or(&0);
            }
            if self.last_window == Some(index) {
                self.last_window = None;
            }
            true
        } else {
            false
        }
    }

    /// Rename window
    pub fn rename_window(&mut self, index: usize, new_name: &str) -> bool {
        if let Some(window) = self.windows.get(&index) {
            let mut w = window.lock().unwrap();
            w.name = new_name.to_string();
            true
        } else {
            false
        }
    }

    /// List windows
    pub fn list_windows(&self) -> Vec<WindowInfo> {
        let mut windows: Vec<_> = self.windows.iter().collect();
        windows.sort_by_key(|(id, _)| *id);

        windows.into_iter().map(|(id, window)| {
            let w = window.lock().unwrap();
            WindowInfo {
                index: *id,
                name: w.name.clone(),
                active: *id == self.current_window,
                panes: w.panes.len(),
                layout: w.layout.clone(),
            }
        }).collect()
    }

    /// Get window count
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Attach to session
    pub fn attach(&mut self) {
        self.attached = true;
        self.last_attached = Some(Instant::now());
        self.activity = Instant::now();
    }

    /// Detach from session
    pub fn detach(&mut self) {
        self.attached = false;
    }

    /// Check if session is attached
    pub fn is_attached(&self) -> bool {
        self.attached
    }

    /// Renumber windows (close gaps)
    pub fn renumber_windows(&mut self, base_index: usize) {
        let mut new_windows = HashMap::new();
        let mut windows: Vec<_> = self.windows.drain().collect();
        windows.sort_by_key(|(id, _)| *id);

        let mut new_current = self.current_window;
        let mut new_last = self.last_window;

        for (i, (old_id, window)) in windows.into_iter().enumerate() {
            let new_id = base_index + i;

            if old_id == self.current_window {
                new_current = new_id;
            }
            if self.last_window == Some(old_id) {
                new_last = Some(new_id);
            }

            {
                let mut w = window.lock().unwrap();
                w.index = new_id;
            }

            new_windows.insert(new_id, window);
        }

        self.windows = new_windows;
        self.current_window = new_current;
        self.last_window = new_last;
    }
}

/// Window information for listing
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub index: usize,
    pub name: String,
    pub active: bool,
    pub panes: usize,
    pub layout: String,
}

impl std::fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let marker = if self.active { "*" } else { "-" };
        write!(f, "{}: {} {} [{} panes]", self.index, self.name, marker, self.panes)
    }
}

/// Session alerts
#[derive(Debug, Default, Clone)]
pub struct SessionAlerts {
    pub activity: bool,
    pub bell: bool,
    pub silence: bool,
}

/// Session-specific options
#[derive(Debug, Clone)]
pub struct SessionOptions {
    pub activity_action: String,
    pub assume_paste_time: u64,
    pub base_index: usize,
    pub bell_action: String,
    pub default_command: Option<String>,
    pub default_shell: Option<String>,
    pub default_size: Option<(u16, u16)>,
    pub destroy_unattached: bool,
    pub detach_on_destroy: String,
    pub display_panes_active_colour: String,
    pub display_panes_colour: String,
    pub display_panes_time: u64,
    pub display_time: u64,
    pub history_limit: usize,
    pub key_table: String,
    pub lock_after_time: u64,
    pub lock_command: String,
    pub message_limit: usize,
    pub mouse: bool,
    pub prefix: String,
    pub prefix2: Option<String>,
    pub renumber_windows: bool,
    pub repeat_time: u64,
    pub set_titles: bool,
    pub set_titles_string: String,
    pub silence_action: String,
    pub status: String,
    pub status_interval: u64,
    pub visual_activity: String,
    pub visual_bell: String,
    pub visual_silence: String,
    pub word_separators: String,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            activity_action: "other".to_string(),
            assume_paste_time: 1,
            base_index: 0,
            bell_action: "any".to_string(),
            default_command: None,
            default_shell: None,
            default_size: None,
            destroy_unattached: false,
            detach_on_destroy: "on".to_string(),
            display_panes_active_colour: "red".to_string(),
            display_panes_colour: "blue".to_string(),
            display_panes_time: 1000,
            display_time: 750,
            history_limit: 2000,
            key_table: "root".to_string(),
            lock_after_time: 0,
            lock_command: "lock -np".to_string(),
            message_limit: 1000,
            mouse: false,
            prefix: "C-b".to_string(),
            prefix2: None,
            renumber_windows: false,
            repeat_time: 500,
            set_titles: false,
            set_titles_string: "#S:#I:#W - \"#T\" #{session_alerts}".to_string(),
            silence_action: "other".to_string(),
            status: "on".to_string(),
            status_interval: 15,
            visual_activity: "off".to_string(),
            visual_bell: "off".to_string(),
            visual_silence: "off".to_string(),
            word_separators: " ".to_string(),
        }
    }
}

