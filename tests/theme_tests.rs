//! Theme module tests

use franken_shell::theme::{expand_prompt, get_builtin_theme, git_prompt_info, list_builtin_themes, GitInfo, PromptContext, Theme, ThemeManager};
use std::path::PathBuf;
use chrono::Local;
use std::collections::HashMap;

// =============================================================================
// Theme Structure Tests
// =============================================================================

#[test]
fn test_theme_new() {
    let theme = Theme {
        prompt: "%~ $ ".to_string(),
        rprompt: None,
        prompt2: "> ".to_string(),
        colors: HashMap::new(),
    };
    assert_eq!(theme.prompt, "%~ $ ");
    assert_eq!(theme.prompt2, "> ");
}

#[test]
fn test_theme_with_rprompt() {
    let theme = Theme {
        prompt: "$ ".to_string(),
        rprompt: Some("%T".to_string()),
        prompt2: "> ".to_string(),
        colors: HashMap::new(),
    };
    assert!(theme.rprompt.is_some());
    assert_eq!(theme.rprompt.unwrap(), "%T");
}

#[test]
fn test_theme_with_colors() {
    let mut colors = HashMap::new();
    colors.insert("user".to_string(), "green".to_string());
    colors.insert("host".to_string(), "blue".to_string());

    let theme = Theme {
        prompt: "$ ".to_string(),
        rprompt: None,
        prompt2: "> ".to_string(),
        colors,
    };

    assert_eq!(theme.colors.get("user"), Some(&"green".to_string()));
    assert_eq!(theme.colors.get("host"), Some(&"blue".to_string()));
}

// =============================================================================
// GitInfo Tests
// =============================================================================

#[test]
fn test_git_info_default() {
    let info = GitInfo::default();
    assert!(!info.is_repo);
    assert!(info.branch.is_none());
    assert!(!info.is_dirty);
    assert!(!info.has_staged);
    assert!(!info.has_untracked);
}

#[test]
fn test_git_info_with_branch() {
    let info = GitInfo {
        is_repo: true,
        branch: Some("main".to_string()),
        commit_short: Some("abc123".to_string()),
        is_dirty: false,
        has_staged: false,
        has_untracked: false,
        ahead: 0,
        behind: 0,
        remote: None,
    };

    assert!(info.is_repo);
    assert_eq!(info.branch, Some("main".to_string()));
    assert_eq!(info.commit_short, Some("abc123".to_string()));
}

#[test]
fn test_git_info_dirty() {
    let info = GitInfo {
        is_repo: true,
        branch: Some("main".to_string()),
        commit_short: None,
        is_dirty: true,
        has_staged: true,
        has_untracked: true,
        ahead: 0,
        behind: 0,
        remote: None,
    };

    assert!(info.is_dirty);
    assert!(info.has_staged);
    assert!(info.has_untracked);
}

// =============================================================================
// PromptContext Tests
// =============================================================================

#[test]
fn test_prompt_context_pwd_tilde() {
    let home = "/home/testuser";
    let cwd = PathBuf::from("/home/testuser/projects/fsh");
    let ctx = PromptContext {
        cwd: &cwd,
        home,
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(ctx.pwd_tilde(), "~/projects/fsh");
}

#[test]
fn test_prompt_context_pwd_no_tilde() {
    let home = "/home/testuser";
    let cwd = PathBuf::from("/usr/local/bin");
    let ctx = PromptContext {
        cwd: &cwd,
        home,
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(ctx.pwd_tilde(), "/usr/local/bin");
}

#[test]
fn test_prompt_context_pwd_short() {
    let cwd = PathBuf::from("/home/testuser/projects/fsh");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(ctx.pwd_short(), "fsh");
}

// =============================================================================
// Expand Prompt Tests
// =============================================================================

#[test]
fn test_expand_prompt_username() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%n", &ctx), "testuser");
}

#[test]
fn test_expand_prompt_hostname() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "myhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%m", &ctx), "myhost");
}

#[test]
fn test_expand_prompt_pwd_tilde() {
    let cwd = PathBuf::from("/home/testuser/projects");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%~", &ctx), "~/projects");
}

#[test]
fn test_expand_prompt_basename() {
    let cwd = PathBuf::from("/home/testuser/projects/fsh");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %c or %C should give basename
    let result = expand_prompt("%c", &ctx);
    assert!(result.contains("fsh") || result == "fsh");
}

#[test]
fn test_expand_prompt_exit_status() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 42,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%?", &ctx), "42");
}

#[test]
fn test_expand_prompt_history_number() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 123,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%h", &ctx), "123");
}

#[test]
fn test_expand_prompt_percent() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    assert_eq!(expand_prompt("%%", &ctx), "%");
}

#[test]
fn test_expand_prompt_complex() {
    let cwd = PathBuf::from("/home/testuser/projects");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "myhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 42,
        time: Local::now(),
    };

    let result = expand_prompt("%n@%m:%~ $ ", &ctx);
    assert!(result.contains("testuser"));
    assert!(result.contains("myhost"));
    assert!(result.contains("~/projects"));
}

// =============================================================================
// Git Prompt Info Tests
// =============================================================================

#[test]
fn test_git_prompt_info_not_repo() {
    let info = GitInfo::default();
    let result = git_prompt_info(&info, " (%b)");
    assert!(result.is_empty());
}

#[test]
fn test_git_prompt_info_with_branch() {
    let info = GitInfo {
        is_repo: true,
        branch: Some("main".to_string()),
        commit_short: None,
        is_dirty: false,
        has_staged: false,
        has_untracked: false,
        ahead: 0,
        behind: 0,
        remote: None,
    };

    let result = git_prompt_info(&info, " (%b)");
    assert!(result.contains("main"));
}

#[test]
fn test_git_prompt_info_dirty() {
    let info = GitInfo {
        is_repo: true,
        branch: Some("feature".to_string()),
        commit_short: None,
        is_dirty: true,
        has_staged: false,
        has_untracked: false,
        ahead: 0,
        behind: 0,
        remote: None,
    };

    let result = git_prompt_info(&info, " (%b%d)");
    assert!(result.contains("feature"));
}

// =============================================================================
// Builtin Theme Tests
// =============================================================================

#[test]
fn test_get_builtin_theme_robbyrussell() {
    let theme = get_builtin_theme("robbyrussell");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_agnoster() {
    let theme = get_builtin_theme("agnoster");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_jsh() {
    let theme = get_builtin_theme("fsh");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_gallifrey() {
    let theme = get_builtin_theme("gallifrey");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_nonexistent() {
    let theme = get_builtin_theme("nonexistent_theme_xyz");
    assert!(theme.is_none());
}

#[test]
fn test_list_builtin_themes() {
    let themes = list_builtin_themes();
    assert!(!themes.is_empty());
    assert!(themes.contains(&"robbyrussell"));
    assert!(themes.contains(&"agnoster"));
    assert!(themes.contains(&"fsh")); // fsh instead of "default"
}

// =============================================================================
// ThemeManager Tests
// =============================================================================

#[test]
fn test_theme_manager_new() {
    let manager = ThemeManager::new();
    // Should have default theme
    assert!(!manager.current_theme.prompt.is_empty());
}

#[test]
fn test_theme_manager_load_theme() {
    let mut manager = ThemeManager::new();
    let result = manager.load_theme("robbyrussell");
    assert!(result.is_ok());
}

#[test]
fn test_theme_manager_load_nonexistent() {
    let mut manager = ThemeManager::new();
    let result = manager.load_theme("nonexistent_xyz");
    assert!(result.is_err());
}

#[test]
fn test_theme_manager_set_prompt() {
    let mut manager = ThemeManager::new();
    manager.set_prompt(">>> ");
    assert_eq!(manager.current_theme.prompt, ">>> ");
}

#[test]
fn test_theme_manager_set_rprompt() {
    let mut manager = ThemeManager::new();
    manager.set_rprompt(Some("%T"));
    assert_eq!(manager.current_theme.rprompt, Some("%T".to_string()));
}

#[test]
fn test_theme_manager_clear_rprompt() {
    let mut manager = ThemeManager::new();
    manager.set_rprompt(Some("%T"));
    manager.set_rprompt(None);
    assert!(manager.current_theme.rprompt.is_none());
}

// =============================================================================
// Color Code Tests
// =============================================================================

#[test]
fn test_expand_prompt_color_red() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %F{red} should produce color codes
    let result = expand_prompt("%F{red}test%f", &ctx);
    // Result should contain ANSI codes or the text
    assert!(result.contains("test"));
}

#[test]
fn test_expand_prompt_bold() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %B and %b for bold
    let result = expand_prompt("%Bbold%b", &ctx);
    assert!(result.contains("bold"));
}

// =============================================================================
// Time Format Tests
// =============================================================================

#[test]
fn test_expand_prompt_time() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %T should give time in some format
    let result = expand_prompt("%T", &ctx);
    // Should be non-empty and contain digits
    assert!(!result.is_empty());
}

#[test]
fn test_expand_prompt_date() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %D should give date
    let result = expand_prompt("%D", &ctx);
    assert!(!result.is_empty());
}

// =============================================================================
// Root Prompt Tests
// =============================================================================

#[test]
fn test_expand_prompt_hash() {
    let cwd = PathBuf::from("/home/testuser");
    let ctx = PromptContext {
        cwd: &cwd,
        home: "/home/testuser",
        user: "testuser",
        host: "localhost",
        last_status: 0,
        git: GitInfo::default(),
        shell_name: "fsh",
        history_num: 1,
        time: Local::now(),
    };

    // %# should be $ for normal user, # for root
    let result = expand_prompt("%#", &ctx);
    // For a normal user, should be $ or %
    assert!(result == "$" || result == "%" || result == "#");
}

// =============================================================================
// Oh-My-ZSH Theme Tests
// =============================================================================

#[test]
fn test_get_builtin_theme_bira() {
    let theme = get_builtin_theme("bira");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Bira uses box drawing characters
    assert!(theme.prompt.contains("╭") || theme.prompt.contains("─"));
}

#[test]
fn test_get_builtin_theme_fino() {
    let theme = get_builtin_theme("fino");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_avit() {
    let theme = get_builtin_theme("avit");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_bureau() {
    let theme = get_builtin_theme("bureau");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_ys() {
    let theme = get_builtin_theme("ys");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_lambda() {
    let theme = get_builtin_theme("lambda");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Lambda theme should have the λ symbol
    assert!(theme.prompt.contains("λ"));
}

#[test]
fn test_get_builtin_theme_half_life() {
    let theme = get_builtin_theme("half-life");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Half-Life theme should have the λ symbol
    assert!(theme.prompt.contains("λ"));
}

#[test]
fn test_get_builtin_theme_cloud() {
    let theme = get_builtin_theme("cloud");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Cloud theme should have cloud emoji
    assert!(theme.prompt.contains("☁"));
}

#[test]
fn test_get_builtin_theme_emotty() {
    let theme = get_builtin_theme("emotty");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Emotty theme should have emoji
    assert!(theme.prompt.contains("😊") || theme.prompt.contains("😢"));
}

#[test]
fn test_get_builtin_theme_fishy() {
    let theme = get_builtin_theme("fishy");
    assert!(theme.is_some());
    let theme = theme.unwrap();
    // Fishy theme should have >>> arrows
    assert!(theme.prompt.contains(">"));
}

#[test]
fn test_get_builtin_theme_spaceship() {
    let theme = get_builtin_theme("spaceship");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_dracula() {
    let theme = get_builtin_theme("dracula");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_nord() {
    let theme = get_builtin_theme("nord");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_gruvbox() {
    let theme = get_builtin_theme("gruvbox");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_catppuccin() {
    let theme = get_builtin_theme("catppuccin");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_tokyo_night() {
    let theme = get_builtin_theme("tokyo-night");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_onedark() {
    let theme = get_builtin_theme("onedark");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_solarized() {
    let theme = get_builtin_theme("solarized");
    assert!(theme.is_some());
}

#[test]
fn test_get_builtin_theme_monokai() {
    let theme = get_builtin_theme("monokai");
    assert!(theme.is_some());
}

#[test]
fn test_list_builtin_themes_count() {
    let themes = list_builtin_themes();
    // We should have at least 130 themes now (with all Oh-My-ZSH themes)
    assert!(themes.len() >= 130, "Expected at least 130 themes, got {}", themes.len());
}

#[test]
fn test_all_listed_themes_exist() {
    let themes = list_builtin_themes();
    for theme_name in themes {
        let theme = get_builtin_theme(theme_name);
        assert!(theme.is_some(), "Theme '{}' is listed but doesn't exist", theme_name);
    }
}
