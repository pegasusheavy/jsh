//! Plugin management built-in commands

use crate::error::Result;
use crate::interpreter::{ExitStatus, Interpreter};
use crate::plugins::{PluginManager, PluginSource, plugins_dir, omz_dir};
use std::sync::Mutex;

// Global plugin manager - using lazy_static pattern with std::sync
static PLUGIN_MANAGER: Mutex<Option<PluginManager>> = Mutex::new(None);

fn get_plugin_manager() -> std::sync::MutexGuard<'static, Option<PluginManager>> {
    let mut guard = PLUGIN_MANAGER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(PluginManager::new());
    }
    guard
}

/// plug - Register a plugin
/// Usage: plug "source" [options...]
///
/// Sources:
///   plug "user/repo"                    # GitHub repository
///   plug "oh-my-zsh:plugins/git"        # Oh-My-Zsh plugin
///   plug "omz:themes/robbyrussell"      # Oh-My-Zsh theme (shorthand)
///   plug "fish:jorgebucaran/fisher"     # Fish plugin
///   plug "local:/path/to/plugin"        # Local directory
///   plug "https://github.com/..."       # Direct Git URL
///
/// Options:
///   as:theme      - Load as a theme
///   as:command    - Add to PATH only
///   as:defer      - Lazy load
///   branch:name   - Use specific branch
///   tag:version   - Use specific tag
///   depth:1       - Clone depth (default: 1)
///   use:file.sh   - Specific file to source
///   hook:cmd      - Run command after loading
///   frozen        - Don't update this plugin
pub fn builtin_plug(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("Usage: plug \"source\" [options...]");
        eprintln!();
        eprintln!("Sources:");
        eprintln!("  plug \"user/repo\"                    # GitHub repository");
        eprintln!("  plug \"oh-my-zsh:plugins/git\"        # Oh-My-Zsh plugin");
        eprintln!("  plug \"omz:themes/robbyrussell\"      # Oh-My-Zsh theme");
        eprintln!("  plug \"fish:jorgebucaran/fisher\"     # Fish plugin");
        eprintln!("  plug \"local:/path/to/plugin\"        # Local directory");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  as:theme      - Load as a theme");
        eprintln!("  as:defer      - Lazy load");
        eprintln!("  branch:name   - Use specific branch");
        eprintln!("  tag:version   - Use specific tag");
        eprintln!("  use:file.sh   - Specific file to source");
        eprintln!("  frozen        - Don't update this plugin");
        return Ok(ExitStatus::success());
    }

    let mut manager = get_plugin_manager();
    if let Some(ref mut mgr) = *manager {
        mgr.parse_plug(args)?;
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_install - Install registered plugins
/// Usage: plug_install [plugin_name]
pub fn builtin_plug_install(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        if args.is_empty() {
            // Install all
            let installed = mgr.install_all()?;
            if installed.is_empty() {
                println!("All plugins already installed.");
            } else {
                println!("\nInstalled {} plugin(s).", installed.len());
            }
        } else {
            // Install specific plugin
            let name = &args[0];
            if let Some(plugin) = mgr.plugins.get(name) {
                if plugin.is_installed() {
                    println!("Plugin '{}' is already installed.", name);
                } else {
                    mgr.install_all()?;
                }
            } else {
                eprintln!("Plugin '{}' not registered. Use 'plug' first.", name);
                return Ok(ExitStatus::failure(1));
            }
        }
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_update - Update installed plugins
/// Usage: plug_update [plugin_name]
pub fn builtin_plug_update(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        if args.is_empty() {
            // Update all
            println!("Updating plugins...\n");
            let updated = mgr.update_all()?;
            println!("\nUpdated {} plugin(s).", updated.len());
        } else {
            // Update specific
            let name = &args[0];
            if let Some(plugin) = mgr.plugins.get(name) {
                if !plugin.is_installed() {
                    eprintln!("Plugin '{}' is not installed.", name);
                    return Ok(ExitStatus::failure(1));
                }
                mgr.update_all()?;
            } else {
                eprintln!("Plugin '{}' not registered.", name);
                return Ok(ExitStatus::failure(1));
            }
        }
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_clean - Remove unregistered plugins
/// Usage: plug_clean
pub fn builtin_plug_clean(_args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        let removed = mgr.clean()?;
        if removed.is_empty() {
            println!("No unused plugins to remove.");
        } else {
            println!("\nRemoved {} plugin(s).", removed.len());
        }
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_list - List plugins
/// Usage: plug_list [--installed]
pub fn builtin_plug_list(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        if args.contains(&"--installed".to_string()) {
            // List installed only
            let installed = mgr.list_installed();
            if installed.is_empty() {
                println!("No plugins installed.");
            } else {
                println!("Installed plugins:");
                for name in installed {
                    println!("  {}", name);
                }
            }
        } else {
            // List registered plugins
            if mgr.plugins.is_empty() {
                println!("No plugins registered.");
                println!("Use 'plug \"source\"' to register plugins.");
            } else {
                println!("Registered plugins:");
                for (name, plugin) in &mgr.plugins {
                    let status = if plugin.is_installed() { "✓" } else { "○" };
                    let source = match &plugin.source {
                        PluginSource::GitHub(repo) => format!("github:{}", repo),
                        PluginSource::OhMyZsh(path) => format!("omz:{}", path),
                        PluginSource::OhMyZshTheme(name) => format!("omz:themes/{}", name),
                        PluginSource::Fish(repo) => format!("fish:{}", repo),
                        PluginSource::Local(path) => format!("local:{}", path.display()),
                        PluginSource::Git(url) => url.clone(),
                    };
                    println!("  {} {} ({})", status, name, source);
                }
            }
        }
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_load - Load all registered plugins
/// Usage: plug_load
pub fn builtin_plug_load(_args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        let commands = mgr.generate_load_commands();
        for cmd in commands {
            let _ = interp.execute_string(&cmd);
        }
        Ok(ExitStatus::success())
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_source - Source a specific plugin
/// Usage: plug_source <plugin_name>
pub fn builtin_plug_source(args: &[String], interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        eprintln!("Usage: plug_source <plugin_name>");
        return Ok(ExitStatus::failure(1));
    }

    let name = &args[0];
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        if let Some(plugin) = mgr.plugins.get(name) {
            if !plugin.is_installed() {
                eprintln!("Plugin '{}' is not installed. Run 'plug_install'.", name);
                return Ok(ExitStatus::failure(1));
            }

            for file in plugin.get_source_files() {
                let cmd = format!("source \"{}\"", file.display());
                let _ = interp.execute_string(&cmd);
            }

            if let Some(ref hook) = plugin.hook {
                let _ = interp.execute_string(hook);
            }

            Ok(ExitStatus::success())
        } else {
            eprintln!("Plugin '{}' not registered.", name);
            Ok(ExitStatus::failure(1))
        }
    } else {
        Ok(ExitStatus::failure(1))
    }
}

/// plug_info - Show plugin information
/// Usage: plug_info <plugin_name>
pub fn builtin_plug_info(args: &[String], _interp: &mut Interpreter) -> Result<ExitStatus> {
    if args.is_empty() {
        // Show general info
        println!("jsh Plugin Manager");
        println!();
        println!("Directories:");
        println!("  Plugins: {}", plugins_dir().display());
        println!("  Oh-My-Zsh: {}", omz_dir().display());
        println!();
        println!("Commands:");
        println!("  plug \"source\"     - Register a plugin");
        println!("  plug_install      - Install registered plugins");
        println!("  plug_update       - Update plugins");
        println!("  plug_clean        - Remove unused plugins");
        println!("  plug_list         - List plugins");
        println!("  plug_load         - Load all plugins");
        println!("  plug_source       - Source a specific plugin");
        return Ok(ExitStatus::success());
    }

    let name = &args[0];
    let manager = get_plugin_manager();
    if let Some(ref mgr) = *manager {
        if let Some(plugin) = mgr.plugins.get(name) {
            println!("Plugin: {}", plugin.name);
            println!("Source: {:?}", plugin.source);
            println!("Installed: {}", plugin.is_installed());
            println!("Enabled: {}", plugin.enabled);
            println!("Frozen: {}", plugin.frozen);
            println!("Load type: {:?}", plugin.load);
            if let Some(ref branch) = plugin.branch {
                println!("Branch: {}", branch);
            }
            if let Some(ref tag) = plugin.tag {
                println!("Tag: {}", tag);
            }
            println!("Directory: {}", plugin.install_dir().display());
            
            let files = plugin.get_source_files();
            if !files.is_empty() {
                println!("Source files:");
                for f in files {
                    println!("  {}", f.display());
                }
            }

            Ok(ExitStatus::success())
        } else {
            eprintln!("Plugin '{}' not registered.", name);
            Ok(ExitStatus::failure(1))
        }
    } else {
        Ok(ExitStatus::failure(1))
    }
}

