use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::feed::{
    dependency::{classify_dependency_result, DependencyCheck},
    process::{CommandInvocation, ProcessRunner},
};

const AZ_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(15);
const HOSTED_ADO_ROOT_MESSAGE: &str = "must be a hosted Azure DevOps organization root (`https://dev.azure.com/<organization>` or `https://<organization>.visualstudio.com`)";

pub(crate) const AZ_MISSING_MESSAGE: &str =
    "Azure DevOps feed requires `az` CLI. Install it from https://aka.ms/install-azure-cli and run `az login`.";
pub(crate) const AZ_EXTENSION_MISSING_MESSAGE: &str =
    "Azure DevOps feed requires `azure-devops` extension. Run `az extension add --name azure-devops`.";
pub(crate) const AZ_UNAUTHENTICATED_MESSAGE: &str =
    "Azure DevOps feed requires `az` authentication. Run `az login` and retry.";

/// Validates and canonicalizes a hosted Azure DevOps organization root.
pub(crate) fn validate_hosted_ado_organization(raw: &str) -> Result<String> {
    let url = reqwest::Url::parse(raw.trim()).context(HOSTED_ADO_ROOT_MESSAGE)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.port_or_known_default() != Some(443)
    {
        bail!(HOSTED_ADO_ROOT_MESSAGE);
    }

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let mut segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| anyhow::anyhow!(HOSTED_ADO_ROOT_MESSAGE))?
        .collect();
    if segments.last() == Some(&"") {
        segments.pop();
    }

    if host == "dev.azure.com" {
        if segments.len() != 1 || segments[0].is_empty() {
            bail!(HOSTED_ADO_ROOT_MESSAGE);
        }
        return Ok(format!("https://dev.azure.com/{}", segments[0]));
    }

    let Some(organization) = host.strip_suffix(".visualstudio.com") else {
        bail!(HOSTED_ADO_ROOT_MESSAGE);
    };
    if organization.is_empty() || organization.contains('.') || !segments.is_empty() {
        bail!(HOSTED_ADO_ROOT_MESSAGE);
    }

    Ok(format!("https://{host}"))
}

/// Checks the shared Azure DevOps CLI dependencies in the specified contract order.
pub(crate) async fn ensure_az_ready(runner: &dyn ProcessRunner) -> Result<()> {
    let version_invocation = CommandInvocation::new("az", ["--version"], AZ_PREFLIGHT_TIMEOUT);
    let version_display = version_invocation.display();
    let version_check =
        classify_dependency_result(&version_display, runner.run(version_invocation).await);

    match version_check {
        DependencyCheck::MissingBinary => bail!(AZ_MISSING_MESSAGE),
        DependencyCheck::InvocationError(error) => bail!("{error}"),
        DependencyCheck::Healthy(_) => {}
    }

    let extension_invocation = CommandInvocation::new(
        "az",
        [
            "extension",
            "show",
            "--name",
            "azure-devops",
            "--output",
            "none",
        ],
        AZ_PREFLIGHT_TIMEOUT,
    );
    let extension_display = extension_invocation.display();
    let extension_check =
        classify_dependency_result(&extension_display, runner.run(extension_invocation).await);

    match extension_check {
        DependencyCheck::MissingBinary => bail!(AZ_MISSING_MESSAGE),
        DependencyCheck::Healthy(_) => {}
        DependencyCheck::InvocationError(error) => {
            if looks_like_missing_extension(&error.stdout, &error.stderr) {
                bail!(AZ_EXTENSION_MISSING_MESSAGE);
            }
            bail!("{error}");
        }
    }

    let auth_invocation = CommandInvocation::new(
        "az",
        ["account", "show", "--output", "json"],
        AZ_PREFLIGHT_TIMEOUT,
    );
    let auth_display = auth_invocation.display();
    let auth_check = classify_dependency_result(&auth_display, runner.run(auth_invocation).await);

    match auth_check {
        DependencyCheck::MissingBinary => bail!(AZ_MISSING_MESSAGE),
        DependencyCheck::Healthy(_) => Ok(()),
        DependencyCheck::InvocationError(error) => {
            if looks_like_az_auth_error(&error.stdout, &error.stderr) {
                bail!(AZ_UNAUTHENTICATED_MESSAGE);
            }
            bail!("{error}");
        }
    }
}

pub(crate) fn looks_like_missing_extension(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    combined.contains("azure-devops") && combined.contains("extension") && combined.contains("not")
}

pub(crate) fn looks_like_az_auth_error(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    combined.contains("az login")
        || combined.contains("please run 'az login'")
        || combined.contains("not logged in")
        || combined.contains("aadsts")
        || combined.contains("tf400813")
        || combined.contains("unauthorized")
}

pub(crate) fn non_zero_exit_context(exit_code: Option<i32>, stdout: &str, stderr: &str) -> String {
    let status = match exit_code {
        Some(code) => format!("exit code {code}"),
        None => "unknown exit status".to_string(),
    };

    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return format!("{status}: {stderr}");
    }

    let stdout = stdout.trim();
    if !stdout.is_empty() {
        return format!("{status}: {stdout}");
    }

    status
}

#[cfg(test)]
mod tests {
    use super::validate_hosted_ado_organization;

    #[test]
    fn hosted_organization_accepts_supported_roots() {
        assert_eq!(
            validate_hosted_ado_organization("https://dev.azure.com/acme/").unwrap(),
            "https://dev.azure.com/acme"
        );
        assert_eq!(
            validate_hosted_ado_organization("https://dev.azure.com:443/acme").unwrap(),
            "https://dev.azure.com/acme"
        );
        assert_eq!(
            validate_hosted_ado_organization("https://acme.visualstudio.com/").unwrap(),
            "https://acme.visualstudio.com"
        );
    }

    #[test]
    fn hosted_organization_rejects_untrusted_or_non_root_urls() {
        for value in [
            "https://dev.azure.com.attacker.example/acme",
            "https://attacker.example/dev.azure.com/acme",
            "https://dev.azure.com/acme/project",
            "https://dev.azure.com/",
            "https://team.extra.visualstudio.com",
            "https://visualstudio.com",
            "https://dev.azure.com:444/acme",
            "https://user:secret@dev.azure.com/acme",
            "https://dev.azure.com/acme?x=1",
            "https://dev.azure.com/acme#fragment",
            "https://dev.azure.com/acme?",
            "https://dev.azure.com/acme#",
            "http://dev.azure.com/acme",
        ] {
            assert!(
                validate_hosted_ado_organization(value).is_err(),
                "{value} should be rejected"
            );
        }
    }
}
