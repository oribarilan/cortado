/// Terminal-specific focus strategies.
///
/// Each submodule implements a `try_focus(ctx) -> FocusResult` function for one
/// terminal emulator. The registry in [`try_focus`] iterates them; the first
/// `Focused` result wins.
///
/// Adding a new terminal: create a new file, implement `try_focus`, add one
/// line to the `STRATEGIES` array below.
mod cmux;
pub(crate) mod ghostty;
mod iterm2;
mod kitty;
mod terminal_app;
mod wezterm;

use super::{FocusContext, FocusResult};

/// A terminal focus strategy function.
type Strategy = fn(&FocusContext) -> FocusResult;

/// Registered terminal strategies, tried in order.
/// Each entry: (human label, function).
const STRATEGIES: &[(&str, Strategy)] = &[
    ("cmux", cmux::try_focus),
    ("ghostty", ghostty::try_focus),
    ("terminal_app", terminal_app::try_focus),
    ("iterm2", iterm2::try_focus),
    ("wezterm", wezterm::try_focus),
    ("kitty", kitty::try_focus),
];

/// Tries each terminal-specific strategy in order.
/// Returns `Focused` on first success, or `NotApplicable` if none matched.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    for (name, strategy) in STRATEGIES {
        match strategy(ctx) {
            FocusResult::Focused => {
                eprintln!("focus: terminal/{name} succeeded");
                return FocusResult::Focused;
            }
            FocusResult::NotApplicable => {
                eprintln!("focus: terminal/{name} not applicable");
            }
            FocusResult::Failed(ref reason) => {
                eprintln!("focus: terminal/{name} failed: {reason}");
            }
        }
    }
    FocusResult::NotApplicable
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolves the TTY device path for a process via `ps -p <pid> -o tty=`.
///
/// Returns the full device path (e.g. `/dev/ttys003`).
///
/// When tmux is in use, the returned TTY is a tmux PTY -- not the terminal
/// tab's TTY. Callers should skip TTY matching when tmux is detected.
pub fn resolve_tty(pid: u32) -> Option<String> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "tty="])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() || raw == "??" {
        return None;
    }

    // ps returns e.g. "ttys003"; prepend /dev/ for the full path.
    Some(format!("/dev/{raw}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_for_bundle(bundle: Option<&str>) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/tmp".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: None,
            terminal_app_name: None,
            terminal_app_bundle: bundle.map(String::from),
        }
    }

    fn ctx_full(bundle: &str, name: &str, tmux: bool) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/home/user/project".to_string(),
            ancestors: vec![2, 3, 4],
            tmux_server_pid: if tmux { Some(100) } else { None },
            terminal_app_pid: Some(200),
            terminal_app_name: Some(name.to_string()),
            terminal_app_bundle: Some(bundle.to_string()),
        }
    }

    // --- Registry dispatch ---

    #[test]
    fn try_focus_returns_not_applicable_for_unknown_terminal() {
        let ctx = ctx_for_bundle(Some("com.unknown.terminal"));
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn try_focus_returns_not_applicable_without_bundle() {
        let ctx = ctx_for_bundle(None);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn registry_has_all_expected_strategies() {
        let names: Vec<&str> = STRATEGIES.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"cmux"), "missing cmux");
        assert!(names.contains(&"ghostty"), "missing ghostty");
        assert!(names.contains(&"terminal_app"), "missing terminal_app");
        assert!(names.contains(&"iterm2"), "missing iterm2");
        assert!(names.contains(&"wezterm"), "missing wezterm");
        assert!(names.contains(&"kitty"), "missing kitty");
    }

    #[test]
    fn each_strategy_returns_not_applicable_for_wrong_bundle() {
        // Every strategy should gate on bundle ID and return NotApplicable
        // for a bundle that doesn't match.
        let fake_bundle = "com.fake.nonexistent";
        let ctx = ctx_full(fake_bundle, "FakeApp", false);

        for (name, strategy) in STRATEGIES {
            let result = strategy(&ctx);
            assert_eq!(
                result,
                FocusResult::NotApplicable,
                "strategy '{name}' should return NotApplicable for bundle '{fake_bundle}'"
            );
        }
    }

    #[test]
    fn alacritty_not_handled_by_any_strategy() {
        let ctx = ctx_full("io.alacritty", "Alacritty", false);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn warp_not_handled_by_any_strategy() {
        let ctx = ctx_full("dev.warp.Warp-Stable", "Warp", false);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    // --- TTY resolution ---

    #[test]
    fn resolve_tty_current_process() {
        let tty = resolve_tty(std::process::id());
        if let Some(ref path) = tty {
            assert!(
                path.starts_with("/dev/"),
                "expected /dev/ prefix, got: {path}"
            );
        }
    }

    #[test]
    fn resolve_tty_nonexistent_pid() {
        assert!(resolve_tty(99_999_999).is_none());
    }
}
