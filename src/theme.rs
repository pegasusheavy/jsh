//! Oh-my-zsh compatible theme system for jsh

use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Theme configuration
#[derive(Debug, Clone, Default)]
pub struct Theme {
    /// The prompt string (PROMPT or PS1)
    pub prompt: String,
    /// Right-side prompt (RPROMPT)
    pub rprompt: Option<String>,
    /// Prompt for continuation lines (PS2)
    pub prompt2: String,
    /// Custom colors
    pub colors: HashMap<String, String>,
}

/// Git repository information
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub is_dirty: bool,
    pub has_staged: bool,
    pub has_untracked: bool,
    pub ahead: usize,
    pub behind: usize,
    pub is_repo: bool,
    pub remote: Option<String>,
    pub commit_short: Option<String>,
}

impl GitInfo {
    /// Get git info for the current directory
    pub fn from_path(path: &Path) -> Self {
        let mut info = GitInfo::default();

        // Check if we're in a git repo
        let branch_output = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(path)
            .output();

        if branch_output.is_err() || !branch_output.unwrap().status.success() {
            return info;
        }

        info.is_repo = true;

        // Get current branch
        if let Ok(output) = std::process::Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                info.branch = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            } else {
                // Detached HEAD - get short commit hash
                if let Ok(output) = std::process::Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .current_dir(path)
                    .output()
                {
                    if output.status.success() {
                        info.commit_short = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                    }
                }
            }
        }

        // Check for uncommitted changes (dirty)
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout);
                for line in status.lines() {
                    if line.starts_with("??") {
                        info.has_untracked = true;
                    } else if line.starts_with(' ') {
                        info.is_dirty = true;
                    } else if !line.is_empty() {
                        info.has_staged = true;
                    }
                }
            }
        }

        // Check ahead/behind
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                let counts = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = counts.trim().split('\t').collect();
                if parts.len() == 2 {
                    info.ahead = parts[0].parse().unwrap_or(0);
                    info.behind = parts[1].parse().unwrap_or(0);
                }
            }
        }

        // Get remote
        if let Ok(output) = std::process::Command::new("git")
            .args(["remote"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                let remote = String::from_utf8_lossy(&output.stdout);
                if let Some(first_remote) = remote.lines().next() {
                    info.remote = Some(first_remote.to_string());
                }
            }
        }

        info
    }
}

/// Prompt context for expansion
pub struct PromptContext<'a> {
    pub cwd: &'a Path,
    pub home: &'a str,
    pub user: &'a str,
    pub host: &'a str,
    pub last_status: i32,
    pub git: GitInfo,
    pub shell_name: &'a str,
    pub history_num: usize,
    pub time: chrono::DateTime<chrono::Local>,
}

impl<'a> PromptContext<'a> {
    /// Get the current directory with ~ substitution
    pub fn pwd_tilde(&self) -> String {
        let cwd = self.cwd.to_string_lossy();
        if cwd.starts_with(self.home) {
            cwd.replacen(self.home, "~", 1)
        } else {
            cwd.to_string()
        }
    }

    /// Get just the current directory name
    pub fn pwd_short(&self) -> String {
        self.cwd
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string())
    }
}

/// Expand ZSH-style prompt escape sequences
pub fn expand_prompt(prompt: &str, ctx: &PromptContext) -> String {
    let mut result = String::new();
    let mut chars = prompt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                chars.next();
                match next {
                    // Colors
                    'F' | 'f' => {
                        // %F{color} - foreground color
                        if chars.peek() == Some(&'{') {
                            chars.next();
                            let mut color = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '}' {
                                    chars.next();
                                    break;
                                }
                                color.push(c);
                                chars.next();
                            }
                            result.push_str(&start_color(&color, true));
                        } else {
                            result.push_str("\x1b[0m"); // Reset
                        }
                    }
                    'K' | 'k' => {
                        // %K{color} - background color
                        if chars.peek() == Some(&'{') {
                            chars.next();
                            let mut color = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '}' {
                                    chars.next();
                                    break;
                                }
                                color.push(c);
                                chars.next();
                            }
                            result.push_str(&start_color(&color, false));
                        } else {
                            result.push_str("\x1b[0m");
                        }
                    }
                    'B' => result.push_str("\x1b[1m"),  // Bold on
                    'b' => result.push_str("\x1b[22m"), // Bold off
                    'U' => result.push_str("\x1b[4m"),  // Underline on
                    'u' => result.push_str("\x1b[24m"), // Underline off
                    'S' => result.push_str("\x1b[9m"),  // Strikethrough on
                    's' => result.push_str("\x1b[29m"), // Strikethrough off

                    // Path
                    '~' => result.push_str(&ctx.pwd_tilde()),
                    '/' => result.push_str(&ctx.cwd.to_string_lossy()),
                    'c' | '.' | 'C' => {
                        // %c, %., %C - trailing component of cwd
                        // Check for number modifier
                        let mut num = 1usize;
                        while let Some(&digit) = chars.peek() {
                            if digit.is_ascii_digit() {
                                num = num * 10 + (digit as usize - '0' as usize);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let path = ctx.pwd_tilde();
                        let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                        let start = components.len().saturating_sub(num);
                        result.push_str(&components[start..].join("/"));
                    }

                    // User/Host
                    'n' => result.push_str(ctx.user),
                    'm' => {
                        // Short hostname (up to first .)
                        if let Some(dot_pos) = ctx.host.find('.') {
                            result.push_str(&ctx.host[..dot_pos]);
                        } else {
                            result.push_str(ctx.host);
                        }
                    }
                    'M' => result.push_str(ctx.host), // Full hostname

                    // Time
                    'T' => result.push_str(&ctx.time.format("%H:%M").to_string()),
                    't' | '@' => result.push_str(&ctx.time.format("%I:%M %p").to_string()),
                    '*' => result.push_str(&ctx.time.format("%H:%M:%S").to_string()),
                    'D' => {
                        // Date with optional format
                        if chars.peek() == Some(&'{') {
                            chars.next();
                            let mut fmt = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '}' {
                                    chars.next();
                                    break;
                                }
                                fmt.push(c);
                                chars.next();
                            }
                            result.push_str(&ctx.time.format(&fmt).to_string());
                        } else {
                            result.push_str(&ctx.time.format("%y-%m-%d").to_string());
                        }
                    }
                    'w' => result.push_str(&ctx.time.format("%a %b %d").to_string()),
                    'W' => result.push_str(&ctx.time.format("%m/%d/%y").to_string()),

                    // Shell state
                    '#' => {
                        // # if root, % otherwise
                        if ctx.user == "root" {
                            result.push('#');
                        } else {
                            result.push('%');
                        }
                    }
                    '?' => result.push_str(&ctx.last_status.to_string()),
                    'h' | '!' => result.push_str(&ctx.history_num.to_string()),
                    'l' => {
                        // TTY name
                        result.push_str("pts/0"); // Simplified
                    }
                    'N' => result.push_str(ctx.shell_name),

                    // Conditionals
                    '(' => {
                        // %(condition.true.false)
                        let mut cond = String::new();
                        let mut depth = 1;
                        while let Some(c) = chars.next() {
                            if c == '(' {
                                depth += 1;
                            } else if c == ')' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            cond.push(c);
                        }
                        result.push_str(&expand_conditional(&cond, ctx));
                    }

                    // Literal %
                    '%' => result.push('%'),

                    // Newline and other
                    '{' => {
                        // %{...%} - literal escape sequence (don't count for prompt length)
                        while let Some(c) = chars.next() {
                            if c == '%' && chars.peek() == Some(&'}') {
                                chars.next();
                                break;
                            }
                            result.push(c);
                        }
                    }

                    _ => {
                        result.push('%');
                        result.push(next);
                    }
                }
            } else {
                result.push('%');
            }
        } else if c == '$' {
            // Variable expansion
            if chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(c);
                    chars.next();
                }
                if let Ok(val) = std::env::var(&var_name) {
                    result.push_str(&val);
                }
            } else if let Some(&next) = chars.peek() {
                if next.is_alphabetic() || next == '_' {
                    let mut var_name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            var_name.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Ok(val) = std::env::var(&var_name) {
                        result.push_str(&val);
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push('$');
            }
        } else if c == '\\' {
            // Bash-style escapes
            if let Some(&next) = chars.peek() {
                chars.next();
                match next {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'e' | 'E' => result.push('\x1b'),
                    '[' => result.push_str("\x1b["), // Start color sequence
                    ']' => {}, // End color sequence (for bash length calc)
                    'u' => result.push_str(ctx.user),
                    'h' => {
                        if let Some(dot_pos) = ctx.host.find('.') {
                            result.push_str(&ctx.host[..dot_pos]);
                        } else {
                            result.push_str(ctx.host);
                        }
                    }
                    'H' => result.push_str(ctx.host),
                    'w' => result.push_str(&ctx.pwd_tilde()),
                    'W' => result.push_str(&ctx.pwd_short()),
                    '$' => {
                        if ctx.user == "root" {
                            result.push('#');
                        } else {
                            result.push('$');
                        }
                    }
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Expand conditional prompt sequences
fn expand_conditional(cond: &str, ctx: &PromptContext) -> String {
    // Format: condition.true-text.false-text
    let parts: Vec<&str> = cond.splitn(3, '.').collect();
    if parts.len() < 2 {
        return String::new();
    }

    let condition = parts[0];
    let true_text = parts.get(1).unwrap_or(&"");
    let false_text = parts.get(2).unwrap_or(&"");

    let result_text = match condition {
        "?" => {
            // %(?.true.false) - based on exit status
            if ctx.last_status == 0 { true_text } else { false_text }
        }
        "#" => {
            // %(#.true.false) - root check
            if ctx.user == "root" { true_text } else { false_text }
        }
        _ => {
            // Check for number comparison like "1?"
            if condition.ends_with('?') {
                let num_str = &condition[..condition.len()-1];
                if let Ok(num) = num_str.parse::<i32>() {
                    if ctx.last_status == num { true_text } else { false_text }
                } else {
                    false_text
                }
            } else {
                false_text
            }
        }
    };

    expand_prompt(result_text, ctx)
}

/// Convert color name to ANSI escape sequence
fn start_color(color: &str, foreground: bool) -> String {
    let code = match color.to_lowercase().as_str() {
        "black" | "0" => "0",
        "red" | "1" => "1",
        "green" | "2" => "2",
        "yellow" | "3" => "3",
        "blue" | "4" => "4",
        "magenta" | "5" => "5",
        "cyan" | "6" => "6",
        "white" | "7" => "7",
        "default" => "9",
        // Bright colors
        "bright_black" | "8" => "8",
        "bright_red" | "9" => "9",
        "bright_green" | "10" => "10",
        "bright_yellow" | "11" => "11",
        "bright_blue" | "12" => "12",
        "bright_magenta" | "13" => "13",
        "bright_cyan" | "14" => "14",
        "bright_white" | "15" => "15",
        // 256 color support
        _ => {
            if let Ok(num) = color.parse::<u8>() {
                return if foreground {
                    format!("\x1b[38;5;{}m", num)
                } else {
                    format!("\x1b[48;5;{}m", num)
                };
            }
            // Hex color support
            if color.starts_with('#') && color.len() == 7 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&color[1..3], 16),
                    u8::from_str_radix(&color[3..5], 16),
                    u8::from_str_radix(&color[5..7], 16),
                ) {
                    return if foreground {
                        format!("\x1b[38;2;{};{};{}m", r, g, b)
                    } else {
                        format!("\x1b[48;2;{};{};{}m", r, g, b)
                    };
                }
            }
            "9"
        }
    };

    let base = if foreground { 30 } else { 40 };
    let num: u8 = code.parse().unwrap_or(9);
    if num < 8 {
        format!("\x1b[{}m", base + num)
    } else if num < 16 {
        format!("\x1b[{}m", base + 60 + num - 8)
    } else {
        format!("\x1b[{}m", base + 9) // Default
    }
}

/// Git prompt info formatter (like oh-my-zsh git_prompt_info)
pub fn git_prompt_info(git: &GitInfo, format: &str) -> String {
    if !git.is_repo {
        return String::new();
    }

    let branch = git.branch.as_ref()
        .or(git.commit_short.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    let mut result = format.to_string();

    // Standard oh-my-zsh variables
    result = result.replace("$(git_current_branch)", branch);
    result = result.replace("%b", branch);

    // Dirty indicator
    let dirty = if git.is_dirty || git.has_staged { "✗" } else { "" };
    result = result.replace("$(git_prompt_dirty)", dirty);
    result = result.replace("%d", dirty);

    // Clean indicator
    let clean = if !git.is_dirty && !git.has_staged { "✓" } else { "" };
    result = result.replace("$(git_prompt_clean)", clean);

    // Untracked indicator
    let untracked = if git.has_untracked { "?" } else { "" };
    result = result.replace("%u", untracked);

    // Ahead/behind
    let ahead = if git.ahead > 0 { format!("↑{}", git.ahead) } else { String::new() };
    let behind = if git.behind > 0 { format!("↓{}", git.behind) } else { String::new() };
    result = result.replace("%a", &ahead);
    result = result.replace("%A", &behind);

    // Status symbols
    let status = format!("{}{}{}",
        if git.has_staged { "●" } else { "" },
        if git.is_dirty { "✚" } else { "" },
        if git.has_untracked { "…" } else { "" }
    );
    result = result.replace("%s", &status);

    result
}

/// Built-in themes
pub fn get_builtin_theme(name: &str) -> Option<Theme> {
    match name {
        "robbyrussell" => Some(Theme {
            prompt: "%F{green}➜%f %F{cyan}%c%f $(git_prompt_info)".to_string(),
            rprompt: None,
            prompt2: "> ".to_string(),
            colors: HashMap::new(),
        }),
        "agnoster" => Some(Theme {
            prompt: concat!(
                "%K{blue}%F{black} %n@%m %f%k",
                "%K{cyan}%F{blue}%f%F{black} %~ %f%k",
                "$(git_prompt_info)",
                "%F{cyan}%f "
            ).to_string(),
            rprompt: None,
            prompt2: "→ ".to_string(),
            colors: HashMap::new(),
        }),
        "minimal" => Some(Theme {
            prompt: "%F{yellow}%~%f %(?.%F{green}.%F{red})❯%f ".to_string(),
            rprompt: None,
            prompt2: "❯ ".to_string(),
            colors: HashMap::new(),
        }),
        "jsh" | "default" => Some(Theme {
            prompt: concat!(
                "%F{cyan}%n%f %F{8}@%f %F{blue}%m%f",
                "$(git_prompt_info)\n",
                "%F{yellow}%~%f %(?.%F{green}.%F{red})❯%f "
            ).to_string(),
            rprompt: Some("%F{8}%T%f".to_string()),
            prompt2: "%F{yellow}❯%f ".to_string(),
            colors: HashMap::new(),
        }),
        "powerlevel" => Some(Theme {
            prompt: concat!(
                "%K{blue}%F{white} %n %f%k",
                "%K{black}%F{blue}%f%F{white} %~ %f%k",
                "$(git_prompt_info)",
                "%F{black}%f "
            ).to_string(),
            rprompt: Some("%K{black}%F{white} %T %f%k".to_string()),
            prompt2: "… ".to_string(),
            colors: HashMap::new(),
        }),
        "simple" => Some(Theme {
            prompt: "%n@%m:%~$ ".to_string(),
            rprompt: None,
            prompt2: "> ".to_string(),
            colors: HashMap::new(),
        }),
        "pure" => Some(Theme {
            prompt: concat!(
                "%F{blue}%~%f",
                "$(git_prompt_info)\n",
                "%(?.%F{magenta}.%F{red})❯%f "
            ).to_string(),
            rprompt: None,
            prompt2: "❯ ".to_string(),
            colors: HashMap::new(),
        }),
        _ => None,
    }
}

/// List available built-in themes
pub fn list_builtin_themes() -> Vec<&'static str> {
    vec![
        "robbyrussell",
        "agnoster",
        "minimal",
        "jsh",
        "powerlevel",
        "simple",
        "pure",
    ]
}

/// Theme manager
pub struct ThemeManager {
    pub current_theme: Theme,
    pub git_format: String,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: get_builtin_theme("jsh").unwrap(),
            git_format: " %F{yellow}git:(%F{red}%b%F{yellow})%f%F{red}%d%f".to_string(),
        }
    }

    /// Load a theme by name
    pub fn load_theme(&mut self, name: &str) -> Result<()> {
        if let Some(theme) = get_builtin_theme(name) {
            self.current_theme = theme;
            Ok(())
        } else {
            Err(crate::error::JshError::runtime(format!("Theme '{}' not found", name)))
        }
    }

    /// Set a custom prompt
    pub fn set_prompt(&mut self, prompt: &str) {
        self.current_theme.prompt = prompt.to_string();
    }

    /// Set a custom right prompt
    pub fn set_rprompt(&mut self, rprompt: Option<&str>) {
        self.current_theme.rprompt = rprompt.map(|s| s.to_string());
    }

    /// Generate the prompt string
    pub fn generate_prompt(&self, ctx: &PromptContext) -> String {
        let prompt = &self.current_theme.prompt;

        // First expand git info
        let git_info = git_prompt_info(&ctx.git, &self.git_format);
        let prompt = prompt.replace("$(git_prompt_info)", &git_info);

        // Then expand prompt sequences
        let expanded = expand_prompt(&prompt, ctx);

        // Reset colors at end
        format!("{}\x1b[0m", expanded)
    }

    /// Generate the right prompt string
    pub fn generate_rprompt(&self, ctx: &PromptContext) -> Option<String> {
        self.current_theme.rprompt.as_ref().map(|rprompt| {
            let git_info = git_prompt_info(&ctx.git, &self.git_format);
            let rprompt = rprompt.replace("$(git_prompt_info)", &git_info);
            let expanded = expand_prompt(&rprompt, ctx);
            format!("{}\x1b[0m", expanded)
        })
    }
}

