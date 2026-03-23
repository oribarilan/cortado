use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::feed::process::{CommandError, CommandOutput};

/// Normalized dependency check outcomes for command-based feeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyCheck {
    MissingBinary,
    InvocationError(DependencyInvocationError),
    Healthy(CommandOutput),
}

/// Detailed invocation failure information for command-based dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInvocationError {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

impl Display for DependencyInvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DependencyInvocationError {}

/// Classifies command execution results into dependency outcomes.
pub fn classify_dependency_result(
    command_display: &str,
    result: std::result::Result<CommandOutput, CommandError>,
) -> DependencyCheck {
    match result {
        Err(CommandError::NotFound { .. }) => DependencyCheck::MissingBinary,
        Err(CommandError::Spawn { message, .. }) => {
            DependencyCheck::InvocationError(DependencyInvocationError {
                command: command_display.to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                message: format!("failed invoking `{command_display}`: {message}"),
            })
        }
        Err(CommandError::Timeout { timeout, .. }) => {
            DependencyCheck::InvocationError(DependencyInvocationError {
                command: command_display.to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                message: format!(
                    "command `{command_display}` timed out after {}s",
                    timeout.as_secs()
                ),
            })
        }
        Ok(output) if output.succeeded() => DependencyCheck::Healthy(output),
        Ok(output) => {
            let exit_code = output.exit_code;
            let stdout = output.stdout;
            let stderr = output.stderr;

            let message = format_non_zero_exit(command_display, exit_code, &stdout, &stderr);

            DependencyCheck::InvocationError(DependencyInvocationError {
                command: command_display.to_string(),
                exit_code,
                stdout,
                stderr,
                message,
            })
        }
    }
}

fn format_non_zero_exit(
    command_display: &str,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> String {
    let exit_context = match exit_code {
        Some(code) => format!("exit code {code}"),
        None => "unknown exit status".to_string(),
    };

    let stderr = stderr.trim();

    if !stderr.is_empty() {
        return format!("`{command_display}` failed with {exit_context}: {stderr}");
    }

    let stdout = stdout.trim();

    if !stdout.is_empty() {
        return format!("`{command_display}` failed with {exit_context}: {stdout}");
    }

    format!("`{command_display}` failed with {exit_context}")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::feed::process::{CommandError, CommandOutput};

    use super::{classify_dependency_result, DependencyCheck};

    #[test]
    fn classify_dependency_result_missing_binary() {
        let result = classify_dependency_result(
            "gh --version",
            Err(CommandError::NotFound {
                program: "gh".to_string(),
            }),
        );

        assert!(matches!(result, DependencyCheck::MissingBinary));
    }

    #[test]
    fn classify_dependency_result_non_zero_exit_keeps_stderr() {
        let result = classify_dependency_result(
            "gh pr list",
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "gh auth login".to_string(),
            }),
        );

        let DependencyCheck::InvocationError(error) = result else {
            panic!("expected invocation error");
        };

        assert!(error.message.contains("gh auth login"));
        assert_eq!(error.exit_code, Some(1));
    }

    #[test]
    fn classify_dependency_result_timeout() {
        let result = classify_dependency_result(
            "gh auth status",
            Err(CommandError::Timeout {
                command: "gh auth status".to_string(),
                timeout: Duration::from_secs(10),
            }),
        );

        let DependencyCheck::InvocationError(error) = result else {
            panic!("expected invocation error");
        };

        assert!(error.message.contains("timed out after 10s"));
    }
}
