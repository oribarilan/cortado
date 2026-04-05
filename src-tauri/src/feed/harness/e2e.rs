//! E2E tests for harness session tracking.
//!
//! These tests run real CLI sessions (Copilot CLI and OpenCode) and verify
//! that the harness interchange files are created, contain correct data,
//! and transition through the expected lifecycle.
//!
//! All tests are `#[ignore]` — they require the respective CLI tools
//! installed and authenticated. Run with `just e2e`.
//!
//! Tests will **fail loudly** if a required binary is missing — this is
//! intentional so test setup issues are immediately visible.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{fs, thread};

use serde::Deserialize;

use super::HarnessProvider;

/// Parsed interchange file for assertions.
#[derive(Debug, Deserialize)]
struct InterchangeFile {
    version: u32,
    harness: String,
    id: String,
    pid: u32,
    cwd: String,
    status: String,
    last_active_at: Option<String>,
    repository: Option<String>,
    branch: Option<String>,
}

fn harness_dir() -> PathBuf {
    dirs::home_dir()
        .expect("home dir should exist")
        .join(".config/cortado/harness")
}

/// Returns the set of existing harness file names (to detect new files).
fn snapshot_harness_files() -> std::collections::HashSet<String> {
    let dir = harness_dir();
    if !dir.exists() {
        return std::collections::HashSet::new();
    }
    fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".json") && !n.starts_with('.'))
        .collect()
}

/// Polls the harness directory for a new file matching the given harness name.
/// Returns the parsed interchange file and its path.
fn wait_for_new_harness_file(
    before: &std::collections::HashSet<String>,
    harness_name: &str,
    timeout: Duration,
) -> (InterchangeFile, PathBuf) {
    let dir = harness_dir();
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            panic!(
                "Timed out waiting for new {harness_name} harness file after {timeout:?}. \
                 Existing files: {before:?}"
            );
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".json") || name.starts_with('.') {
                    continue;
                }
                if before.contains(&name) {
                    continue;
                }
                // New file — try to parse it.
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(interchange) = serde_json::from_str::<InterchangeFile>(&content) {
                        if interchange.harness == harness_name {
                            return (interchange, entry.path());
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/// Reads and parses an interchange file, returning None if missing or invalid.
fn read_interchange(path: &std::path::Path) -> Option<InterchangeFile> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Checks if a process is still alive.
fn pid_is_alive(pid: u32) -> bool {
    // kill(pid, 0) checks liveness without sending a signal.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

// ---------------------------------------------------------------------------
// Copilot CLI e2e tests
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn e2e_copilot_session_lifecycle() {
    let before = snapshot_harness_files();

    // Spawn copilot in a background thread so we can poll files concurrently.
    let handle = thread::spawn(|| {
        let output = Command::new("copilot")
            .args([
                "-p",
                "respond with the single word 'hello'",
                "--allow-all-tools",
                "--no-custom-instructions",
            ])
            .output()
            .expect("copilot binary must be installed and on PATH");
        assert!(
            output.status.success(),
            "copilot session should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    });

    // 1. Wait for harness file to appear.
    let (interchange, path) =
        wait_for_new_harness_file(&before, "copilot", Duration::from_secs(30));

    // 2. Verify fields while session is active.
    assert_eq!(interchange.version, 1);
    assert_eq!(interchange.harness, "copilot");
    assert!(!interchange.id.is_empty(), "session ID should be set");
    assert!(!interchange.cwd.is_empty(), "cwd should be set");
    assert!(
        interchange.last_active_at.is_some(),
        "last_active_at should be set"
    );
    // Working status should appear during the session (userPromptSubmitted
    // fires first and sets "working").
    assert!(
        interchange.status == "working" || interchange.status == "idle",
        "status should be working or idle during session, got: {}",
        interchange.status
    );

    // 3. Wait for session to complete.
    handle.join().expect("copilot thread should not panic");

    // 4. After session ends, the hook writes "idle". If the Cortado app
    //    is running, GenericProvider may clean up the file before we read it.
    //    Both outcomes are valid: "idle" file exists, or file already cleaned up.
    let session_id = interchange.id.clone();
    let session_pid = interchange.pid;
    if let Some(final_state) = read_interchange(&path) {
        assert_eq!(
            final_state.status, "idle",
            "if file still exists, status should be idle after session ends"
        );
    }

    // 5. PID should be dead after session.
    assert!(
        !pid_is_alive(session_pid),
        "copilot PID {session_pid} should be dead after session",
    );

    // 6. GenericProvider should filter out dead-PID sessions.
    let provider =
        super::generic::GenericProvider::new("copilot").expect("GenericProvider should initialize");
    let sessions = provider
        .discover_sessions()
        .expect("discover_sessions should succeed");
    assert!(
        !sessions.iter().any(|s| s.id == session_id),
        "dead-PID session should be cleaned up by GenericProvider"
    );

    // File should be gone after GenericProvider cleaned it up.
    assert!(
        !path.exists(),
        "harness file should be deleted after PID cleanup"
    );
}

#[test]
#[ignore]
fn e2e_copilot_session_fields() {
    let before = snapshot_harness_files();

    let output = Command::new("copilot")
        .args([
            "-p",
            "respond with the single word 'hello'",
            "--allow-all-tools",
            "--no-custom-instructions",
        ])
        .output()
        .expect("copilot binary must be installed and on PATH");
    assert!(output.status.success());

    // File should exist with idle status.
    let (interchange, path) = wait_for_new_harness_file(&before, "copilot", Duration::from_secs(5));

    assert_eq!(interchange.harness, "copilot");

    // When run from the cortado repo, git metadata should be present.
    if let Some(repo) = &interchange.repository {
        assert!(
            repo.contains("cortado"),
            "repo should contain 'cortado', got: {repo}"
        );
    }
    if let Some(branch) = &interchange.branch {
        assert!(!branch.is_empty(), "branch should not be empty");
    }

    // Clean up.
    let _ = fs::remove_file(&path);
}

// ---------------------------------------------------------------------------
// OpenCode e2e tests
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn e2e_opencode_session_lifecycle() {
    let before = snapshot_harness_files();

    let handle = thread::spawn(|| {
        let output = Command::new("opencode")
            .args(["run", "respond with the single word 'hello'"])
            .output()
            .expect("opencode binary must be installed and on PATH");
        assert!(
            output.status.success(),
            "opencode session should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    });

    // 1. Wait for harness file to appear.
    let (interchange, path) =
        wait_for_new_harness_file(&before, "opencode", Duration::from_secs(30));

    // 2. Verify fields while session is active.
    assert_eq!(interchange.version, 1);
    assert_eq!(interchange.harness, "opencode");
    assert!(!interchange.id.is_empty(), "session ID should be set");
    assert!(!interchange.cwd.is_empty(), "cwd should be set");
    assert!(
        interchange.last_active_at.is_some(),
        "last_active_at should be set"
    );
    assert!(
        interchange.status == "working" || interchange.status == "idle",
        "status should be working or idle during session, got: {}",
        interchange.status
    );

    // 3. Wait for session to complete.
    handle.join().expect("opencode thread should not panic");

    // 4. After session ends, the plugin writes "idle". If the Cortado app
    //    is running, GenericProvider may clean up the file before we read it.
    //    Both outcomes are valid: "idle" file exists, or file already cleaned up.
    let session_id = interchange.id.clone();
    let session_pid = interchange.pid;
    if let Some(final_state) = read_interchange(&path) {
        assert_eq!(
            final_state.status, "idle",
            "if file still exists, status should be idle after session ends"
        );
    }

    // 5. PID should be dead after session.
    assert!(
        !pid_is_alive(session_pid),
        "opencode PID {session_pid} should be dead after session",
    );

    // 6. GenericProvider should filter out dead-PID sessions.
    let provider = super::generic::GenericProvider::new("opencode")
        .expect("GenericProvider should initialize");
    let sessions = provider
        .discover_sessions()
        .expect("discover_sessions should succeed");
    assert!(
        !sessions.iter().any(|s| s.id == session_id),
        "dead-PID session should be cleaned up by GenericProvider"
    );

    assert!(
        !path.exists(),
        "harness file should be deleted after PID cleanup"
    );
}

#[test]
#[ignore]
fn e2e_opencode_session_fields() {
    let before = snapshot_harness_files();

    let output = Command::new("opencode")
        .args(["run", "respond with the single word 'hello'"])
        .output()
        .expect("opencode binary must be installed and on PATH");
    assert!(output.status.success());

    let (interchange, path) =
        wait_for_new_harness_file(&before, "opencode", Duration::from_secs(5));

    assert_eq!(interchange.harness, "opencode");

    if let Some(repo) = &interchange.repository {
        assert!(
            repo.contains("cortado"),
            "repo should contain 'cortado', got: {repo}"
        );
    }
    if let Some(branch) = &interchange.branch {
        assert!(!branch.is_empty(), "branch should not be empty");
    }

    let _ = fs::remove_file(&path);
}
