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
            // Already past tmux — still look for terminal.
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
    // try finding a tmux client and walking its ancestry for the terminal app.
    if tmux_server_pid.is_some() && terminal_app_pid.is_none() {
        if let Some(client_pid) = find_tmux_client_pid() {
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
    ("ghostty", "Ghostty", "com.mitchellh.ghostty"),
    ("Ghostty", "Ghostty", "com.mitchellh.ghostty"),
    ("iTerm2", "iTerm2", "com.googlecode.iterm2"),
    ("Terminal", "Terminal", "com.apple.Terminal"),
    ("Alacritty", "Alacritty", "io.alacritty"),
    ("kitty", "kitty", "net.kovidgoyal.kitty"),
    ("WezTerm", "WezTerm", "org.wezfurlong.wezterm"),
    ("wezterm-gui", "WezTerm", "org.wezfurlong.wezterm"),
];

/// Checks if a process name matches a known terminal app.
/// Returns (display name, bundle ID) if it matches.
fn is_terminal_app(process_name: &str) -> Option<(&'static str, &'static str)> {
    KNOWN_TERMINALS
        .iter()
        .find(|(name, _, _)| *name == process_name)
        .map(|(_, display, bundle)| (*display, *bundle))
}

/// Finds the PID of a tmux client process (for terminal resolution via tmux).
fn find_tmux_client_pid() -> Option<u32> {
    use std::process::Command;

    let output = Command::new("tmux")
        .args(["list-clients", "-F", "#{client_pid}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .and_then(|line| line.trim().parse::<u32>().ok())
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
}
