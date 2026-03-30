use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use super::{HarnessProvider, SessionInfo, SessionStatus};

const MAX_SESSIONS: usize = 20;

/// Copilot CLI session discovery provider.
///
/// Reads session state from `~/.copilot/session-state/`, detecting active
/// sessions via lock files and inferring status from the last event in
/// `events.jsonl`.
#[allow(dead_code)] // Used by HarnessFeed (task 02).
pub struct CopilotProvider {
    state_dir: PathBuf,
}

impl CopilotProvider {
    /// Creates a provider using the default Copilot CLI session state directory.
    pub fn new() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not resolve home directory"))?;
        Ok(Self {
            state_dir: home.join(".copilot").join("session-state"),
        })
    }

    /// Creates a provider with a custom state directory (for testing).
    #[cfg(test)]
    fn with_state_dir(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }
}

impl HarnessProvider for CopilotProvider {
    fn harness_name(&self) -> &str {
        "Copilot"
    }

    fn feed_type(&self) -> &str {
        "copilot-session"
    }

    fn discover_sessions(&self) -> Result<Vec<SessionInfo>> {
        if !self.state_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();

        let entries = fs::read_dir(&self.state_dir)
            .with_context(|| format!("failed reading {}", self.state_dir.display()))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let session_dir = entry.path();
            if !session_dir.is_dir() {
                continue;
            }

            if let Some(session) = try_discover_session(&session_dir) {
                sessions.push(session);
                if sessions.len() >= MAX_SESSIONS {
                    break;
                }
            }
        }

        Ok(sessions)
    }
}

/// Attempts to discover a single session from a session directory.
/// Returns `None` if no live lock file exists or parsing fails.
fn try_discover_session(session_dir: &Path) -> Option<SessionInfo> {
    let pid = find_live_pid(session_dir)?;

    let workspace_path = session_dir.join("workspace.yaml");
    let workspace = parse_workspace(&workspace_path)?;

    let events_path = session_dir.join("events.jsonl");
    let (status, last_active_at) = infer_status_from_last_event(&events_path);

    Some(SessionInfo {
        id: workspace.id,
        cwd: workspace.cwd,
        repository: workspace.repository,
        branch: workspace.branch,
        status,
        pid,
        summary: workspace.summary.filter(|s| !s.is_empty()),
        last_active_at,
    })
}

/// Scans for `inuse.*.lock` files and returns the PID if the process is alive.
fn find_live_pid(session_dir: &Path) -> Option<u32> {
    let entries = fs::read_dir(session_dir).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if let Some(pid) = parse_lock_filename(&name_str) {
            if is_pid_alive(pid) {
                return Some(pid);
            }
        }
    }

    None
}

/// Extracts PID from a lock filename like `inuse.12345.lock`.
fn parse_lock_filename(name: &str) -> Option<u32> {
    let name = name.strip_prefix("inuse.")?;
    let name = name.strip_suffix(".lock")?;
    name.parse().ok()
}

/// Checks if a process with the given PID is alive.
fn is_pid_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) only checks process existence — no signal is sent.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Workspace metadata from `workspace.yaml`.
#[derive(Debug, Deserialize)]
struct WorkspaceYaml {
    id: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    summary: Option<String>,
}

/// Parses `workspace.yaml`. Returns `None` if missing or malformed.
fn parse_workspace(path: &Path) -> Option<WorkspaceYaml> {
    let content = fs::read_to_string(path).ok()?;
    serde_saphyr::from_str(&content).ok()
}

/// Reads the last events from `events.jsonl` and infers session status + timestamp.
///
/// Checks both the last line and the penultimate line. When the last event is
/// `tool.execution_start`, the preceding `assistant.message` may carry a more
/// accurate status (e.g., Approval for pending tool requests). The CLI emits
/// both events at the same instant, so the last line alone can be misleading.
fn infer_status_from_last_event(path: &Path) -> (SessionStatus, Option<String>) {
    let lines = match read_last_lines(path, 2) {
        Some(lines) if !lines.is_empty() => lines,
        _ => return (SessionStatus::Unknown, None),
    };

    let last = &lines[lines.len() - 1];
    let timestamp = extract_event_timestamp(last);
    let last_status = parse_event_status(last);

    // If last event is tool.execution_start (Working), check if the preceding
    // assistant.message gives a more specific status (Question or Approval).
    if last_status == SessionStatus::Working && lines.len() == 2 {
        let prev_status = parse_event_status(&lines[0]);
        if matches!(
            prev_status,
            SessionStatus::Question | SessionStatus::Approval
        ) {
            return (prev_status, timestamp);
        }
    }

    (last_status, timestamp)
}

/// Reads the last N non-empty lines of a file via reverse seek.
fn read_last_lines(path: &Path, n: usize) -> Option<Vec<String>> {
    let mut file = fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();

    if file_len == 0 {
        return None;
    }

    // Read up to last 64KB to find the last complete lines.
    let read_size = file_len.min(65_536);
    let start = file_len - read_size;

    file.seek(SeekFrom::Start(start)).ok()?;
    let mut buf = String::with_capacity(read_size as usize);
    file.read_to_string(&mut buf).ok()?;

    let lines: Vec<String> = buf
        .lines()
        .rev()
        .filter(|line| !line.trim().is_empty())
        .take(n)
        .map(String::from)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

/// Parses a JSON event line and maps the event type to a `SessionStatus`.
fn parse_event_status(json_line: &str) -> SessionStatus {
    let event: serde_json::Value = match serde_json::from_str(json_line) {
        Ok(v) => v,
        Err(_) => return SessionStatus::Unknown,
    };

    let event_type = match event.get("type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return SessionStatus::Unknown,
    };

    match event_type {
        "assistant.turn_start" | "user.message" => SessionStatus::Working,
        "tool.execution_start" => classify_tool_execution(&event),
        "assistant.message" => classify_assistant_message(&event),
        "assistant.turn_end" | "tool.execution_complete" => SessionStatus::Idle,
        "session.shutdown" | "abort" => SessionStatus::Idle,
        _ => SessionStatus::Unknown,
    }
}

/// Extracts the `timestamp` field from a JSON event line.
fn extract_event_timestamp(json_line: &str) -> Option<String> {
    let event: serde_json::Value = serde_json::from_str(json_line).ok()?;
    event
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Classifies a `tool.execution_start` event.
/// `ask_user` tool means the agent is waiting for user input (Question).
fn classify_tool_execution(event: &serde_json::Value) -> SessionStatus {
    let tool_name = event
        .get("data")
        .and_then(|d| d.get("toolName"))
        .and_then(|n| n.as_str());

    match tool_name {
        Some("ask_user") => SessionStatus::Question,
        _ => SessionStatus::Working,
    }
}

/// Classifies an `assistant.message` event by checking `data.toolRequests`.
fn classify_assistant_message(event: &serde_json::Value) -> SessionStatus {
    let tool_requests = match event
        .get("data")
        .and_then(|d| d.get("toolRequests"))
        .and_then(|t| t.as_array())
    {
        Some(requests) if !requests.is_empty() => requests,
        _ => return SessionStatus::Idle,
    };

    let has_ask_user = tool_requests.iter().any(|req| {
        req.get("name")
            .and_then(|n| n.as_str())
            .is_some_and(|name| name == "ask_user")
    });

    if has_ask_user {
        SessionStatus::Question
    } else {
        SessionStatus::Approval
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_session_dir(
        base: &Path,
        session_id: &str,
        workspace_yaml: &str,
        events_jsonl: Option<&str>,
        lock_pid: Option<u32>,
    ) -> PathBuf {
        let session_dir = base.join(session_id);
        fs::create_dir_all(&session_dir).unwrap();

        fs::write(session_dir.join("workspace.yaml"), workspace_yaml).unwrap();

        if let Some(events) = events_jsonl {
            fs::write(session_dir.join("events.jsonl"), events).unwrap();
        }

        if let Some(pid) = lock_pid {
            let lock_name = format!("inuse.{pid}.lock");
            fs::write(session_dir.join(&lock_name), pid.to_string()).unwrap();
        }

        session_dir
    }

    fn current_pid() -> u32 {
        std::process::id()
    }

    #[test]
    fn parse_lock_filename_valid() {
        assert_eq!(parse_lock_filename("inuse.12345.lock"), Some(12345));
        assert_eq!(parse_lock_filename("inuse.1.lock"), Some(1));
    }

    #[test]
    fn parse_lock_filename_invalid() {
        assert_eq!(parse_lock_filename("inuse..lock"), None);
        assert_eq!(parse_lock_filename("inuse.abc.lock"), None);
        assert_eq!(parse_lock_filename("other.12345.lock"), None);
        assert_eq!(parse_lock_filename("inuse.12345.txt"), None);
    }

    #[test]
    fn pid_liveness_current_process_is_alive() {
        assert!(is_pid_alive(current_pid()));
    }

    #[test]
    fn pid_liveness_nonexistent_pid_is_dead() {
        // PID 99999999 is extremely unlikely to exist.
        assert!(!is_pid_alive(99_999_999));
    }

    #[test]
    fn parse_workspace_valid() {
        let yaml = r#"
id: abc-123
cwd: /home/user/project
repository: user/repo
branch: main
summary: some summary
"#;
        let ws: WorkspaceYaml = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(ws.id, "abc-123");
        assert_eq!(ws.cwd, "/home/user/project");
        assert_eq!(ws.repository.as_deref(), Some("user/repo"));
        assert_eq!(ws.branch.as_deref(), Some("main"));
    }

    #[test]
    fn parse_workspace_minimal() {
        let yaml = "id: minimal-session\n";
        let ws: WorkspaceYaml = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(ws.id, "minimal-session");
        assert_eq!(ws.cwd, "");
        assert!(ws.repository.is_none());
        assert!(ws.branch.is_none());
    }

    #[test]
    fn parse_workspace_malformed_returns_none() {
        let result = parse_workspace(Path::new("/nonexistent/workspace.yaml"));
        assert!(result.is_none());
    }

    #[test]
    fn event_status_working_events() {
        let events = [
            r#"{"type":"assistant.turn_start","data":{}}"#,
            r#"{"type":"tool.execution_start","data":{"toolName":"bash"}}"#,
            r#"{"type":"tool.execution_start","data":{}}"#,
            r#"{"type":"user.message","data":{}}"#,
        ];
        for event in &events {
            assert_eq!(
                parse_event_status(event),
                SessionStatus::Working,
                "expected Working for: {event}"
            );
        }
    }

    #[test]
    fn event_status_idle_events() {
        let events = [
            r#"{"type":"assistant.turn_end","data":{}}"#,
            r#"{"type":"tool.execution_complete","data":{}}"#,
            r#"{"type":"session.shutdown","data":{}}"#,
            r#"{"type":"abort","data":{}}"#,
        ];
        for event in &events {
            assert_eq!(
                parse_event_status(event),
                SessionStatus::Idle,
                "expected Idle for: {event}"
            );
        }
    }

    #[test]
    fn event_status_question() {
        let event = r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"ask_user","arguments":{}}]}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Question);
    }

    #[test]
    fn event_status_question_from_tool_execution() {
        let event =
            r#"{"type":"tool.execution_start","data":{"toolName":"ask_user","arguments":{}}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Question);
    }

    #[test]
    fn event_status_approval() {
        let event = r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"bash","arguments":{}}]}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Approval);
    }

    #[test]
    fn event_status_assistant_message_no_tool_requests() {
        let event = r#"{"type":"assistant.message","data":{}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Idle);
    }

    #[test]
    fn event_status_assistant_message_empty_tool_requests() {
        let event = r#"{"type":"assistant.message","data":{"toolRequests":[]}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Idle);
    }

    #[test]
    fn event_status_unknown_event_type() {
        let event = r#"{"type":"some.future.event","data":{}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Unknown);
    }

    #[test]
    fn event_status_malformed_json() {
        assert_eq!(parse_event_status("not json"), SessionStatus::Unknown);
        assert_eq!(parse_event_status(""), SessionStatus::Unknown);
    }

    #[test]
    fn read_last_lines_finds_final_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");

        fs::write(
            &path,
            r#"{"type":"session.start","data":{}}
{"type":"user.message","data":{}}
{"type":"assistant.turn_end","data":{}}
"#,
        )
        .unwrap();

        let lines = read_last_lines(&path, 2).unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("user.message"));
        assert!(lines[1].contains("assistant.turn_end"));
    }

    #[test]
    fn read_last_lines_handles_trailing_newlines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");

        fs::write(&path, "{\"type\":\"idle\"}\n\n\n").unwrap();

        let lines = read_last_lines(&path, 1).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("idle"));
    }

    #[test]
    fn read_last_lines_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");

        fs::write(&path, "").unwrap();
        assert!(read_last_lines(&path, 1).is_none());
    }

    #[test]
    fn tool_execution_start_preceded_by_approval_resolves_to_approval() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");

        fs::write(
            &path,
            r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"bash","arguments":{}}]}}
{"type":"tool.execution_start","data":{"toolName":"bash"}}
"#,
        )
        .unwrap();

        let (status, _) = infer_status_from_last_event(&path);
        assert_eq!(status, SessionStatus::Approval);
    }

    #[test]
    fn tool_execution_start_preceded_by_question_resolves_to_question() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");

        fs::write(
            &path,
            r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"ask_user","arguments":{}}]}}
{"type":"tool.execution_start","data":{"toolName":"ask_user"}}
"#,
        )
        .unwrap();

        let (status, _) = infer_status_from_last_event(&path);
        assert_eq!(status, SessionStatus::Question);
    }

    #[test]
    fn discover_session_with_live_lock() {
        let dir = tempfile::tempdir().unwrap();
        let pid = current_pid();

        setup_session_dir(
            dir.path(),
            "test-session",
            "id: test-session\ncwd: /tmp/test\nrepository: user/repo\nbranch: main\n",
            Some("{\"type\":\"assistant.turn_end\",\"data\":{}}\n"),
            Some(pid),
        );

        let session = try_discover_session(&dir.path().join("test-session"));
        let session = session.expect("should discover session with live PID");
        assert_eq!(session.id, "test-session");
        assert_eq!(session.cwd, "/tmp/test");
        assert_eq!(session.repository.as_deref(), Some("user/repo"));
        assert_eq!(session.branch.as_deref(), Some("main"));
        assert_eq!(session.status, SessionStatus::Idle);
        assert_eq!(session.pid, pid);
    }

    #[test]
    fn discover_session_skips_dead_pid() {
        let dir = tempfile::tempdir().unwrap();

        setup_session_dir(
            dir.path(),
            "dead-session",
            "id: dead-session\ncwd: /tmp\n",
            None,
            Some(99_999_999), // Non-existent PID
        );

        let session = try_discover_session(&dir.path().join("dead-session"));
        assert!(session.is_none());
    }

    #[test]
    fn discover_session_no_lock_file() {
        let dir = tempfile::tempdir().unwrap();

        setup_session_dir(
            dir.path(),
            "no-lock",
            "id: no-lock\ncwd: /tmp\n",
            None,
            None, // No lock file
        );

        let session = try_discover_session(&dir.path().join("no-lock"));
        assert!(session.is_none());
    }

    #[test]
    fn discover_session_missing_workspace_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let session_dir = dir.path().join("no-yaml");
        fs::create_dir_all(&session_dir).unwrap();

        let pid = current_pid();
        let lock_name = format!("inuse.{pid}.lock");
        fs::write(session_dir.join(lock_name), pid.to_string()).unwrap();

        let session = try_discover_session(&session_dir);
        assert!(session.is_none());
    }

    #[test]
    fn discover_session_no_events_gives_unknown_status() {
        let dir = tempfile::tempdir().unwrap();
        let pid = current_pid();

        setup_session_dir(
            dir.path(),
            "no-events",
            "id: no-events\ncwd: /tmp\n",
            None, // No events.jsonl
            Some(pid),
        );

        let session = try_discover_session(&dir.path().join("no-events"));
        let session = session.expect("should discover despite missing events");
        assert_eq!(session.status, SessionStatus::Unknown);
    }

    #[test]
    fn provider_empty_state_dir() {
        let dir = tempfile::tempdir().unwrap();
        let provider = CopilotProvider::with_state_dir(dir.path().to_path_buf());

        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn provider_nonexistent_state_dir() {
        let provider = CopilotProvider::with_state_dir(PathBuf::from("/nonexistent/path"));

        let sessions = provider.discover_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn provider_discovers_multiple_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let pid = current_pid();

        setup_session_dir(
            dir.path(),
            "session-1",
            "id: session-1\ncwd: /tmp/a\n",
            Some("{\"type\":\"user.message\",\"data\":{}}\n"),
            Some(pid),
        );

        setup_session_dir(
            dir.path(),
            "session-2",
            "id: session-2\ncwd: /tmp/b\n",
            Some("{\"type\":\"assistant.turn_end\",\"data\":{}}\n"),
            Some(pid),
        );

        let provider = CopilotProvider::with_state_dir(dir.path().to_path_buf());
        let sessions = provider.discover_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn event_status_session_start() {
        let event = r#"{"type":"session.start","data":{}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Unknown);
    }

    #[test]
    fn event_status_missing_type_field() {
        let event = r#"{"data":{}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Unknown);
    }

    #[test]
    fn classify_tool_execution_non_ask_user() {
        let event = r#"{"type":"tool.execution_start","data":{"toolName":"edit"}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Working);
    }

    #[test]
    fn classify_assistant_message_multiple_tools_with_ask_user() {
        // ask_user among other tools should still be Question
        let event = r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"bash","arguments":{}},{"name":"ask_user","arguments":{}}]}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Question);
    }

    #[test]
    fn classify_assistant_message_multiple_tools_without_ask_user() {
        let event = r#"{"type":"assistant.message","data":{"toolRequests":[{"name":"bash","arguments":{}},{"name":"edit","arguments":{}}]}}"#;
        assert_eq!(parse_event_status(event), SessionStatus::Approval);
    }

    #[test]
    fn read_last_lines_single_line_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        fs::write(&path, r#"{"type":"idle"}"#).unwrap();

        let lines = read_last_lines(&path, 2).unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("idle"));
    }

    #[test]
    fn read_last_lines_requests_more_than_available() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        fs::write(&path, "{\"type\":\"a\"}\n{\"type\":\"b\"}\n").unwrap();

        let lines = read_last_lines(&path, 10).unwrap();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn infer_status_tool_start_after_idle_stays_working() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        // assistant.turn_end (Idle) followed by tool.execution_start (Working)
        // The prev is Idle, not Question/Approval, so result stays Working
        fs::write(
            &path,
            r#"{"type":"assistant.turn_end","data":{}}
{"type":"tool.execution_start","data":{"toolName":"bash"}}
"#,
        )
        .unwrap();

        let (status, _) = infer_status_from_last_event(&path);
        assert_eq!(status, SessionStatus::Working);
    }

    #[test]
    fn workspace_yaml_extra_fields_ignored() {
        let yaml = "id: test\ncwd: /tmp\nsummary: hello\nextra_field: ignored\nanother: 42\n";
        let ws: WorkspaceYaml = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(ws.id, "test");
        assert_eq!(ws.summary.as_deref(), Some("hello"));
    }

    #[test]
    fn workspace_yaml_empty_summary_filtered() {
        // Empty summary should become None in SessionInfo
        let yaml = "id: test\ncwd: /tmp\nsummary: \n";
        let ws: WorkspaceYaml = serde_saphyr::from_str(yaml).unwrap();
        // The YAML parser may return Some("") or None for empty value
        let filtered = ws.summary.filter(|s| !s.is_empty());
        assert!(filtered.is_none());
    }
}
