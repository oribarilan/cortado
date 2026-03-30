//! E2E tests for terminal focus strategies.
//!
//! All tests are `#[ignore]` — they open real terminal windows and require
//! macOS with the target terminal installed. Run with `just local-e2e`.

use std::process::Command;

/// Checks if a macOS app is installed.
fn is_app_installed(app_name: &str) -> bool {
    std::path::Path::new(&format!("/Applications/{app_name}.app")).exists()
        || std::path::Path::new(&format!("/System/Applications/{app_name}.app")).exists()
}

// ---------------------------------------------------------------------------
// E2E tests — all #[ignore], run via `just local-e2e`
// ---------------------------------------------------------------------------

/// Verifies Terminal.app AppleScript API: can query TTY and switch tabs.
#[test]
#[ignore]
fn e2e_terminal_app_applescript_api() {
    if !is_app_installed("Utilities/Terminal") {
        eprintln!("Terminal.app not found, skipping");
        return;
    }

    // Open a new window and verify we can read TTY
    let output = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "Terminal"
    activate
    do script "echo e2e_test_marker"
    delay 1
    set t to selected tab of front window
    return tty of t
end tell"#,
        ])
        .output()
        .expect("osascript should run");

    assert!(
        output.status.success(),
        "Terminal.app AppleScript should succeed"
    );
    let tty = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        tty.starts_with("/dev/ttys"),
        "expected TTY path, got: {tty}"
    );

    // Verify we can query selected tab state
    let check = Command::new("osascript")
        .args([
            "-e",
            &format!(
                r#"tell application "Terminal"
    repeat with w in windows
        if tty of selected tab of w is "{tty}" then return "found"
    end repeat
    return "not_found"
end tell"#
            ),
        ])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&check.stdout).trim(),
        "found",
        "should find the tab by TTY"
    );

    // Clean up
    let _ = Command::new("osascript")
        .args(["-e", r#"tell application "Terminal" to close front window"#])
        .output();
}

/// Verifies iTerm2 AppleScript API: can create tab, read TTY, select session.
#[test]
#[ignore]
fn e2e_iterm2_applescript_api() {
    if !is_app_installed("iTerm") && !is_app_installed("iTerm2") {
        eprintln!("iTerm2 not found, skipping");
        return;
    }

    // Determine the correct app name (iTerm vs iTerm2)
    let app_name = if is_app_installed("iTerm2") {
        "iTerm2"
    } else {
        "iTerm"
    };

    // Activate and ensure a window exists, then read TTY
    let script = format!(
        r#"tell application "{app_name}"
    activate
    delay 1
    if (count of windows) is 0 then
        create window with default profile
        delay 1
    end if
    set w to current window
    return tty of current session of current tab of w
end tell"#
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .expect("osascript should run");

    assert!(
        output.status.success(),
        "{app_name} AppleScript should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tty = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        tty.starts_with("/dev/ttys"),
        "expected TTY path, got: {tty}"
    );

    // Verify select works on the session
    let select_script = format!(
        r#"tell application "{app_name}"
    tell current session of current tab of current window to select
    return "ok"
end tell"#
    );
    let select_result = Command::new("osascript")
        .args(["-e", &select_script])
        .output()
        .unwrap();
    assert!(select_result.status.success(), "session select should work");
}

/// Verifies Ghostty AppleScript API: can enumerate windows and tabs.
#[test]
#[ignore]
fn e2e_ghostty_applescript_api() {
    if !is_app_installed("Ghostty") {
        eprintln!("Ghostty not found, skipping");
        return;
    }

    // Verify window enumeration
    let windows = Command::new("osascript")
        .args(["-e", r#"tell application "Ghostty" to get every window"#])
        .output()
        .unwrap();
    assert!(
        windows.status.success(),
        "Ghostty should respond to AppleScript"
    );

    // Verify tab enumeration
    let tabs = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "Ghostty"
    set tabNames to {}
    repeat with w in every window
        repeat with t in every tab of w
            set end of tabNames to (name of t)
        end repeat
    end repeat
    return tabNames
end tell"#,
        ])
        .output()
        .unwrap();
    assert!(tabs.status.success(), "should enumerate Ghostty tabs");

    // Verify focus command works
    let focus = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "Ghostty"
    focus (focused terminal of tab 1 of window 1)
    return "ok"
end tell"#,
        ])
        .output()
        .unwrap();
    assert!(
        focus.status.success(),
        "focus command should work on Ghostty 1.3+"
    );
}

/// Verifies kitty remote control: can list windows and read PIDs.
#[test]
#[ignore]
fn e2e_kitty_remote_control() {
    let output = Command::new("kitty").args(["@", "ls"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let json = String::from_utf8_lossy(&o.stdout);
            assert!(json.starts_with('['), "kitty @ ls should return JSON array");

            // Parse and verify structure has pid/cwd fields
            let parsed: serde_json::Value =
                serde_json::from_str(&json).expect("kitty @ ls should return valid JSON");
            if let Some(os_window) = parsed.as_array().and_then(|a| a.first()) {
                let tabs = os_window.get("tabs").and_then(|t| t.as_array());
                assert!(tabs.is_some(), "OS window should have tabs");
                if let Some(tab) = tabs.and_then(|t| t.first()) {
                    let windows = tab.get("windows").and_then(|w| w.as_array());
                    assert!(windows.is_some(), "tab should have windows");
                    if let Some(window) = windows.and_then(|w| w.first()) {
                        assert!(window.get("pid").is_some(), "window should have pid");
                        assert!(window.get("cwd").is_some(), "window should have cwd");
                    }
                }
            }
        }
        _ => {
            eprintln!(
                "kitty remote control not available (allow_remote_control not set?), skipping"
            );
        }
    }
}

/// Verifies WezTerm CLI: can list panes with metadata.
#[test]
#[ignore]
fn e2e_wezterm_cli() {
    let output = Command::new("wezterm")
        .args(["cli", "list", "--format", "json"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let json = String::from_utf8_lossy(&o.stdout);
            let parsed: serde_json::Value =
                serde_json::from_str(&json).expect("wezterm cli list should return valid JSON");

            // Verify it's an array of pane objects with expected fields
            let panes = parsed.as_array().expect("should be a JSON array");
            if let Some(pane) = panes.first() {
                assert!(pane.get("pane_id").is_some(), "pane should have pane_id");
                assert!(pane.get("cwd").is_some(), "pane should have cwd");
                assert!(pane.get("title").is_some(), "pane should have title");
                assert!(pane.get("tab_id").is_some(), "pane should have tab_id");
                assert!(
                    pane.get("window_id").is_some(),
                    "pane should have window_id"
                );
            }
        }
        _ => {
            eprintln!("wezterm CLI not available, skipping");
        }
    }
}
