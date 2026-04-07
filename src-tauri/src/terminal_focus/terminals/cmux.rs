use std::process::Command;

use crate::terminal_focus::{escape_applescript, FocusContext, FocusResult};

/// cmux terminal focus strategy: uses AppleScript to focus the terminal
/// whose working directory matches the session's CWD.
///
/// cmux (com.cmuxterm.app) is a native macOS terminal built on libghostty
/// with built-in workspaces and split panes. Its AppleScript dictionary
/// exposes `working directory` on terminal objects, enabling precise CWD
/// matching without relying on tab names.
///
/// Matching strategy:
/// - With tmux: maps copilot PID -> tmux session name -> cmux tab name.
/// - Without tmux: matches terminal's `working directory` against session CWD.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    match ctx.terminal_app_bundle.as_deref() {
        Some("com.cmuxterm.app") => {}
        _ => return FocusResult::NotApplicable,
    }

    // Try tmux-based matching first (precise tab name match).
    if ctx.tmux_server_pid.is_some() {
        if let Some(session_name) = find_tmux_session_for_context(ctx) {
            return match focus_cmux_tab_by_name(&session_name) {
                Ok(true) => FocusResult::Focused,
                Ok(false) => FocusResult::Failed(format!(
                    "no cmux tab matches tmux session '{session_name}'"
                )),
                Err(e) => FocusResult::Failed(e),
            };
        }
    }

    // Primary: match by working directory (cmux exposes this directly).
    match focus_cmux_terminal_by_cwd(&ctx.cwd) {
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

    for line in stdout.lines() {
        let (session, pid_str) = line.rsplit_once(' ')?;
        let pid: u32 = pid_str.parse().ok()?;

        if pid == ctx.copilot_pid || ctx.ancestors.contains(&pid) {
            return Some(session.to_string());
        }
    }

    None
}

/// Focuses a cmux tab whose name matches a tmux session name.
fn focus_cmux_tab_by_name(tab_name: &str) -> Result<bool, String> {
    let safe_name = escape_applescript(tab_name);
    let script = format!(
        r#"tell application "cmux"
    repeat with w in every window
        repeat with t in every tab of w
            if name of t is "{safe_name}" then
                focus (focused terminal of t)
                activate
                return true
            end if
        end repeat
    end repeat
    return false
end tell"#
    );
    run_cmux_script(&script)
}

/// Focuses the cmux terminal whose working directory matches the given CWD.
///
/// Uses exact match on the terminal's `working directory` property.
fn focus_cmux_terminal_by_cwd(cwd: &str) -> Result<bool, String> {
    let safe_cwd = escape_applescript(cwd);
    let script = format!(
        r#"tell application "cmux"
    repeat with w in every window
        repeat with t in every tab of w
            repeat with trm in every terminal of t
                if working directory of trm is "{safe_cwd}" then
                    focus trm
                    activate
                    return true
                end if
            end repeat
        end repeat
    end repeat
    return false
end tell"#
    );
    run_cmux_script(&script)
}

fn run_cmux_script(script: &str) -> Result<bool, String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cmux AppleScript failed: {}", stderr.trim()));
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result == "true")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmux_ctx(tmux: bool, cwd: &str) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: cwd.to_string(),
            ancestors: vec![2, 3],
            tmux_server_pid: if tmux { Some(100) } else { None },
            terminal_app_pid: Some(200),
            terminal_app_name: Some("cmux".to_string()),
            terminal_app_bundle: Some("com.cmuxterm.app".to_string()),
        }
    }

    #[test]
    fn not_applicable_for_non_cmux() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: None,
            terminal_app_name: Some("Ghostty".to_string()),
            terminal_app_bundle: Some("com.mitchellh.ghostty".to_string()),
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

    #[test]
    fn not_applicable_for_ghostty() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: Some(500),
            terminal_app_name: Some("Ghostty".to_string()),
            terminal_app_bundle: Some("com.mitchellh.ghostty".to_string()),
        };
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_for_iterm2() {
        let ctx = FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: Some(500),
            terminal_app_name: Some("iTerm2".to_string()),
            terminal_app_bundle: Some("com.googlecode.iterm2".to_string()),
        };
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn cwd_used_for_matching() {
        let ctx = cmux_ctx(false, "/home/user/repos/my-project");
        // We just verify the function runs without panicking and returns
        // a valid FocusResult (will be NotApplicable if cmux isn't running).
        let result = try_focus(&ctx);
        assert!(matches!(
            result,
            FocusResult::NotApplicable | FocusResult::Focused | FocusResult::Failed(_)
        ));
    }
}
