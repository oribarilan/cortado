use std::process::Command;

use crate::terminal_focus::{FocusContext, FocusResult};

const BUNDLE_ID: &str = "com.github.wez.wezterm";

/// WezTerm focus strategy: uses the `wezterm` CLI to list panes and
/// activate the one matching by CWD (primary) or TTY (fallback).
///
/// `activate-pane` focuses within WezTerm's internal mux but doesn't
/// raise the OS window — app activation is handled by the parent waterfall.
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    if ctx.terminal_app_bundle.as_deref() != Some(BUNDLE_ID) {
        return FocusResult::NotApplicable;
    }

    let panes = match list_panes() {
        Ok(p) => p,
        Err(_) => return FocusResult::NotApplicable, // CLI not available
    };

    // Try CWD match first (most reliable).
    if let Some(pane_id) = find_pane_by_cwd(&panes, &ctx.cwd) {
        return match activate_pane(pane_id) {
            Ok(()) => activate_wezterm_app(ctx),
            Err(e) => FocusResult::Failed(e),
        };
    }

    // Fallback: TTY match (only without tmux).
    if ctx.tmux_server_pid.is_none() {
        if let Some(tty) = super::resolve_tty(ctx.copilot_pid) {
            if let Some(pane_id) = find_pane_by_tty(&panes, &tty) {
                return match activate_pane(pane_id) {
                    Ok(()) => activate_wezterm_app(ctx),
                    Err(e) => FocusResult::Failed(e),
                };
            }
        }
    }

    FocusResult::Failed("no WezTerm pane matches session CWD or TTY".into())
}

/// Brings WezTerm to the front after pane activation.
fn activate_wezterm_app(ctx: &FocusContext) -> FocusResult {
    if let Some(pid) = ctx.terminal_app_pid {
        let _ = crate::terminal_focus::activate_app_by_pid(pid);
    }
    FocusResult::Focused
}

// ---------------------------------------------------------------------------
// Pane model
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct WezPane {
    pane_id: u64,
    cwd: Option<String>,
    tty_name: Option<String>,
}

/// Runs `wezterm cli list --format json` and parses the output.
fn list_panes() -> Result<Vec<WezPane>, String> {
    let output = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .output()
        .map_err(|e| format!("failed to run wezterm cli: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wezterm cli list failed: {}", stderr.trim()));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_panes_json(&json_str)
}

/// Parses the JSON output from `wezterm cli list --format json`.
fn parse_panes_json(json_str: &str) -> Result<Vec<WezPane>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("invalid JSON from wezterm: {e}"))?;

    let arr = parsed
        .as_array()
        .ok_or("expected JSON array from wezterm")?;

    Ok(arr
        .iter()
        .filter_map(|obj| {
            let pane_id = obj.get("pane_id")?.as_u64()?;
            let cwd = obj
                .get("cwd")
                .and_then(|v| v.as_str())
                .map(extract_cwd_path);
            let tty_name = obj
                .get("tty_name")
                .and_then(|v| v.as_str())
                .map(String::from);
            Some(WezPane {
                pane_id,
                cwd,
                tty_name,
            })
        })
        .collect())
}

/// Extracts the filesystem path from a WezTerm CWD URL.
/// WezTerm reports CWD as `file://hostname/path` — we extract `/path`.
fn extract_cwd_path(cwd_url: &str) -> String {
    if let Some(rest) = cwd_url.strip_prefix("file://") {
        // Skip the hostname: find the next '/' after the authority.
        if let Some(slash_idx) = rest.find('/') {
            return rest[slash_idx..].to_string();
        }
    }
    // Not a file:// URL — return as-is (may already be a plain path).
    cwd_url.to_string()
}

/// Finds a pane whose CWD matches the session's working directory.
fn find_pane_by_cwd(panes: &[WezPane], cwd: &str) -> Option<u64> {
    // Normalize: strip trailing slashes for comparison.
    let target = cwd.trim_end_matches('/');
    panes.iter().find_map(|p| {
        let pane_cwd = p.cwd.as_deref()?.trim_end_matches('/');
        (pane_cwd == target).then_some(p.pane_id)
    })
}

/// Finds a pane whose TTY matches.
fn find_pane_by_tty(panes: &[WezPane], tty: &str) -> Option<u64> {
    panes.iter().find_map(|p| {
        let pane_tty = p.tty_name.as_deref()?;
        (pane_tty == tty).then_some(p.pane_id)
    })
}

/// Activates a WezTerm pane by ID.
fn activate_pane(pane_id: u64) -> Result<(), String> {
    let output = Command::new("wezterm")
        .args(["cli", "activate-pane", "--pane-id", &pane_id.to_string()])
        .output()
        .map_err(|e| format!("failed to run wezterm activate-pane: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wezterm activate-pane failed: {}", stderr.trim()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(bundle: Option<&str>) -> FocusContext {
        FocusContext {
            copilot_pid: 1,
            cwd: "/home/user/project".to_string(),
            ancestors: vec![],
            tmux_server_pid: None,
            terminal_app_pid: Some(200),
            terminal_app_name: Some("WezTerm".to_string()),
            terminal_app_bundle: bundle.map(String::from),
        }
    }

    #[test]
    fn not_applicable_wrong_bundle() {
        let ctx = make_ctx(Some("com.apple.Terminal"));
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    #[test]
    fn not_applicable_no_bundle() {
        let ctx = make_ctx(None);
        assert_eq!(try_focus(&ctx), FocusResult::NotApplicable);
    }

    // --- CWD URL parsing ---

    #[test]
    fn extract_cwd_path_file_url() {
        let path = extract_cwd_path("file://hostname/Users/me/project");
        assert_eq!(path, "/Users/me/project");
    }

    #[test]
    fn extract_cwd_path_file_url_localhost() {
        let path = extract_cwd_path("file://localhost/home/user");
        assert_eq!(path, "/home/user");
    }

    #[test]
    fn extract_cwd_path_plain_path() {
        let path = extract_cwd_path("/home/user/project");
        assert_eq!(path, "/home/user/project");
    }

    #[test]
    fn extract_cwd_path_empty_host() {
        let path = extract_cwd_path("file:///home/user");
        assert_eq!(path, "/home/user");
    }

    // --- JSON parsing ---

    #[test]
    fn parse_panes_json_valid() {
        let json = r#"[
            {"pane_id": 0, "cwd": "file://host/home/user", "tty_name": "/dev/ttys003", "tab_id": 1, "window_id": 0},
            {"pane_id": 1, "cwd": "file://host/tmp", "tty_name": null, "tab_id": 1, "window_id": 0}
        ]"#;

        let panes = parse_panes_json(json).unwrap();
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].pane_id, 0);
        assert_eq!(panes[0].cwd.as_deref(), Some("/home/user"));
        assert_eq!(panes[0].tty_name.as_deref(), Some("/dev/ttys003"));
        assert_eq!(panes[1].pane_id, 1);
        assert_eq!(panes[1].cwd.as_deref(), Some("/tmp"));
        assert!(panes[1].tty_name.is_none());
    }

    #[test]
    fn parse_panes_json_empty_array() {
        let panes = parse_panes_json("[]").unwrap();
        assert!(panes.is_empty());
    }

    #[test]
    fn parse_panes_json_missing_pane_id() {
        let json = r#"[{"cwd": "/tmp"}]"#;
        let panes = parse_panes_json(json).unwrap();
        assert!(panes.is_empty(), "pane without pane_id should be skipped");
    }

    #[test]
    fn parse_panes_json_invalid() {
        assert!(parse_panes_json("not json").is_err());
    }

    // --- CWD matching ---

    #[test]
    fn find_pane_by_cwd_exact_match() {
        let panes = vec![
            WezPane {
                pane_id: 0,
                cwd: Some("/other".into()),
                tty_name: None,
            },
            WezPane {
                pane_id: 1,
                cwd: Some("/home/user/project".into()),
                tty_name: None,
            },
        ];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user/project"), Some(1));
    }

    #[test]
    fn find_pane_by_cwd_trailing_slash() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: Some("/home/user/project/".into()),
            tty_name: None,
        }];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user/project"), Some(0));
    }

    #[test]
    fn find_pane_by_cwd_no_match() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: Some("/other".into()),
            tty_name: None,
        }];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user/project"), None);
    }

    // --- TTY matching ---

    #[test]
    fn find_pane_by_tty_match() {
        let panes = vec![
            WezPane {
                pane_id: 0,
                cwd: None,
                tty_name: Some("/dev/ttys003".into()),
            },
            WezPane {
                pane_id: 1,
                cwd: None,
                tty_name: Some("/dev/ttys007".into()),
            },
        ];
        assert_eq!(find_pane_by_tty(&panes, "/dev/ttys007"), Some(1));
    }

    #[test]
    fn find_pane_by_tty_no_match() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: None,
            tty_name: Some("/dev/ttys003".into()),
        }];
        assert_eq!(find_pane_by_tty(&panes, "/dev/ttys999"), None);
    }

    #[test]
    fn find_pane_by_tty_none_tty() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: None,
            tty_name: None,
        }];
        assert_eq!(find_pane_by_tty(&panes, "/dev/ttys003"), None);
    }

    // --- CWD URL edge cases ---

    #[test]
    fn extract_cwd_path_with_spaces() {
        let path = extract_cwd_path("file://host/Users/me/my%20project");
        assert_eq!(path, "/Users/me/my%20project");
    }

    #[test]
    fn extract_cwd_path_deep_nested() {
        let path = extract_cwd_path("file://host/a/b/c/d/e/f");
        assert_eq!(path, "/a/b/c/d/e/f");
    }

    #[test]
    fn extract_cwd_path_root() {
        let path = extract_cwd_path("file://host/");
        assert_eq!(path, "/");
    }

    // --- CWD matching edge cases ---

    #[test]
    fn find_pane_by_cwd_empty_panes() {
        let panes: Vec<WezPane> = vec![];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user"), None);
    }

    #[test]
    fn find_pane_by_cwd_none_cwd() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: None,
            tty_name: None,
        }];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user"), None);
    }

    #[test]
    fn find_pane_by_cwd_target_trailing_slash() {
        let panes = vec![WezPane {
            pane_id: 0,
            cwd: Some("/home/user/project".into()),
            tty_name: None,
        }];
        assert_eq!(find_pane_by_cwd(&panes, "/home/user/project/"), Some(0));
    }

    #[test]
    fn find_pane_by_cwd_first_match_wins() {
        let panes = vec![
            WezPane {
                pane_id: 5,
                cwd: Some("/same/path".into()),
                tty_name: None,
            },
            WezPane {
                pane_id: 9,
                cwd: Some("/same/path".into()),
                tty_name: None,
            },
        ];
        assert_eq!(find_pane_by_cwd(&panes, "/same/path"), Some(5));
    }

    // --- JSON parsing edge cases ---

    #[test]
    fn parse_panes_json_extra_fields_ignored() {
        let json = r#"[{"pane_id": 7, "cwd": "/tmp", "tty_name": "/dev/ttys001", "extra": true, "workspace": "default"}]"#;
        let panes = parse_panes_json(json).unwrap();
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].pane_id, 7);
    }

    #[test]
    fn parse_panes_json_string_pane_id_skipped() {
        let json = r#"[{"pane_id": "not_a_number", "cwd": "/tmp"}]"#;
        let panes = parse_panes_json(json).unwrap();
        assert!(panes.is_empty());
    }
}
