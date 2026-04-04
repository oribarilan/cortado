use std::process::Command;

use super::FocusContext;

/// tmux pane navigation: navigates to the exact pane containing the copilot session.
///
/// This is a pre-step, not a competing strategy — it handles tmux-level navigation
/// while terminal strategies handle app-level focus (tab switching, activation).
///
/// When the target session already has a client attached, uses select-window + select-pane
/// to navigate within it (non-destructive). Only uses switch-client when the target session
/// has no attached client.
///
/// Returns `Ok(true)` if navigation succeeded, `Ok(false)` if tmux is not applicable.
pub fn try_navigate(ctx: &FocusContext) -> Result<bool, String> {
    if ctx.tmux_server_pid.is_none() {
        return Ok(false);
    }

    let panes = list_panes()?;

    let target_pane = find_target_pane(&panes, &ctx.ancestors, ctx.copilot_pid)
        .ok_or_else(|| "no tmux pane matches copilot process ancestry".to_string())?;

    let clients = list_clients()?;

    if clients.is_empty() {
        return Err("no tmux clients attached".into());
    }

    let target_session = target_pane.pane_id.split(':').next().unwrap_or("");
    let has_own_client = clients.iter().any(|c| c.client_session == target_session);

    if has_own_client {
        // Target session already has a client viewing it — just navigate within it.
        select_within_session(&target_pane.pane_id)?;
    } else {
        // No client on this session — switch an existing client to it.
        let client = &clients[0];
        switch_client_to_pane(&client.client_tty, &target_pane.pane_id)?;
    }

    Ok(true)
}

/// A parsed tmux pane entry.
#[derive(Debug, Clone)]
struct TmuxPane {
    /// Full pane identifier (e.g., "main:0.1").
    pane_id: String,
    /// PID of the pane's shell process.
    pane_pid: u32,
}

/// A parsed tmux client entry.
#[derive(Debug, Clone)]
struct TmuxClient {
    /// Client TTY (e.g., "/dev/ttys001").
    client_tty: String,
    /// Session this client is attached to.
    client_session: String,
}

/// Runs `tmux list-panes -a` and parses the output.
fn list_panes() -> Result<Vec<TmuxPane>, String> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{session_name}:#{window_index}.#{pane_index} #{pane_pid}",
        ])
        .output()
        .map_err(|e| format!("failed to run tmux list-panes: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux list-panes failed: {}", stderr.trim()));
    }

    Ok(parse_panes_output(&String::from_utf8_lossy(&output.stdout)))
}

/// Parses `tmux list-panes` output into structured entries.
fn parse_panes_output(output: &str) -> Vec<TmuxPane> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (pane_id, pid_str) = line.rsplit_once(' ')?;
            let pane_pid = pid_str.parse().ok()?;
            Some(TmuxPane {
                pane_id: pane_id.to_string(),
                pane_pid,
            })
        })
        .collect()
}

/// Runs `tmux list-clients` and parses the output.
fn list_clients() -> Result<Vec<TmuxClient>, String> {
    let output = Command::new("tmux")
        .args(["list-clients", "-F", "#{client_tty} #{client_session}"])
        .output()
        .map_err(|e| format!("failed to run tmux list-clients: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux list-clients failed: {}", stderr.trim()));
    }

    Ok(parse_clients_output(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

/// Parses `tmux list-clients` output into structured entries.
fn parse_clients_output(output: &str) -> Vec<TmuxClient> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (tty, session) = line.split_once(' ')?;
            Some(TmuxClient {
                client_tty: tty.to_string(),
                client_session: session.to_string(),
            })
        })
        .collect()
}

/// Finds the tmux pane containing the copilot process (or its ancestor).
fn find_target_pane(panes: &[TmuxPane], ancestors: &[u32], copilot_pid: u32) -> Option<TmuxPane> {
    // Direct match on copilot PID.
    if let Some(pane) = panes.iter().find(|p| p.pane_pid == copilot_pid) {
        return Some(pane.clone());
    }

    // Match on ancestors (shell -> copilot).
    for ancestor in ancestors {
        if let Some(pane) = panes.iter().find(|p| p.pane_pid == *ancestor) {
            return Some(pane.clone());
        }
    }

    None
}

/// Navigates within a session that already has a client: select window + pane.
fn select_within_session(pane_id: &str) -> Result<(), String> {
    // Extract session:window from the full pane_id (e.g., "main:0.1" -> "main:0").
    let window_id = pane_id.rsplit_once('.').map(|(w, _)| w).unwrap_or(pane_id);

    run_tmux(&["select-window", "-t", window_id])?;
    run_tmux(&["select-pane", "-t", pane_id])?;
    Ok(())
}

/// Switches a client to a different session/pane (used when target has no client).
fn switch_client_to_pane(client_tty: &str, pane_id: &str) -> Result<(), String> {
    run_tmux(&["switch-client", "-c", client_tty, "-t", pane_id])?;
    run_tmux(&["select-pane", "-t", pane_id])?;
    Ok(())
}

fn run_tmux(args: &[&str]) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run tmux {}: {e}", args[0]))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux {} failed: {}", args[0], stderr.trim()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_panes_valid_output() {
        let output = "main:0.0 12345\nmain:0.1 67890\nwork:1.0 11111\n";
        let panes = parse_panes_output(output);
        assert_eq!(panes.len(), 3);
        assert_eq!(panes[0].pane_id, "main:0.0");
        assert_eq!(panes[0].pane_pid, 12345);
        assert_eq!(panes[1].pane_id, "main:0.1");
        assert_eq!(panes[1].pane_pid, 67890);
        assert_eq!(panes[2].pane_id, "work:1.0");
        assert_eq!(panes[2].pane_pid, 11111);
    }

    #[test]
    fn parse_panes_empty_output() {
        let panes = parse_panes_output("");
        assert!(panes.is_empty());
    }

    #[test]
    fn parse_panes_ignores_malformed_lines() {
        let output = "main:0.0 12345\nbadline\nmain:0.1 notanumber\n";
        let panes = parse_panes_output(output);
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].pane_id, "main:0.0");
    }

    #[test]
    fn parse_clients_valid_output() {
        let output = "/dev/ttys001 main\n/dev/ttys002 work\n";
        let clients = parse_clients_output(output);
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].client_tty, "/dev/ttys001");
        assert_eq!(clients[0].client_session, "main");
    }

    #[test]
    fn parse_clients_empty_output() {
        let clients = parse_clients_output("");
        assert!(clients.is_empty());
    }

    #[test]
    fn find_target_pane_direct_match() {
        let panes = vec![
            TmuxPane {
                pane_id: "main:0.0".to_string(),
                pane_pid: 100,
            },
            TmuxPane {
                pane_id: "main:0.1".to_string(),
                pane_pid: 200,
            },
        ];

        let result = find_target_pane(&panes, &[], 200);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pane_id, "main:0.1");
    }

    #[test]
    fn find_target_pane_ancestor_match() {
        let panes = vec![TmuxPane {
            pane_id: "main:0.0".to_string(),
            pane_pid: 100,
        }];

        // copilot_pid=300 is not in panes, but ancestor 100 is.
        let result = find_target_pane(&panes, &[200, 100], 300);
        assert!(result.is_some());
        assert_eq!(result.unwrap().pane_pid, 100);
    }

    #[test]
    fn find_target_pane_no_match() {
        let panes = vec![TmuxPane {
            pane_id: "main:0.0".to_string(),
            pane_pid: 100,
        }];

        let result = find_target_pane(&panes, &[200, 300], 400);
        assert!(result.is_none());
    }

    #[test]
    fn find_target_pane_prefers_direct_over_ancestor() {
        let panes = vec![
            TmuxPane {
                pane_id: "main:0.0".to_string(),
                pane_pid: 100, // ancestor
            },
            TmuxPane {
                pane_id: "main:0.1".to_string(),
                pane_pid: 300, // direct match
            },
        ];

        let result = find_target_pane(&panes, &[100], 300);
        assert_eq!(result.unwrap().pane_id, "main:0.1");
    }

    #[test]
    fn find_target_pane_ancestor_priority_order() {
        // First ancestor in the list should match first.
        let panes = vec![
            TmuxPane {
                pane_id: "a:0.0".to_string(),
                pane_pid: 50,
            },
            TmuxPane {
                pane_id: "b:0.0".to_string(),
                pane_pid: 60,
            },
        ];

        // ancestors: [60, 50] — 60 comes first, should match first.
        let result = find_target_pane(&panes, &[60, 50], 999);
        assert_eq!(result.unwrap().pane_pid, 60);
    }

    #[test]
    fn parse_clients_with_named_sessions() {
        let output = "/dev/ttys001 my-project\n/dev/ttys002 dotfiles\n";
        let clients = parse_clients_output(output);
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].client_session, "my-project");
        assert_eq!(clients[1].client_session, "dotfiles");
    }

    #[test]
    fn try_navigate_not_applicable_without_tmux() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: None,
            terminal_app_name: None,
            terminal_app_bundle: None,
        };
        assert_eq!(try_navigate(&ctx).unwrap(), false);
    }

    #[test]
    fn has_own_client_detection() {
        let clients = [
            TmuxClient {
                client_tty: "/dev/ttys001".to_string(),
                client_session: "work".to_string(),
            },
            TmuxClient {
                client_tty: "/dev/ttys002".to_string(),
                client_session: "personal".to_string(),
            },
        ];

        assert!(clients.iter().any(|c| c.client_session == "work"));
        assert!(clients.iter().any(|c| c.client_session == "personal"));
        assert!(!clients.iter().any(|c| c.client_session == "nonexistent"));
    }

    #[test]
    fn parse_panes_with_named_sessions() {
        let output = "my-project:0.0 12345\ndotfiles:0.0 67890\n";
        let panes = parse_panes_output(output);
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].pane_id, "my-project:0.0");
        assert_eq!(panes[1].pane_id, "dotfiles:0.0");
    }

    #[test]
    fn parse_panes_with_spaces_in_session_name() {
        // Session names with spaces shouldn't happen in practice,
        // but our parser splits on last space so it should still work.
        let output = "my project:0.0 12345\n";
        let panes = parse_panes_output(output);
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].pane_id, "my project:0.0");
        assert_eq!(panes[0].pane_pid, 12345);
    }

    #[test]
    fn find_target_pane_empty_panes() {
        let result = find_target_pane(&[], &[100, 200], 300);
        assert!(result.is_none());
    }

    #[test]
    fn find_target_pane_empty_ancestors() {
        let panes = [TmuxPane {
            pane_id: "main:0.0".to_string(),
            pane_pid: 100,
        }];
        // No direct match, no ancestors to check.
        let result = find_target_pane(&panes, &[], 999);
        assert!(result.is_none());
    }
}
