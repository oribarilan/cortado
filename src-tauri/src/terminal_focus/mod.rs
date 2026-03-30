use crate::feed::harness::SessionInfo;

mod pid_ancestry;
pub(crate) mod tmux;

/// Context gathered during PID ancestry walk, shared by all strategies.
#[derive(Debug)]
#[allow(dead_code)] // Fields used by focus strategies (tasks 04-06).
pub struct FocusContext {
    /// The session's copilot process PID.
    pub copilot_pid: u32,
    /// Working directory of the session (for CWD-based matching).
    pub cwd: String,
    /// Ancestor PIDs collected during the walk (copilot -> ... -> root).
    pub ancestors: Vec<u32>,
    /// If tmux was detected, the tmux server PID.
    pub tmux_server_pid: Option<u32>,
    /// The resolved terminal app PID.
    pub terminal_app_pid: Option<u32>,
    /// The terminal app name (e.g., "Ghostty", "iTerm2", "Terminal").
    pub terminal_app_name: Option<String>,
    /// The terminal app bundle ID (e.g., "com.mitchellh.ghostty").
    pub terminal_app_bundle: Option<String>,
}

/// Result of a focus attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum FocusResult {
    /// Strategy succeeded -- stop the waterfall.
    Focused,
    /// Strategy doesn't apply to this context -- try the next one.
    NotApplicable,
    /// Strategy applies but failed -- try the next one.
    Failed(String),
}

/// Attempts to focus the terminal containing the given session.
///
/// Runs strategies in priority order: tmux > terminal_script > accessibility > app_activation.
/// Stops at the first successful strategy. Strategies can be disabled via settings.
pub fn focus_terminal(
    session: &SessionInfo,
    tmux_enabled: bool,
    accessibility_enabled: bool,
) -> Result<(), String> {
    let ctx = pid_ancestry::build_focus_context(session)
        .map_err(|e| format!("failed to build focus context: {e}"))?;

    type Strategy = fn(&FocusContext) -> FocusResult;

    let strategies: &[(&str, Strategy, bool)] = &[
        ("tmux", tmux::try_focus, tmux_enabled),
        ("terminal_script", stub_not_applicable, true), // Task 05 (stretch).
        ("accessibility", stub_not_applicable, accessibility_enabled), // Task 06 (stretch).
        ("app_activation", try_app_activation, true),
    ];

    for (name, strategy, enabled) in strategies {
        if !enabled {
            continue;
        }
        match strategy(&ctx) {
            FocusResult::Focused => {
                eprintln!("focus: {name} succeeded");
                return Ok(());
            }
            FocusResult::NotApplicable => continue,
            FocusResult::Failed(reason) => {
                eprintln!("focus: {name} failed: {reason}");
                continue;
            }
        }
    }

    Err("no focus strategy succeeded".to_string())
}

/// Stub strategy that always returns `NotApplicable`.
fn stub_not_applicable(_ctx: &FocusContext) -> FocusResult {
    FocusResult::NotApplicable
}

/// Fallback strategy: activate the terminal app without targeting a specific window.
fn try_app_activation(ctx: &FocusContext) -> FocusResult {
    if let Some(pid) = ctx.terminal_app_pid {
        match activate_app_by_pid(pid) {
            Ok(()) => return FocusResult::Focused,
            Err(e) => {
                eprintln!("focus: app activation by PID failed: {e}");
            }
        }
    }

    // Fallback: activate by app name if we know it.
    if let Some(name) = &ctx.terminal_app_name {
        match activate_app_by_name(name) {
            Ok(()) => return FocusResult::Focused,
            Err(e) => return FocusResult::Failed(e),
        }
    }

    FocusResult::NotApplicable
}

/// Activates a macOS app by PID using NSRunningApplication.
pub(crate) fn activate_app_by_pid(pid: u32) -> Result<(), String> {
    use std::process::Command;

    // Use osascript to activate the app — avoids direct objc FFI.
    let script = format!(
        r#"tell application "System Events"
    set targetProcess to first process whose unix id is {pid}
    set frontmost of targetProcess to true
end tell"#
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("osascript failed: {}", stderr.trim()))
    }
}

/// Activates a macOS app by name.
fn activate_app_by_name(name: &str) -> Result<(), String> {
    use std::process::Command;

    let script = format!(r#"tell application "{name}" to activate"#);

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("failed to run osascript: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("osascript failed: {}", stderr.trim()))
    }
}

/// Builds a `FocusContext` for label generation (no focus attempt).
/// Returns `None` if the PID walk fails.
pub fn build_context_for_label(session: &SessionInfo) -> Option<FocusContext> {
    pid_ancestry::build_focus_context(session).ok()
}

/// Queries current focus capabilities for the settings UI.
///
/// Only performs cheap checks (tmux binary, AX permission).
/// Does NOT do PID ancestry walks — those happen during poll/focus.
pub fn get_capabilities() -> FocusCapabilities {
    FocusCapabilities {
        has_active_session: false,
        tmux_installed: is_tmux_installed(),
        tmux_detected: false,
        terminal_app: None,
        terminal_scriptable: false,
        accessibility_permitted: check_accessibility_permission(),
    }
}

/// Focus capabilities for the settings UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FocusCapabilities {
    pub has_active_session: bool,
    pub tmux_installed: bool,
    pub tmux_detected: bool,
    pub terminal_app: Option<String>,
    pub terminal_scriptable: bool,
    pub accessibility_permitted: bool,
}

/// Checks whether the terminal app supports AppleScript-based focus.
#[allow(dead_code)] // Used when terminal scripting strategy is implemented (task 05).
fn is_scriptable_terminal(bundle_id: Option<&str>) -> bool {
    matches!(
        bundle_id,
        Some("com.apple.Terminal" | "com.googlecode.iterm2" | "com.mitchellh.ghostty")
    )
}

/// Checks if tmux is installed (available on PATH).
fn is_tmux_installed() -> bool {
    std::process::Command::new("tmux")
        .arg("-V")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Checks if Accessibility permission is granted via AXIsProcessTrusted().
fn check_accessibility_permission() -> bool {
    // Link against ApplicationServices for AXIsProcessTrusted.
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    // SAFETY: AXIsProcessTrusted is a safe, side-effect-free query.
    unsafe { AXIsProcessTrusted() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_context(terminal_pid: Option<u32>) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: terminal_pid,
            terminal_app_name: None,
            terminal_app_bundle: None,
        }
    }

    #[test]
    fn stub_returns_not_applicable() {
        let ctx = mock_context(None);
        assert_eq!(stub_not_applicable(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn app_activation_not_applicable_without_terminal_pid() {
        let ctx = mock_context(None);
        assert_eq!(try_app_activation(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn is_scriptable_terminal_known_apps() {
        assert!(is_scriptable_terminal(Some("com.apple.Terminal")));
        assert!(is_scriptable_terminal(Some("com.googlecode.iterm2")));
        assert!(is_scriptable_terminal(Some("com.mitchellh.ghostty")));
    }

    #[test]
    fn is_scriptable_terminal_unknown_app() {
        assert!(!is_scriptable_terminal(Some("com.unknown.app")));
        assert!(!is_scriptable_terminal(None));
    }
}
