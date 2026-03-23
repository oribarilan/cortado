use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    dependency::{classify_dependency_result, DependencyCheck},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    process::{CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const AZ_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(15);
const AZ_POLL_TIMEOUT: Duration = Duration::from_secs(30);

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
        let org_url = required_non_empty_type_specific_string(config, "org")?;
        let project = required_non_empty_type_specific_string(config, "project")?;
        let repo = required_non_empty_type_specific_string(config, "repo")?;
        let user = config
            .type_specific
            .get("user")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "me".to_string());

        if !org_url.starts_with("https://") {
            bail!(
                "feed `{}` (type ado-pr) requires `org` to be an https URL",
                config.name
            );
        }

        let org_url = org_url.trim_end_matches('/').to_string();

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
                FieldDefinition {
                    name: "labels".to_string(),
                    label: "Labels".to_string(),
                    field_type: FieldType::Text,
                    description: "Applied PR labels".to_string(),
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
                "100",
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

        let activities = prs
            .into_iter()
            .map(|pr| {
                map_pr_to_activity(
                    pr,
                    &self.org_url,
                    &self.project,
                    &self.repo,
                    &self.config_overrides,
                )
            })
            .collect();

        Ok(activities)
    }
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
) -> Activity {
    let review = map_review(pr.reviewers.as_deref().unwrap_or_default());
    let mergeable = map_merge_status(pr.merge_status.as_deref());
    let draft = map_draft(pr.is_draft.unwrap_or(false));
    let labels = map_labels(pr.labels.as_deref());

    let fields = apply_activity_overrides(
        vec![
            Field {
                name: "review".to_string(),
                label: "Review".to_string(),
                value: review,
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
            Field {
                name: "labels".to_string(),
                label: "Labels".to_string(),
                value: labels,
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
        return status_field("rejected", StatusKind::Warning);
    }

    if has_waiting_author {
        return status_field("changes requested", StatusKind::Warning);
    }

    if required_count > 0 && required_approved == required_count {
        return status_field("approved", StatusKind::Success);
    }

    status_field("awaiting", StatusKind::Pending)
}

fn map_merge_status(raw: Option<&str>) -> FieldValue {
    match raw.unwrap_or("notSet") {
        "succeeded" => status_field("yes", StatusKind::Success),
        "conflicts" => status_field("no", StatusKind::Error),
        "rejectedByPolicy" => status_field("blocked", StatusKind::Warning),
        "queued" => status_field("checking", StatusKind::Pending),
        "failure" => status_field("failed", StatusKind::Error),
        "notSet" => status_field("notSet (unknown)", StatusKind::Neutral),
        other => status_field(&format!("{other} (unknown)"), StatusKind::Neutral),
    }
}

fn map_draft(is_draft: bool) -> FieldValue {
    if is_draft {
        status_field("yes", StatusKind::Pending)
    } else {
        status_field("no", StatusKind::Neutral)
    }
}

fn map_labels(labels: Option<&[AdoLabel]>) -> FieldValue {
    let mut names: Vec<String> = labels
        .unwrap_or_default()
        .iter()
        .filter_map(|label| label.name.clone())
        .collect();
    names.sort();

    FieldValue::Text {
        value: names.join(", "),
    }
}

fn status_field(value: &str, severity: StatusKind) -> FieldValue {
    FieldValue::Status {
        value: value.to_string(),
        severity,
    }
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
    labels: Option<Vec<AdoLabel>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoReviewer {
    vote: Option<i16>,
    is_required: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoLabel {
    name: Option<String>,
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
        map_merge_status, map_review, AdoPrFeed, AZ_CREATOR_IDENTITY_MESSAGE,
        AZ_EXTENSION_MISSING_MESSAGE, AZ_MISSING_MESSAGE, AZ_UNAUTHENTICATED_MESSAGE,
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

    #[tokio::test]
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
        ]));

        let feed = AdoPrFeed::from_config_with_runner(&base_config(), runner.clone())
            .expect("feed should build");

        let activities = feed.poll().await.expect("poll should succeed");
        assert_eq!(activities.len(), 2);

        let invocations = runner.invocations().await;
        assert_eq!(invocations.len(), 4);
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
    fn from_config_requires_org_project_repo_and_defaults_user() {
        let mut config = base_config();
        config.type_specific.remove("org");
        let error = match AdoPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("org should be required"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing required `org`"));

        let mut config = base_config();
        config.type_specific.remove("project");
        let error = match AdoPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("project should be required"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing required `project`"));

        let mut config = base_config();
        config.type_specific.remove("repo");
        let error = match AdoPrFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("repo should be required"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing required `repo`"));

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
        let FieldValue::Status { value, severity } = map_merge_status(Some("mystery")) else {
            panic!("status expected");
        };
        assert_eq!(value, "mystery (unknown)");
        assert!(matches!(severity, StatusKind::Neutral));

        let reviewers = vec![super::AdoReviewer {
            vote: Some(-10),
            is_required: Some(true),
        }];

        let FieldValue::Status { value, severity } = map_review(&reviewers) else {
            panic!("status expected");
        };
        assert_eq!(value, "rejected");
        assert!(matches!(severity, StatusKind::Warning));
    }

    fn base_config() -> FeedConfig {
        let mut type_specific = Table::new();
        type_specific.insert(
            "org".to_string(),
            Value::String("https://dev.azure.com/my-org".to_string()),
        );
        type_specific.insert(
            "project".to_string(),
            Value::String("my-project".to_string()),
        );
        type_specific.insert("repo".to_string(), Value::String("my-repo".to_string()));
        type_specific.insert("user".to_string(), Value::String("me".to_string()));

        FeedConfig {
            name: "ADO PRs".to_string(),
            feed_type: "ado-pr".to_string(),
            interval: Some(Duration::from_secs(120)),
            retain: None,
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
}
