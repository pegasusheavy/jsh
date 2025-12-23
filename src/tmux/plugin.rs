//! Tmux Plugin Manager (TPM) compatible plugin system

use crate::error::{JshError, Result};
use crate::shell::jsh_data_dir;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// TPM-compatible plugin manager
#[derive(Debug)]
pub struct TmuxPluginManager {
    pub plugins: HashMap<String, TmuxPlugin>,
    pub plugins_dir: PathBuf,
}

impl Default for TmuxPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TmuxPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir: tpm_plugins_dir(),
        }
    }

    /// Register a plugin from TPM format
    /// Format: "github_username/plugin_name" or "git@github.com:user/plugin"
    pub fn add(&mut self, spec: &str) -> Result<()> {
        let plugin = TmuxPlugin::from_spec(spec)?;
        self.plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    /// Install all registered plugins
    pub fn install_all(&self) -> Result<Vec<String>> {
        let mut installed = vec![];
        
        // Ensure plugins directory exists
        fs::create_dir_all(&self.plugins_dir)
            .map_err(|e| JshError::runtime(format!("Failed to create plugins dir: {}", e)))?;

        for plugin in self.plugins.values() {
            if plugin.is_installed(&self.plugins_dir) {
                continue;
            }

            match self.install_plugin(plugin) {
                Ok(_) => {
                    installed.push(plugin.name.clone());
                    println!("✓ Installed: {}", plugin.name);
                }
                Err(e) => {
                    eprintln!("✗ Failed to install {}: {}", plugin.name, e);
                }
            }
        }

        Ok(installed)
    }

    /// Install a single plugin
    fn install_plugin(&self, plugin: &TmuxPlugin) -> Result<()> {
        let install_dir = self.plugins_dir.join(&plugin.name);

        let status = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                &plugin.git_url,
                &install_dir.to_string_lossy(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| JshError::runtime(format!("git clone failed: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime(format!("Failed to clone {}", plugin.name)));
        }

        // Run plugin's install script if it exists
        let install_script = install_dir.join("install.sh");
        if install_script.exists() {
            let _ = Command::new("sh")
                .arg(&install_script)
                .current_dir(&install_dir)
                .status();
        }

        Ok(())
    }

    /// Update all plugins
    pub fn update_all(&self) -> Result<Vec<String>> {
        let mut updated = vec![];

        for plugin in self.plugins.values() {
            if !plugin.is_installed(&self.plugins_dir) {
                continue;
            }

            match self.update_plugin(plugin) {
                Ok(had_updates) => {
                    if had_updates {
                        updated.push(plugin.name.clone());
                        println!("✓ Updated: {}", plugin.name);
                    } else {
                        println!("• Up to date: {}", plugin.name);
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to update {}: {}", plugin.name, e);
                }
            }
        }

        Ok(updated)
    }

    /// Update a single plugin
    fn update_plugin(&self, plugin: &TmuxPlugin) -> Result<bool> {
        let install_dir = self.plugins_dir.join(&plugin.name);

        // Get current commit
        let old_commit = Command::new("git")
            .args(["-C", &install_dir.to_string_lossy(), "rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        // Pull updates
        let status = Command::new("git")
            .args(["-C", &install_dir.to_string_lossy(), "pull", "--rebase"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| JshError::runtime(format!("git pull failed: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime("git pull failed"));
        }

        // Check if commit changed
        let new_commit = Command::new("git")
            .args(["-C", &install_dir.to_string_lossy(), "rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let had_updates = old_commit != new_commit;

        // Run update script if it exists and there were updates
        if had_updates {
            let update_script = install_dir.join("update.sh");
            if update_script.exists() {
                let _ = Command::new("sh")
                    .arg(&update_script)
                    .current_dir(&install_dir)
                    .status();
            }
        }

        Ok(had_updates)
    }

    /// Clean unregistered plugins
    pub fn clean(&self) -> Result<Vec<String>> {
        let mut removed = vec![];

        if !self.plugins_dir.exists() {
            return Ok(removed);
        }

        if let Ok(entries) = fs::read_dir(&self.plugins_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip tpm itself
                if name == "tpm" {
                    continue;
                }

                if !self.plugins.contains_key(&name) {
                    let path = entry.path();
                    if let Err(e) = fs::remove_dir_all(&path) {
                        eprintln!("Failed to remove {}: {}", name, e);
                    } else {
                        removed.push(name.clone());
                        println!("✓ Removed: {}", name);
                    }
                }
            }
        }

        Ok(removed)
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Vec<String> {
        if !self.plugins_dir.exists() {
            return vec![];
        }

        fs::read_dir(&self.plugins_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    }

    /// Source a plugin's configuration
    pub fn source_plugin(&self, name: &str) -> Option<String> {
        let plugin_dir = self.plugins_dir.join(name);
        
        if !plugin_dir.exists() {
            return None;
        }

        // Look for main tmux file
        let candidates = vec![
            format!("{}.tmux", name),
            format!("{}.conf", name),
            "plugin.tmux".to_string(),
            "main.tmux".to_string(),
        ];

        for candidate in candidates {
            let file = plugin_dir.join(&candidate);
            if file.exists() {
                return Some(format!("run-shell '{}'", file.display()));
            }
        }

        None
    }

    /// Generate commands to load all plugins
    pub fn generate_load_commands(&self) -> Vec<String> {
        let mut commands = vec![];

        for plugin in self.plugins.values() {
            if let Some(cmd) = self.source_plugin(&plugin.name) {
                commands.push(cmd);
            }
        }

        commands
    }

    /// Initialize TPM (install TPM itself)
    pub fn init_tpm(&self) -> Result<()> {
        let tpm_dir = self.plugins_dir.join("tpm");
        
        if tpm_dir.exists() {
            return Ok(());
        }

        println!("Installing TPM (Tmux Plugin Manager)...");
        
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/tmux-plugins/tpm",
                &tpm_dir.to_string_lossy(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| JshError::runtime(format!("Failed to clone TPM: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime("Failed to install TPM"));
        }

        println!("TPM installed successfully.");
        Ok(())
    }
}

/// A tmux plugin
#[derive(Debug, Clone)]
pub struct TmuxPlugin {
    pub name: String,
    pub git_url: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
}

impl TmuxPlugin {
    /// Parse plugin specification
    pub fn from_spec(spec: &str) -> Result<Self> {
        let spec = spec.trim().trim_matches(|c| c == '"' || c == '\'');
        
        // Handle git@ URLs
        if spec.starts_with("git@") {
            let name = spec
                .rsplit('/')
                .next()
                .map(|s| s.trim_end_matches(".git"))
                .unwrap_or("plugin")
                .to_string();
            
            return Ok(Self {
                name,
                git_url: spec.to_string(),
                branch: None,
                tag: None,
            });
        }

        // Handle https URLs
        if spec.starts_with("https://") || spec.starts_with("http://") {
            let name = spec
                .rsplit('/')
                .next()
                .map(|s| s.trim_end_matches(".git"))
                .unwrap_or("plugin")
                .to_string();
            
            return Ok(Self {
                name,
                git_url: spec.to_string(),
                branch: None,
                tag: None,
            });
        }

        // Handle user/repo format (GitHub)
        if spec.contains('/') {
            let parts: Vec<&str> = spec.split('#').collect();
            let repo = parts[0];
            let branch = parts.get(1).map(|s| s.to_string());
            
            let name = repo
                .split('/')
                .last()
                .unwrap_or("plugin")
                .to_string();
            
            let git_url = format!("https://github.com/{}.git", repo);
            
            return Ok(Self {
                name,
                git_url,
                branch,
                tag: None,
            });
        }

        Err(JshError::runtime(format!("Invalid plugin spec: {}", spec)))
    }

    /// Check if plugin is installed
    pub fn is_installed(&self, plugins_dir: &PathBuf) -> bool {
        plugins_dir.join(&self.name).exists()
    }
}

/// Get TPM plugins directory
pub fn tpm_plugins_dir() -> PathBuf {
    // Check for TMUX_PLUGIN_MANAGER_PATH environment variable
    if let Ok(path) = std::env::var("TMUX_PLUGIN_MANAGER_PATH") {
        return PathBuf::from(path);
    }

    // Default to ~/.tmux/plugins for tmux compatibility
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let tmux_plugins = PathBuf::from(&home).join(".tmux/plugins");
    
    if tmux_plugins.exists() {
        return tmux_plugins;
    }

    // Fallback to XDG location
    jsh_data_dir().join("tmux/plugins")
}

/// Popular tmux plugins
pub fn popular_plugins() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tmux-plugins/tpm", "Tmux Plugin Manager"),
        ("tmux-plugins/tmux-sensible", "Basic tmux settings"),
        ("tmux-plugins/tmux-resurrect", "Persist sessions across restarts"),
        ("tmux-plugins/tmux-continuum", "Continuous saving/restoring"),
        ("tmux-plugins/tmux-yank", "Copy to system clipboard"),
        ("tmux-plugins/tmux-pain-control", "Better pane navigation"),
        ("tmux-plugins/tmux-copycat", "Enhanced search"),
        ("tmux-plugins/tmux-open", "Open highlighted selection"),
        ("tmux-plugins/tmux-prefix-highlight", "Highlight prefix key"),
        ("tmux-plugins/tmux-sidebar", "File tree sidebar"),
        ("tmux-plugins/tmux-urlview", "Open URLs from terminal"),
        ("tmux-plugins/tmux-fpp", "Facebook PathPicker integration"),
        ("dracula/tmux", "Dracula theme"),
        ("catppuccin/tmux", "Catppuccin theme"),
        ("nordtheme/tmux", "Nord theme"),
        ("egel/tmux-gruvbox", "Gruvbox theme"),
        ("wfxr/tmux-power", "Powerline-like theme"),
        ("christoomey/vim-tmux-navigator", "Vim-tmux navigation"),
        ("sainnhe/tmux-fzf", "FZF integration"),
        ("laktak/extrakto", "Copy text from terminal"),
        ("fcsonline/tmux-thumbs", "Copy hint mode"),
        ("schasse/tmux-jump", "Easy motion for tmux"),
        ("roosta/tmux-fuzzback", "Fuzzy search scrollback"),
        ("jaclu/tmux-menus", "Popup menus"),
        ("tmux-plugins/tmux-logging", "Log pane output"),
        ("tmux-plugins/tmux-cpu", "CPU usage indicator"),
        ("tmux-plugins/tmux-battery", "Battery indicator"),
        ("tmux-plugins/tmux-online-status", "Online status indicator"),
    ]
}

