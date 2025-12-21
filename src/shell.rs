//! Shell REPL and configuration for jsh

use crate::error::{JshError, Result};
use crate::interpreter::Interpreter;
use crate::theme::{GitInfo, PromptContext, ThemeManager, list_builtin_themes};
use chrono::Local;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;
use std::path::PathBuf;

/// Shell configuration
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub history_file: PathBuf,
    pub history_size: usize,
    pub prompt: String,
    pub continuation_prompt: String,
    pub vi_mode: bool,
    pub theme_name: String,
    pub is_login_shell: bool,
    pub is_interactive: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        let history_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".jsh_history");

        Self {
            history_file,
            history_size: 10000,
            prompt: String::new(), // Will be computed dynamically by theme
            continuation_prompt: "> ".to_string(),
            vi_mode: false,
            theme_name: "jsh".to_string(),
            is_login_shell: false,
            is_interactive: true,
        }
    }
}

/// Helper for rustyline
struct JshHelper;

impl Helper for JshHelper {}

impl Completer for JshHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find word start
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line[start..pos];

        let mut completions = Vec::new();

        if word.starts_with('$') {
            // Variable completion
            let var_prefix = &word[1..];
            for (key, _) in std::env::vars() {
                if key.starts_with(var_prefix) {
                    completions.push(Pair {
                        display: format!("${}", key),
                        replacement: format!("${}", key),
                    });
                }
            }
        } else if word.contains('/') || word.starts_with('.') || word.starts_with('~') {
            // Path completion
            let path = if word.starts_with('~') {
                let home = dirs::home_dir().unwrap_or_default();
                if word.len() > 1 {
                    home.join(&word[2..])
                } else {
                    home
                }
            } else {
                PathBuf::from(word)
            };

            let (dir, prefix) = if path.is_dir() {
                (path.clone(), String::new())
            } else {
                let parent = path.parent().unwrap_or(&path);
                let prefix = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                (parent.to_path_buf(), prefix)
            };

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) {
                        let mut full_path = if word.starts_with('~') {
                            format!(
                                "~/{}",
                                dir.strip_prefix(dirs::home_dir().unwrap_or_default())
                                    .unwrap_or(&dir)
                                    .join(&name)
                                    .display()
                            )
                        } else {
                            dir.join(&name).to_string_lossy().to_string()
                        };

                        if entry.path().is_dir() {
                            full_path.push('/');
                        }
                        completions.push(Pair {
                            display: full_path.clone(),
                            replacement: full_path,
                        });
                    }
                }
            }
        } else {
            // Command completion
            // First, check builtins
            const BUILTINS: &[&str] = &[
                "cd", "pwd", "echo", "printf", "test", "true", "false", "exit", "return", "source",
                "eval", "exec", "set", "unset", "export", "alias", "unalias", "type", "which",
                "command", "builtin", "help", "read", "jobs", "fg", "bg", "wait", "trap", "umask",
                "ulimit", "pushd", "popd", "dirs", "history", "hash", "getopts", "if", "then",
                "else", "elif", "fi", "case", "esac", "for", "in", "do", "done", "while", "until",
                "function", "match", "when", "loop", "fn", "let", "const", "try", "catch",
                "finally", "theme",
            ];

            for builtin in BUILTINS {
                if builtin.starts_with(word) {
                    completions.push(Pair {
                        display: builtin.to_string(),
                        replacement: builtin.to_string(),
                    });
                }
            }

            // Then check PATH
            if let Ok(path_var) = std::env::var("PATH") {
                for dir in path_var.split(':') {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.starts_with(word)
                                && !completions.iter().any(|c| c.replacement == name)
                            {
                                completions.push(Pair {
                                    display: name.clone(),
                                    replacement: name,
                                });
                            }
                        }
                    }
                }
            }
        }

        completions.sort_by(|a, b| a.display.cmp(&b.display));
        Ok((start, completions))
    }
}

impl Highlighter for JshHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Basic syntax highlighting
        let mut result = String::new();
        let mut in_string = false;
        let mut string_char = '"';
        let mut chars = line.chars().peekable();
        let mut word = String::new();

        while let Some(c) = chars.next() {
            if in_string {
                word.push(c);
                if c == string_char {
                    result.push_str(&word.green().to_string());
                    word.clear();
                    in_string = false;
                }
            } else {
                match c {
                    '"' | '\'' => {
                        if !word.is_empty() {
                            result.push_str(&highlight_word(&word));
                            word.clear();
                        }
                        in_string = true;
                        string_char = c;
                        word.push(c);
                    }
                    ' ' | '\t' => {
                        if !word.is_empty() {
                            result.push_str(&highlight_word(&word));
                            word.clear();
                        }
                        result.push(c);
                    }
                    '$' => {
                        if !word.is_empty() {
                            result.push_str(&highlight_word(&word));
                            word.clear();
                        }
                        word.push(c);
                        // Consume variable name
                        while let Some(&next) = chars.peek() {
                            if next.is_alphanumeric() || next == '_' || next == '{' || next == '}' {
                                word.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        result.push_str(&word.cyan().to_string());
                        word.clear();
                    }
                    '|' | '&' | ';' | '<' | '>' => {
                        if !word.is_empty() {
                            result.push_str(&highlight_word(&word));
                            word.clear();
                        }
                        result.push_str(&c.to_string().yellow().to_string());
                    }
                    '#' => {
                        if !word.is_empty() {
                            result.push_str(&highlight_word(&word));
                            word.clear();
                        }
                        // Rest is comment
                        let comment: String = std::iter::once(c).chain(chars.by_ref()).collect();
                        result.push_str(&comment.bright_black().to_string());
                    }
                    _ => {
                        word.push(c);
                    }
                }
            }
        }

        if !word.is_empty() {
            if in_string {
                result.push_str(&word.green().to_string());
            } else {
                result.push_str(&highlight_word(&word));
            }
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
}

impl Hinter for JshHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Validator for JshHelper {
    fn validate(&self, _ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

/// Highlight a word based on its type
fn highlight_word(word: &str) -> String {
    // Keywords
    const KEYWORDS: &[&str] = &[
        "if", "then", "else", "elif", "fi", "case", "esac", "for", "in", "do", "done", "while",
        "until", "select", "function", "return", "break", "continue", "local", "export",
        "readonly", "declare", "typeset", "unset", "shift", "time", "coproc", "match", "when",
        "loop", "fn", "let", "const", "try", "catch", "finally", "throw",
    ];

    // Builtins
    const BUILTINS: &[&str] = &[
        "cd", "pwd", "echo", "printf", "test", "[", "true", "false", ":", "exit", "return",
        "source", ".", "eval", "exec", "set", "unset", "export", "alias", "unalias", "type",
        "which", "command", "builtin", "help", "read", "jobs", "fg", "bg", "wait", "kill", "trap",
        "umask", "ulimit", "pushd", "popd", "dirs", "history", "hash", "getopts", "enable", "shopt",
    ];

    if KEYWORDS.contains(&word) {
        word.magenta().bold().to_string()
    } else if BUILTINS.contains(&word) {
        word.blue().to_string()
    } else if word.parse::<i64>().is_ok() || word.parse::<f64>().is_ok() {
        word.red().to_string()
    } else {
        word.to_string()
    }
}

/// The main shell
pub struct Shell {
    pub config: ShellConfig,
    pub interpreter: Interpreter,
    pub theme_manager: ThemeManager,
    editor: Editor<JshHelper, DefaultHistory>,
    history_num: usize,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Self::with_options(false, true)
    }

    /// Create a new shell with specific options
    pub fn with_options(is_login: bool, is_interactive: bool) -> Result<Self> {
        let mut config = ShellConfig::default();
        config.is_login_shell = is_login;
        config.is_interactive = is_interactive;

        let interpreter = Interpreter::new();
        let theme_manager = ThemeManager::new();

        let mut editor = Editor::new()?;
        editor.set_max_history_size(config.history_size)?;
        editor.set_helper(Some(JshHelper));

        // Load history
        let _ = editor.load_history(&config.history_file);

        let mut shell = Self {
            config,
            interpreter,
            theme_manager,
            editor,
            history_num: 0,
        };

        // Initialize environment and source profiles
        shell.initialize_environment();

        Ok(shell)
    }

    /// Initialize the shell environment
    fn initialize_environment(&mut self) {
        // Always source ~/.jshenv first (like zsh's .zshenv - for ALL shell types)
        self.source_jshenv();

        // Source login shell profiles if this is a login shell
        if self.config.is_login_shell {
            self.source_login_profiles();
        }

        // Source interactive shell profiles if this is interactive
        if self.config.is_interactive {
            self.source_interactive_profiles();
        }
    }

    /// Source ~/.jshenv - always sourced for ALL shell invocations
    /// This is the right place for environment variables, PATH modifications, etc.
    fn source_jshenv(&mut self) {
        // System-wide jshenv
        self.source_file_if_exists("/etc/jshenv");

        // User jshenv
        if let Some(home) = dirs::home_dir() {
            let jshenv = home.join(".jshenv");
            if jshenv.exists() {
                self.source_file_if_exists(&jshenv.to_string_lossy());
            }
        }

        // XDG location: ~/.config/jsh/env
        let xdg_config = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")));

        if let Some(xdg) = xdg_config {
            let xdg_env = xdg.join("jsh").join("env");
            if xdg_env.exists() {
                self.source_file_if_exists(&xdg_env.to_string_lossy());
            }
        }
    }

    /// Source login shell profile files
    /// Order: /etc/environment, /etc/profile, then first of ~/.bash_profile, ~/.bash_login, ~/.profile
    fn source_login_profiles(&mut self) {
        // Parse /etc/environment (simple KEY=VALUE format, not a shell script)
        self.parse_environment_file("/etc/environment");

        // System-wide profile
        self.source_file_if_exists("/etc/profile");

        // Source /etc/profile.d/*.sh files (common on Linux)
        self.source_profile_d("/etc/profile.d");

        // System-wide jsh login profile
        self.source_file_if_exists("/etc/jsh_profile");

        // User profile (first one that exists, in order)
        let home = dirs::home_dir();
        if let Some(ref home) = home {
            let profiles = [
                home.join(".jsh_profile"),    // jsh-specific first
                home.join(".bash_profile"),
                home.join(".bash_login"),
                home.join(".profile"),
            ];

            for profile in &profiles {
                if profile.exists() {
                    self.source_file_if_exists(&profile.to_string_lossy());
                    break; // Only source the first one found
                }
            }
        }
    }

    /// Parse /etc/environment style file (simple KEY=VALUE format)
    fn parse_environment_file(&mut self, path: &str) {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let path = PathBuf::from(path);
        if !path.exists() {
            return;
        }

        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(|l| l.ok()) {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Parse KEY=VALUE or KEY="VALUE"
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let mut value = value.trim();

                    // Remove surrounding quotes if present
                    if (value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\''))
                    {
                        value = &value[1..value.len() - 1];
                    }

                    // Set the environment variable
                    self.interpreter.env.insert(key.to_string(), value.to_string());
                    // SAFETY: Single-threaded shell environment setup
                    unsafe { std::env::set_var(key, value) };
                }
            }
        }
    }

    /// Source all *.sh files in a profile.d directory
    fn source_profile_d(&mut self, dir: &str) {
        let path = PathBuf::from(dir);
        if !path.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&path) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "sh")
                        .unwrap_or(false)
                })
                .collect();

            // Sort for consistent ordering
            files.sort_by_key(|e| e.path());

            for entry in files {
                self.source_file_if_exists(&entry.path().to_string_lossy());
            }
        }
    }

    /// Source interactive shell profile files
    fn source_interactive_profiles(&mut self) {
        // System-wide jshrc
        self.source_file_if_exists("/etc/jsh.jshrc");
        self.source_file_if_exists("/etc/jshrc");
        self.source_file_if_exists("/etc/bash.bashrc");

        // User rc file - check in priority order
        if let Some(home) = dirs::home_dir() {
            // 1. ~/.jshrc takes highest priority
            let jshrc = home.join(".jshrc");
            if jshrc.exists() {
                self.source_file_if_exists(&jshrc.to_string_lossy());
                return;
            }

            // 2. XDG location: ~/.config/jsh/jshrc
            let xdg_config = std::env::var("XDG_CONFIG_HOME")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));

            let xdg_jshrc = xdg_config.join("jsh").join("jshrc");
            if xdg_jshrc.exists() {
                self.source_file_if_exists(&xdg_jshrc.to_string_lossy());
                return;
            }

            // 3. Fallback to bashrc for compatibility
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                self.source_file_if_exists(&bashrc.to_string_lossy());
                return;
            }

            // 4. Fallback to zshrc (limited compatibility)
            let zshrc = home.join(".zshrc");
            if zshrc.exists() {
                self.source_file_if_exists(&zshrc.to_string_lossy());
            }
        }
    }

    /// Source a file if it exists, silently ignore errors
    fn source_file_if_exists(&mut self, path: &str) {
        let path = PathBuf::from(path);
        if path.exists() && path.is_file() {
            // Check if file is readable
            if std::fs::metadata(&path).map(|m| !m.permissions().readonly()).unwrap_or(false) ||
               std::fs::read_to_string(&path).is_ok() {
                let _ = self.interpreter.run_script(&path.to_string_lossy());
            }
        }
    }

    /// Source a file (public API)
    pub fn source_file(&mut self, path: &str) -> Result<()> {
        self.interpreter.run_script(path)?;
        Ok(())
    }

    /// Generate the prompt string using the theme system
    fn generate_prompt(&self) -> String {
        let home = self.interpreter.get_var("HOME").unwrap_or("").to_string();
        let user = self.interpreter.get_var("USER").unwrap_or("user").to_string();
        let host = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "localhost".to_string());

        let git_info = GitInfo::from_path(&self.interpreter.cwd);

        let ctx = PromptContext {
            cwd: &self.interpreter.cwd,
            home: &home,
            user: &user,
            host: &host,
            last_status: self.interpreter.last_status.code,
            git: git_info,
            shell_name: "jsh",
            history_num: self.history_num,
            time: Local::now(),
        };

        self.theme_manager.generate_prompt(&ctx)
    }

    /// Generate the right prompt
    #[allow(dead_code)]
    fn generate_rprompt(&self) -> Option<String> {
        let home = self.interpreter.get_var("HOME").unwrap_or("").to_string();
        let user = self.interpreter.get_var("USER").unwrap_or("user").to_string();
        let host = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "localhost".to_string());

        let git_info = GitInfo::from_path(&self.interpreter.cwd);

        let ctx = PromptContext {
            cwd: &self.interpreter.cwd,
            home: &home,
            user: &user,
            host: &host,
            last_status: self.interpreter.last_status.code,
            git: git_info,
            shell_name: "jsh",
            history_num: self.history_num,
            time: Local::now(),
        };

        self.theme_manager.generate_rprompt(&ctx)
    }

    /// Set the theme by name
    pub fn set_theme(&mut self, name: &str) -> Result<()> {
        self.theme_manager.load_theme(name)?;
        self.config.theme_name = name.to_string();
        Ok(())
    }

    /// Set a custom PROMPT
    pub fn set_prompt(&mut self, prompt: &str) {
        self.theme_manager.set_prompt(prompt);
    }

    /// Set a custom RPROMPT
    pub fn set_rprompt(&mut self, rprompt: Option<&str>) {
        self.theme_manager.set_rprompt(rprompt);
    }

    /// List available themes
    pub fn list_themes(&self) -> Vec<&'static str> {
        list_builtin_themes()
    }

    /// Run the interactive shell
    pub fn run(&mut self) -> Result<i32> {
        // Print welcome message
        println!(
            "{} {} - A ZSH/Bash-compatible shell",
            "jsh".cyan().bold(),
            env!("CARGO_PKG_VERSION")
        );
        println!("Type {} for available commands", "help".yellow());
        println!("Current theme: {} (use {} to list themes)\n",
            self.config.theme_name.cyan(),
            "theme list".yellow()
        );

        // Apply theme and prompt settings from environment
        self.apply_theme_from_env();

        loop {
            // Check for dynamic theme/prompt changes
            self.sync_theme_from_env();

            let prompt = self.generate_prompt();

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    self.history_num += 1;
                    let _ = self.editor.add_history_entry(line);

                    // Handle theme command internally for immediate effect
                    if line.starts_with("theme ") {
                        self.handle_theme_command(line);
                        continue;
                    }

                    match self.interpreter.execute_string(line) {
                        Ok(_) => {}
                        Err(JshError::Exit(code)) => {
                            self.save_history();
                            return Ok(code);
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "error".red().bold(), e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    self.save_history();
                    return Ok(0);
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    continue;
                }
            }
        }
    }

    /// Sync theme settings from environment variables
    /// Apply theme settings from environment (called once at startup)
    fn apply_theme_from_env(&mut self) {
        // Check for JSH_THEME environment variable
        if let Some(theme) = self.interpreter.get_var("JSH_THEME").map(|s| s.to_string()) {
            if !theme.is_empty() {
                let _ = self.set_theme(&theme);
            }
        }

        // Check for custom PROMPT
        if let Some(prompt) = self.interpreter.get_var("PROMPT").map(|s| s.to_string()) {
            if !prompt.is_empty() {
                self.set_prompt(&prompt);
            }
        }

        // Check for custom RPROMPT
        if let Some(rprompt) = self.interpreter.get_var("RPROMPT").map(|s| s.to_string()) {
            if !rprompt.is_empty() {
                self.set_rprompt(Some(&rprompt));
            }
        }
    }

    fn sync_theme_from_env(&mut self) {
        // Check if JSH_THEME changed
        let theme = self.interpreter.get_var("JSH_THEME").map(|s| s.to_string());
        if let Some(theme) = theme {
            if !theme.is_empty() && theme != self.config.theme_name {
                let _ = self.set_theme(&theme);
            }
        }

        // Dynamic PROMPT changes
        let prompt = self.interpreter.get_var("PROMPT").map(|s| s.to_string());
        let current_prompt = self.theme_manager.current_theme.prompt.clone();
        if let Some(prompt) = prompt {
            if !prompt.is_empty() && prompt != current_prompt {
                self.set_prompt(&prompt);
            }
        }
    }

    /// Handle theme command for immediate effect
    fn handle_theme_command(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts.get(1).map(|s| *s) {
            Some("list") | Some("ls") => {
                println!("{}", "Available themes:".cyan().bold());
                for theme in self.list_themes() {
                    let marker = if theme == self.config.theme_name { "* " } else { "  " };
                    println!("{}{}", marker.green(), theme);
                }
            }
            Some("set") | Some("use") => {
                if let Some(name) = parts.get(2) {
                    match self.set_theme(name) {
                        Ok(()) => println!("Theme set to: {}", name.green()),
                        Err(e) => eprintln!("{}: {}", "error".red().bold(), e),
                    }
                } else {
                    eprintln!("Usage: theme set <name>");
                }
            }
            Some("current") => {
                println!("Current theme: {}", self.config.theme_name.cyan());
            }
            Some("preview") => {
                if let Some(name) = parts.get(2) {
                    let old_theme = self.config.theme_name.clone();
                    if self.set_theme(name).is_ok() {
                        println!("{}", "Preview:".yellow().bold());
                        println!("{}", self.generate_prompt());
                        let _ = self.set_theme(&old_theme);
                    } else {
                        eprintln!("Theme '{}' not found", name);
                    }
                } else {
                    // Preview all themes
                    println!("{}", "Theme previews:".cyan().bold());
                    let old_theme = self.config.theme_name.clone();
                    for theme in self.list_themes() {
                        if self.set_theme(theme).is_ok() {
                            println!("\n{}:", theme.yellow().bold());
                            println!("{}", self.generate_prompt());
                        }
                    }
                    let _ = self.set_theme(&old_theme);
                }
            }
            Some(name) => {
                // Direct theme name (shorthand for theme set)
                match self.set_theme(name) {
                    Ok(()) => println!("Theme set to: {}", name.green()),
                    Err(e) => eprintln!("{}: {}", "error".red().bold(), e),
                }
            }
            None => {
                println!("{}", "Theme commands:".cyan().bold());
                println!("  theme list           - List available themes");
                println!("  theme set <name>     - Set theme");
                println!("  theme <name>         - Set theme (shorthand)");
                println!("  theme current        - Show current theme");
                println!("  theme preview        - Preview all themes");
                println!("  theme preview <name> - Preview a specific theme");
                println!();
                println!("You can also set themes via:");
                println!("  export JSH_THEME=<name>");
                println!("  export PROMPT='<prompt string>'");
                println!("  export RPROMPT='<right prompt>'");
            }
        }
    }

    /// Run a script file
    pub fn run_script(&mut self, path: &str, args: &[String]) -> Result<i32> {
        self.interpreter.positional_params = args.to_vec();

        match self.interpreter.run_script(path) {
            Ok(status) => Ok(status.code),
            Err(JshError::Exit(code)) => Ok(code),
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                Ok(1)
            }
        }
    }

    /// Run a command string
    pub fn run_command(&mut self, command: &str) -> Result<i32> {
        match self.interpreter.execute_string(command) {
            Ok(status) => Ok(status.code),
            Err(JshError::Exit(code)) => Ok(code),
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                Ok(1)
            }
        }
    }

    /// Save history
    fn save_history(&mut self) {
        let _ = self.editor.save_history(&self.config.history_file);
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new().expect("Failed to create shell")
    }
}
