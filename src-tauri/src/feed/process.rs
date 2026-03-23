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

    /// Creates a shell invocation (`sh -c <command>`).
    pub fn shell(command: impl Into<String>, timeout: Duration) -> Self {
        Self::new("sh", ["-c".to_string(), command.into()], timeout)
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
