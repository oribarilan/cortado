use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    github_common::{
        ensure_gh_available, looks_like_gh_auth_error, non_zero_exit_context, GH_COMMAND_TIMEOUT,
    },
    process::{CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const MAX_ACTIVITIES_PER_FEED: usize = 20;

/// Feed that polls GitHub Actions workflow runs via `gh run list`.
pub struct GithubActionsFeed {
    name: String,
    repo: String,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
    branch: Option<String>,
    workflow: Option<String>,
    event: Option<String>,
    actor: Option<String>,
}

impl GithubActionsFeed {
    /// Builds a GitHub Actions feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds a GitHub Actions feed with an injected process runner (used by tests).
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let repo = config
            .type_specific
            .get("repo")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type github-actions) is missing required `repo` string",
                    config.name
                )
            })?
            .trim()
            .to_string();

        if repo.is_empty() {
            bail!(
                "feed `{}` (type github-actions) requires non-empty `repo`",
                config.name
            );
        }

        let branch = optional_string(config, "branch");
        let workflow = optional_string(config, "workflow");
        let event = optional_string(config, "event");
        let actor = optional_string(config, "user");

        Ok(Self {
            name: config.name.clone(),
            repo,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            process_runner,
            branch,
            workflow,
            event,
            actor,
        })
    }
}

/// Reads an optional, non-blank string from `config.type_specific`.
fn optional_string(config: &FeedConfig, key: &str) -> Option<String> {
    config
        .type_specific
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

#[async_trait::async_trait]
impl Feed for GithubActionsFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "github-actions"
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn retain_for(&self) -> Option<Duration> {
        self.retain_for
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        apply_definition_overrides(
            base_field_definitions(),
            &HashMap::new(),
            &self.config_overrides,
        )
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        ensure_gh_available(self.process_runner.as_ref()).await?;

        let mut args = vec![
            "run".to_string(),
            "list".to_string(),
            "--repo".to_string(),
            self.repo.clone(),
            "--limit".to_string(),
            "20".to_string(),
            "--json".to_string(),
            "name,status,conclusion,headBranch,event,url,updatedAt,workflowName,databaseId,number"
                .to_string(),
        ];

        if let Some(branch) = &self.branch {
            args.push("--branch".to_string());
            args.push(branch.clone());
        }
        if let Some(workflow) = &self.workflow {
            args.push("--workflow".to_string());
            args.push(workflow.clone());
        }
        if let Some(event) = &self.event {
            args.push("--event".to_string());
            args.push(event.clone());
        }
        if let Some(actor) = &self.actor {
            args.push("--actor".to_string());
            args.push(actor.clone());
        }

        let invocation =
            CommandInvocation::new("gh", args.iter().map(String::as_str), GH_COMMAND_TIMEOUT);

        let command_display = invocation.display();
        let output = self
            .process_runner
            .run(invocation)
            .await
            .map_err(|error| anyhow!("failed invoking `{command_display}`: {error}"))?;

        if !output.succeeded() {
            if looks_like_gh_auth_error(&output.stdout, &output.stderr) {
                bail!("{}", crate::feed::github_common::GH_UNAUTHENTICATED_MESSAGE);
            }

            bail!(
                "`{command_display}` failed with {}",
                non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
            );
        }

        let runs = serde_json::from_str::<Vec<GhWorkflowRun>>(&output.stdout)
            .map_err(|error| anyhow!("failed parsing `gh run list` JSON output: {error}"))?;

        // Deduplicate by workflow name: `gh run list` returns runs newest-first,
        // so the first occurrence of each name is the latest run.
        let mut seen = HashSet::new();
        let activities = runs
            .into_iter()
            .filter(|run| seen.insert(run.name.clone()))
            .map(|run| map_run_to_activity(run, &self.repo, &self.config_overrides))
            .take(MAX_ACTIVITIES_PER_FEED)
            .collect();

        Ok(activities)
    }
}

fn base_field_definitions() -> Vec<FieldDefinition> {
    vec![
        FieldDefinition {
            name: "status".to_string(),
            label: "Status".to_string(),
            field_type: FieldType::Status,
            description: "Run status".to_string(),
        },
        FieldDefinition {
            name: "branch".to_string(),
            label: "Branch".to_string(),
            field_type: FieldType::Text,
            description: "Head branch".to_string(),
        },
        FieldDefinition {
            name: "workflow".to_string(),
            label: "Workflow".to_string(),
            field_type: FieldType::Text,
            description: "Workflow name".to_string(),
        },
        FieldDefinition {
            name: "event".to_string(),
            label: "Event".to_string(),
            field_type: FieldType::Text,
            description: "Trigger event".to_string(),
        },
    ]
}

fn map_run_to_activity(
    run: GhWorkflowRun,
    repo: &str,
    config_overrides: &HashMap<String, FieldOverride>,
) -> Activity {
    let status_value = map_run_status(&run.status, run.conclusion.as_deref());

    let branch_value = FieldValue::Text {
        value: run.head_branch.clone().unwrap_or_default(),
    };

    let workflow_value = FieldValue::Text {
        value: run.workflow_name.clone().unwrap_or_default(),
    };

    let event_value = FieldValue::Text {
        value: run.event.clone().unwrap_or_default(),
    };

    let fields = apply_activity_overrides(
        vec![
            Field {
                name: "status".to_string(),
                label: "Status".to_string(),
                value: status_value,
            },
            Field {
                name: "branch".to_string(),
                label: "Branch".to_string(),
                value: branch_value,
            },
            Field {
                name: "workflow".to_string(),
                label: "Workflow".to_string(),
                value: workflow_value,
            },
            Field {
                name: "event".to_string(),
                label: "Event".to_string(),
                value: event_value,
            },
        ],
        &HashMap::new(),
        config_overrides,
    );

    let display_name = run.workflow_name.as_deref().unwrap_or(run.name.as_str());

    let title = match run.number {
        Some(n) => format!("{display_name} #{n}"),
        None => display_name.to_string(),
    };

    let id = run.url.clone().unwrap_or_else(|| match run.database_id {
        Some(db_id) => format!("{repo}/actions/runs/{db_id}"),
        None => format!("{repo}/actions/runs/unknown"),
    });

    Activity {
        id,
        title,
        fields,
        retained: false,
        retained_at_unix_ms: None,
        sort_ts: None,
        action: None,
    }
}

fn map_run_status(status: &str, conclusion: Option<&str>) -> FieldValue {
    let status_lower = status.to_ascii_lowercase();
    let conclusion_lower = conclusion.unwrap_or_default().to_ascii_lowercase();

    match (status_lower.as_str(), conclusion_lower.as_str()) {
        (_, "failure" | "timed_out" | "startup_failure") => {
            status_field("failing", StatusKind::AttentionNegative)
        }
        (_, "cancelled") => status_field("cancelled", StatusKind::Idle),
        ("in_progress", _) => status_field("running", StatusKind::Running),
        ("queued" | "waiting" | "requested" | "pending", _) => {
            status_field("queued", StatusKind::Waiting)
        }
        (_, "success") => status_field("passing", StatusKind::Idle),
        (_, "skipped" | "neutral") => status_field("skipped", StatusKind::Idle),
        _ => status_field("unknown", StatusKind::Idle),
    }
}

fn status_field(value: &str, kind: StatusKind) -> FieldValue {
    FieldValue::Status {
        value: value.to_string(),
        kind,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhWorkflowRun {
    name: String,
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    head_branch: Option<String>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    database_id: Option<u64>,
    #[serde(default)]
    number: Option<u64>,
    #[serde(default)]
    workflow_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use tokio::sync::Mutex;
    use toml::{Table, Value};

    use crate::app_settings::FeedNotifyOverride;
    use crate::feed::{
        config::{FeedConfig, FieldOverride},
        github_common::{GH_MISSING_MESSAGE, GH_UNAUTHENTICATED_MESSAGE},
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldValue, StatusKind,
    };

    use super::{map_run_status, GithubActionsFeed};

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

        async fn commands(&self) -> Vec<String> {
            self.invocations
                .lock()
                .await
                .iter()
                .map(CommandInvocation::display)
                .collect()
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

    // --- Config validation ---

    #[test]
    fn from_config_requires_repo() {
        let mut config = base_config();
        config.type_specific.remove("repo");

        let error = match GithubActionsFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("repo should be required"),
            Err(e) => e,
        };

        assert!(error.to_string().contains("missing required `repo`"));
        assert!(error.to_string().contains("github-actions"));
    }

    #[test]
    fn from_config_rejects_empty_repo() {
        let mut config = base_config();
        config
            .type_specific
            .insert("repo".to_string(), Value::String("   ".to_string()));

        let error = match GithubActionsFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("empty repo should fail"),
            Err(e) => e,
        };

        assert!(error.to_string().contains("non-empty `repo`"));
    }

    #[test]
    fn from_config_parses_optional_filters() {
        let mut config = base_config();
        config
            .type_specific
            .insert("branch".to_string(), Value::String("main".to_string()));
        config
            .type_specific
            .insert("workflow".to_string(), Value::String("CI".to_string()));
        config
            .type_specific
            .insert("event".to_string(), Value::String("push".to_string()));
        config
            .type_specific
            .insert("user".to_string(), Value::String("octocat".to_string()));

        let feed = GithubActionsFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        )
        .expect("should build with filters");

        assert_eq!(feed.branch.as_deref(), Some("main"));
        assert_eq!(feed.workflow.as_deref(), Some("CI"));
        assert_eq!(feed.event.as_deref(), Some("push"));
        assert_eq!(feed.actor.as_deref(), Some("octocat"));
    }

    #[test]
    fn from_config_blank_filters_are_none() {
        let mut config = base_config();
        config
            .type_specific
            .insert("branch".to_string(), Value::String("   ".to_string()));
        config
            .type_specific
            .insert("workflow".to_string(), Value::String("".to_string()));

        let feed = GithubActionsFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        )
        .expect("should build");

        assert!(feed.branch.is_none());
        assert!(feed.workflow.is_none());
    }

    #[test]
    fn from_config_default_interval() {
        let mut config = base_config();
        config.interval = None;

        let feed = GithubActionsFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        )
        .expect("should build");

        assert_eq!(feed.interval, Duration::from_secs(120));
    }

    // --- Preflight ---

    #[tokio::test]
    async fn poll_missing_gh_binary_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::NotFound {
            program: "gh".to_string(),
        })]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert_eq!(error.to_string(), GH_MISSING_MESSAGE);
    }

    #[tokio::test]
    async fn poll_unauthenticated_returns_exact_error() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "Run `gh auth login`".to_string(),
            }),
        ]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert_eq!(error.to_string(), GH_UNAUTHENTICATED_MESSAGE);
    }

    // --- CLI invocation ---

    #[tokio::test]
    async fn poll_invokes_run_list_after_preflight() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: gh_run_list_fixture(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubActionsFeed::from_config_with_runner(&base_config(), runner.clone())
            .expect("builds");

        let activities = feed.poll().await.expect("poll should succeed");
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].title, "CI #482");
        assert_eq!(
            activities[0].id,
            "https://github.com/personal/cortado/actions/runs/12345"
        );

        let commands = runner.commands().await;
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "gh --version");
        assert_eq!(commands[1], "gh auth status");
        assert_eq!(
            commands[2],
            "gh run list --repo personal/cortado --limit 20 --json \"name,status,conclusion,headBranch,event,url,updatedAt,workflowName,databaseId,number\""
        );
    }

    #[tokio::test]
    async fn poll_includes_optional_filter_flags() {
        let mut config = base_config();
        config
            .type_specific
            .insert("branch".to_string(), Value::String("main".to_string()));
        config
            .type_specific
            .insert("workflow".to_string(), Value::String("CI".to_string()));
        config
            .type_specific
            .insert("event".to_string(), Value::String("push".to_string()));
        config
            .type_specific
            .insert("user".to_string(), Value::String("octocat".to_string()));

        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&config, runner.clone()).expect("builds");

        feed.poll().await.expect("poll should succeed");

        let commands = runner.commands().await;
        let run_cmd = &commands[2];
        assert!(run_cmd.contains("--branch main"));
        assert!(run_cmd.contains("--workflow CI"));
        assert!(run_cmd.contains("--event push"));
        assert!(run_cmd.contains("--actor octocat"));
    }

    #[tokio::test]
    async fn poll_non_auth_api_failure_includes_context() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(1),
                stdout: String::new(),
                stderr: "GraphQL error: resource not accessible".to_string(),
            }),
        ]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");

        assert!(error
            .to_string()
            .contains("GraphQL error: resource not accessible"));
    }

    #[tokio::test]
    async fn empty_run_set_returns_empty_success() {
        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "[]".to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let activities = feed.poll().await.expect("should succeed");
        assert!(activities.is_empty());
    }

    #[tokio::test]
    async fn poll_deduplicates_by_workflow_name() {
        let fixture = r#"[
            {
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "headBranch": "main",
                "event": "push",
                "url": "https://github.com/personal/cortado/actions/runs/100",
                "workflowName": "CI",
                "databaseId": 100,
                "number": 14
            },
            {
                "name": "CI",
                "status": "completed",
                "conclusion": "failure",
                "headBranch": "main",
                "event": "push",
                "url": "https://github.com/personal/cortado/actions/runs/99",
                "workflowName": "CI",
                "databaseId": 99,
                "number": 13
            },
            {
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "headBranch": "main",
                "event": "push",
                "url": "https://github.com/personal/cortado/actions/runs/98",
                "workflowName": "CI",
                "databaseId": 98,
                "number": 12
            },
            {
                "name": "Release",
                "status": "completed",
                "conclusion": "success",
                "headBranch": "main",
                "event": "release",
                "url": "https://github.com/personal/cortado/actions/runs/97",
                "workflowName": "Release",
                "databaseId": 97,
                "number": 5
            }
        ]"#;

        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: fixture.to_string(),
                stderr: String::new(),
            }),
        ]));

        let feed =
            GithubActionsFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let activities = feed.poll().await.expect("poll should succeed");

        // 3 CI runs + 1 Release run should collapse to 2 activities
        assert_eq!(activities.len(), 2);
        // Latest CI run is kept (newest-first means #14 wins)
        assert_eq!(activities[0].title, "CI #14");
        assert_eq!(
            activities[0].id,
            "https://github.com/personal/cortado/actions/runs/100"
        );
        // Release is kept
        assert_eq!(activities[1].title, "Release #5");
    }

    // --- Status mapping ---

    #[test]
    fn status_mapping_failure() {
        assert_status(
            map_run_status("completed", Some("failure")),
            "failing",
            StatusKind::AttentionNegative,
        );
        assert_status(
            map_run_status("completed", Some("timed_out")),
            "failing",
            StatusKind::AttentionNegative,
        );
        assert_status(
            map_run_status("completed", Some("startup_failure")),
            "failing",
            StatusKind::AttentionNegative,
        );
    }

    #[test]
    fn status_mapping_cancelled() {
        assert_status(
            map_run_status("completed", Some("cancelled")),
            "cancelled",
            StatusKind::Idle,
        );
    }

    #[test]
    fn status_mapping_running() {
        assert_status(
            map_run_status("in_progress", None),
            "running",
            StatusKind::Running,
        );
        assert_status(
            map_run_status("in_progress", Some("")),
            "running",
            StatusKind::Running,
        );
    }

    #[test]
    fn status_mapping_queued() {
        assert_status(
            map_run_status("queued", None),
            "queued",
            StatusKind::Waiting,
        );
        assert_status(
            map_run_status("waiting", None),
            "queued",
            StatusKind::Waiting,
        );
        assert_status(
            map_run_status("requested", None),
            "queued",
            StatusKind::Waiting,
        );
        assert_status(
            map_run_status("pending", None),
            "queued",
            StatusKind::Waiting,
        );
    }

    #[test]
    fn status_mapping_success() {
        assert_status(
            map_run_status("completed", Some("success")),
            "passing",
            StatusKind::Idle,
        );
    }

    #[test]
    fn status_mapping_skipped() {
        assert_status(
            map_run_status("completed", Some("skipped")),
            "skipped",
            StatusKind::Idle,
        );
        assert_status(
            map_run_status("completed", Some("neutral")),
            "skipped",
            StatusKind::Idle,
        );
    }

    #[test]
    fn status_mapping_unknown() {
        assert_status(
            map_run_status("completed", Some("something_new")),
            "unknown",
            StatusKind::Idle,
        );
    }

    #[test]
    fn status_mapping_case_insensitive() {
        assert_status(
            map_run_status("COMPLETED", Some("FAILURE")),
            "failing",
            StatusKind::AttentionNegative,
        );
        assert_status(
            map_run_status("IN_PROGRESS", None),
            "running",
            StatusKind::Running,
        );
        assert_status(
            map_run_status("QUEUED", None),
            "queued",
            StatusKind::Waiting,
        );
    }

    // --- Field overrides ---

    #[tokio::test]
    async fn provided_fields_and_activity_fields_apply_overrides() {
        let mut config = base_config();
        config.field_overrides.insert(
            "event".to_string(),
            FieldOverride {
                visible: Some(false),
                label: Some("Trigger".to_string()),
            },
        );

        let runner = Arc::new(StubRunner::new(vec![
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "gh version 2.60.0".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: "github.com\n  Logged in".to_string(),
                stderr: String::new(),
            }),
            Ok(CommandOutput {
                exit_code: Some(0),
                stdout: gh_run_list_fixture(),
                stderr: String::new(),
            }),
        ]));

        let feed = GithubActionsFeed::from_config_with_runner(&config, runner).expect("builds");

        let fields = feed.provided_fields();
        let event_meta = fields
            .iter()
            .find(|d| d.name == "event")
            .expect("event metadata exists");
        assert_eq!(event_meta.label, "Trigger");

        let activities = feed.poll().await.expect("polls");
        for activity in activities {
            assert!(activity.fields.iter().all(|f| f.name != "event"));
        }
    }

    // --- Activity identity and title ---

    #[test]
    fn activity_title_with_number() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "workflowName": "CI",
                "number": 482
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.title, "CI #482");
    }

    #[test]
    fn activity_title_without_number() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "workflowName": "CI"
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.title, "CI");
    }

    #[test]
    fn activity_title_falls_back_to_name_when_workflow_name_missing() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "Fallback Name",
                "status": "completed",
                "conclusion": "success",
                "number": 10
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.title, "Fallback Name #10");
    }

    #[test]
    fn activity_id_uses_url() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "CI",
                "status": "completed",
                "url": "https://github.com/org/repo/actions/runs/99",
                "databaseId": 99
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.id, "https://github.com/org/repo/actions/runs/99");
    }

    #[test]
    fn activity_id_falls_back_to_database_id() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "CI",
                "status": "completed",
                "databaseId": 12345
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.id, "personal/cortado/actions/runs/12345");
    }

    #[test]
    fn activity_id_falls_back_to_unknown() {
        let run: super::GhWorkflowRun = serde_json::from_str(
            r#"{
                "name": "CI",
                "status": "completed"
            }"#,
        )
        .expect("json");

        let activity = super::map_run_to_activity(run, "personal/cortado", &HashMap::new());
        assert_eq!(activity.id, "personal/cortado/actions/runs/unknown");
    }

    // --- Helpers ---

    fn assert_status(value: FieldValue, expected_value: &str, expected_kind: StatusKind) {
        let FieldValue::Status { value, kind } = value else {
            panic!("expected status field");
        };

        assert_eq!(value, expected_value);
        assert_eq!(kind, expected_kind);
    }

    fn base_config() -> FeedConfig {
        let mut type_specific = Table::new();
        type_specific.insert(
            "repo".to_string(),
            Value::String("personal/cortado".to_string()),
        );

        FeedConfig {
            name: "CI Runs".to_string(),
            feed_type: "github-actions".to_string(),
            interval: Some(Duration::from_secs(60)),
            retain: None,
            notify: FeedNotifyOverride::Global,
            type_specific,
            field_overrides: HashMap::new(),
        }
    }

    fn gh_run_list_fixture() -> String {
        r#"[
            {
                "name": "CI",
                "status": "completed",
                "conclusion": "success",
                "headBranch": "main",
                "event": "push",
                "url": "https://github.com/personal/cortado/actions/runs/12345",
                "workflowName": "CI",
                "databaseId": 12345,
                "number": 482
            },
            {
                "name": "Deploy",
                "status": "in_progress",
                "conclusion": null,
                "headBranch": "release/v2",
                "event": "workflow_dispatch",
                "url": "https://github.com/personal/cortado/actions/runs/12346",
                "workflowName": "Deploy",
                "databaseId": 12346,
                "number": 15
            },
            {
                "name": "CI",
                "status": "completed",
                "conclusion": "failure",
                "headBranch": "main",
                "event": "push",
                "url": "https://github.com/personal/cortado/actions/runs/12340",
                "workflowName": "CI",
                "databaseId": 12340,
                "number": 481
            }
        ]"#
        .to_string()
    }
}
