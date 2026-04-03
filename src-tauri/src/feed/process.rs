use std::{
    error::Error,
    fmt::{self, Display},
    io::ErrorKind,
    process::Stdio,
    time::Duration,
};

use async_trait::async_trait;
use tokio::process::Command;

/// Command invocation details for external process execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub program: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}

impl CommandInvocation {
    /// Creates a command invocation with the provided timeout.
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
        timeout: Duration,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            timeout,
        }
    }

    /// Returns a human-readable representation of the command.
    pub fn display(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(printable_arg(&self.program));

        for arg in &self.args {
            parts.push(printable_arg(arg));
        }

        parts.join(" ")
    }
}

/// Captured command output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    /// Returns `true` when the process exited successfully.
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Process execution failures that occur before obtaining exit output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    NotFound { program: String },
    Spawn { program: String, message: String },
    Timeout { command: String, timeout: Duration },
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { program } => {
                write!(f, "required binary `{program}` was not found")
            }
            Self::Spawn { program, message } => {
                write!(f, "failed spawning `{program}`: {message}")
            }
            Self::Timeout { command, timeout } => {
                write!(
                    f,
                    "command `{command}` timed out after {}s",
                    timeout.as_secs()
                )
            }
        }
    }
}

impl Error for CommandError {}

/// Async process runner used by feed implementations.
#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn run(
        &self,
        invocation: CommandInvocation,
    ) -> std::result::Result<CommandOutput, CommandError>;
}

/// Default tokio-based process runner.
#[derive(Debug, Default)]
pub struct TokioProcessRunner;

#[async_trait]
impl ProcessRunner for TokioProcessRunner {
    async fn run(
        &self,
        invocation: CommandInvocation,
    ) -> std::result::Result<CommandOutput, CommandError> {
        let command_repr = invocation.display();
        let program = invocation.program.clone();

        let mut command = Command::new(&invocation.program);
        command.args(&invocation.args);
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let output = match tokio::time::timeout(invocation.timeout, command.output()).await {
            Ok(result) => result,
            Err(_) => {
                return Err(CommandError::Timeout {
                    command: command_repr,
                    timeout: invocation.timeout,
                })
            }
        };

        let output = match output {
            Ok(output) => output,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                return Err(CommandError::NotFound { program });
            }
            Err(err) => {
                return Err(CommandError::Spawn {
                    program,
                    message: err.to_string(),
                });
            }
        };

        Ok(CommandOutput {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn printable_arg(arg: &str) -> String {
    if arg
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:=@".contains(character))
    {
        return arg.to_string();
    }

    format!("{arg:?}")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn command_invocation_new_builds_correctly() {
        let inv = CommandInvocation::new("git", ["status", "--short"], Duration::from_secs(5));
        assert_eq!(inv.program, "git");
        assert_eq!(inv.args, vec!["status", "--short"]);
        assert_eq!(inv.timeout, Duration::from_secs(5));
    }

    #[test]
    fn command_invocation_display_joins_args() {
        let inv = CommandInvocation::new(
            "gh",
            ["pr", "list", "--repo", "org/repo"],
            Duration::from_secs(5),
        );
        assert_eq!(inv.display(), "gh pr list --repo org/repo");
    }

    #[test]
    fn command_invocation_display_quotes_special_args() {
        let inv = CommandInvocation::new("sh", ["-c", "echo hello world"], Duration::from_secs(5));
        let display = inv.display();
        // "-c" contains only safe chars so it's unquoted; "echo hello world" has spaces so it's quoted
        assert!(display.starts_with("sh -c "));
        assert!(display.contains("echo hello world"));
    }

    #[test]
    fn command_output_succeeded_for_zero_exit() {
        let output = CommandOutput {
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(output.succeeded());
    }

    #[test]
    fn command_output_not_succeeded_for_nonzero_exit() {
        let output = CommandOutput {
            exit_code: Some(1),
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(!output.succeeded());
    }

    #[test]
    fn command_output_not_succeeded_for_none_exit() {
        let output = CommandOutput {
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(!output.succeeded());
    }

    #[test]
    fn command_error_display_not_found() {
        let err = CommandError::NotFound {
            program: "gh".to_string(),
        };
        assert_eq!(err.to_string(), "required binary `gh` was not found");
    }

    #[test]
    fn command_error_display_spawn() {
        let err = CommandError::Spawn {
            program: "gh".to_string(),
            message: "permission denied".to_string(),
        };
        assert_eq!(err.to_string(), "failed spawning `gh`: permission denied");
    }

    #[test]
    fn command_error_display_timeout() {
        let err = CommandError::Timeout {
            command: "gh pr list".to_string(),
            timeout: Duration::from_secs(30),
        };
        assert_eq!(err.to_string(), "command `gh pr list` timed out after 30s");
    }

    #[test]
    fn printable_arg_safe_chars_unquoted() {
        assert_eq!(printable_arg("hello"), "hello");
        assert_eq!(printable_arg("--repo"), "--repo");
        assert_eq!(printable_arg("org/repo"), "org/repo");
        assert_eq!(printable_arg("user@host"), "user@host");
        assert_eq!(printable_arg("key=value"), "key=value");
    }

    #[test]
    fn printable_arg_special_chars_quoted() {
        let result = printable_arg("echo hello world");
        assert!(result.starts_with('"'));
        assert!(result.ends_with('"'));
    }
}
