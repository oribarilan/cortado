//! Shared helpers for GitHub-backed feeds (`github-pr`, `github-actions`).
//!
//! Houses the `gh` CLI preflight check and error-message helpers so that
//! every GitHub feed reuses the same binary/auth validation logic.

use std::time::Duration;

use anyhow::{bail, Result};

use crate::feed::{
    dependency::{classify_dependency_result, DependencyCheck},
    process::{CommandInvocation, ProcessRunner},
};

/// Timeout applied to `gh --version` and `gh auth status` preflight commands.
pub(crate) const GH_COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

/// Error shown when `gh` is not found on PATH.
pub(crate) const GH_MISSING_MESSAGE: &str =
    "GitHub feed requires `gh` CLI. Install it from https://cli.github.com/ and run `gh auth login`.";

/// Error shown when `gh` is installed but the user is not authenticated.
pub(crate) const GH_UNAUTHENTICATED_MESSAGE: &str =
    "GitHub feed requires `gh` authentication. Run `gh auth login` and retry.";

/// Verifies that `gh` is installed and authenticated.
///
/// Runs `gh --version` then `gh auth status`, mapping failures to
/// user-friendly error messages.
pub(crate) async fn ensure_gh_available(process_runner: &dyn ProcessRunner) -> Result<()> {
    let version_invocation = CommandInvocation::new("gh", ["--version"], GH_COMMAND_TIMEOUT);
    let version_display = version_invocation.display();
    let version_check = classify_dependency_result(
        &version_display,
        process_runner.run(version_invocation).await,
    );

    match version_check {
        DependencyCheck::MissingBinary => {
            bail!(GH_MISSING_MESSAGE);
        }
        DependencyCheck::InvocationError(error) => {
            bail!("{error}");
        }
        DependencyCheck::Healthy(_) => {}
    }

    let auth_invocation = CommandInvocation::new("gh", ["auth", "status"], GH_COMMAND_TIMEOUT);
    let auth_display = auth_invocation.display();
    let auth_check =
        classify_dependency_result(&auth_display, process_runner.run(auth_invocation).await);

    match auth_check {
        DependencyCheck::MissingBinary => bail!(GH_MISSING_MESSAGE),
        DependencyCheck::Healthy(_) => Ok(()),
        DependencyCheck::InvocationError(error) => {
            if looks_like_gh_auth_error(&error.stdout, &error.stderr) {
                bail!(GH_UNAUTHENTICATED_MESSAGE);
            }

            bail!("{error}");
        }
    }
}

/// Heuristic: returns `true` when combined stdout/stderr looks like a
/// `gh` authentication error.
pub(crate) fn looks_like_gh_auth_error(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{}\n{}", stdout, stderr).to_ascii_lowercase();

    combined.contains("gh auth login")
        || combined.contains("not logged into")
        || combined.contains("authentication") && combined.contains("required")
        || combined.contains("log in to github")
}

/// Formats an exit-code + output pair for inclusion in error messages.
pub(crate) fn non_zero_exit_context(exit_code: Option<i32>, stdout: &str, stderr: &str) -> String {
    let exit = match exit_code {
        Some(code) => format!("exit code {code}"),
        None => "unknown exit status".to_string(),
    };

    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return format!("{exit}: {stderr}");
    }

    let stdout = stdout.trim();
    if !stdout.is_empty() {
        return format!("{exit}: {stdout}");
    }

    exit
}
