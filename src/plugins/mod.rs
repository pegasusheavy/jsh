//! Plugin Manager for Franken Shell
//!
//! A zplug-inspired plugin manager supporting:
//! - Oh-My-Zsh plugins and themes
//! - Fish plugins
//! - Bash plugins
//! - Generic Git-based plugins
//!
//! Plugins are stored in $XDG_DATA_HOME/fsh/plugins/

use crate::error::{JshError, Result};
use crate::shell::{fsh_cache_dir, fsh_data_dir};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Plugin source type
#[derive(Debug, Clone, PartialEq)]
pub enum PluginSource {
    /// GitHub repository (user/repo)
    GitHub(String),
    /// Oh-My-Zsh plugin (plugins/name or themes/name)
    OhMyZsh(String),
    /// Oh-My-Zsh theme
    OhMyZshTheme(String),
    /// Fish plugin (from oh-my-fish or fisher)
    Fish(String),
    /// Local path
    Local(PathBuf),
    /// Direct Git URL
    Git(String),
}

/// Plugin loading type
#[derive(Debug, Clone, PartialEq)]
pub enum PluginLoad {
    /// Source the main file
    Source,
    /// Add to PATH/fpath
    Path,
    /// Load as theme
    Theme,
    /// Defer loading (lazy)
    Defer,
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub source: PluginSource,
    pub load: PluginLoad,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub depth: Option<u32>,
    pub frozen: bool,
    pub enabled: bool,
    /// Files to source (globs supported)
    pub use_files: Vec<String>,
    /// Hook to run after loading
    pub hook: Option<String>,
}

impl Plugin {
    pub fn new(name: &str, source: PluginSource) -> Self {
        Self {
            name: name.to_string(),
            source,
            load: PluginLoad::Source,
            branch: None,
            tag: None,
            depth: Some(1),
            frozen: false,
            enabled: true,
            use_files: vec![],
            hook: None,
        }
    }

    /// Get the installation directory for this plugin
    pub fn install_dir(&self) -> PathBuf {
        plugins_dir().join(&self.name)
    }

    /// Check if plugin is installed
    pub fn is_installed(&self) -> bool {
        self.install_dir().exists()
    }

    /// Get files to source
    pub fn get_source_files(&self) -> Vec<PathBuf> {
        let dir = self.install_dir();
        if !dir.exists() {
            return vec![];
        }

        // If specific files are requested, use those
        if !self.use_files.is_empty() {
            return self
                .use_files
                .iter()
                .flat_map(|pattern| {
                    glob::glob(&dir.join(pattern).to_string_lossy())
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(|r| r.ok())
                })
                .collect();
        }

        // Auto-detect source files based on plugin type
        let candidates = match &self.source {
            PluginSource::OhMyZsh(_) => vec![
                format!("{}.plugin.zsh", self.name),
                format!("{}.plugin.sh", self.name),
                "init.zsh".to_string(),
                "init.sh".to_string(),
            ],
            PluginSource::OhMyZshTheme(_) => vec![format!("{}.zsh-theme", self.name)],
            PluginSource::Fish(_) => vec![
                format!("conf.d/{}.fish", self.name),
                format!("functions/{}.fish", self.name),
                "init.fish".to_string(),
            ],
            _ => vec![
                format!("{}.plugin.zsh", self.name),
                format!("{}.plugin.sh", self.name),
                format!("{}.sh", self.name),
                format!("{}.bash", self.name),
                "init.zsh".to_string(),
                "init.sh".to_string(),
                "init.bash".to_string(),
                format!("{}.zsh", self.name),
            ],
        };

        for candidate in candidates {
            let path = dir.join(&candidate);
            if path.exists() {
                return vec![path];
            }
        }

        // Look for any .sh, .zsh, .bash files in the root
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut files: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .map(|ext| ext == "sh" || ext == "zsh" || ext == "bash")
                        .unwrap_or(false)
                })
                .collect();
            files.sort();
            if !files.is_empty() {
                return vec![files[0].clone()];
            }
        }

        vec![]
    }
}

/// Get the plugins directory
pub fn plugins_dir() -> PathBuf {
    fsh_data_dir().join("plugins")
}

/// Get the oh-my-zsh installation directory
pub fn omz_dir() -> PathBuf {
    fsh_data_dir().join("oh-my-zsh")
}

/// Get the plugin cache directory
pub fn plugin_cache_dir() -> PathBuf {
    fsh_cache_dir().join("plugins")
}

/// Plugin Manager
pub struct PluginManager {
    pub plugins: HashMap<String, Plugin>,
    pub loaded: Vec<String>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loaded: vec![],
        }
    }

    /// Register a plugin
    pub fn add(&mut self, plugin: Plugin) {
        self.plugins.insert(plugin.name.clone(), plugin);
    }

    /// Parse a plug declaration
    /// Format: plug "source" [options...]
    /// Examples:
    ///   plug "zsh-users/zsh-autosuggestions"
    ///   plug "oh-my-zsh:plugins/git"
    ///   plug "oh-my-zsh:themes/robbyrussell" as:theme
    ///   plug "local:/path/to/plugin"
    pub fn parse_plug(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Err(JshError::runtime("plug: missing plugin source"));
        }

        let source_str = &args[0];
        let (source, default_name) = parse_plugin_source(source_str)?;

        let mut plugin = Plugin::new(&default_name, source);

        // Parse options
        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            if let Some((key, value)) = arg.split_once(':') {
                match key {
                    "as" => {
                        plugin.load = match value {
                            "theme" => PluginLoad::Theme,
                            "command" | "path" => PluginLoad::Path,
                            "defer" | "lazy" => PluginLoad::Defer,
                            _ => PluginLoad::Source,
                        };
                    }
                    "branch" => plugin.branch = Some(value.to_string()),
                    "tag" => plugin.tag = Some(value.to_string()),
                    "depth" => plugin.depth = value.parse().ok(),
                    "use" => plugin.use_files.push(value.to_string()),
                    "hook" => plugin.hook = Some(value.to_string()),
                    "name" | "rename-to" => plugin.name = value.to_string(),
                    "if"
                        // Conditional loading - simple check
                        if value != "true" && value != "1" => {
                            plugin.enabled = false;
                        }
                    _ => {}
                }
            } else {
                match arg.as_str() {
                    "frozen" => plugin.frozen = true,
                    "defer" | "lazy" => plugin.load = PluginLoad::Defer,
                    _ => {}
                }
            }
            i += 1;
        }

        self.plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    /// Install all registered plugins
    pub fn install_all(&self) -> Result<Vec<String>> {
        let mut installed = vec![];

        // Create plugins directory
        let _ = fs::create_dir_all(plugins_dir());

        for plugin in self.plugins.values() {
            if !plugin.enabled {
                continue;
            }
            if plugin.is_installed() {
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
    fn install_plugin(&self, plugin: &Plugin) -> Result<()> {
        let install_dir = plugin.install_dir();

        // Check if oh-my-zsh is needed and install it first
        if matches!(
            plugin.source,
            PluginSource::OhMyZsh(_) | PluginSource::OhMyZshTheme(_)
        ) {
            self.ensure_oh_my_zsh()?;
        }

        match &plugin.source {
            PluginSource::GitHub(repo) => {
                let url = format!("https://github.com/{}.git", repo);
                self.git_clone(&url, &install_dir, plugin)?;
            }
            PluginSource::Git(url) => {
                self.git_clone(url, &install_dir, plugin)?;
            }
            PluginSource::OhMyZsh(path) => {
                // Symlink from oh-my-zsh
                let omz_path = omz_dir().join(path);
                if omz_path.exists() {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&omz_path, &install_dir)
                        .map_err(|e| JshError::runtime(format!("Failed to symlink: {}", e)))?;
                    #[cfg(windows)]
                    fs::copy(&omz_path, &install_dir)
                        .map_err(|e| JshError::runtime(format!("Failed to copy: {}", e)))?;
                } else {
                    return Err(JshError::runtime(format!(
                        "Oh-My-Zsh path not found: {}",
                        path
                    )));
                }
            }
            PluginSource::OhMyZshTheme(name) => {
                let theme_path = omz_dir().join("themes").join(format!("{}.zsh-theme", name));
                let _ = fs::create_dir_all(&install_dir);
                if theme_path.exists() {
                    fs::copy(&theme_path, install_dir.join(format!("{}.zsh-theme", name)))
                        .map_err(|e| JshError::runtime(format!("Failed to copy theme: {}", e)))?;
                } else {
                    return Err(JshError::runtime(format!(
                        "Oh-My-Zsh theme not found: {}",
                        name
                    )));
                }
            }
            PluginSource::Fish(repo) => {
                // Fish plugins are typically on GitHub
                let url = format!("https://github.com/{}.git", repo);
                self.git_clone(&url, &install_dir, plugin)?;
            }
            PluginSource::Local(path) => {
                // Symlink local directory
                #[cfg(unix)]
                std::os::unix::fs::symlink(path, &install_dir)
                    .map_err(|e| JshError::runtime(format!("Failed to symlink: {}", e)))?;
                #[cfg(windows)]
                {
                    let _ = fs::create_dir_all(&install_dir);
                    copy_dir_all(path, &install_dir)?;
                }
            }
        }

        Ok(())
    }

    /// Ensure Oh-My-Zsh is installed
    fn ensure_oh_my_zsh(&self) -> Result<()> {
        let omz = omz_dir();
        if omz.exists() {
            return Ok(());
        }

        println!("Installing Oh-My-Zsh...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "https://github.com/ohmyzsh/ohmyzsh.git",
                &omz.to_string_lossy(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| JshError::runtime(format!("Failed to clone oh-my-zsh: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime("Failed to install Oh-My-Zsh"));
        }

        Ok(())
    }

    /// Clone a git repository
    fn git_clone(&self, url: &str, dest: &Path, plugin: &Plugin) -> Result<()> {
        let mut args = vec!["clone"];

        if let Some(depth) = plugin.depth {
            args.push("--depth");
            // We need to own the string
            let depth_str = depth.to_string();
            args.push(Box::leak(depth_str.into_boxed_str()));
        }

        if let Some(ref branch) = plugin.branch {
            args.push("--branch");
            args.push(branch);
        }

        args.push(url);
        args.push(Box::leak(
            dest.to_string_lossy().into_owned().into_boxed_str(),
        ));

        let status = Command::new("git")
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| JshError::runtime(format!("git clone failed: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime(format!("Failed to clone {}", url)));
        }

        // Checkout specific tag if requested
        if let Some(ref tag) = plugin.tag {
            let status = Command::new("git")
                .args(["-C", &dest.to_string_lossy(), "checkout", tag])
                .status()
                .map_err(|e| JshError::runtime(format!("git checkout failed: {}", e)))?;

            if !status.success() {
                return Err(JshError::runtime(format!("Failed to checkout tag {}", tag)));
            }
        }

        Ok(())
    }

    /// Update all plugins
    pub fn update_all(&self) -> Result<Vec<String>> {
        let mut updated = vec![];

        for plugin in self.plugins.values() {
            if !plugin.enabled || plugin.frozen || !plugin.is_installed() {
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

        // Update oh-my-zsh if installed
        if omz_dir().exists() {
            println!("Updating Oh-My-Zsh...");
            let _ = Command::new("git")
                .args(["-C", &omz_dir().to_string_lossy(), "pull", "--rebase"])
                .status();
        }

        Ok(updated)
    }

    /// Update a single plugin
    fn update_plugin(&self, plugin: &Plugin) -> Result<bool> {
        let dir = plugin.install_dir();

        // Skip symlinks (local plugins, oh-my-zsh plugins)
        if dir.is_symlink() {
            return Ok(false);
        }

        // Check if it's a git repo
        if !dir.join(".git").exists() {
            return Ok(false);
        }

        // Get current commit
        let old_commit = Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        // Pull updates
        let status = Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "pull", "--rebase"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| JshError::runtime(format!("git pull failed: {}", e)))?;

        if !status.success() {
            return Err(JshError::runtime("git pull failed"));
        }

        // Check if commit changed
        let new_commit = Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        Ok(old_commit != new_commit)
    }

    /// Clean unregistered plugins
    pub fn clean(&self) -> Result<Vec<String>> {
        let mut removed = vec![];
        let plugins_path = plugins_dir();

        if !plugins_path.exists() {
            return Ok(removed);
        }

        if let Ok(entries) = fs::read_dir(&plugins_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !self.plugins.contains_key(&name) {
                    let path = entry.path();
                    if path.is_symlink() {
                        let _ = fs::remove_file(&path);
                    } else {
                        let _ = fs::remove_dir_all(&path);
                    }
                    removed.push(name);
                    println!("✓ Removed: {}", entry.file_name().to_string_lossy());
                }
            }
        }

        Ok(removed)
    }

    /// Get list of installed plugins
    pub fn list_installed(&self) -> Vec<String> {
        let plugins_path = plugins_dir();
        if !plugins_path.exists() {
            return vec![];
        }

        fs::read_dir(&plugins_path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    }

    /// Generate source commands for loading plugins
    pub fn generate_load_commands(&self) -> Vec<String> {
        let mut commands = vec![];

        for plugin in self.plugins.values() {
            if !plugin.enabled || !plugin.is_installed() {
                continue;
            }

            if plugin.load == PluginLoad::Defer {
                // Deferred plugins are loaded later
                continue;
            }

            for file in plugin.get_source_files() {
                commands.push(format!("source \"{}\"", file.display()));
            }

            if let Some(ref hook) = plugin.hook {
                commands.push(hook.clone());
            }
        }

        commands
    }
}

/// Parse plugin source string
fn parse_plugin_source(source: &str) -> Result<(PluginSource, String)> {
    if let Some(path) = source.strip_prefix("local:") {
        let name = PathBuf::from(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "local-plugin".to_string());
        return Ok((PluginSource::Local(PathBuf::from(path)), name));
    }

    if let Some(path) = source.strip_prefix("oh-my-zsh:plugins/") {
        return Ok((
            PluginSource::OhMyZsh(format!("plugins/{}", path)),
            path.to_string(),
        ));
    }

    if let Some(theme) = source.strip_prefix("oh-my-zsh:themes/") {
        return Ok((
            PluginSource::OhMyZshTheme(theme.to_string()),
            theme.to_string(),
        ));
    }

    if let Some(path) = source.strip_prefix("omz:plugins/") {
        return Ok((
            PluginSource::OhMyZsh(format!("plugins/{}", path)),
            path.to_string(),
        ));
    }

    if let Some(theme) = source.strip_prefix("omz:themes/") {
        return Ok((
            PluginSource::OhMyZshTheme(theme.to_string()),
            theme.to_string(),
        ));
    }

    if let Some(repo) = source.strip_prefix("fish:") {
        let name = repo.split('/').next_back().unwrap_or(repo).to_string();
        return Ok((PluginSource::Fish(repo.to_string()), name));
    }

    if source.starts_with("https://") || source.starts_with("git@") || source.starts_with("git://")
    {
        let name = source
            .split('/')
            .next_back()
            .unwrap_or("plugin")
            .trim_end_matches(".git")
            .to_string();
        return Ok((PluginSource::Git(source.to_string()), name));
    }

    // Default: GitHub user/repo
    if source.contains('/') {
        let name = source.split('/').next_back().unwrap_or(source).to_string();
        return Ok((PluginSource::GitHub(source.to_string()), name));
    }

    Err(JshError::runtime(format!(
        "Invalid plugin source: {}",
        source
    )))
}

#[cfg(windows)]
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
