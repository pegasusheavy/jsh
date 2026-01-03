//! Tmux window management

use super::pane::Pane;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A tmux window containing panes
#[derive(Debug)]
pub struct Window {
    pub index: usize,
    pub name: String,
    pub panes: HashMap<usize, Arc<Mutex<Pane>>>,
    pub active_pane: usize,
    pub last_pane: Option<usize>,
    pub layout: String,
    pub created: Instant,
    pub activity: Instant,
    pub flags: WindowFlags,
    pub options: WindowOptions,
    next_pane_id: usize,
}

impl Window {
    pub fn new(index: usize, name: &str) -> Self {
        let mut window = Self {
            index,
            name: name.to_string(),
            panes: HashMap::new(),
            active_pane: 0,
            last_pane: None,
            layout: "even-horizontal".to_string(),
            created: Instant::now(),
            activity: Instant::now(),
            flags: WindowFlags::default(),
            options: WindowOptions::default(),
            next_pane_id: 0,
        };

        // Create default pane
        window.new_pane(None);
        window
    }

    /// Create a new pane
    pub fn new_pane(&mut self, shell: Option<&str>) -> Arc<Mutex<Pane>> {
        let id = self.next_pane_id;
        self.next_pane_id += 1;

        let pane = Arc::new(Mutex::new(Pane::new(id, shell)));
        self.panes.insert(id, pane.clone());

        if self.panes.len() == 1 {
            self.active_pane = id;
        }

        pane
    }

    /// Get active pane
    pub fn active_pane(&self) -> Option<Arc<Mutex<Pane>>> {
        self.panes.get(&self.active_pane).cloned()
    }

    /// Select a pane
    pub fn select_pane(&mut self, id: usize) -> bool {
        if self.panes.contains_key(&id) {
            self.last_pane = Some(self.active_pane);
            self.active_pane = id;
            self.activity = Instant::now();
            true
        } else {
            false
        }
    }

    /// Select last pane
    pub fn select_last_pane(&mut self) -> bool {
        if let Some(last) = self.last_pane {
            self.select_pane(last)
        } else {
            false
        }
    }

    /// Select next pane
    pub fn next_pane(&mut self) -> bool {
        let mut ids: Vec<usize> = self.panes.keys().cloned().collect();
        ids.sort();

        if let Some(pos) = ids.iter().position(|&i| i == self.active_pane) {
            let next = if pos + 1 < ids.len() {
                ids[pos + 1]
            } else {
                ids[0]
            };
            self.select_pane(next)
        } else {
            false
        }
    }

    /// Select previous pane
    pub fn previous_pane(&mut self) -> bool {
        let mut ids: Vec<usize> = self.panes.keys().cloned().collect();
        ids.sort();

        if let Some(pos) = ids.iter().position(|&i| i == self.active_pane) {
            let prev = if pos > 0 {
                ids[pos - 1]
            } else {
                ids[ids.len() - 1]
            };
            self.select_pane(prev)
        } else {
            false
        }
    }

    /// Split pane horizontally (new pane below)
    pub fn split_horizontal(&mut self, shell: Option<&str>) -> Option<Arc<Mutex<Pane>>> {
        let pane = self.new_pane(shell);
        self.recalculate_layout();
        Some(pane)
    }

    /// Split pane vertically (new pane to the right)
    pub fn split_vertical(&mut self, shell: Option<&str>) -> Option<Arc<Mutex<Pane>>> {
        let pane = self.new_pane(shell);
        self.recalculate_layout();
        Some(pane)
    }

    /// Kill a pane
    pub fn kill_pane(&mut self, id: usize) -> bool {
        if self.panes.len() <= 1 {
            return false; // Can't kill last pane
        }

        if self.panes.remove(&id).is_some() {
            if self.active_pane == id {
                self.active_pane = *self.panes.keys().next().unwrap_or(&0);
            }
            if self.last_pane == Some(id) {
                self.last_pane = None;
            }
            self.recalculate_layout();
            true
        } else {
            false
        }
    }

    /// Swap panes
    pub fn swap_panes(&mut self, pane1: usize, pane2: usize) -> bool {
        if !self.panes.contains_key(&pane1) || !self.panes.contains_key(&pane2) {
            return false;
        }

        // Swap the panes in the HashMap
        let p1 = self.panes.remove(&pane1).unwrap();
        let p2 = self.panes.remove(&pane2).unwrap();

        // Update their IDs
        {
            let mut p1_lock = p1.lock().unwrap();
            let mut p2_lock = p2.lock().unwrap();
            std::mem::swap(&mut p1_lock.id, &mut p2_lock.id);
        }

        self.panes.insert(pane1, p2);
        self.panes.insert(pane2, p1);

        true
    }

    /// Break pane (move to new window) - returns the pane
    pub fn break_pane(&mut self, id: usize) -> Option<Arc<Mutex<Pane>>> {
        if self.panes.len() <= 1 {
            return None;
        }

        let pane = self.panes.remove(&id)?;

        if self.active_pane == id {
            self.active_pane = *self.panes.keys().next().unwrap_or(&0);
        }

        self.recalculate_layout();
        Some(pane)
    }

    /// Set layout
    pub fn set_layout(&mut self, layout: &str) {
        self.layout = layout.to_string();
        self.recalculate_layout();
    }

    /// Recalculate pane positions based on layout
    fn recalculate_layout(&mut self) {
        // Get terminal size
        let (width, height) = terminal_size();
        let pane_count = self.panes.len();

        if pane_count == 0 {
            return;
        }

        let mut ids: Vec<usize> = self.panes.keys().cloned().collect();
        ids.sort();

        match self.layout.as_str() {
            "even-horizontal" => {
                let pane_width = width / pane_count as u16;
                for (i, id) in ids.iter().enumerate() {
                    if let Some(pane) = self.panes.get(id) {
                        let mut p = pane.lock().unwrap();
                        p.x = (i as u16) * pane_width;
                        p.y = 0;
                        p.width = pane_width;
                        p.height = height;
                    }
                }
            }
            "even-vertical" => {
                let pane_height = height / pane_count as u16;
                for (i, id) in ids.iter().enumerate() {
                    if let Some(pane) = self.panes.get(id) {
                        let mut p = pane.lock().unwrap();
                        p.x = 0;
                        p.y = (i as u16) * pane_height;
                        p.width = width;
                        p.height = pane_height;
                    }
                }
            }
            "main-horizontal" => {
                if pane_count == 1 {
                    if let Some(pane) = self.panes.get(&ids[0]) {
                        let mut p = pane.lock().unwrap();
                        p.x = 0;
                        p.y = 0;
                        p.width = width;
                        p.height = height;
                    }
                } else {
                    let main_height = height / 2;
                    if let Some(pane) = self.panes.get(&ids[0]) {
                        let mut p = pane.lock().unwrap();
                        p.x = 0;
                        p.y = 0;
                        p.width = width;
                        p.height = main_height;
                    }

                    let other_width = width / (pane_count - 1) as u16;
                    for (i, id) in ids[1..].iter().enumerate() {
                        if let Some(pane) = self.panes.get(id) {
                            let mut p = pane.lock().unwrap();
                            p.x = (i as u16) * other_width;
                            p.y = main_height;
                            p.width = other_width;
                            p.height = height - main_height;
                        }
                    }
                }
            }
            "main-vertical" => {
                if pane_count == 1 {
                    if let Some(pane) = self.panes.get(&ids[0]) {
                        let mut p = pane.lock().unwrap();
                        p.x = 0;
                        p.y = 0;
                        p.width = width;
                        p.height = height;
                    }
                } else {
                    let main_width = width / 2;
                    if let Some(pane) = self.panes.get(&ids[0]) {
                        let mut p = pane.lock().unwrap();
                        p.x = 0;
                        p.y = 0;
                        p.width = main_width;
                        p.height = height;
                    }

                    let other_height = height / (pane_count - 1) as u16;
                    for (i, id) in ids[1..].iter().enumerate() {
                        if let Some(pane) = self.panes.get(id) {
                            let mut p = pane.lock().unwrap();
                            p.x = main_width;
                            p.y = (i as u16) * other_height;
                            p.width = width - main_width;
                            p.height = other_height;
                        }
                    }
                }
            }
            "tiled" => {
                let cols = (pane_count as f64).sqrt().ceil() as u16;
                let rows = (pane_count as u16 + cols - 1) / cols;
                let pane_width = width / cols;
                let pane_height = height / rows;

                for (i, id) in ids.iter().enumerate() {
                    if let Some(pane) = self.panes.get(id) {
                        let mut p = pane.lock().unwrap();
                        let col = (i as u16) % cols;
                        let row = (i as u16) / cols;
                        p.x = col * pane_width;
                        p.y = row * pane_height;
                        p.width = pane_width;
                        p.height = pane_height;
                    }
                }
            }
            _ => {
                // Default to even-horizontal
                let pane_width = width / pane_count as u16;
                for (i, id) in ids.iter().enumerate() {
                    if let Some(pane) = self.panes.get(id) {
                        let mut p = pane.lock().unwrap();
                        p.x = (i as u16) * pane_width;
                        p.y = 0;
                        p.width = pane_width;
                        p.height = height;
                    }
                }
            }
        }
    }

    /// Rotate panes clockwise
    pub fn rotate_panes(&mut self, clockwise: bool) {
        let mut ids: Vec<usize> = self.panes.keys().cloned().collect();
        ids.sort();

        if ids.len() < 2 {
            return;
        }

        if clockwise {
            ids.rotate_right(1);
        } else {
            ids.rotate_left(1);
        }

        // Reassign IDs
        let panes: Vec<_> = ids.iter().map(|id| self.panes.remove(id).unwrap()).collect();

        let new_ids: Vec<usize> = self.panes.keys().cloned().collect();
        for (i, pane) in panes.into_iter().enumerate() {
            let new_id = if i < new_ids.len() { new_ids[i] } else { i };
            {
                let mut p = pane.lock().unwrap();
                p.id = new_id;
            }
            self.panes.insert(new_id, pane);
        }

        self.recalculate_layout();
    }

    /// Get pane count
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    /// List panes
    pub fn list_panes(&self) -> Vec<PaneInfo> {
        let mut panes: Vec<_> = self.panes.iter().collect();
        panes.sort_by_key(|(id, _)| *id);

        panes.into_iter().map(|(id, pane)| {
            let p = pane.lock().unwrap();
            PaneInfo {
                id: *id,
                active: *id == self.active_pane,
                width: p.width,
                height: p.height,
                x: p.x,
                y: p.y,
                title: p.title.clone(),
                current_command: p.current_command.clone(),
            }
        }).collect()
    }

    /// Resize pane
    pub fn resize_pane(&mut self, id: usize, direction: ResizeDirection, amount: i16) {
        // In a real implementation, this would adjust adjacent panes
        if let Some(pane) = self.panes.get(&id) {
            let mut p = pane.lock().unwrap();
            match direction {
                ResizeDirection::Up => {
                    if p.height as i16 + amount > 0 {
                        p.height = (p.height as i16 + amount) as u16;
                    }
                }
                ResizeDirection::Down => {
                    if p.height as i16 - amount > 0 {
                        p.height = (p.height as i16 - amount) as u16;
                    }
                }
                ResizeDirection::Left => {
                    if p.width as i16 + amount > 0 {
                        p.width = (p.width as i16 + amount) as u16;
                    }
                }
                ResizeDirection::Right => {
                    if p.width as i16 - amount > 0 {
                        p.width = (p.width as i16 - amount) as u16;
                    }
                }
            }
        }
    }

    /// Next layout
    pub fn next_layout(&mut self) {
        let layouts = ["even-horizontal", "even-vertical", "main-horizontal", "main-vertical", "tiled"];
        let current = layouts.iter().position(|&l| l == self.layout).unwrap_or(0);
        let next = (current + 1) % layouts.len();
        self.set_layout(layouts[next]);
    }
}

/// Pane info for listing
#[derive(Debug, Clone)]
pub struct PaneInfo {
    pub id: usize,
    pub active: bool,
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub title: String,
    pub current_command: Option<String>,
}

impl std::fmt::Display for PaneInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let marker = if self.active { "*" } else { " " };
        write!(f, "{}:{} [{}x{}] [{},{}]",
            self.id, marker, self.width, self.height, self.x, self.y)
    }
}

/// Window flags
#[derive(Debug, Default, Clone)]
pub struct WindowFlags {
    pub activity: bool,
    pub bell: bool,
    pub silence: bool,
    pub zoomed: bool,
    pub last: bool,
    pub linked: bool,
    pub marked: bool,
}

impl WindowFlags {
    pub fn to_string(&self) -> String {
        let mut flags = String::new();
        if self.activity { flags.push('!'); }
        if self.bell { flags.push('#'); }
        if self.silence { flags.push('~'); }
        if self.zoomed { flags.push('Z'); }
        if self.last { flags.push('-'); }
        if self.linked { flags.push('L'); }
        if self.marked { flags.push('M'); }
        flags
    }
}

/// Window-specific options
#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub aggressive_resize: bool,
    pub automatic_rename: bool,
    pub automatic_rename_format: String,
    pub clock_mode_colour: String,
    pub clock_mode_style: String,
    pub main_pane_height: Option<u16>,
    pub main_pane_width: Option<u16>,
    pub mode_keys: String,
    pub monitor_activity: bool,
    pub monitor_bell: bool,
    pub monitor_silence: u64,
    pub other_pane_height: Option<u16>,
    pub other_pane_width: Option<u16>,
    pub pane_base_index: usize,
    pub synchronize_panes: bool,
    pub window_status_current_format: String,
    pub window_status_format: String,
    pub window_status_separator: String,
    pub wrap_search: bool,
    pub xterm_keys: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            aggressive_resize: false,
            automatic_rename: true,
            automatic_rename_format: "#{?pane_in_mode,[tmux],#{pane_current_command}}".to_string(),
            clock_mode_colour: "blue".to_string(),
            clock_mode_style: "24".to_string(),
            main_pane_height: None,
            main_pane_width: None,
            mode_keys: "emacs".to_string(),
            monitor_activity: false,
            monitor_bell: true,
            monitor_silence: 0,
            other_pane_height: None,
            other_pane_width: None,
            pane_base_index: 0,
            synchronize_panes: false,
            window_status_current_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_format: "#I:#W#{?window_flags,#{window_flags}, }".to_string(),
            window_status_separator: " ".to_string(),
            wrap_search: true,
            xterm_keys: true,
        }
    }
}

/// Resize direction
#[derive(Debug, Clone, Copy)]
pub enum ResizeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Get terminal size
fn terminal_size() -> (u16, u16) {
    #[cfg(unix)]
    {
        use nix::libc;
        use std::mem::MaybeUninit;

        unsafe {
            let mut size: MaybeUninit<libc::winsize> = MaybeUninit::uninit();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, size.as_mut_ptr()) == 0 {
                let size = size.assume_init();
                return (size.ws_col, size.ws_row);
            }
        }
    }

    // Default fallback
    (80, 24)
}

