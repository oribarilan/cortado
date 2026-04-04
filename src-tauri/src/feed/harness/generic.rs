use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use super::{HarnessProvider, SessionInfo, SessionStatus};

#[allow(dead_code)] // Used inside discover_sessions.
const MAX_SESSIONS: usize = 20;

/// Generic harness session discovery provider.
///
/// Reads session state from the generic interchange format: JSON files in
/// `~/.config/cortado/harness/`. Each file represents one active session
/// written by a harness adapter. Stale files (dead PIDs) are cleaned up
/// automatically.
#[allow(dead_code)] // Used by HarnessFeed when generic feeds are configured.
pub struct GenericProvider {
    harness_name: String,
    feed_type: String,
    harness_dir: PathBuf,
}

impl GenericProvider {
    /// Creates a provider for the given harness name.
    ///
    /// The `harness_name` identifies which harness sessions to discover
    /// (e.g., `"opencode"`). Only interchange files whose `harness` field
    /// matches this name are included.
    #[allow(dead_code)] // Will be used when generic feeds are wired into instantiate_harness_feed.
    pub fn new(harness_name: &str) -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not resolve home directory"))?;
        Ok(Self {
            harness_name: harness_name.to_string(),
            feed_type: format!("{harness_name}-session"),
            harness_dir: home.join(".config/cortado/harness"),
        })
    }

    /// Creates a provider with a custom harness directory (for testing).
    #[cfg(test)]
    fn with_harness_dir(harness_name: &str, harness_dir: PathBuf) -> Self {
        Self {
            harness_name: harness_name.to_string(),
            feed_type: format!("{harness_name}-session"),
            harness_dir,
        }
    }
}

impl HarnessProvider for GenericProvider {
    fn harness_name(&self) -> &str {
        &self.harness_name
    }

    fn feed_type(&self) -> &str {
        &self.feed_type
    }

    fn discover_sessions(&self) -> Result<Vec<SessionInfo>> {
        if !self.harness_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();

        let entries = fs::read_dir(&self.harness_dir)
            .with_context(|| format!("failed reading {}", self.harness_dir.display()))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Only process .json files (non-recursive).
            let is_json = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
            if !is_json || !path.is_file() {
                continue;
            }

            match try_parse_interchange(&path) {
                Ok(interchange) => {
                    // Skip files with unsupported version.
                    if interchange.version != 1 {
                        eprintln!(
                            "generic provider: skipping {} (unsupported version {})",
                            path.display(),
                            interchange.version
                        );
                        continue;
                    }

                    // Skip files for a different harness.
                    if interchange.harness != self.harness_name {
                        continue;
                    }

                    // Check PID liveness; clean up stale files.
                    if !is_pid_alive(interchange.pid) {
                        let _ = fs::remove_file(&path);
                        continue;
                    }

                    sessions.push(SessionInfo {
                        id: interchange.id,
                        cwd: interchange.cwd,
                        repository: interchange.repository,
                        branch: interchange.branch,
                        status: parse_status(&interchange.status),
                        pid: interchange.pid,
                        summary: interchange.summary,
                        last_active_at: Some(interchange.last_active_at),
                    });

                    if sessions.len() >= MAX_SESSIONS {
                        break;
                    }
                }
                Err(_) => {
                    eprintln!(
                        "generic provider: skipping malformed file {}",
                        path.display()
                    );
                    continue;
                }
            }
        }

        Ok(sessions)
    }

    fn watch_paths(&self) -> Option<Vec<std::path::PathBuf>> {
        Some(vec![self.harness_dir.clone()])
    }
}

/// Session data in the generic interchange JSON format.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields read via serde deserialization.
struct InterchangeSession {
    version: u32,
    harness: String,
    id: String,
    pid: u32,
    cwd: String,
    status: String,
    last_active_at: String,
    repository: Option<String>,
    branch: Option<String>,
    summary: Option<String>,
}

/// Parses a JSON interchange file into an `InterchangeSession`.
#[allow(dead_code)] // Used inside discover_sessions.
fn try_parse_interchange(path: &Path) -> Result<InterchangeSession> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?;
    let session: InterchangeSession = serde_json::from_str(&content)
        .with_context(|| format!("failed parsing {}", path.display()))?;
    Ok(session)
}

/// Maps a status string from the interchange format to `SessionStatus`.
#[allow(dead_code)] // Used inside discover_sessions.
fn parse_status(s: &str) -> SessionStatus {
    match s {
        "working" => SessionStatus::Working,
        "question" => SessionStatus::Question,
        "approval" => SessionStatus::Approval,
        "idle" => SessionStatus::Idle,
        _ => SessionStatus::Unknown,
    }
}

/// Checks if a process with the given PID is alive.
#[allow(dead_code)] // Used inside discover_sessions.
fn is_pid_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) only checks process existence -- no signal is sent.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_interchange_file(dir: &Path, pid: u32, harness: &str, status: &str) {
        let json = serde_json::json!({
            "version": 1,
            "harness": harness,
            "id": format!("test-{pid}"),
            "pid": pid,
            "cwd": "/tmp/test",
            "status": status,
            "last_active_at": "2026-01-01T00:00:00Z"
        });
        let path = dir.join(format!("{pid}.json"));
        fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    }

    #[test]
    fn parse_status_values() {
        assert_eq!(parse_status("working"), SessionStatus::Working);
        assert_eq!(parse_status("question"), SessionStatus::Question);
        assert_eq!(parse_status("approval"), SessionStatus::Approval);
        assert_eq!(parse_status("idle"), SessionStatus::Idle);
        assert_eq!(parse_status("something_else"), SessionStatus::Unknown);
        assert_eq!(parse_status(""), SessionStatus::Unknown);
    }

    #[test]
    fn valid_file_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let pid = std::process::id();
        let json = serde_json::json!({
            "version": 1,
            "harness": "opencode",
            "id": "session-abc",
            "pid": pid,
            "cwd": "/home/user/project",
            "status": "working",
            "last_active_at": "2026-01-15T10:30:00Z",
            "repository": "user/repo",
            "branch": "feature-branch",
            "summary": "Implementing generic provider"
        });
        let path = dir.path().join("session.json");
        fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert_eq!(sessions.len(), 1);

        let s = &sessions[0];
        assert_eq!(s.id, "session-abc");
        assert_eq!(s.cwd, "/home/user/project");
        assert_eq!(s.repository.as_deref(), Some("user/repo"));
        assert_eq!(s.branch.as_deref(), Some("feature-branch"));
        assert_eq!(s.summary.as_deref(), Some("Implementing generic provider"));
        assert_eq!(s.status, SessionStatus::Working);
        assert_eq!(s.pid, pid);
        assert_eq!(s.last_active_at.as_deref(), Some("2026-01-15T10:30:00Z"));
    }

    #[test]
    fn filters_by_harness_name() {
        let dir = tempfile::tempdir().unwrap();
        let pid = std::process::id();

        // File for "opencode" harness
        write_interchange_file(dir.path(), pid, "opencode", "working");

        // File for "other" harness (different filename to avoid collision)
        let json = serde_json::json!({
            "version": 1,
            "harness": "other",
            "id": "other-session",
            "pid": pid,
            "cwd": "/tmp/other",
            "status": "idle",
            "last_active_at": "2026-01-01T00:00:00Z"
        });
        fs::write(
            dir.path().join("other.json"),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, format!("test-{pid}"));
    }

    #[test]
    fn skips_unknown_version() {
        let dir = tempfile::tempdir().unwrap();
        let pid = std::process::id();
        let json = serde_json::json!({
            "version": 2,
            "harness": "opencode",
            "id": "future-session",
            "pid": pid,
            "cwd": "/tmp/test",
            "status": "working",
            "last_active_at": "2026-01-01T00:00:00Z"
        });
        fs::write(
            dir.path().join("future.json"),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn skips_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("bad.json"), "this is not valid json").unwrap();

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn missing_directory_returns_empty() {
        let provider = GenericProvider::with_harness_dir(
            "opencode",
            PathBuf::from("/nonexistent/cortado/harness"),
        );
        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn empty_directory_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn stale_pid_cleaned_up() {
        let dir = tempfile::tempdir().unwrap();
        let dead_pid: u32 = 99_999_999;
        write_interchange_file(dir.path(), dead_pid, "opencode", "working");

        let file_path = dir.path().join(format!("{dead_pid}.json"));
        assert!(file_path.exists(), "file should exist before discovery");

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
        assert!(
            !file_path.exists(),
            "stale file should be deleted after discovery"
        );
    }

    #[test]
    fn live_pid_included() {
        let dir = tempfile::tempdir().unwrap();
        let pid = std::process::id();
        write_interchange_file(dir.path(), pid, "opencode", "idle");

        let provider = GenericProvider::with_harness_dir("opencode", dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].pid, pid);
        assert_eq!(sessions[0].status, SessionStatus::Idle);
    }

    #[test]
    fn pid_liveness_current_process() {
        assert!(is_pid_alive(std::process::id()));
    }

    #[test]
    fn pid_liveness_dead_process() {
        assert!(!is_pid_alive(99_999_999));
    }
}
