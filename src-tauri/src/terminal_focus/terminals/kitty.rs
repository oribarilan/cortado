use std::process::Command;

use crate::terminal_focus::{FocusContext, FocusResult};

const BUNDLE_ID: &str = "net.kovidgoyal.kitty";

/// kitty focus strategy: uses kitty's remote control protocol to find
/// and focus the window matching by PID.
///
/// Requires `allow_remote_control yes` (or `socket-only`) in `kitty.conf`.
/// When remote control is not enabled, returns `NotApplicable` (not an error).
pub fn try_focus(ctx: &FocusContext) -> FocusResult {
    if ctx.terminal_app_bundle.as_deref() != Some(BUNDLE_ID) {
        return FocusResult::NotApplicable;
    }

    let os_windows = match kitty_ls() {
        Ok(w) => w,
        Err(_) => return FocusResult::NotApplicable, // remote control not enabled or kitty not found
    };

    // Match by copilot PID or any ancestor PID.
    if let Some(matched_pid) = find_matching_pid(&os_windows, ctx.copilot_pid, &ctx.ancestors) {
        return match focus_window_by_pid(matched_pid) {
            Ok(()) => FocusResult::Focused,
            Err(e) => FocusResult::Failed(e),
        };
    }

    FocusResult::Failed("no kitty window matches copilot process PID".into())
}

// ---------------------------------------------------------------------------
// kitty data model
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct KittyOsWindow {
    tabs: Vec<KittyTab>,
}

#[derive(Debug)]
struct KittyTab {
    windows: Vec<KittyWindow>,
}

#[derive(Debug)]
struct KittyWindow {
    pid: u32,
}

/// Runs `kitty @ ls` and parses the JSON output.
fn kitty_ls() -> Result<Vec<KittyOsWindow>, String> {
    let output = Command::new("kitty")
        .args(["@", "ls"])
        .output()
        .map_err(|e| format!("failed to run kitty @ ls: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("kitty @ ls failed: {}", stderr.trim()));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_kitty_ls(&json_str)
}

/// Parses the `kitty @ ls` JSON into our simplified model.
fn parse_kitty_ls(json_str: &str) -> Result<Vec<KittyOsWindow>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("invalid JSON from kitty: {e}"))?;

    let os_windows_arr = parsed
        .as_array()
        .ok_or("expected JSON array from kitty @ ls")?;

    let mut os_windows = Vec::new();
    for os_win in os_windows_arr {
        let tabs_arr = os_win
            .get("tabs")
            .and_then(|t| t.as_array())
            .unwrap_or(&Vec::new())
            .clone();

        let mut tabs = Vec::new();
        for tab in &tabs_arr {
            let windows_arr = tab
                .get("windows")
                .and_then(|w| w.as_array())
                .unwrap_or(&Vec::new())
                .clone();

            let windows: Vec<KittyWindow> = windows_arr
                .iter()
                .filter_map(|w| {
                    let pid = w.get("pid")?.as_u64()? as u32;
                    Some(KittyWindow { pid })
                })
                .collect();

            tabs.push(KittyTab { windows });
        }

        os_windows.push(KittyOsWindow { tabs });
    }

    Ok(os_windows)
}

/// Finds a PID (copilot or ancestor) that exists in any kitty window.
fn find_matching_pid(
    os_windows: &[KittyOsWindow],
    copilot_pid: u32,
    ancestors: &[u32],
) -> Option<u32> {
    let all_pids: Vec<u32> = os_windows
        .iter()
        .flat_map(|ow| &ow.tabs)
        .flat_map(|t| &t.windows)
        .map(|w| w.pid)
        .collect();

    // Direct match on copilot PID.
    if all_pids.contains(&copilot_pid) {
        return Some(copilot_pid);
    }

    // Match on ancestors (closest first).
    ancestors.iter().find(|a| all_pids.contains(a)).copied()
}

/// Focuses a kitty window by PID using `kitty @ focus-window`.
fn focus_window_by_pid(pid: u32) -> Result<(), String> {
    let output = Command::new("kitty")
        .args(["@", "focus-window", "--match", &format!("pid:{pid}")])
        .output()
        .map_err(|e| format!("failed to run kitty @ focus-window: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("kitty @ focus-window failed: {}", stderr.trim()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(bundle: Option<&str>) -> FocusContext {
        FocusContext {
            copilot_pid: 42,
            cwd: "/home/user/project".to_string(),
            ancestors: vec![30, 20, 10, 1],
            tmux_server_pid: None,
            terminal_app_pid: Some(200),
            terminal_app_name: Some("kitty".to_string()),
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

    // --- JSON parsing ---

    #[test]
    fn parse_kitty_ls_valid() {
        let json = r#"[
            {
                "id": 1,
                "tabs": [
                    {
                        "id": 7,
                        "windows": [
                            {"id": 1, "pid": 12345, "cwd": "/home/user"},
                            {"id": 2, "pid": 67890, "cwd": "/tmp"}
                        ]
                    }
                ]
            }
        ]"#;

        let os_windows = parse_kitty_ls(json).unwrap();
        assert_eq!(os_windows.len(), 1);
        assert_eq!(os_windows[0].tabs.len(), 1);
        assert_eq!(os_windows[0].tabs[0].windows.len(), 2);
        assert_eq!(os_windows[0].tabs[0].windows[0].pid, 12345);
        assert_eq!(os_windows[0].tabs[0].windows[1].pid, 67890);
    }

    #[test]
    fn parse_kitty_ls_empty() {
        let os_windows = parse_kitty_ls("[]").unwrap();
        assert!(os_windows.is_empty());
    }

    #[test]
    fn parse_kitty_ls_no_tabs() {
        let json = r#"[{"id": 1}]"#;
        let os_windows = parse_kitty_ls(json).unwrap();
        assert_eq!(os_windows.len(), 1);
        assert!(os_windows[0].tabs.is_empty());
    }

    #[test]
    fn parse_kitty_ls_invalid_json() {
        assert!(parse_kitty_ls("not json").is_err());
    }

    // --- PID matching ---

    #[test]
    fn find_matching_pid_direct() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![KittyWindow { pid: 100 }, KittyWindow { pid: 42 }],
            }],
        }];

        assert_eq!(find_matching_pid(&os_windows, 42, &[30, 20]), Some(42));
    }

    #[test]
    fn find_matching_pid_ancestor() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![KittyWindow { pid: 20 }],
            }],
        }];

        assert_eq!(find_matching_pid(&os_windows, 42, &[30, 20, 10]), Some(20));
    }

    #[test]
    fn find_matching_pid_prefers_closest_ancestor() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![KittyWindow { pid: 10 }, KittyWindow { pid: 30 }],
            }],
        }];

        // ancestors: [30, 20, 10] -- 30 comes first, should match first.
        assert_eq!(find_matching_pid(&os_windows, 42, &[30, 20, 10]), Some(30));
    }

    #[test]
    fn find_matching_pid_no_match() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![KittyWindow { pid: 999 }],
            }],
        }];

        assert_eq!(find_matching_pid(&os_windows, 42, &[30, 20]), None);
    }

    #[test]
    fn find_matching_pid_empty_windows() {
        let os_windows: Vec<KittyOsWindow> = vec![];
        assert_eq!(find_matching_pid(&os_windows, 42, &[30, 20]), None);
    }

    #[test]
    fn find_matching_pid_prefers_direct_over_ancestor() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![
                    KittyWindow { pid: 30 }, // ancestor
                    KittyWindow { pid: 42 }, // direct match
                ],
            }],
        }];

        assert_eq!(find_matching_pid(&os_windows, 42, &[30]), Some(42));
    }

    // --- Multi-OS-window / multi-tab scenarios ---

    #[test]
    fn find_matching_pid_across_os_windows() {
        let os_windows = vec![
            KittyOsWindow {
                tabs: vec![KittyTab {
                    windows: vec![KittyWindow { pid: 100 }],
                }],
            },
            KittyOsWindow {
                tabs: vec![KittyTab {
                    windows: vec![KittyWindow { pid: 42 }],
                }],
            },
        ];

        assert_eq!(find_matching_pid(&os_windows, 42, &[]), Some(42));
    }

    #[test]
    fn find_matching_pid_across_tabs() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![
                KittyTab {
                    windows: vec![KittyWindow { pid: 100 }],
                },
                KittyTab {
                    windows: vec![KittyWindow { pid: 42 }],
                },
            ],
        }];

        assert_eq!(find_matching_pid(&os_windows, 42, &[]), Some(42));
    }

    #[test]
    fn find_matching_pid_empty_ancestors() {
        let os_windows = vec![KittyOsWindow {
            tabs: vec![KittyTab {
                windows: vec![KittyWindow { pid: 100 }],
            }],
        }];

        // No direct match, no ancestors to check.
        assert_eq!(find_matching_pid(&os_windows, 42, &[]), None);
    }

    // --- JSON edge cases ---

    #[test]
    fn parse_kitty_ls_no_windows_in_tab() {
        let json = r#"[{"id": 1, "tabs": [{"id": 7}]}]"#;
        let os_windows = parse_kitty_ls(json).unwrap();
        assert_eq!(os_windows.len(), 1);
        assert_eq!(os_windows[0].tabs.len(), 1);
        assert!(os_windows[0].tabs[0].windows.is_empty());
    }

    #[test]
    fn parse_kitty_ls_window_without_pid_skipped() {
        let json = r#"[{"id": 1, "tabs": [{"id": 7, "windows": [{"id": 1, "cwd": "/tmp"}]}]}]"#;
        let os_windows = parse_kitty_ls(json).unwrap();
        assert!(os_windows[0].tabs[0].windows.is_empty());
    }

    #[test]
    fn parse_kitty_ls_multiple_os_windows() {
        let json = r#"[
            {"id": 1, "tabs": [{"id": 1, "windows": [{"id": 1, "pid": 100}]}]},
            {"id": 2, "tabs": [{"id": 2, "windows": [{"id": 2, "pid": 200}]}]}
        ]"#;
        let os_windows = parse_kitty_ls(json).unwrap();
        assert_eq!(os_windows.len(), 2);
        assert_eq!(os_windows[0].tabs[0].windows[0].pid, 100);
        assert_eq!(os_windows[1].tabs[0].windows[0].pid, 200);
    }
}
