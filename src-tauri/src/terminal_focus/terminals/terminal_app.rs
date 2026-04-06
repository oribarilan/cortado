use std::process::Command;

use crate::terminal_focus::{escape_applescript, FocusContext, FocusResult};

const BUNDLE_ID: &str = "com.apple.Terminal";

/// macOS Terminal.app focus strategy: uses AppleScript to enumerate tabs
/// and match by TTY device path.
///
/// Only works without tmux -- when tmux is in the process ancestry the
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

    match focus_tab_by_tty(&tty) {
        Ok(true) => FocusResult::Focused,
        Ok(false) => FocusResult::Failed(format!("no Terminal.app tab with TTY {tty}")),
        Err(e) => FocusResult::Failed(e),
    }
}

/// AppleScript: find the tab whose TTY matches, select it, bring window to front.
fn focus_tab_by_tty(tty: &str) -> Result<bool, String> {
    let safe_tty = escape_applescript(tty);
    let script = format!(
        r#"tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            if tty of t is "{safe_tty}" then
                set selected tab of w to t
                set index of w to 1
                activate
                return true
            end if
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
        return Err(format!(
            "Terminal.app AppleScript failed: {}",
            stderr.trim()
        ));
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
            terminal_app_name: Some("Terminal".to_string()),
            terminal_app_bundle: bundle.map(String::from),
        }
    }

    #[test]
    fn not_applicable_wrong_bundle() {
        let ctx = make_ctx(Some("com.mitchellh.ghostty"), false);
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
    fn tty_path_format() {
        // Verify the TTY helper produces the expected format.
        let tty = super::super::resolve_tty(std::process::id());
        if let Some(ref path) = tty {
            assert!(
                path.starts_with("/dev/ttys"),
                "expected /dev/ttys prefix, got: {path}"
            );
        }
    }
}
