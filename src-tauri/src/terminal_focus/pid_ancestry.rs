use crate::feed::harness::SessionInfo;

use super::FocusContext;

/// Builds a `FocusContext` from a `SessionInfo` by walking the PID ancestry tree.
pub fn build_focus_context(session: &SessionInfo) -> Result<FocusContext, String> {
    let mut ancestors = Vec::new();
    let mut tmux_server_pid = None;
    let mut terminal_app_pid = None;
    let mut terminal_app_name = None;
    let mut terminal_app_bundle = None;

    let mut current_pid = session.pid;

    // Walk up the process tree until we hit PID 1 or can't go further.
    for _ in 0..64 {
        let parent = match get_parent_pid(current_pid) {
            Some(ppid) if ppid != current_pid && ppid != 0 => ppid,
            _ => break,
        };

        ancestors.push(parent);

        // Detect tmux server in ancestry.
        if tmux_server_pid.is_none() {
            if let Some(name) = get_process_name(parent) {
                if name == "tmux: server" || name.starts_with("tmux") {
                    tmux_server_pid = Some(parent);
                }

                // Detect terminal app by process name.
                if terminal_app_pid.is_none() {
                    if let Some((display, bundle)) = is_terminal_app(&name) {
                        terminal_app_pid = Some(parent);
                        terminal_app_name = Some(display.to_string());
                        terminal_app_bundle = Some(bundle.to_string());
                    }
                }
            }
        } else if terminal_app_pid.is_none() {
            // Already past tmux -- still look for terminal.
            if let Some(name) = get_process_name(parent) {
                if let Some((display, bundle)) = is_terminal_app(&name) {
                    terminal_app_pid = Some(parent);
                    terminal_app_name = Some(display.to_string());
                    terminal_app_bundle = Some(bundle.to_string());
                }
            }
        }

        if parent == 1 {
            break;
        }

        current_pid = parent;
    }

    // If tmux was detected but no terminal app found via direct ancestry,
    // find the tmux client attached to the copilot's session and walk its ancestry.
    if tmux_server_pid.is_some() && terminal_app_pid.is_none() {
        if let Some(client_pid) = find_tmux_client_for_session(session.pid, &ancestors) {
            let mut pid = client_pid;
            for _ in 0..32 {
                let parent = match get_parent_pid(pid) {
                    Some(ppid) if ppid != pid && ppid != 0 => ppid,
                    _ => break,
                };

                if let Some(name) = get_process_name(parent) {
                    if let Some((display, bundle)) = is_terminal_app(&name) {
                        terminal_app_pid = Some(parent);
                        terminal_app_name = Some(display.to_string());
                        terminal_app_bundle = Some(bundle.to_string());
                        break;
                    }
                }

                if parent == 1 {
                    break;
                }
                pid = parent;
            }
        }
    }

    Ok(FocusContext {
        copilot_pid: session.pid,
        cwd: session.cwd.clone(),
        ancestors,
        tmux_server_pid,
        terminal_app_pid,
        terminal_app_name,
        terminal_app_bundle,
    })
}

/// Gets the parent PID of a process via `ps`.
fn get_parent_pid(pid: u32) -> Option<u32> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "ppid="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

/// Gets the process name via `ps`.
fn get_process_name(pid: u32) -> Option<String> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() {
        None
    } else {
        // ps returns the full path; extract just the command name.
        Some(name.rsplit('/').next().unwrap_or(&name).to_string())
    }
}

/// Known terminal app process names and their display names / bundle IDs.
const KNOWN_TERMINALS: &[(&str, &str, &str)] = &[
    ("cmux", "cmux", "com.cmuxterm.app"),
    ("ghostty", "Ghostty", "com.mitchellh.ghostty"),
    ("Ghostty", "Ghostty", "com.mitchellh.ghostty"),
    ("iTerm2", "iTerm2", "com.googlecode.iterm2"),
    ("Terminal", "Terminal", "com.apple.Terminal"),
    ("Alacritty", "Alacritty", "io.alacritty"),
    ("kitty", "kitty", "net.kovidgoyal.kitty"),
    ("WezTerm", "WezTerm", "com.github.wez.wezterm"),
    ("wezterm-gui", "WezTerm", "com.github.wez.wezterm"),
];

/// Checks if a process name matches a known terminal app.
/// Returns (display name, bundle ID) if it matches.
fn is_terminal_app(process_name: &str) -> Option<(&'static str, &'static str)> {
    KNOWN_TERMINALS
        .iter()
        .find(|(name, _, _)| *name == process_name)
        .map(|(_, display, bundle)| (*display, *bundle))
}

/// Finds the PID of the tmux client attached to the session containing `copilot_pid`.
fn find_tmux_client_for_session(copilot_pid: u32, ancestors: &[u32]) -> Option<u32> {
    use std::process::Command;

    // First, find which tmux session contains the copilot process.
    let panes_output = Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{session_name} #{pane_pid}"])
        .output()
        .ok()?;

    if !panes_output.status.success() {
        return None;
    }

    let panes_stdout = String::from_utf8_lossy(&panes_output.stdout);
    let target_session = panes_stdout.lines().find_map(|line| {
        let (session, pid_str) = line.rsplit_once(' ')?;
        let pid: u32 = pid_str.parse().ok()?;
        if pid == copilot_pid || ancestors.contains(&pid) {
            Some(session.to_string())
        } else {
            None
        }
    })?;

    // Find the client attached to that session, falling back to any client.
    let clients_output = Command::new("tmux")
        .args(["list-clients", "-F", "#{client_pid} #{client_session}"])
        .output()
        .ok()?;

    if !clients_output.status.success() {
        return None;
    }

    let clients_stdout = String::from_utf8_lossy(&clients_output.stdout);

    // Prefer the client attached to the target session.
    let exact_match = clients_stdout.lines().find_map(|line| {
        let (pid_str, session) = line.split_once(' ')?;
        if session == target_session {
            pid_str.parse().ok()
        } else {
            None
        }
    });

    if exact_match.is_some() {
        return exact_match;
    }

    // Fallback: any client (so we can at least detect the terminal app).
    clients_stdout.lines().find_map(|line| {
        let (pid_str, _session) = line.split_once(' ')?;
        pid_str.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_parent_pid_of_current_process() {
        let pid = std::process::id();
        let ppid = get_parent_pid(pid);
        assert!(ppid.is_some());
        assert!(ppid.unwrap() > 0);
    }

    #[test]
    fn get_parent_pid_nonexistent() {
        let ppid = get_parent_pid(99_999_999);
        assert!(ppid.is_none());
    }

    #[test]
    fn get_process_name_of_launchd() {
        let name = get_process_name(1);
        assert!(name.is_some());
        assert_eq!(name.unwrap(), "launchd");
    }

    #[test]
    fn get_process_name_nonexistent() {
        let name = get_process_name(99_999_999);
        assert!(name.is_none());
    }

    #[test]
    fn build_context_for_current_process() {
        let session = SessionInfo {
            id: "test".to_string(),
            cwd: "/tmp".to_string(),
            repository: None,
            branch: None,
            status: crate::feed::harness::SessionStatus::Idle,
            pid: std::process::id(),
            summary: None,
            last_active_at: None,
        };

        let ctx = build_focus_context(&session).expect("should build context");
        assert_eq!(ctx.copilot_pid, std::process::id());
        assert_eq!(ctx.cwd, "/tmp");
        assert!(!ctx.ancestors.is_empty());
    }

    // --- Known terminal detection ---

    #[test]
    fn is_terminal_app_cmux() {
        let result = is_terminal_app("cmux");
        assert!(result.is_some());
        let (name, bundle) = result.unwrap();
        assert_eq!(name, "cmux");
        assert_eq!(bundle, "com.cmuxterm.app");
    }

    #[test]
    fn is_terminal_app_ghostty() {
        let result = is_terminal_app("ghostty");
        assert!(result.is_some());
        let (name, bundle) = result.unwrap();
        assert_eq!(name, "Ghostty");
        assert_eq!(bundle, "com.mitchellh.ghostty");
    }

    #[test]
    fn is_terminal_app_ghostty_capitalized() {
        assert!(is_terminal_app("Ghostty").is_some());
    }

    #[test]
    fn is_terminal_app_iterm2() {
        let result = is_terminal_app("iTerm2");
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, "com.googlecode.iterm2");
    }

    #[test]
    fn is_terminal_app_macos_terminal() {
        let result = is_terminal_app("Terminal");
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, "com.apple.Terminal");
    }

    #[test]
    fn is_terminal_app_alacritty() {
        assert!(is_terminal_app("Alacritty").is_some());
    }

    #[test]
    fn is_terminal_app_kitty() {
        assert!(is_terminal_app("kitty").is_some());
    }

    #[test]
    fn is_terminal_app_wezterm() {
        assert!(is_terminal_app("WezTerm").is_some());
        assert!(is_terminal_app("wezterm-gui").is_some());
    }

    #[test]
    fn is_terminal_app_unknown() {
        assert!(is_terminal_app("vim").is_none());
        assert!(is_terminal_app("zsh").is_none());
        assert!(is_terminal_app("tmux").is_none());
        assert!(is_terminal_app("node").is_none());
        assert!(is_terminal_app("").is_none());
    }

    // --- Process name extraction ---

    #[test]
    fn get_process_name_extracts_basename() {
        // ps returns paths for some processes; we extract the basename.
        let pid = std::process::id();
        let name = get_process_name(pid);
        assert!(name.is_some());
        // The test binary name won't contain '/'.
        assert!(!name.unwrap().contains('/'));
    }
}
