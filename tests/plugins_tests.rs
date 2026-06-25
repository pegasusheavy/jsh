//! Plugin manager module tests

use franken_shell::plugins::{
    Plugin, PluginLoad, PluginManager, PluginSource, omz_dir, plugin_cache_dir, plugins_dir,
};
use std::path::PathBuf;

// =============================================================================
// PluginSource Tests
// =============================================================================

#[test]
fn test_plugin_source_github() {
    let source = PluginSource::GitHub("user/repo".to_string());
    assert!(matches!(source, PluginSource::GitHub(s) if s == "user/repo"));
}

#[test]
fn test_plugin_source_omz() {
    let source = PluginSource::OhMyZsh("git".to_string());
    assert!(matches!(source, PluginSource::OhMyZsh(s) if s == "git"));
}

#[test]
fn test_plugin_source_omz_theme() {
    let source = PluginSource::OhMyZshTheme("robbyrussell".to_string());
    assert!(matches!(source, PluginSource::OhMyZshTheme(s) if s == "robbyrussell"));
}

#[test]
fn test_plugin_source_fish() {
    let source = PluginSource::Fish("bass".to_string());
    assert!(matches!(source, PluginSource::Fish(s) if s == "bass"));
}

#[test]
fn test_plugin_source_local() {
    let source = PluginSource::Local(PathBuf::from("/path/to/plugin"));
    assert!(matches!(source, PluginSource::Local(p) if p == *"/path/to/plugin"));
}

#[test]
fn test_plugin_source_git() {
    let source = PluginSource::Git("https://github.com/user/repo.git".to_string());
    assert!(matches!(source, PluginSource::Git(s) if s == "https://github.com/user/repo.git"));
}

// =============================================================================
// PluginLoad Tests
// =============================================================================

#[test]
fn test_plugin_load_source() {
    let load = PluginLoad::Source;
    assert!(matches!(load, PluginLoad::Source));
}

#[test]
fn test_plugin_load_path() {
    let load = PluginLoad::Path;
    assert!(matches!(load, PluginLoad::Path));
}

#[test]
fn test_plugin_load_theme() {
    let load = PluginLoad::Theme;
    assert!(matches!(load, PluginLoad::Theme));
}

#[test]
fn test_plugin_load_defer() {
    let load = PluginLoad::Defer;
    assert!(matches!(load, PluginLoad::Defer));
}

// =============================================================================
// Plugin Tests
// =============================================================================

#[test]
fn test_plugin_new() {
    let plugin = Plugin::new("test-plugin", PluginSource::GitHub("user/repo".to_string()));
    assert_eq!(plugin.name, "test-plugin");
    assert!(matches!(plugin.source, PluginSource::GitHub(_)));
    assert!(matches!(plugin.load, PluginLoad::Source));
    assert!(plugin.enabled);
    assert!(!plugin.frozen);
}

#[test]
fn test_plugin_install_dir() {
    let plugin = Plugin::new("my-plugin", PluginSource::GitHub("user/repo".to_string()));
    let install_dir = plugin.install_dir();
    assert!(install_dir.ends_with("my-plugin"));
}

#[test]
fn test_plugin_with_branch() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.branch = Some("develop".to_string());
    assert_eq!(plugin.branch, Some("develop".to_string()));
}

#[test]
fn test_plugin_with_tag() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.tag = Some("v1.0.0".to_string());
    assert_eq!(plugin.tag, Some("v1.0.0".to_string()));
}

#[test]
fn test_plugin_frozen() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.frozen = true;
    assert!(plugin.frozen);
}

#[test]
fn test_plugin_disabled() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.enabled = false;
    assert!(!plugin.enabled);
}

#[test]
fn test_plugin_use_files() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.use_files = vec!["*.sh".to_string(), "init.zsh".to_string()];
    assert_eq!(plugin.use_files.len(), 2);
}

#[test]
fn test_plugin_hook() {
    let mut plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    plugin.hook = Some("echo 'loaded'".to_string());
    assert!(plugin.hook.is_some());
}

// =============================================================================
// Directory Functions Tests
// =============================================================================

#[test]
fn test_plugins_dir() {
    let dir = plugins_dir();
    // Should be in XDG_DATA_HOME/fsh/plugins or ~/.local/share/fsh/plugins
    assert!(dir.to_string_lossy().contains("fsh") || dir.to_string_lossy().contains("plugins"));
}

#[test]
fn test_omz_dir() {
    let dir = omz_dir();
    // Should be in XDG_DATA_HOME/fsh/oh-my-zsh
    assert!(dir.to_string_lossy().contains("fsh") || dir.to_string_lossy().contains("oh-my-zsh"));
}

#[test]
fn test_plugin_cache_dir() {
    let dir = plugin_cache_dir();
    // Should be in XDG_CACHE_HOME/fsh/plugins
    assert!(dir.to_string_lossy().contains("fsh"));
}

// =============================================================================
// PluginManager Tests
// =============================================================================

#[test]
fn test_plugin_manager_new() {
    let manager = PluginManager::new();
    assert!(manager.plugins.is_empty());
}

#[test]
fn test_plugin_manager_add() {
    let mut manager = PluginManager::new();
    let plugin = Plugin::new("test-plugin", PluginSource::GitHub("user/repo".to_string()));
    manager.add(plugin);
    assert_eq!(manager.plugins.len(), 1);
}

#[test]
fn test_plugin_manager_add_multiple() {
    let mut manager = PluginManager::new();
    manager.add(Plugin::new(
        "plugin1",
        PluginSource::GitHub("user/repo1".to_string()),
    ));
    manager.add(Plugin::new(
        "plugin2",
        PluginSource::GitHub("user/repo2".to_string()),
    ));
    manager.add(Plugin::new(
        "plugin3",
        PluginSource::OhMyZsh("git".to_string()),
    ));
    assert_eq!(manager.plugins.len(), 3);
}

#[test]
fn test_plugin_manager_plugins_accessible() {
    let mut manager = PluginManager::new();
    manager.add(Plugin::new(
        "my-plugin",
        PluginSource::GitHub("user/repo".to_string()),
    ));

    // Access via the public plugins field (HashMap)
    let plugin = manager.plugins.get("my-plugin");
    assert!(plugin.is_some());
    assert_eq!(plugin.unwrap().name, "my-plugin");
}

#[test]
fn test_plugin_manager_list_installed() {
    let manager = PluginManager::new();
    // list_installed returns installed plugins from disk
    // This returns whatever is actually installed, so just check it doesn't crash.
    let _installed = manager.list_installed();
}

#[test]
fn test_plugin_manager_generate_load_commands() {
    let mut manager = PluginManager::new();
    manager.add(Plugin::new(
        "plugin1",
        PluginSource::GitHub("user/repo1".to_string()),
    ));

    // Generate load commands; the list may be empty if plugins are not
    // installed, so just confirm the call succeeds without panicking.
    let _commands = manager.generate_load_commands();
}

// =============================================================================
// Plugin Clone Tests
// =============================================================================

#[test]
fn test_plugin_clone() {
    let plugin = Plugin::new("test", PluginSource::GitHub("user/repo".to_string()));
    let cloned = plugin.clone();

    assert_eq!(plugin.name, cloned.name);
    assert_eq!(plugin.source, cloned.source);
    assert_eq!(plugin.enabled, cloned.enabled);
}

#[test]
fn test_plugin_source_clone() {
    let source = PluginSource::GitHub("user/repo".to_string());
    let cloned = source.clone();
    assert_eq!(source, cloned);
}

#[test]
fn test_plugin_load_clone() {
    let load = PluginLoad::Source;
    let cloned = load.clone();
    assert_eq!(load, cloned);
}

// =============================================================================
// Plugin Equality Tests
// =============================================================================

#[test]
fn test_plugin_source_equality() {
    let s1 = PluginSource::GitHub("user/repo".to_string());
    let s2 = PluginSource::GitHub("user/repo".to_string());
    let s3 = PluginSource::GitHub("other/repo".to_string());

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn test_plugin_load_equality() {
    let l1 = PluginLoad::Source;
    let l2 = PluginLoad::Source;
    let l3 = PluginLoad::Path;

    assert_eq!(l1, l2);
    assert_ne!(l1, l3);
}

// =============================================================================
// Plugin Source Variants Tests
// =============================================================================

#[test]
fn test_all_plugin_sources() {
    let sources = [
        PluginSource::GitHub("user/repo".to_string()),
        PluginSource::OhMyZsh("git".to_string()),
        PluginSource::OhMyZshTheme("robbyrussell".to_string()),
        PluginSource::Fish("bass".to_string()),
        PluginSource::Local(PathBuf::from("/path/to/plugin")),
        PluginSource::Git("https://example.com/repo.git".to_string()),
    ];

    assert_eq!(sources.len(), 6);
}

#[test]
fn test_all_plugin_loads() {
    let loads = [
        PluginLoad::Source,
        PluginLoad::Path,
        PluginLoad::Theme,
        PluginLoad::Defer,
    ];

    assert_eq!(loads.len(), 4);
}

// =============================================================================
// Plugin Manager Operations
// =============================================================================

#[test]
fn test_plugin_is_installed_uninstalled() {
    let plugin = Plugin::new(
        "nonexistent-xyz-123",
        PluginSource::GitHub("user/repo".to_string()),
    );
    assert!(!plugin.is_installed());
}

#[test]
fn test_plugin_get_source_files_uninstalled() {
    let plugin = Plugin::new(
        "nonexistent-xyz-123",
        PluginSource::GitHub("user/repo".to_string()),
    );
    let files = plugin.get_source_files();
    assert!(files.is_empty());
}

// =============================================================================
// Parse Plug Tests
// =============================================================================

#[test]
fn test_plugin_manager_parse_plug_github() {
    let mut manager = PluginManager::new();
    let args = vec!["user/repo".to_string()];
    let result = manager.parse_plug(&args);
    assert!(result.is_ok());
    assert_eq!(manager.plugins.len(), 1);
}

#[test]
fn test_plugin_manager_parse_plug_with_branch() {
    let mut manager = PluginManager::new();
    // Use colon-separated format as expected by parse_plug
    let args = vec!["user/repo".to_string(), "branch:develop".to_string()];
    let result = manager.parse_plug(&args);
    assert!(result.is_ok());
    // Find the plugin by name (user/repo gets the name "repo")
    let plugin = manager.plugins.values().next();
    assert!(plugin.is_some());
    assert_eq!(plugin.unwrap().branch, Some("develop".to_string()));
}

#[test]
fn test_plugin_manager_parse_plug_frozen() {
    let mut manager = PluginManager::new();
    // Use single word format for flags
    let args = vec!["user/repo".to_string(), "frozen".to_string()];
    let result = manager.parse_plug(&args);
    assert!(result.is_ok());
    let plugin = manager.plugins.values().next();
    assert!(plugin.is_some());
    assert!(plugin.unwrap().frozen);
}

#[test]
fn test_plugin_manager_parse_plug_with_rename() {
    let mut manager = PluginManager::new();
    let args = vec!["user/repo".to_string(), "name:custom-name".to_string()];
    let result = manager.parse_plug(&args);
    assert!(result.is_ok());
    assert!(manager.plugins.contains_key("custom-name"));
}

#[test]
fn test_plugin_manager_parse_plug_defer() {
    let mut manager = PluginManager::new();
    let args = vec!["user/repo".to_string(), "defer".to_string()];
    let result = manager.parse_plug(&args);
    assert!(result.is_ok());
    let plugin = manager.plugins.values().next();
    assert!(plugin.is_some());
    assert!(matches!(plugin.unwrap().load, PluginLoad::Defer));
}
