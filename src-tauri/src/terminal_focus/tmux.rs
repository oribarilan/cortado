use std::process::Command;

use super::{FocusContext, FocusResult};

/// tmux focus strategy: switches the tmux client to the exact pane
/// containing the copilot session.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    if ctx.tmux_server_pid.is_none() {
        return FocusResult::NotApplicable;
    }

    // Find the pane containing an ancestor of the copilot process.
    let panes = match list_panes() {
        Ok(panes) => panes,
        Err(e) => return FocusResult::Failed(e),
    };

    let target_pane = match find_target_pane(&panes, &ctx.ancestors, ctx.copilot_pid) {
        Some(pane) => pane,
        None => return FocusResult::Failed("no tmux pane matches copilot process ancestry".into()),
    };

    // Find a client to switch.
    let clients = match list_clients() {
        Ok(clients) => clients,
        Err(e) => return FocusResult::Failed(e),
    };

    if clients.is_empty() {
        return FocusResult::Failed("no tmux clients attached".into());
    }

    let target_session = target_pane.pane_id.split(':').next().unwrap_or("");
    let client = pick_best_client(&clients, target_session);

    // Switch client to the target pane.
    if let Err(e) = switch_to_pane(&client.client_tty, &target_pane.pane_id) {
        return FocusResult::Failed(e);
    }

    // Activate the terminal app.
    if let Some(terminal_pid) = ctx.terminal_app_pid {
        let _ = super::activate_app_by_pid(terminal_pid);
    }

    FocusResult::Focused
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

/// Picks the best client: prefer one already attached to the target session.
fn pick_best_client<'a>(clients: &'a [TmuxClient], target_session: &str) -> &'a TmuxClient {
    clients
        .iter()
        .find(|c| c.client_session == target_session)
        .unwrap_or(&clients[0])
}

/// Switches a tmux client to the given pane.
fn switch_to_pane(client_tty: &str, pane_id: &str) -> Result<(), String> {
    let switch = Command::new("tmux")
        .args(["switch-client", "-c", client_tty, "-t", pane_id])
        .output()
        .map_err(|e| format!("failed to run tmux switch-client: {e}"))?;

    if !switch.status.success() {
        let stderr = String::from_utf8_lossy(&switch.stderr);
        return Err(format!("tmux switch-client failed: {}", stderr.trim()));
    }

    let select = Command::new("tmux")
        .args(["select-pane", "-t", pane_id])
        .output()
        .map_err(|e| format!("failed to run tmux select-pane: {e}"))?;

    if !select.status.success() {
        let stderr = String::from_utf8_lossy(&select.stderr);
        return Err(format!("tmux select-pane failed: {}", stderr.trim()));
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
    fn pick_best_client_prefers_target_session() {
        let clients = vec![
            TmuxClient {
                client_tty: "/dev/ttys001".to_string(),
                client_session: "work".to_string(),
            },
            TmuxClient {
                client_tty: "/dev/ttys002".to_string(),
                client_session: "main".to_string(),
            },
        ];

        let best = pick_best_client(&clients, "main");
        assert_eq!(best.client_tty, "/dev/ttys002");
    }

    #[test]
    fn pick_best_client_falls_back_to_first() {
        let clients = vec![TmuxClient {
            client_tty: "/dev/ttys001".to_string(),
            client_session: "work".to_string(),
        }];

        let best = pick_best_client(&clients, "nonexistent");
        assert_eq!(best.client_tty, "/dev/ttys001");
    }

    #[test]
    fn try_focus_not_applicable_without_tmux() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: None,
            terminal_app_name: None,
            terminal_app_bundle: None,
        };
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }
}
