use std::process::Command;

use super::{FocusContext, FocusResult};

/// Ghostty tab focus strategy: uses Ghostty AppleScript to switch to the
/// tab running the session that contains the copilot process.
///
/// Requires Ghostty 1.3+ (AppleScript support).
///
/// Matching strategy:
/// - With tmux: maps copilot PID -> tmux session name -> Ghostty tab name.
/// - Without tmux: matches tab name against the session's working directory
///   (best-effort, depends on shell title config).
///
/// Ghostty 1.3 does not expose PID or TTY on terminal objects (tracked in
/// ghostty-org/ghostty#11592). When that's available, we can match precisely
/// without relying on tab names.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    match ctx.terminal_app_bundle.as_deref() {
        Some("com.mitchellh.ghostty") => {}
        _ => return FocusResult::NotApplicable,
    }

    if !is_ghostty_scriptable() {
        return FocusResult::NotApplicable;
    }

    // Try tmux-based matching first (precise).
    if ctx.tmux_server_pid.is_some() {
        if let Some(session_name) = find_tmux_session_for_context(ctx) {
            return match focus_ghostty_tab_by_name(&session_name) {
                Ok(true) => FocusResult::Focused,
                Ok(false) => FocusResult::Failed(format!(
                    "no Ghostty tab matches tmux session '{session_name}'"
                )),
                Err(e) => FocusResult::Failed(e),
            };
        }
    }

    // Fallback: match tab name against working directory (best-effort).
    let cwd_name = ctx
        .cwd
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or(&ctx.cwd);

    match focus_ghostty_tab_by_substring(cwd_name) {
        Ok(true) => FocusResult::Focused,
        Ok(false) => FocusResult::NotApplicable,
        Err(e) => FocusResult::Failed(e),
    }
}

/// Finds the tmux session name containing the copilot process.
fn find_tmux_session_for_context(ctx: &FocusContext) -> Option<String> {
    let output = Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{session_name} #{pane_pid}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Match by copilot PID or its ancestors.
    for line in stdout.lines() {
        let (session, pid_str) = line.rsplit_once(' ')?;
        let pid: u32 = pid_str.parse().ok()?;

        if pid == ctx.copilot_pid || ctx.ancestors.contains(&pid) {
            return Some(session.to_string());
        }
    }

    None
}

/// Uses Ghostty AppleScript to find a tab by exact name and focus its terminal.
fn focus_ghostty_tab_by_name(tab_name: &str) -> Result<bool, String> {
    let script = format!(
        r#"tell application "Ghostty"
    repeat with w in every window
        repeat with t in every tab of w
            if name of t is "{tab_name}" then
                focus (focused terminal of t)
                activate
                return true
            end if
        end repeat
    end repeat
    return false
end tell"#
    );
    run_ghostty_script(&script)
}

/// Uses Ghostty AppleScript to find a tab whose name contains a substring.
fn focus_ghostty_tab_by_substring(substring: &str) -> Result<bool, String> {
    let script = format!(
        r#"tell application "Ghostty"
    repeat with w in every window
        repeat with t in every tab of w
            if name of t contains "{substring}" then
                focus (focused terminal of t)
                activate
                return true
            end if
        end repeat
    end repeat
    return false
end tell"#
    );
    run_ghostty_script(&script)
}

fn run_ghostty_script(script: &str) -> Result<bool, String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ghostty AppleScript failed: {}", stderr.trim()));
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result == "true")
}

/// Checks if the installed Ghostty version supports AppleScript (>= 1.3).
fn is_ghostty_scriptable() -> bool {
    ghostty_version().map(|v| v >= (1, 3)).unwrap_or(false)
}

/// Returns the Ghostty version as (major, minor), or None if not available.
fn ghostty_version() -> Option<(u32, u32)> {
    let output = Command::new("ghostty").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    // First line: "Ghostty X.Y.Z"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_str = stdout.lines().next()?.strip_prefix("Ghostty ")?;
    let mut parts = version_str.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Returns the Ghostty version string for display in settings, or None.
pub fn ghostty_version_string() -> Option<String> {
    let output = Command::new("ghostty").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .next()?
        .strip_prefix("Ghostty ")
        .map(|s| s.to_string())
}

/// Returns whether the Ghostty scripting strategy is available.
pub fn is_available() -> bool {
    is_ghostty_scriptable()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_applicable_for_non_ghostty() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: None,
            terminal_app_name: Some("Terminal".to_string()),
            terminal_app_bundle: Some("com.apple.Terminal".to_string()),
        };
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_without_bundle() {
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
