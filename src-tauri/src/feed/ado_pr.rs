use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    concurrent,
    config::{FeedConfig, FieldOverride},
    dependency::{classify_dependency_result, DependencyCheck},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    process::{CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const AZ_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(15);
const AZ_POLL_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_ACTIVITIES_PER_FEED: usize = 20;
const MAX_POLICY_CONCURRENCY: usize = 5;

const AZ_MISSING_MESSAGE: &str =
    "Azure DevOps feed requires `az` CLI. Install it from https://aka.ms/install-azure-cli and run `az login`.";
const AZ_EXTENSION_MISSING_MESSAGE: &str =
    "Azure DevOps feed requires `azure-devops` extension. Run `az extension add --name azure-devops`.";
const AZ_UNAUTHENTICATED_MESSAGE: &str =
    "Azure DevOps feed requires `az` authentication. Run `az login` and retry.";
const AZ_CREATOR_IDENTITY_MESSAGE: &str =
    "Azure DevOps feed `user` was not resolved to a unique identity. Use a more specific creator value (prefer email/UPN).";

/// Feed that polls Azure DevOps pull requests via `az repos pr list`.
pub struct AdoPrFeed {
    name: String,
    org_url: String,
    project: String,
    repo: String,
    user: String,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
}

impl AdoPrFeed {
    /// Builds an Azure DevOps PR feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds an Azure DevOps PR feed with an injected process runner (used by tests).
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let url = required_non_empty_type_specific_string(config, "url")?;
        let (org_url, project, repo) = parse_ado_repo_url(&url)
            .map_err(|err| anyhow!("feed `{}` (type ado-pr): {err}", config.name))?;
        let user = config
            .type_specific
            .get("user")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "me".to_string());

        Ok(Self {
            name: config.name.clone(),
            org_url,
            project,
            repo,
            user,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            process_runner,
        })
    }

    async fn ensure_az_ready(&self) -> Result<()> {
        let version_invocation = CommandInvocation::new("az", ["--version"], AZ_PREFLIGHT_TIMEOUT);
        let version_display = version_invocation.display();
        let version_check = classify_dependency_result(
            &version_display,
            self.process_runner.run(version_invocation).await,
        );

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
        let extension_check = classify_dependency_result(
            &extension_display,
            self.process_runner.run(extension_invocation).await,
        );

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
        let auth_check = classify_dependency_result(
            &auth_display,
            self.process_runner.run(auth_invocation).await,
        );

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
}

#[async_trait::async_trait]
impl Feed for AdoPrFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "ado-pr"
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn retain_for(&self) -> Option<Duration> {
        self.retain_for
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        apply_definition_overrides(
            vec![
                FieldDefinition {
                    name: "review".to_string(),
                    label: "Review".to_string(),
                    field_type: FieldType::Status,
                    description: "Current reviewer decision state".to_string(),
                },
                FieldDefinition {
                    name: "checks".to_string(),
                    label: "Checks".to_string(),
                    field_type: FieldType::Status,
                    description: "Policy evaluation rollup from Azure DevOps".to_string(),
                },
                FieldDefinition {
                    name: "mergeable".to_string(),
                    label: "Mergeable".to_string(),
                    field_type: FieldType::Status,
                    description: "Merge readiness from Azure DevOps mergeStatus".to_string(),
                },
                FieldDefinition {
                    name: "draft".to_string(),
                    label: "Draft".to_string(),
                    field_type: FieldType::Status,
                    description: "Draft status".to_string(),
                },
            ],
            &HashMap::new(),
            &self.config_overrides,
        )
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        self.ensure_az_ready().await?;

        let invocation = CommandInvocation::new(
            "az",
            [
                "repos",
                "pr",
                "list",
                "--organization",
                &self.org_url,
                "--project",
                &self.project,
                "--repository",
                &self.repo,
                "--creator",
                &self.user,
                "--status",
                "active",
                "--top",
                "20",
                "--output",
                "json",
                "--detect",
                "false",
            ],
            AZ_POLL_TIMEOUT,
        );

        let command_display = invocation.display();
        let output = self
            .process_runner
            .run(invocation)
            .await
            .map_err(|error| anyhow!("failed invoking `{command_display}`: {error}"))?;

        if !output.succeeded() {
            if looks_like_missing_extension(&output.stdout, &output.stderr) {
                bail!(AZ_EXTENSION_MISSING_MESSAGE);
            }

            if looks_like_az_auth_error(&output.stdout, &output.stderr) {
                bail!(AZ_UNAUTHENTICATED_MESSAGE);
            }

            if looks_like_creator_identity_error(&output.stdout, &output.stderr) {
                bail!(AZ_CREATOR_IDENTITY_MESSAGE);
            }

            bail!(
                "`{command_display}` failed with {}",
                non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
            );
        }

        let prs = serde_json::from_str::<Vec<AdoPullRequest>>(&output.stdout)
            .map_err(|error| anyhow!("failed parsing `az repos pr list` JSON output: {error}"))?;

        let prs: Vec<AdoPullRequest> = prs.into_iter().take(MAX_ACTIVITIES_PER_FEED).collect();

        let runner = self.process_runner.clone();
        let org_url = self.org_url.clone();

        let policy_results = concurrent::map_concurrent(
            prs.iter().map(|pr| pr.pull_request_id).collect(),
            MAX_POLICY_CONCURRENCY,
            move |pr_id: Option<u64>| {
                let runner = runner.clone();
                let org_url = org_url.clone();
                async move {
                    match pr_id {
                        Some(id) => fetch_policy_status(&*runner, &org_url, id).await,
                        None => Ok(Vec::new()),
                    }
                }
            },
        )
        .await;

        let activities = prs
            .into_iter()
            .zip(policy_results)
            .map(|(pr, policy_result)| {
                let checks = match policy_result {
                    Ok(policies) => map_checks_rollup(&policies),
                    Err(_) => status_field("unknown", StatusKind::Idle),
                };
                map_pr_to_activity(
                    pr,
                    &self.org_url,
                    &self.project,
                    &self.repo,
                    &self.config_overrides,
                    checks,
                )
            })
            .collect();

        Ok(activities)
    }
}

/// Parses an Azure DevOps repository URL into (org_url, project, repo).
///
/// Accepts URLs like:
///   `https://dev.azure.com/{org}/{project}/_git/{repo}`
///   `https://{host}/{collection}/{project}/_git/{repo}`
///   `https://{host}.visualstudio.com/{project}/_git/{repo}`
fn parse_ado_repo_url(url: &str) -> Result<(String, String, String)> {
    if !url.starts_with("https://") {
        bail!("`url` must be an https:// URL");
    }

    // Find `_git` segment to split project and repo.
    let Some(git_idx) = url.find("/_git/") else {
        bail!("`url` must contain `/_git/` (e.g., https://dev.azure.com/org/project/_git/repo)");
    };

    let after_git = &url[git_idx + "/_git/".len()..];
    let repo = after_git
        .split(&['?', '#'][..])
        .next()
        .unwrap_or(after_git)
        .trim_end_matches('/');
    if repo.is_empty() {
        bail!("`url` is missing the repository name after `/_git/`");
    }

    let before_git = &url[..git_idx];

    // Split off the last path segment as the project.
    let Some(slash_idx) = before_git.rfind('/') else {
        bail!("`url` is missing the project segment before `/_git/`");
    };

    let project = &before_git[slash_idx + 1..];
    if project.is_empty() {
        bail!("`url` is missing the project name before `/_git/`");
    }

    let org_url = &before_git[..slash_idx];

    // Sanity: org_url should still start with https:// and have a host.
    if org_url.len() <= "https://".len() {
        bail!("`url` is missing the organization segment");
    }

    Ok((
        org_url.trim_end_matches('/').to_string(),
        project.to_string(),
        repo.to_string(),
    ))
}

fn required_non_empty_type_specific_string(config: &FeedConfig, key: &str) -> Result<String> {
    let value = config
        .type_specific
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow!(
                "feed `{}` (type ado-pr) is missing required `{key}` string",
                config.name
            )
        })?
        .trim()
        .to_string();

    if value.is_empty() {
        bail!(
            "feed `{}` (type ado-pr) requires non-empty `{key}`",
            config.name
        );
    }

    Ok(value)
}

fn map_pr_to_activity(
    pr: AdoPullRequest,
    org_url: &str,
    project: &str,
    repo: &str,
    config_overrides: &HashMap<String, FieldOverride>,
    checks: FieldValue,
) -> Activity {
    let review = map_review(pr.reviewers.as_deref().unwrap_or_default());
    let mergeable = map_merge_status(pr.merge_status.as_deref());
    let draft = map_draft(pr.is_draft.unwrap_or(false));

    let fields = apply_activity_overrides(
        vec![
            Field {
                name: "review".to_string(),
                label: "Review".to_string(),
                value: review,
            },
            Field {
                name: "checks".to_string(),
                label: "Checks".to_string(),
                value: checks,
            },
            Field {
                name: "mergeable".to_string(),
                label: "Mergeable".to_string(),
                value: mergeable,
            },
            Field {
                name: "draft".to_string(),
                label: "Draft".to_string(),
                value: draft,
            },
        ],
        &HashMap::new(),
        config_overrides,
    );

    let id = pr
        .pull_request_id
        .map_or_else(|| "unknown".to_string(), |id| id.to_string());

    Activity {
        id: format!("{org_url}/{project}/_git/{repo}/pullrequest/{id}"),
        title: format!(
            "#{} {}",
            id,
            pr.title.unwrap_or_else(|| "Untitled".to_string())
        ),
        fields,
        retained: false,
        retained_at_unix_ms: None,
        sort_ts: None,
    }
}

fn map_review(reviewers: &[AdoReviewer]) -> FieldValue {
    let mut has_rejected = false;
    let mut has_waiting_author = false;
    let mut required_count = 0_u32;
    let mut required_approved = 0_u32;

    for reviewer in reviewers {
        let vote = reviewer.vote.unwrap_or(0);

        if vote == -10 {
            has_rejected = true;
        } else if vote == -5 {
            has_waiting_author = true;
        }

        if reviewer.is_required.unwrap_or(false) {
            required_count += 1;
            if vote >= 5 {
                required_approved += 1;
            }
        }
    }

    if has_rejected {
        return status_field("rejected", StatusKind::AttentionNegative);
    }

    if has_waiting_author {
        return status_field("changes requested", StatusKind::AttentionNegative);
    }

    if required_count > 0 && required_approved == required_count {
        return status_field("approved", StatusKind::AttentionPositive);
    }

    status_field("awaiting", StatusKind::Waiting)
}

fn map_merge_status(raw: Option<&str>) -> FieldValue {
    match raw.unwrap_or("notSet") {
        "succeeded" => status_field("yes", StatusKind::Idle),
        "conflicts" => status_field("no", StatusKind::AttentionNegative),
        "rejectedByPolicy" => status_field("blocked", StatusKind::Waiting),
        "queued" => status_field("checking", StatusKind::Running),
        "failure" => status_field("failed", StatusKind::AttentionNegative),
        "notSet" => status_field("notSet (unknown)", StatusKind::Idle),
        other => status_field(&format!("{other} (unknown)"), StatusKind::Idle),
    }
}

fn map_draft(is_draft: bool) -> FieldValue {
    if is_draft {
        status_field("yes", StatusKind::AttentionPositive)
    } else {
        status_field("no", StatusKind::Idle)
    }
}

fn status_field(value: &str, kind: StatusKind) -> FieldValue {
    FieldValue::Status {
        value: value.to_string(),
        kind,
    }
}

/// Fetches policy evaluation states for a single PR via `az repos pr policy list`.
///
/// Note: `az repos pr policy list` does not accept `--project` — the PR ID
/// is unique within an organization, so only `--organization` is needed.
async fn fetch_policy_status(
    runner: &dyn ProcessRunner,
    org_url: &str,
    pr_id: u64,
) -> Result<Vec<AdoPolicyEvaluation>> {
    let invocation = CommandInvocation::new(
        "az",
        [
            "repos",
            "pr",
            "policy",
            "list",
            "--id",
            &pr_id.to_string(),
            "--organization",
            org_url,
            "--detect",
            "false",
            "--output",
            "json",
        ],
        AZ_POLL_TIMEOUT,
    );

    let command_display = invocation.display();
    let output = runner
        .run(invocation)
        .await
        .map_err(|error| anyhow!("failed invoking `{command_display}`: {error}"))?;

    if !output.succeeded() {
        bail!(
            "`{command_display}` failed with {}",
            non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
        );
    }

    serde_json::from_str::<Vec<AdoPolicyEvaluation>>(&output.stdout)
        .map_err(|error| anyhow!("failed parsing policy list JSON: {error}"))
}

/// Returns `true` if this policy evaluation is a CI/build check (Build or Status type).
/// Policies without a recognized type ID are excluded from the rollup.
fn is_ci_check(policy: &AdoPolicyEvaluation) -> bool {
    policy
        .configuration
        .as_ref()
        .and_then(|c| c.policy_type.as_ref())
        .and_then(|t| t.id.as_deref())
        .is_some_and(|id| id == BUILD_POLICY_TYPE_ID || id == STATUS_POLICY_TYPE_ID)
}

/// Returns `true` if a Build policy's context indicates an expired evaluation.
///
/// ADO auto-requeues build evaluations when the source changes, but the build
/// may never actually run (e.g., file-pattern scoped policies where the PR
/// doesn't touch those paths). These show up as `queued` with `isExpired: true`
/// indefinitely. We treat these as failed since they block completion.
fn is_expired_build(policy: &AdoPolicyEvaluation) -> bool {
    policy
        .context
        .as_ref()
        .and_then(|ctx| ctx.get("isExpired"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Rolls up policy evaluation states into a single checks status field.
///
/// Only Build and Status (external check) policies are considered — reviewer,
/// work-item-linking, and other policy types are excluded. The `review` field
/// already covers reviewer status.
///
/// Rollup precedence:
/// 1. any `rejected` or `broken` → `failed` (error)
/// 2. any `queued`/`running` with expired build context → `failed` (error)
/// 3. else any `queued` or `running` → `running` (pending)
/// 4. `notApplicable` is ignored
/// 5. else → `succeeded` (success)
///
/// Unknown states are ignored for rollup. If every non-`notApplicable` policy
/// has an unknown state, the result is `"<state> (unknown)"` (neutral).
fn map_checks_rollup(policies: &[AdoPolicyEvaluation]) -> FieldValue {
    let mut has_running = false;
    let mut has_approved = false;
    let mut first_unknown_state: Option<&str> = None;

    for policy in policies.iter().filter(|p| is_ci_check(p)) {
        match policy.status.as_deref().unwrap_or("") {
            "rejected" | "broken" => return status_field("failed", StatusKind::AttentionNegative),
            "queued" | "running" => {
                if is_expired_build(policy) {
                    return status_field("failed", StatusKind::AttentionNegative);
                }
                has_running = true;
            }
            "approved" => has_approved = true,
            "notApplicable" | "" => {}
            other => {
                if first_unknown_state.is_none() {
                    first_unknown_state = Some(other);
                }
            }
        }
    }

    if has_running {
        return status_field("running", StatusKind::Running);
    }

    if has_approved {
        return status_field("succeeded", StatusKind::Idle);
    }

    if let Some(state) = first_unknown_state {
        return status_field(&format!("{state} (unknown)"), StatusKind::Idle);
    }

    // All notApplicable, empty statuses, or no CI policies at all.
    status_field("succeeded", StatusKind::Idle)
}

fn looks_like_missing_extension(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    combined.contains("azure-devops") && combined.contains("extension") && combined.contains("not")
}

fn looks_like_az_auth_error(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    combined.contains("az login")
        || combined.contains("please run 'az login'")
        || combined.contains("not logged in")
        || combined.contains("aadsts")
        || combined.contains("tf400813")
        || combined.contains("unauthorized")
}

fn looks_like_creator_identity_error(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    (combined.contains("multiple identities") || combined.contains("identity"))
        && (combined.contains("creator")
            || combined.contains("reviewer")
            || combined.contains("cannot resolve"))
}

fn non_zero_exit_context(exit_code: Option<i32>, stdout: &str, stderr: &str) -> String {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoPullRequest {
    pull_request_id: Option<u64>,
    title: Option<String>,
    is_draft: Option<bool>,
    merge_status: Option<String>,
    reviewers: Option<Vec<AdoReviewer>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoReviewer {
    vote: Option<i16>,
    is_required: Option<bool>,
}

/// Well-known Azure DevOps policy type GUIDs for build / CI checks.
/// Only policies matching these types are included in the `checks` rollup.
/// Reviewer policies (min-reviewers, required-reviewers) are excluded because
/// the `review` field already covers that, and their `running` status while
/// waiting for human approval would pollute the CI-focused rollup.
const BUILD_POLICY_TYPE_ID: &str = "0609b952-1397-4640-95ec-e00a01b2c241";
const STATUS_POLICY_TYPE_ID: &str = "cbdc66da-9728-4af8-aada-9a5a32e4a226";

/// A single policy evaluation entry from `az repos pr policy list`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoPolicyEvaluation {
    status: Option<String>,
    configuration: Option<AdoPolicyConfiguration>,
    /// Polymorphic context — shape varies by policy type. For Build policies
    /// this contains `isExpired` and `buildIsNotCurrent` among other fields.
    context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoPolicyConfiguration {
    #[serde(rename = "type")]
    policy_type: Option<AdoPolicyType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoPolicyType {
    id: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use toml::{Table, Value};

    use crate::feed::{
        config::FeedConfig,
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldValue, StatusKind,
    };

    use super::{
        map_checks_rollup, map_merge_status, map_review, parse_ado_repo_url,
        AdoPolicyConfiguration, AdoPolicyEvaluation, AdoPolicyType, AdoPrFeed,
        AZ_CREATOR_IDENTITY_MESSAGE, AZ_EXTENSION_MISSING_MESSAGE, AZ_MISSING_MESSAGE,
        AZ_UNAUTHENTICATED_MESSAGE, BUILD_POLICY_TYPE_ID, STATUS_POLICY_TYPE_ID,
    };

    #[derive(Clone)]
    struct StubRunner {
        responses: Arc<Mutex<Vec<std::result::Result<CommandOutput, CommandError>>>>,
        invocations: Arc<Mutex<Vec<CommandInvocation>>>,
    }

    impl StubRunner {
        fn new(responses: Vec<std::result::Result<CommandOutput, CommandError>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
                invocations: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn invocations(&self) -> Vec<CommandInvocation> {
            self.invocations.lock().await.clone()
        }
    }

    #[async_trait]
    impl ProcessRunner for StubRunner {
        async fn run(
            &self,
            invocation: CommandInvocation,
        ) -> std::result::Result<CommandOutput, CommandError> {
            self.invocations.lock().await.push(invocation);
            self.responses.lock().await.remove(0)
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn poll_invokes_az_with_expected_flags() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "azure-cli 2.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "{}".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: ado_list_json_fixture(),
                stderr: String::new(),
            }),
            // Policy responses for PR 42 and PR 43 (order may vary under concurrency).
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner.clone())
            .expect("feed should build");

        let activities = feed.poll().await.expect("poll should succeed");
        assert_eq!(activities.len(), 2);

        let invocations = runner.invocations().await;
        assert_eq!(invocations.len(), 6);
        assert_eq!(invocations[0].program, "az");
        assert_eq!(
            invocations[1].args,
            vec![
                "extension",
                "show",
                "--name",
                "azure-devops",
                "--output",
                "none"
            ]
        );
        assert!(invocations[3].args.contains(&"--organization".to_string()));
        assert!(invocations[3]
            .args
            .contains(&"https://dev.azure.com/my-org".to_string()));
        assert!(invocations[3].args.contains(&"--creator".to_string()));
        assert!(invocations[3].args.contains(&"me".to_string()));
        assert!(invocations[3].args.contains(&"--detect".to_string()));
        assert!(invocations[3].args.contains(&"false".to_string()));
        assert!(invocations[3].args.contains(&"--top".to_string()));
        assert!(invocations[3].args.contains(&"20".to_string()));

        // Verify policy list calls were made.
        let policy_invocations: Vec<_> = invocations[4..]
            .iter()
            .filter(|inv| inv.args.contains(&"policy".to_string()))
            .collect();
        assert_eq!(policy_invocations.len(), 2);
        for inv in &policy_invocations {
            assert!(inv.args.contains(&"--detect".to_string()));
            assert!(inv.args.contains(&"false".to_string()));
            assert!(inv.args.contains(&"--output".to_string()));
            assert!(inv.args.contains(&"json".to_string()));
        }
    }

    #[tokio::test]
    async fn poll_missing_az_binary_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::NotFound {
            program: "az".to_string(),
        })]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("missing az should fail");
        assert_eq!(error.to_string(), AZ_MISSING_MESSAGE);
    }

    #[tokio::test]
    async fn poll_missing_extension_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "azure-cli".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "extension azure-devops is not installed".to_string(),
            }),
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed
            .poll()
            .await
            .expect_err("missing extension should fail");
        assert_eq!(error.to_string(), AZ_EXTENSION_MISSING_MESSAGE);
    }

    #[tokio::test]
    async fn poll_unauthenticated_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "azure-cli".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "Please run 'az login' to setup account.".to_string(),
            }),
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("auth should fail");
        assert_eq!(error.to_string(), AZ_UNAUTHENTICATED_MESSAGE);
    }

    #[tokio::test]
    async fn poll_creator_identity_error_returns_exact_message() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "azure-cli".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "{}".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "multiple identities found for creator".to_string(),
            }),
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("identity should fail");
        assert_eq!(error.to_string(), AZ_CREATOR_IDENTITY_MESSAGE);
    }

    #[test]
    fn from_config_requires_url_and_defaults_user() {
        let mut config = base_config();
        config.type_specific.remove("url");
        let error = match AdoPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("url should be required"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing required `url`"));

        // Invalid URL (no _git segment)
        let mut config = base_config();
        config.type_specific.insert(
            "url".to_string(),
            Value::String("https://dev.azure.com/my-org/my-project".to_string()),
        );
        let error = match AdoPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("should require _git in URL"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("_git"));

        let mut config = base_config();
        config.type_specific.remove("user");
        let feed =
            AdoPrFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("missing user should default to me");
        assert_eq!(feed.user, "me");

        let mut config = base_config();
        config
            .type_specific
            .insert("user".to_string(), Value::String("  ".to_string()));
        let feed =
            AdoPrFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("blank user should default to me");
        assert_eq!(feed.user, "me");
    }

    #[test]
    fn mapping_merge_status_unknown_and_review_rejected_are_deterministic() {
        let FieldValue::Status { value, kind } = map_merge_status(Some("mystery")) else {
            panic!("status expected");
        };
        assert_eq!(value, "mystery (unknown)");
        assert!(matches!(kind, StatusKind::Idle));

        let reviewers = vec![super::AdoReviewer {
            vote: Some(-10),
            is_required: Some(true),
        }];

        let FieldValue::Status { value, kind } = map_review(&reviewers) else {
            panic!("status expected");
        };
        assert_eq!(value, "rejected");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn parse_ado_repo_url_extracts_components() {
        // Standard dev.azure.com URL
        let (org, proj, repo) =
            parse_ado_repo_url("https://dev.azure.com/my-org/my-project/_git/my-repo").unwrap();
        assert_eq!(org, "https://dev.azure.com/my-org");
        assert_eq!(proj, "my-project");
        assert_eq!(repo, "my-repo");

        // visualstudio.com URL
        let (org, proj, repo) =
            parse_ado_repo_url("https://microsoft.visualstudio.com/WDATP/_git/OneCyber.Content")
                .unwrap();
        assert_eq!(org, "https://microsoft.visualstudio.com");
        assert_eq!(proj, "WDATP");
        assert_eq!(repo, "OneCyber.Content");

        // Trailing slash
        let (org, proj, repo) =
            parse_ado_repo_url("https://dev.azure.com/my-org/proj/_git/repo/").unwrap();
        assert_eq!(org, "https://dev.azure.com/my-org");
        assert_eq!(proj, "proj");
        assert_eq!(repo, "repo");

        // URL with query parameters
        let (org, proj, repo) =
            parse_ado_repo_url("https://dev.azure.com/org/proj/_git/repo?version=GBmain").unwrap();
        assert_eq!(org, "https://dev.azure.com/org");
        assert_eq!(proj, "proj");
        assert_eq!(repo, "repo");

        // URL with fragment
        let (_, _, repo) =
            parse_ado_repo_url("https://dev.azure.com/org/proj/_git/repo#path=/src").unwrap();
        assert_eq!(repo, "repo");

        // Missing https
        assert!(parse_ado_repo_url("http://dev.azure.com/o/p/_git/r").is_err());

        // Missing _git
        assert!(parse_ado_repo_url("https://dev.azure.com/o/p/r").is_err());

        // Missing repo after _git
        assert!(parse_ado_repo_url("https://dev.azure.com/o/p/_git/").is_err());
    }

    fn base_config() -> FeedConfig {
        let mut type_specific = Table::new();
        type_specific.insert(
            "url".to_string(),
            Value::String("https://dev.azure.com/my-org/my-project/_git/my-repo".to_string()),
        );
        type_specific.insert("user".to_string(), Value::String("me".to_string()));

        FeedConfig {
            name: "ADO PRs".to_string(),
            feed_type: "ado-pr".to_string(),
            interval: Some(Duration::from_secs(120)),
            retain: None,
            notify: None,
            type_specific,
            field_overrides: HashMap::new(),
        }
    }

    fn ado_list_json_fixture() -> String {
        r#"[
            {
                "pullRequestId": 42,
                "title": "Add retained activities",
                "isDraft": false,
                "mergeStatus": "queued",
                "reviewers": [
                    {"vote": 10, "isRequired": true},
                    {"vote": 5, "isRequired": false}
                ],
                "labels": [{"name": "feature"}, {"name": "backend"}]
            },
            {
                "pullRequestId": 43,
                "title": "Fix flaky tests",
                "isDraft": true,
                "mergeStatus": "conflicts",
                "reviewers": [
                    {"vote": -10, "isRequired": true}
                ],
                "labels": [{"name": "bug"}]
            }
        ]"#
        .to_string()
    }

    fn policy(status: &str) -> AdoPolicyEvaluation {
        build_policy(status)
    }

    fn build_policy(status: &str) -> AdoPolicyEvaluation {
        AdoPolicyEvaluation {
            status: Some(status.to_string()),
            configuration: Some(AdoPolicyConfiguration {
                policy_type: Some(AdoPolicyType {
                    id: Some(BUILD_POLICY_TYPE_ID.to_string()),
                }),
            }),
            context: None,
        }
    }

    fn expired_build_policy(status: &str) -> AdoPolicyEvaluation {
        AdoPolicyEvaluation {
            status: Some(status.to_string()),
            configuration: Some(AdoPolicyConfiguration {
                policy_type: Some(AdoPolicyType {
                    id: Some(BUILD_POLICY_TYPE_ID.to_string()),
                }),
            }),
            context: Some(serde_json::json!({
                "isExpired": true,
                "buildIsNotCurrent": true,
            })),
        }
    }

    fn status_check_policy(status: &str) -> AdoPolicyEvaluation {
        AdoPolicyEvaluation {
            status: Some(status.to_string()),
            configuration: Some(AdoPolicyConfiguration {
                policy_type: Some(AdoPolicyType {
                    id: Some(STATUS_POLICY_TYPE_ID.to_string()),
                }),
            }),
            context: None,
        }
    }

    fn reviewer_policy(status: &str) -> AdoPolicyEvaluation {
        AdoPolicyEvaluation {
            status: Some(status.to_string()),
            configuration: Some(AdoPolicyConfiguration {
                policy_type: Some(AdoPolicyType {
                    id: Some("fa4e907d-c16b-4a4c-9dfa-4906e5d171dd".to_string()),
                }),
            }),
            context: None,
        }
    }

    fn untyped_policy(status: &str) -> AdoPolicyEvaluation {
        AdoPolicyEvaluation {
            status: Some(status.to_string()),
            configuration: None,
            context: None,
        }
    }

    // --- checks rollup tests ---

    #[test]
    fn checks_rollup_rejected_returns_failed() {
        let policies = vec![policy("approved"), policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_broken_returns_failed() {
        let policies = vec![policy("approved"), policy("broken")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_queued_returns_running() {
        let policies = vec![policy("approved"), policy("queued")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "running");
        assert!(matches!(kind, StatusKind::Running));
    }

    #[test]
    fn checks_rollup_running_returns_running() {
        let policies = vec![policy("running"), policy("approved")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "running");
        assert!(matches!(kind, StatusKind::Running));
    }

    #[test]
    fn checks_rollup_all_approved_returns_succeeded() {
        let policies = vec![policy("approved"), policy("approved")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_empty_returns_succeeded() {
        let FieldValue::Status { value, kind } = map_checks_rollup(&[]) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_all_not_applicable_returns_succeeded() {
        let policies = vec![policy("notApplicable"), policy("notApplicable")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_unknown_state_only_returns_idle() {
        let policies = vec![policy("mystery")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "mystery (unknown)");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_unknown_with_approved_returns_succeeded() {
        let policies = vec![policy("approved"), policy("mystery")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_rejected_beats_running() {
        let policies = vec![policy("running"), policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_not_applicable_plus_unknown_returns_idle() {
        let policies = vec![policy("notApplicable"), policy("newstatus")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "newstatus (unknown)");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_missing_status_field_ignored() {
        let policies = vec![AdoPolicyEvaluation {
            status: None,
            configuration: Some(AdoPolicyConfiguration {
                policy_type: Some(AdoPolicyType {
                    id: Some(BUILD_POLICY_TYPE_ID.to_string()),
                }),
            }),
            context: None,
        }];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    // --- policy type filtering tests ---

    #[test]
    fn checks_rollup_ignores_reviewer_running_policy() {
        // Reviewer policy is "running" (waiting for approval), build is "approved".
        // Without filtering this would return "running"; with filtering it returns "succeeded".
        let policies = vec![build_policy("approved"), reviewer_policy("running")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_ignores_reviewer_rejected_policy() {
        // Reviewer policy is "rejected" but it's not a CI check — should not affect rollup.
        let policies = vec![build_policy("approved"), reviewer_policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_includes_status_check_policy() {
        // Status (external check) policies should be included alongside build policies.
        let policies = vec![build_policy("approved"), status_check_policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_ignores_untyped_policy() {
        // Policies without configuration type info are excluded from the rollup.
        let policies = vec![build_policy("approved"), untyped_policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_only_reviewer_policies_returns_succeeded() {
        // If the only policies are reviewer types, no CI checks exist → succeeded.
        let policies = vec![reviewer_policy("running"), reviewer_policy("rejected")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "succeeded");
        assert!(matches!(kind, StatusKind::Idle));
    }

    #[test]
    fn checks_rollup_build_rejected_with_reviewer_running() {
        // The exact bug scenario: build rejected + reviewer running.
        // Should return "failed" from the build, not "running" from the reviewer.
        let policies = vec![
            build_policy("approved"),
            build_policy("approved"),
            build_policy("approved"),
            build_policy("rejected"),
            reviewer_policy("running"),
            reviewer_policy("approved"),
        ];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    // --- expired build tests ---

    #[test]
    fn checks_rollup_expired_queued_build_returns_failed() {
        // ADO auto-requeues builds but they may never run (e.g., file-pattern
        // scoped). These show up as queued + isExpired indefinitely.
        let policies = vec![
            build_policy("approved"),
            expired_build_policy("queued"),
            status_check_policy("approved"),
        ];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_expired_running_build_returns_failed() {
        let policies = vec![expired_build_policy("running")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "failed");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[test]
    fn checks_rollup_non_expired_queued_build_returns_running() {
        // A fresh queued build (no expired context) is genuinely pending.
        let policies = vec![build_policy("queued")];
        let FieldValue::Status { value, kind } = map_checks_rollup(&policies) else {
            panic!("expected status");
        };
        assert_eq!(value, "running");
        assert!(matches!(kind, StatusKind::Running));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn poll_policy_failure_produces_neutral_unknown() {
        let runner = Arc::new(StubRunner::new(vec![
            // Preflight (3 calls).
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "azure-cli".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "{}".to_string(),
                stderr: String::new(),
            }),
            // PR list with 1 PR.
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: r#"[{"pullRequestId": 99, "title": "Solo PR"}]"#.to_string(),
                stderr: String::new(),
            }),
            // Policy call fails.
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "service unavailable".to_string(),
            }),
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let activities = feed.poll().await.expect("poll should still succeed");
        assert_eq!(activities.len(), 1);

        let checks_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "checks")
            .expect("checks field should exist");

        let FieldValue::Status { value, kind } = &checks_field.value else {
            panic!("expected status");
        };
        assert_eq!(value, "unknown");
        assert!(matches!(kind, StatusKind::Idle));
    }
}
