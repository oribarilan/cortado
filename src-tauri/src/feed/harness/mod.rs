use anyhow::Result;

/// Status of a harness session, shared across all providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Agent is actively working (processing, executing tools).
    Working,
    /// Agent asked the user a question.
    Question,
    /// Agent requested tool approval from user.
    Approval,
    /// Session is idle (agent finished, waiting for user input).
    Idle,
    /// Status could not be determined.
    Unknown,
}

/// A discovered harness session, provider-agnostic.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub id: String,
    /// Working directory of the session.
    pub cwd: String,
    /// Repository name (e.g., "oribarilan/cortado").
    pub repository: Option<String>,
    /// Git branch name.
    pub branch: Option<String>,
    /// Current session status.
    pub status: SessionStatus,
    /// PID of the owning process (for terminal focus).
    pub pid: u32,
    /// Agent-generated session summary.
    pub summary: Option<String>,
    /// ISO 8601 timestamp of the last event (last activity).
    pub last_active_at: Option<String>,
}

/// Abstracts harness session discovery and status inference.
///
/// Each provider implements session discovery for a specific coding harness
/// (e.g., Copilot CLI, Claude Code). The generic `HarnessFeed` delegates
/// to a provider and maps results into activities.
pub trait HarnessProvider: Send + Sync {
    /// Human-readable name of the harness (e.g., "Copilot", "Claude Code").
    #[allow(dead_code)] // Used for display/logging in future.
    fn harness_name(&self) -> &str;

    /// Feed type identifier (e.g., "copilot-session", "claude-code-session").
    fn feed_type(&self) -> &str;

    /// Discover all active sessions.
    fn discover_sessions(&self) -> Result<Vec<SessionInfo>>;

    /// Returns directories that should be watched for file changes.
    /// When files change in these directories, the feed re-polls immediately.
    /// Returns `None` if file watching is not supported (uses timer-only polling).
    #[allow(dead_code)] // Will be used when file-watch polling is wired up.
    fn watch_paths(&self) -> Option<Vec<std::path::PathBuf>> {
        None
    }
}

pub mod feed;
pub mod generic;

#[cfg(test)]
mod e2e;
