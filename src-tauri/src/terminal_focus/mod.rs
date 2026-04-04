use crate::feed::harness::SessionInfo;

mod pid_ancestry;
mod terminals;
pub(crate) mod tmux;

/// Escapes a string for safe interpolation into an AppleScript double-quoted string.
/// Handles backslashes and double quotes — the two special characters in AppleScript strings.
pub(crate) fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Context gathered during PID ancestry walk, shared by all strategies.
#[derive(Debug)]
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
/// Two-phase approach:
/// 1. **tmux pre-step**: navigates to the correct tmux pane (if tmux is detected and enabled).
/// 2. **Terminal waterfall**: tries terminal-specific strategies to switch to the right
///    tab/window and activate the app. Falls back to generic app activation.
///
/// This separation lets tmux pane navigation compose with terminal tab focus
/// (e.g., Ghostty tab switching) instead of competing with it.
pub fn focus_terminal(
    session: &SessionInfo,
    tmux_enabled: bool,
    accessibility_enabled: bool,
) -> Result<(), String> {
    let ctx = pid_ancestry::build_focus_context(session)
        .map_err(|e| format!("failed to build focus context: {e}"))?;

    eprintln!(
        "focus: context for session {} — terminal={:?} bundle={:?} tmux={:?}",
        session.id, ctx.terminal_app_name, ctx.terminal_app_bundle, ctx.tmux_server_pid
    );

    // Phase 1: tmux pane navigation (pre-step, not part of the waterfall).
    if tmux_enabled {
        match tmux::try_navigate(&ctx) {
            Ok(true) => eprintln!("focus: tmux pane navigation succeeded"),
            Ok(false) => eprintln!("focus: tmux not applicable"),
            Err(e) => eprintln!("focus: tmux pane navigation failed: {e}"),
        }
    } else {
        eprintln!("focus: tmux skipped (disabled)");
    }

    // Phase 2: terminal focus waterfall.
    type Strategy = fn(&FocusContext) -> FocusResult;

    let strategies: &[(&str, Strategy, bool)] = &[
        ("terminals", terminals::try_focus, true),
        ("accessibility", stub_not_applicable, accessibility_enabled),
        ("app_activation", try_app_activation, true),
    ];

    for (name, strategy, enabled) in strategies {
        if !enabled {
            eprintln!("focus: {name} skipped (disabled)");
            continue;
        }
        match strategy(&ctx) {
            FocusResult::Focused => {
                eprintln!("focus: {name} succeeded");
                return Ok(());
            }
            FocusResult::NotApplicable => {
                eprintln!("focus: {name} not applicable");
                continue;
            }
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

    let safe_name = escape_applescript(name);
    let script = format!(r#"tell application "{safe_name}" to activate"#);

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
        ghostty_scriptable: terminals::ghostty::is_available(),
        ghostty_version: terminals::ghostty::ghostty_version_string(),
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
    pub ghostty_scriptable: bool,
    pub ghostty_version: Option<String>,
    pub accessibility_permitted: bool,
}

/// Checks whether the terminal app supports AppleScript-based focus.
#[cfg(test)]
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
mod e2e;

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(
        terminal_pid: Option<u32>,
        terminal_name: Option<&str>,
        terminal_bundle: Option<&str>,
        tmux: bool,
    ) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/home/user/project".to_string(),
            ancestors: vec![2, 3, 4],
            tmux_server_pid: if tmux { Some(100) } else { None },
            terminal_app_pid: terminal_pid,
            terminal_app_name: terminal_name.map(String::from),
            terminal_app_bundle: terminal_bundle.map(String::from),
        }
    }

    // --- AppleScript escaping ---

    #[test]
    fn escape_applescript_no_special_chars() {
        assert_eq!(escape_applescript("hello world"), "hello world");
    }

    #[test]
    fn escape_applescript_quotes() {
        assert_eq!(escape_applescript(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn escape_applescript_backslashes() {
        assert_eq!(escape_applescript(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn escape_applescript_injection_attempt() {
        let malicious = "foo\" then do shell script \"rm -rf ~";
        let escaped = escape_applescript(malicious);
        assert_eq!(escaped, "foo\\\" then do shell script \\\"rm -rf ~");
    }

    // --- Stub strategy ---

    #[test]
    fn stub_returns_not_applicable() {
        let ctx = ctx_with(None, None, None, false);
        assert_eq!(stub_not_applicable(&ctx), FocusResult::NotApplicable);
    }

    // --- App activation fallback ---

    #[test]
    fn app_activation_not_applicable_without_terminal() {
        let ctx = ctx_with(None, None, None, false);
        assert_eq!(try_app_activation(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn app_activation_not_applicable_without_pid_or_name() {
        let ctx = ctx_with(None, None, None, true);
        assert_eq!(try_app_activation(&ctx), FocusResult::NotApplicable);
    }

    // --- Terminal detection ---

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

    // --- Waterfall logic ---

    #[test]
    fn waterfall_skips_disabled_strategies() {
        // Verify the strategy array respects enabled flags.
        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("disabled", stub_not_applicable, false),
            ("also_disabled", stub_not_applicable, false),
        ];

        let mut ran = Vec::new();
        for (name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            ran.push(*name);
            let _ = strategy(&ctx);
        }
        assert!(ran.is_empty(), "no strategies should run when all disabled");
    }

    #[test]
    fn waterfall_stops_at_first_focused() {
        fn always_focused(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Focused
        }
        fn should_not_run(_ctx: &FocusContext) -> FocusResult {
            panic!("should not be called");
        }

        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("first", always_focused, true),
            ("second", should_not_run, true),
        ];

        for (_name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            match strategy(&ctx) {
                FocusResult::Focused => break,
                _ => continue,
            }
        }
    }

    #[test]
    fn waterfall_continues_on_not_applicable() {
        fn not_applicable(_ctx: &FocusContext) -> FocusResult {
            FocusResult::NotApplicable
        }
        fn fallback_focused(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Focused
        }

        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("skip", not_applicable, true),
            ("fallback", fallback_focused, true),
        ];

        let mut result = FocusResult::NotApplicable;
        for (_name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            match strategy(&ctx) {
                FocusResult::Focused => {
                    result = FocusResult::Focused;
                    break;
                }
                _ => continue,
            }
        }
        assert_eq!(result, FocusResult::Focused);
    }

    #[test]
    fn waterfall_continues_on_failed() {
        fn failed(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Failed("broken".into())
        }
        fn fallback_focused(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Focused
        }

        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("broken", failed, true),
            ("fallback", fallback_focused, true),
        ];

        let mut result = FocusResult::NotApplicable;
        for (_name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            match strategy(&ctx) {
                FocusResult::Focused => {
                    result = FocusResult::Focused;
                    break;
                }
                _ => continue,
            }
        }
        assert_eq!(result, FocusResult::Focused);
    }

    // --- Mixed waterfall scenarios ---

    #[test]
    fn waterfall_mixed_disabled_failed_then_success() {
        fn failed(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Failed("nope".into())
        }
        fn focused(_ctx: &FocusContext) -> FocusResult {
            FocusResult::Focused
        }

        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("disabled", stub_not_applicable, false),
            ("failed", failed, true),
            ("na", stub_not_applicable, true),
            ("success", focused, true),
        ];

        let mut result = FocusResult::NotApplicable;
        let mut ran = Vec::new();
        for (name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            ran.push(*name);
            match strategy(&ctx) {
                FocusResult::Focused => {
                    result = FocusResult::Focused;
                    break;
                }
                _ => continue,
            }
        }
        assert_eq!(result, FocusResult::Focused);
        assert_eq!(ran, vec!["failed", "na", "success"]);
    }

    #[test]
    fn waterfall_all_not_applicable_yields_no_success() {
        type Strategy = fn(&FocusContext) -> FocusResult;

        let ctx = ctx_with(None, None, None, false);
        let strategies: &[(&str, Strategy, bool)] = &[
            ("a", stub_not_applicable, true),
            ("b", stub_not_applicable, true),
            ("c", stub_not_applicable, true),
        ];

        let mut result = FocusResult::NotApplicable;
        for (_name, strategy, enabled) in strategies {
            if !enabled {
                continue;
            }
            match strategy(&ctx) {
                FocusResult::Focused => {
                    result = FocusResult::Focused;
                    break;
                }
                _ => continue,
            }
        }
        assert_eq!(result, FocusResult::NotApplicable);
    }

    // --- FocusResult equality ---

    #[test]
    fn focus_result_equality() {
        assert_eq!(FocusResult::Focused, FocusResult::Focused);
        assert_eq!(FocusResult::NotApplicable, FocusResult::NotApplicable);
        assert_eq!(
            FocusResult::Failed("a".into()),
            FocusResult::Failed("a".into())
        );
        assert_ne!(FocusResult::Focused, FocusResult::NotApplicable);
        assert_ne!(
            FocusResult::Failed("a".into()),
            FocusResult::Failed("b".into())
        );
    }

    // --- AppleScript escaping edge cases ---

    #[test]
    fn escape_applescript_empty_string() {
        assert_eq!(escape_applescript(""), "");
    }

    #[test]
    fn escape_applescript_mixed_special_chars() {
        let input = r#"path\to\"file""#;
        let escaped = escape_applescript(input);
        assert_eq!(escaped, r#"path\\to\\\"file\""#);
    }

    #[test]
    fn escape_applescript_only_backslashes() {
        assert_eq!(escape_applescript(r"\\"), r"\\\\");
    }

    // --- Scriptable terminal exhaustive check ---

    #[test]
    fn is_scriptable_terminal_all_scriptable_bundles() {
        let scriptable = [
            "com.apple.Terminal",
            "com.googlecode.iterm2",
            "com.mitchellh.ghostty",
        ];
        for bundle in &scriptable {
            assert!(
                is_scriptable_terminal(Some(bundle)),
                "{bundle} should be scriptable"
            );
        }
    }

    #[test]
    fn is_scriptable_terminal_non_scriptable_bundles() {
        let non_scriptable = [
            "io.alacritty",
            "net.kovidgoyal.kitty",
            "com.github.wez.wezterm",
            "dev.warp.Warp-Stable",
        ];
        for bundle in &non_scriptable {
            assert!(
                !is_scriptable_terminal(Some(bundle)),
                "{bundle} should NOT be scriptable"
            );
        }
    }
}
