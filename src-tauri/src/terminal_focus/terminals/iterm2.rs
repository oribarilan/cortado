use std::process::Command;

use crate::terminal_focus::{escape_applescript, FocusContext, FocusResult};

const BUNDLE_ID: &str = "com.googlecode.iterm2";

/// iTerm2 focus strategy: uses AppleScript to enumerate sessions (split panes)
/// and match by TTY device path.
///
/// Focus sequence: select window > select tab > select session > activate.
/// Handles split panes naturally (sessions are siblings within a tab).
///
/// Only works without tmux — when tmux is in the process ancestry the
/// copilot TTY is a tmux PTY, not the terminal's.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    if ctx.terminal_app_bundle.as_deref() != Some(BUNDLE_ID) {
        return FocusResult::NotApplicable;
    }

    // TTY matching doesn't work under tmux (different PTY namespace).
    if ctx.tmux_server_pid.is_some() {
        return FocusResult::NotApplicable;
    }

    let tty = match super::resolve_tty(ctx.copilot_pid) {
        Some(t) => t,
        None => return FocusResult::Failed("could not resolve TTY for copilot process".into()),
    };

    match focus_session_by_tty(&tty) {
        Ok(true) => FocusResult::Focused,
        Ok(false) => FocusResult::Failed(format!("no iTerm2 session with TTY {tty}")),
        Err(e) => FocusResult::Failed(e),
    }
}

/// AppleScript: enumerate windows > tabs > sessions, match by TTY.
/// Selects the window, tab, and session to focus the exact pane.
fn focus_session_by_tty(tty: &str) -> Result<bool, String> {
    let safe_tty = escape_applescript(tty);

    // iTerm2 may be installed as "iTerm" or "iTerm2" — the AppleScript
    // application name is "iTerm2" on modern versions. We use "iTerm2"
    // which covers the common case; macOS will match the bundle regardless
    // of the .app folder name.
    let script = format!(
        r#"tell application "iTerm2"
    repeat with w in windows
        repeat with t in tabs of w
            repeat with s in sessions of t
                if (tty of s) is "{safe_tty}" then
                    tell w to select
                    tell t to select
                    tell s to select
                    activate
                    return true
                end if
            end repeat
        end repeat
    end repeat
    return false
end tell"#
    );

    run_applescript(&script)
}

fn run_applescript(script: &str) -> Result<bool, String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("iTerm2 AppleScript failed: {}", stderr.trim()));
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result == "true")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(bundle: Option<&str>, tmux: bool) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: if tmux { Some(100) } else { None },
            terminal_app_pid: Some(200),
            terminal_app_name: Some("iTerm2".to_string()),
            terminal_app_bundle: bundle.map(String::from),
        }
    }

    #[test]
    fn not_applicable_wrong_bundle() {
        let ctx = make_ctx(Some("com.apple.Terminal"), false);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_no_bundle() {
        let ctx = make_ctx(None, false);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_with_tmux() {
        let ctx = make_ctx(Some(BUNDLE_ID), true);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_for_ghostty() {
        let ctx = make_ctx(Some("com.mitchellh.ghostty"), false);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }
}
