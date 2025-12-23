//! Tmux command implementation

use crate::error::{JshError, Result};
use super::TmuxServer;
use super::window::ResizeDirection;
use super::theme::StatusBarTheme;

/// Execute a tmux command
pub fn execute_command(server: &mut TmuxServer, cmd: &str, args: &[&str]) -> Result<String> {
    match cmd {
        // Session commands
        "new-session" | "new" => cmd_new_session(server, args),
        "kill-session" => cmd_kill_session(server, args),
        "rename-session" | "rename" => cmd_rename_session(server, args),
        "list-sessions" | "ls" => cmd_list_sessions(server),
        "attach-session" | "attach" | "a" => cmd_attach_session(server, args),
        "detach-client" | "detach" | "d" => cmd_detach_client(server),
        "switch-client" => cmd_switch_client(server, args),
        "has-session" => cmd_has_session(server, args),

        // Window commands
        "new-window" | "neww" => cmd_new_window(server, args),
        "kill-window" | "killw" => cmd_kill_window(server, args),
        "rename-window" | "renamew" => cmd_rename_window(server, args),
        "list-windows" | "lsw" => cmd_list_windows(server, args),
        "select-window" | "selectw" => cmd_select_window(server, args),
        "next-window" | "next" => cmd_next_window(server),
        "previous-window" | "prev" => cmd_previous_window(server),
        "last-window" | "last" => cmd_last_window(server),
        "rotate-window" | "rotatew" => cmd_rotate_window(server, args),
        "swap-window" | "swapw" => cmd_swap_window(server, args),
        "move-window" | "movew" => cmd_move_window(server, args),
        "link-window" | "linkw" => cmd_link_window(server, args),
        "unlink-window" | "unlinkw" => cmd_unlink_window(server, args),
        "find-window" | "findw" => cmd_find_window(server, args),

        // Pane commands
        "split-window" | "splitw" => cmd_split_window(server, args),
        "kill-pane" | "killp" => cmd_kill_pane(server, args),
        "select-pane" | "selectp" => cmd_select_pane(server, args),
        "last-pane" | "lastp" => cmd_last_pane(server),
        "resize-pane" | "resizep" => cmd_resize_pane(server, args),
        "swap-pane" | "swapp" => cmd_swap_pane(server, args),
        "break-pane" | "breakp" => cmd_break_pane(server, args),
        "join-pane" | "joinp" => cmd_join_pane(server, args),
        "capture-pane" | "capturep" => cmd_capture_pane(server, args),
        "display-panes" | "displayp" => cmd_display_panes(server),
        "pipe-pane" | "pipep" => cmd_pipe_pane(server, args),
        "respawn-pane" | "respawnp" => cmd_respawn_pane(server, args),
        "list-panes" | "lsp" => cmd_list_panes(server, args),

        // Layout commands
        "select-layout" | "selectl" => cmd_select_layout(server, args),
        "next-layout" | "nextl" => cmd_next_layout(server),

        // Configuration commands
        "set-option" | "set" => cmd_set_option(server, args),
        "set-window-option" | "setw" => cmd_set_window_option(server, args),
        "show-options" | "show" => cmd_show_options(server, args),
        "source-file" | "source" => cmd_source_file(server, args),

        // Key binding commands
        "bind-key" | "bind" => cmd_bind_key(server, args),
        "unbind-key" | "unbind" => cmd_unbind_key(server, args),
        "list-keys" | "lsk" => cmd_list_keys(server, args),

        // Copy/paste commands
        "copy-mode" => cmd_copy_mode(server, args),
        "paste-buffer" | "pasteb" => cmd_paste_buffer(server, args),
        "list-buffers" | "lsb" => cmd_list_buffers(server),
        "set-buffer" | "setb" => cmd_set_buffer(server, args),
        "show-buffer" | "showb" => cmd_show_buffer(server, args),
        "delete-buffer" | "deleteb" => cmd_delete_buffer(server, args),

        // Input commands
        "send-keys" | "send" => cmd_send_keys(server, args),
        "send-prefix" => cmd_send_prefix(server),

        // Display commands
        "display-message" | "display" => cmd_display_message(server, args),
        "clock-mode" => cmd_clock_mode(server),
        "show-messages" | "showmsgs" => cmd_show_messages(server),

        // Misc commands
        "run-shell" | "run" => cmd_run_shell(args),
        "confirm-before" | "confirm" => cmd_confirm_before(server, args),
        "command-prompt" => cmd_command_prompt(server, args),
        "choose-tree" => cmd_choose_tree(server, args),
        "choose-buffer" => cmd_choose_buffer(server),
        "if-shell" | "if" => cmd_if_shell(server, args),
        "wait-for" | "wait" => cmd_wait_for(server, args),
        "lock-server" | "lock" => cmd_lock_server(server),
        "lock-session" | "locks" => cmd_lock_session(server),
        "lock-client" | "lockc" => cmd_lock_client(server),
        "refresh-client" | "refresh" => cmd_refresh_client(server),
        "suspend-client" | "suspendc" => cmd_suspend_client(server),
        "clear-history" | "clearhist" => cmd_clear_history(server, args),
        "info" => cmd_info(server),
        "server-info" => cmd_server_info(server),
        "start-server" | "start" => cmd_start_server(server),
        "kill-server" => cmd_kill_server(server),

        // Plugin commands
        "tpm-install" => cmd_tpm_install(server),
        "tpm-update" => cmd_tpm_update(server),
        "tpm-clean" => cmd_tpm_clean(server),
        "tpm-list" => cmd_tpm_list(server),

        // Theme commands
        "set-theme" => cmd_set_theme(server, args),
        "list-themes" => cmd_list_themes(),

        _ => Err(JshError::runtime(format!("Unknown tmux command: {}", cmd))),
    }
}

// Session commands

fn cmd_new_session(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let mut name = format!("{}", server.sessions.len());
    let mut detached = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-d" => detached = true,
            "-s" if i + 1 < args.len() => {
                i += 1;
                name = args[i].to_string();
            }
            "-n" | "-c" | "-x" | "-y" | "-t" | "-e" | "-f" | "-P" | "-F" => {
                i += 1; // Skip value
            }
            s if !s.starts_with('-') && name == format!("{}", server.sessions.len()) => {
                name = s.to_string();
            }
            _ => {}
        }
        i += 1;
    }

    let session = server.new_session(&name)?;
    
    if !detached {
        session.lock().unwrap().attach();
    }

    Ok(format!("Created session: {}", name))
}

fn cmd_kill_session(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let mut target = None;
    let mut all = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-a" => all = true,
            "-t" if i + 1 < args.len() => {
                i += 1;
                target = Some(args[i].to_string());
            }
            s if !s.starts_with('-') => {
                target = Some(s.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    if all {
        let sessions: Vec<String> = server.sessions.keys().cloned().collect();
        for name in sessions {
            if target.as_ref() != Some(&name) {
                let _ = server.kill_session(&name);
            }
        }
        Ok("Killed all sessions".to_string())
    } else if let Some(name) = target {
        server.kill_session(&name)?;
        Ok(format!("Killed session: {}", name))
    } else {
        Err(JshError::runtime("No target session specified"))
    }
}

fn cmd_rename_session(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    if args.len() < 2 {
        return Err(JshError::runtime("Usage: rename-session [-t target] new-name"));
    }

    let new_name = args.last().unwrap();
    let target = server.current_session.clone()
        .ok_or_else(|| JshError::runtime("No session"))?;

    server.rename_session(&target, new_name)?;
    Ok(format!("Renamed session to: {}", new_name))
}

fn cmd_list_sessions(server: &TmuxServer) -> Result<String> {
    if server.sessions.is_empty() {
        return Ok("no server running".to_string());
    }

    let mut output = String::new();
    for (name, session) in &server.sessions {
        let s = session.lock().unwrap();
        let attached = if s.is_attached() { "(attached)" } else { "" };
        output.push_str(&format!("{}: {} windows {}\n", name, s.window_count(), attached));
    }
    Ok(output.trim().to_string())
}

fn cmd_attach_session(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let mut target = server.sessions.keys().next().cloned();
    
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-t" && i + 1 < args.len() {
            target = Some(args[i + 1].to_string());
        }
    }

    if let Some(name) = target {
        server.attach_session(&name)?;
        if let Some(session) = server.get_session(&name) {
            session.lock().unwrap().attach();
        }
        Ok(format!("Attached to session: {}", name))
    } else {
        Err(JshError::runtime("No session to attach to"))
    }
}

fn cmd_detach_client(server: &TmuxServer) -> Result<String> {
    if let Some(session) = server.current() {
        session.lock().unwrap().detach();
        Ok("Detached".to_string())
    } else {
        Err(JshError::runtime("No current session"))
    }
}

fn cmd_switch_client(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let sessions: Vec<String> = server.sessions.keys().cloned().collect();
    
    if sessions.is_empty() {
        return Err(JshError::runtime("No sessions"));
    }

    let current = server.current_session.clone().unwrap_or_default();
    let current_idx = sessions.iter().position(|s| s == &current).unwrap_or(0);

    let mut next_session = false;
    let mut prev_session = false;
    let mut target = None;

    for (i, arg) in args.iter().enumerate() {
        match *arg {
            "-n" => next_session = true,
            "-p" => prev_session = true,
            "-t" if i + 1 < args.len() => target = Some(args[i + 1].to_string()),
            _ => {}
        }
    }

    let new_session = if let Some(t) = target {
        t
    } else if next_session {
        sessions[(current_idx + 1) % sessions.len()].clone()
    } else if prev_session {
        sessions[(current_idx + sessions.len() - 1) % sessions.len()].clone()
    } else {
        return Err(JshError::runtime("No target specified"));
    };

    server.attach_session(&new_session)?;
    Ok(format!("Switched to: {}", new_session))
}

fn cmd_has_session(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let target = args.iter()
        .skip_while(|a| *a != &"-t")
        .nth(1)
        .map(|s| s.to_string());

    if let Some(name) = target {
        if server.sessions.contains_key(&name) {
            Ok(String::new())
        } else {
            Err(JshError::runtime(format!("Session not found: {}", name)))
        }
    } else {
        Err(JshError::runtime("No target specified"))
    }
}

// Window commands

fn cmd_new_window(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let mut name = None;
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-n" && i + 1 < args.len() {
            name = Some(args[i + 1]);
        }
    }

    let mut s = session.lock().unwrap();
    let window = s.new_window(name);
    let w = window.lock().unwrap();
    
    Ok(format!("Created window: {}:{}", s.name, w.index))
}

fn cmd_kill_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let mut s = session.lock().unwrap();
    let mut target = s.current_window;
    
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-t" && i + 1 < args.len() {
            if let Ok(idx) = args[i + 1].parse() {
                target = idx;
            }
        }
    }

    if s.kill_window(target) {
        Ok(format!("Killed window: {}", target))
    } else {
        Err(JshError::runtime("Cannot kill window"))
    }
}

fn cmd_rename_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;

    let new_name = args.last()
        .ok_or_else(|| JshError::runtime("No name specified"))?;

    let mut s = session.lock().unwrap();
    let current = s.current_window;
    s.rename_window(current, new_name);
    Ok(format!("Renamed window to: {}", new_name))
}

fn cmd_list_windows(server: &TmuxServer, _args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    let windows = s.list_windows();
    
    let output: Vec<String> = windows.iter()
        .map(|w| w.to_string())
        .collect();
    
    Ok(output.join("\n"))
}

fn cmd_select_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;

    let mut target = None;
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-t" && i + 1 < args.len() {
            target = args[i + 1].parse().ok();
        } else if !arg.starts_with('-') {
            target = arg.parse().ok();
        }
    }

    if let Some(idx) = target {
        let mut s = session.lock().unwrap();
        if s.select_window(idx) {
            Ok(format!("Selected window: {}", idx))
        } else {
            Err(JshError::runtime(format!("Window not found: {}", idx)))
        }
    } else {
        Err(JshError::runtime("No target specified"))
    }
}

fn cmd_next_window(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let mut s = session.lock().unwrap();
    s.next_window();
    Ok(format!("Window: {}", s.current_window))
}

fn cmd_previous_window(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let mut s = session.lock().unwrap();
    s.previous_window();
    Ok(format!("Window: {}", s.current_window))
}

fn cmd_last_window(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let mut s = session.lock().unwrap();
    if s.select_last_window() {
        Ok(format!("Window: {}", s.current_window))
    } else {
        Err(JshError::runtime("No last window"))
    }
}

fn cmd_rotate_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let clockwise = !args.contains(&"-U");
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        window.lock().unwrap().rotate_panes(clockwise);
        Ok("Rotated panes".to_string())
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_swap_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("swap-window not fully implemented".to_string())
}

fn cmd_move_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("move-window not fully implemented".to_string())
}

fn cmd_link_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("link-window not fully implemented".to_string())
}

fn cmd_unlink_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("unlink-window not fully implemented".to_string())
}

fn cmd_find_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("find-window not fully implemented".to_string())
}

// Pane commands

fn cmd_split_window(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let horizontal = args.contains(&"-h");
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        if horizontal {
            w.split_vertical(None);
        } else {
            w.split_horizontal(None);
        }
        Ok("Split window".to_string())
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_kill_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        
        let mut target = w.active_pane;
        for (i, arg) in args.iter().enumerate() {
            if *arg == "-t" && i + 1 < args.len() {
                if let Ok(idx) = args[i + 1].parse() {
                    target = idx;
                }
            }
        }

        if w.kill_pane(target) {
            Ok("Killed pane".to_string())
        } else {
            Err(JshError::runtime("Cannot kill pane"))
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_select_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();

        // Check for direction flags
        if args.contains(&"-U") {
            // Select pane above (simplified - just previous)
            w.previous_pane();
        } else if args.contains(&"-D") {
            // Select pane below
            w.next_pane();
        } else if args.contains(&"-L") {
            w.previous_pane();
        } else if args.contains(&"-R") {
            w.next_pane();
        } else {
            // Target pane number
            for (i, arg) in args.iter().enumerate() {
                if *arg == "-t" && i + 1 < args.len() {
                    if let Ok(idx) = args[i + 1].parse() {
                        w.select_pane(idx);
                    }
                }
            }
        }

        Ok(format!("Selected pane: {}", w.active_pane))
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_last_pane(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        w.select_last_pane();
        Ok(format!("Pane: {}", w.active_pane))
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_resize_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        let pane_id = w.active_pane;

        let mut amount = 1i16;
        let mut direction = None;

        for (i, arg) in args.iter().enumerate() {
            match *arg {
                "-U" => direction = Some(ResizeDirection::Up),
                "-D" => direction = Some(ResizeDirection::Down),
                "-L" => direction = Some(ResizeDirection::Left),
                "-R" => direction = Some(ResizeDirection::Right),
                "-Z" => {
                    // Toggle zoom (not fully implemented)
                    return Ok("Toggled zoom".to_string());
                }
                s if !s.starts_with('-') => {
                    if let Ok(n) = s.parse::<i16>() {
                        amount = n;
                    }
                }
                _ if i + 1 < args.len() && !args[i + 1].starts_with('-') => {
                    if let Ok(n) = args[i + 1].parse::<i16>() {
                        amount = n;
                    }
                }
                _ => {}
            }
        }

        if let Some(dir) = direction {
            w.resize_pane(pane_id, dir, amount);
            Ok("Resized pane".to_string())
        } else {
            Ok("No direction specified".to_string())
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_swap_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("swap-pane not fully implemented".to_string())
}

fn cmd_break_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("break-pane not fully implemented".to_string())
}

fn cmd_join_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("join-pane not fully implemented".to_string())
}

fn cmd_capture_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let w = window.lock().unwrap();
        if let Some(pane) = w.active_pane() {
            let p = pane.lock().unwrap();
            
            let mut start = None;
            let mut end = None;
            
            for (i, arg) in args.iter().enumerate() {
                if *arg == "-S" && i + 1 < args.len() {
                    start = args[i + 1].parse().ok();
                }
                if *arg == "-E" && i + 1 < args.len() {
                    end = args[i + 1].parse().ok();
                }
            }

            Ok(p.capture(start, end))
        } else {
            Err(JshError::runtime("No active pane"))
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_display_panes(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let w = window.lock().unwrap();
        let panes = w.list_panes();
        
        let output: Vec<String> = panes.iter()
            .map(|p| p.to_string())
            .collect();
        
        Ok(output.join("\n"))
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_pipe_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("pipe-pane not fully implemented".to_string())
}

fn cmd_respawn_pane(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let _ = (server, args);
    Ok("respawn-pane not fully implemented".to_string())
}

fn cmd_list_panes(server: &TmuxServer, _args: &[&str]) -> Result<String> {
    cmd_display_panes(server)
}

// Layout commands

fn cmd_select_layout(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        
        let layout = args.iter()
            .find(|a| !a.starts_with('-'))
            .unwrap_or(&"even-horizontal");
        
        w.set_layout(layout);
        Ok(format!("Set layout: {}", layout))
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_next_layout(server: &TmuxServer) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let mut w = window.lock().unwrap();
        w.next_layout();
        Ok(format!("Layout: {}", w.layout))
    } else {
        Err(JshError::runtime("No current window"))
    }
}

// Configuration commands

fn cmd_set_option(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let mut args = args.iter().peekable();
    
    // Skip flags
    while args.peek().map(|s| s.starts_with('-')).unwrap_or(false) {
        args.next();
    }

    let option = args.next().ok_or_else(|| JshError::runtime("No option specified"))?;
    let value = args.next().map(|s| *s).unwrap_or("");

    server.config.set_option(option, value)?;
    Ok(format!("Set {} = {}", option, value))
}

fn cmd_set_window_option(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    cmd_set_option(server, args)
}

fn cmd_show_options(server: &TmuxServer, args: &[&str]) -> Result<String> {
    if let Some(option) = args.iter().find(|a| !a.starts_with('-')) {
        if let Some(value) = server.config.get_option(option) {
            Ok(format!("{} {}", option, value))
        } else {
            Err(JshError::runtime(format!("Unknown option: {}", option)))
        }
    } else {
        // Show all options
        let mut output = String::new();
        output.push_str(&format!("prefix {}\n", server.config.prefix));
        output.push_str(&format!("base-index {}\n", server.config.base_index));
        output.push_str(&format!("history-limit {}\n", server.config.history_limit));
        output.push_str(&format!("mouse {}\n", if server.config.mouse { "on" } else { "off" }));
        output.push_str(&format!("status {}\n", server.config.status));
        Ok(output)
    }
}

fn cmd_source_file(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let path_str = args.iter()
        .find(|a| !a.starts_with('-'))
        .ok_or_else(|| JshError::runtime("No file specified"))?;
    
    // Manual tilde expansion
    let expanded = if path_str.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            path_str.replacen('~', &home, 1)
        } else {
            path_str.to_string()
        }
    } else {
        path_str.to_string()
    };
    
    let path = std::path::PathBuf::from(&expanded);
    
    server.load_config(Some(&path))?;
    Ok(format!("Sourced: {}", path.display()))
}

// Key binding commands

fn cmd_bind_key(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    server.config.parse_command(&format!("bind {}", args.join(" ")))
        .map(|_| "Key bound".to_string())
}

fn cmd_unbind_key(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    server.config.parse_command(&format!("unbind {}", args.join(" ")))
        .map(|_| "Key unbound".to_string())
}

fn cmd_list_keys(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let table_name = args.iter()
        .skip_while(|a| *a != &"-T")
        .nth(1)
        .map(|s| *s)
        .unwrap_or("prefix");

    if let Some(table) = server.config.key_tables.get(table_name) {
        let mut output = String::new();
        for binding in table.list() {
            let repeat = if binding.repeat { "-r " } else { "" };
            output.push_str(&format!("bind-key {}{} {}\n", repeat, binding.key, binding.command));
        }
        Ok(output)
    } else {
        Err(JshError::runtime(format!("Unknown key table: {}", table_name)))
    }
}

// Copy/paste commands

fn cmd_copy_mode(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let vi_mode = args.contains(&"-e");
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let w = window.lock().unwrap();
        if let Some(pane) = w.active_pane() {
            pane.lock().unwrap().enter_copy_mode(!vi_mode);
            Ok("Entered copy mode".to_string())
        } else {
            Err(JshError::runtime("No active pane"))
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_paste_buffer(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("paste-buffer not fully implemented".to_string())
}

fn cmd_list_buffers(_server: &TmuxServer) -> Result<String> {
    Ok("No paste buffers".to_string())
}

fn cmd_set_buffer(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("set-buffer not fully implemented".to_string())
}

fn cmd_show_buffer(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("No buffer content".to_string())
}

fn cmd_delete_buffer(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("delete-buffer not fully implemented".to_string())
}

// Input commands

fn cmd_send_keys(server: &TmuxServer, args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let w = window.lock().unwrap();
        if let Some(pane) = w.active_pane() {
            let literal = args.contains(&"-l");
            let keys = args.iter()
                .filter(|a| !a.starts_with('-'))
                .map(|s| *s)
                .collect::<Vec<_>>()
                .join(" ");
            
            pane.lock().unwrap().send_keys(&keys, literal);
            Ok("Keys sent".to_string())
        } else {
            Err(JshError::runtime("No active pane"))
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_send_prefix(server: &TmuxServer) -> Result<String> {
    cmd_send_keys(server, &[&server.config.prefix])
}

// Display commands

fn cmd_display_message(_server: &TmuxServer, args: &[&str]) -> Result<String> {
    let message = args.iter()
        .filter(|a| !a.starts_with('-'))
        .map(|s| *s)
        .collect::<Vec<_>>()
        .join(" ");
    
    Ok(message)
}

fn cmd_clock_mode(_server: &TmuxServer) -> Result<String> {
    use std::io::Write;
    
    let now = chrono::Local::now();
    let time = now.format("%H:%M:%S").to_string();
    
    // Simple ASCII art clock display
    let output = format!("\n    {}\n", time);
    print!("{}", output);
    std::io::stdout().flush().ok();
    
    Ok(time)
}

fn cmd_show_messages(_server: &TmuxServer) -> Result<String> {
    Ok("No messages".to_string())
}

// Misc commands

fn cmd_run_shell(args: &[&str]) -> Result<String> {
    let command = args.iter()
        .filter(|a| !a.starts_with('-'))
        .map(|s| *s)
        .collect::<Vec<_>>()
        .join(" ");
    
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| JshError::runtime(format!("Failed to run: {}", e)))?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn cmd_confirm_before(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("confirm-before not fully implemented".to_string())
}

fn cmd_command_prompt(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("command-prompt not fully implemented".to_string())
}

fn cmd_choose_tree(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("choose-tree not fully implemented".to_string())
}

fn cmd_choose_buffer(_server: &TmuxServer) -> Result<String> {
    Ok("choose-buffer not fully implemented".to_string())
}

fn cmd_if_shell(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("if-shell not fully implemented".to_string())
}

fn cmd_wait_for(_server: &TmuxServer, _args: &[&str]) -> Result<String> {
    Ok("wait-for not fully implemented".to_string())
}

fn cmd_lock_server(_server: &TmuxServer) -> Result<String> {
    Ok("lock-server not fully implemented".to_string())
}

fn cmd_lock_session(_server: &TmuxServer) -> Result<String> {
    Ok("lock-session not fully implemented".to_string())
}

fn cmd_lock_client(_server: &TmuxServer) -> Result<String> {
    Ok("lock-client not fully implemented".to_string())
}

fn cmd_refresh_client(_server: &TmuxServer) -> Result<String> {
    Ok("Refreshed".to_string())
}

fn cmd_suspend_client(_server: &TmuxServer) -> Result<String> {
    Ok("suspend-client not fully implemented".to_string())
}

fn cmd_clear_history(server: &TmuxServer, _args: &[&str]) -> Result<String> {
    let session = server.current()
        .ok_or_else(|| JshError::runtime("No current session"))?;
    
    let s = session.lock().unwrap();
    if let Some(window) = s.current_window() {
        let w = window.lock().unwrap();
        if let Some(pane) = w.active_pane() {
            pane.lock().unwrap().clear_history();
            Ok("History cleared".to_string())
        } else {
            Err(JshError::runtime("No active pane"))
        }
    } else {
        Err(JshError::runtime("No current window"))
    }
}

fn cmd_info(server: &TmuxServer) -> Result<String> {
    let mut info = String::new();
    info.push_str(&format!("Sessions: {}\n", server.sessions.len()));
    info.push_str(&format!("Current: {:?}\n", server.current_session));
    info.push_str(&format!("Socket: {}\n", server.socket_path.display()));
    info.push_str(&format!("Uptime: {:?}\n", server.uptime()));
    Ok(info)
}

fn cmd_server_info(server: &TmuxServer) -> Result<String> {
    cmd_info(server)
}

fn cmd_start_server(_server: &TmuxServer) -> Result<String> {
    Ok("Server started".to_string())
}

fn cmd_kill_server(_server: &mut TmuxServer) -> Result<String> {
    Ok("kill-server: use exit to terminate".to_string())
}

// Plugin commands

fn cmd_tpm_install(server: &TmuxServer) -> Result<String> {
    let installed = server.plugins.install_all()?;
    Ok(format!("Installed {} plugins", installed.len()))
}

fn cmd_tpm_update(server: &TmuxServer) -> Result<String> {
    let updated = server.plugins.update_all()?;
    Ok(format!("Updated {} plugins", updated.len()))
}

fn cmd_tpm_clean(server: &TmuxServer) -> Result<String> {
    let removed = server.plugins.clean()?;
    Ok(format!("Removed {} plugins", removed.len()))
}

fn cmd_tpm_list(server: &TmuxServer) -> Result<String> {
    let installed = server.plugins.list_installed();
    if installed.is_empty() {
        Ok("No plugins installed".to_string())
    } else {
        Ok(installed.join("\n"))
    }
}

// Theme commands

fn cmd_set_theme(server: &mut TmuxServer, args: &[&str]) -> Result<String> {
    let theme_name = args.first()
        .ok_or_else(|| JshError::runtime("No theme specified"))?;
    
    if let Some(theme) = StatusBarTheme::from_name(theme_name) {
        // Apply theme settings
        server.config.status_style = theme.status_style;
        server.config.status_left = theme.status_left.into();
        server.config.status_right = theme.status_right.into();
        server.config.window_status_format = theme.window_status_format;
        server.config.window_status_current_format = theme.window_status_current_format;
        server.config.window_status_style = theme.window_status_style;
        server.config.window_status_current_style = theme.window_status_current_style;
        server.config.pane_border_style = theme.pane_border_style;
        server.config.pane_active_border_style = theme.pane_active_border_style;
        server.config.message_style = theme.message_style;
        server.config.mode_style = theme.mode_style;
        
        Ok(format!("Applied theme: {}", theme_name))
    } else {
        Err(JshError::runtime(format!("Unknown theme: {}. Available: {}", 
            theme_name, StatusBarTheme::list_themes().join(", "))))
    }
}

fn cmd_list_themes() -> Result<String> {
    Ok(StatusBarTheme::list_themes().join("\n"))
}

impl From<String> for super::FormatSpec {
    fn from(s: String) -> Self {
        super::FormatSpec::new(&s)
    }
}

