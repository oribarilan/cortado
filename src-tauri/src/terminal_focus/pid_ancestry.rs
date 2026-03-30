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
            }
        }

        // Detect GUI terminal app.
        if terminal_app_pid.is_none() {
            if let Some((app_name, bundle_id)) = is_gui_app(parent) {
                terminal_app_pid = Some(parent);
                terminal_app_name = Some(app_name);
                terminal_app_bundle = Some(bundle_id);
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

                if let Some((app_name, bundle_id)) = is_gui_app(parent) {
                    terminal_app_pid = Some(parent);
                    terminal_app_name = Some(app_name);
                    terminal_app_bundle = Some(bundle_id);
                    break;
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

/// Checks if a PID is a regular GUI app via NSRunningApplication.
/// Returns (localized name, bundle identifier) if it is.
fn is_gui_app(pid: u32) -> Option<(String, String)> {
    use std::process::Command;

    let script = format!(
        r#"use framework "AppKit"
set app to current application's NSRunningApplication's runningApplicationWithProcessIdentifier:{}
if app is missing value then return ""
set appName to (app's localizedName()) as text
set bundleId to (app's bundleIdentifier()) as text
return appName & "|" & bundleId"#,
        pid
    );

    let output = Command::new("osascript")
        .args(["-l", "AppleScript", "-e", &script])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if result.is_empty() {
        return None;
    }

    let parts: Vec<&str> = result.splitn(2, '|').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
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
