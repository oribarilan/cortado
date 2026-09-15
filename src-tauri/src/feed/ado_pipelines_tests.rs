use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::Mutex;
use toml::{Table, Value};

use crate::{
    app_settings::FeedNotifyOverride,
    feed::{
        ado_common::AZ_UNAUTHENTICATED_MESSAGE,
        config::FeedConfig,
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldValue, StatusKind,
    },
};

use super::{map_run_status, AdoPipelinesFeed, MAX_PIPELINES};

#[derive(Clone)]
struct StubRunner {
    responses: Arc<Mutex<Vec<std::result::Result<CommandOutput, CommandError>>>>,
    invocations: Arc<Mutex<Vec<CommandInvocation>>>,
}

impl StubRunner {
    fn new(responses: Vec<std::result::Result<CommandOutput, CommandError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().rev().collect())),
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
        self.responses
            .lock()
            .await
            .pop()
            .expect("stub response exhausted")
    }
}

fn ok(stdout: &str) -> std::result::Result<CommandOutput, CommandError> {
    Ok(CommandOutput {
        exit_code: Some(0),
        stdout: stdout.to_string(),
        stderr: String::new(),
    })
}

fn responses_with_pages(pages: &[&str]) -> Vec<std::result::Result<CommandOutput, CommandError>> {
    let mut responses = vec![ok("az version"), ok(""), ok("{}")];
    responses.extend(pages.iter().map(|page| ok(page)));
    responses
}

fn config_with(entries: impl IntoIterator<Item = (&'static str, Value)>) -> FeedConfig {
    let mut type_specific = Table::new();
    type_specific.insert(
        "organization".to_string(),
        Value::String("https://dev.azure.com/acme".to_string()),
    );
    type_specific.insert(
        "project".to_string(),
        Value::String("Platform Team".to_string()),
    );
    for (key, value) in entries {
        type_specific.insert(key.to_string(), value);
    }
    FeedConfig {
        name: "Team CI".to_string(),
        feed_type: "ado-pipelines".to_string(),
        interval: None,
        retain: None,
        notify: FeedNotifyOverride::Global,
        type_specific,
        field_overrides: HashMap::new(),
    }
}

fn ids(values: &[i64]) -> Value {
    Value::Array(values.iter().copied().map(Value::Integer).collect())
}

fn config_error(config: &FeedConfig) -> String {
    match AdoPipelinesFeed::from_config(config) {
        Ok(_) => panic!("expected config error"),
        Err(error) => error.to_string(),
    }
}

fn field_status(activity: &crate::feed::Activity) -> (&str, StatusKind) {
    let field = activity
        .fields
        .iter()
        .find(|field| field.name == "status")
        .expect("status field");
    let FieldValue::Status { value, kind } = &field.value else {
        panic!("status value expected")
    };
    (value, *kind)
}

#[test]
fn validates_selectors_and_input_security() {
    let no_selector = config_with([]);
    assert!(config_error(&no_selector).contains("pipeline_ids` or `folder"));

    let both = config_with([
        ("pipeline_ids", ids(&[1])),
        ("folder", Value::String("\\Team".to_string())),
    ]);
    assert!(config_error(&both).contains("not both"));

    for invalid in [
        ids(&[]),
        ids(&[0]),
        ids(&[1, 1]),
        ids(&[i32::MAX as i64 + 1]),
    ] {
        assert!(AdoPipelinesFeed::from_config(&config_with([("pipeline_ids", invalid)])).is_err());
    }

    let too_many: Vec<i64> = (1..=(MAX_PIPELINES as i64 + 1)).collect();
    assert!(config_error(&config_with([("pipeline_ids", ids(&too_many))])).contains("at most 20"));

    for folder in ["Team", "\\Team/Sub", "\\Team\nOther"] {
        assert!(AdoPipelinesFeed::from_config(&config_with([(
            "folder",
            Value::String(folder.to_string()),
        )]))
        .is_err());
    }
}

#[tokio::test]
async fn validates_hosted_organization_and_project_before_cli_use() {
    let runner = Arc::new(StubRunner::new(Vec::new()));
    for organization in [
        "http://dev.azure.com/acme",
        "https://user:secret@dev.azure.com/acme",
        "https://dev.azure.com.attacker.example/acme",
        "https://dev.azure.com/acme/project",
        "https://dev.azure.com:444/acme",
        "https://dev.azure.com/acme?x=1",
        "https://dev.azure.com/acme#fragment",
    ] {
        let mut config = config_with([("pipeline_ids", ids(&[1]))]);
        config.type_specific.insert(
            "organization".to_string(),
            Value::String(organization.to_string()),
        );
        assert!(AdoPipelinesFeed::from_config_with_runner(&config, runner.clone()).is_err());
    }

    let mut config = config_with([("pipeline_ids", ids(&[1]))]);
    config
        .type_specific
        .insert("project".to_string(), Value::String("\n".to_string()));
    assert!(AdoPipelinesFeed::from_config_with_runner(&config, runner.clone()).is_err());
    assert!(runner.invocations().await.is_empty());

    for organization in [
        "https://dev.azure.com:443/acme/",
        "https://acme.visualstudio.com/",
    ] {
        let mut config = config_with([("pipeline_ids", ids(&[1]))]);
        config.type_specific.insert(
            "organization".to_string(),
            Value::String(organization.to_string()),
        );
        assert!(AdoPipelinesFeed::from_config_with_runner(&config, runner.clone()).is_ok());
    }
}

#[tokio::test]
async fn polls_explicit_ids_in_one_bulk_request_and_maps_latest_runs() {
    let page = r#"{
      "count": 2,
      "value": [
        {"id": 42, "name": "API", "path": "\\Other", "repository": {"name": "api-repo"}, "latestBuild": {
          "id": 9001, "status": "inProgress", "buildNumber": "2026.7",
          "sourceBranch": "refs/heads/main", "reason": "individualCI",
          "queueTime": "2026-01-02T03:04:05Z"
        }},
        {"id": 73, "name": "Web", "path": "\\Web", "repository": {"name": "web-repo"}, "latestBuild": {
          "id": 9002, "status": "completed", "result": "failed", "buildNumber": "42"
        }}
      ]
    }"#;
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[page])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42, 73]))]),
        runner.clone(),
    )
    .unwrap();

    let activities = feed.poll().await.unwrap();
    assert_eq!(activities.len(), 2);
    assert_eq!(
        activities[0].id,
        "ado-pipeline:https://dev.azure.com/acme/Platform%20Team/_build?definitionId=42"
    );
    assert_eq!(activities[0].title, "API");
    assert_eq!(
        field_status(&activities[0]),
        ("running", StatusKind::Running)
    );
    assert_eq!(activities[0].sort_ts, Some(1_767_323_045_000));
    assert_eq!(
        field_status(&activities[1]),
        ("failing", StatusKind::AttentionNegative)
    );

    let link = activities[0]
        .fields
        .iter()
        .find(|field| field.name == "link")
        .unwrap();
    assert!(
        matches!(&link.value, FieldValue::Url { value } if value == "https://dev.azure.com/acme/Platform%20Team/_build/results?buildId=9001&view=results")
    );

    let invocations = runner.invocations().await;
    assert_eq!(invocations.len(), 4);
    let poll = &invocations[3];
    assert_eq!(poll.program, "az");
    assert_eq!(poll.timeout, Duration::from_secs(30));
    assert!(poll.args.windows(2).any(|pair| pair == ["--area", "build"]));
    assert!(poll
        .args
        .windows(2)
        .any(|pair| pair == ["--resource", "definitions"]));
    assert!(poll.args.contains(&"processType=2".to_string()));
    assert!(poll.args.contains(&"includeLatestBuilds=true".to_string()));
    assert!(poll.args.contains(&"definitionIds=42,73".to_string()));
    assert!(!poll
        .args
        .iter()
        .any(|arg| arg.contains("includeAllProperties")));
}

#[tokio::test]
async fn exact_folder_follows_pages_and_ignores_subfolders() {
    let first = r#"{"value":[{"id":1,"name":"First","path":"\\Team\\CI"}],"continuation_token":"next token"}"#;
    let second = r#"{"value":[
      {"id":2,"name":"Nested","path":"\\Team\\CI\\Nested"},
      {"id":3,"name":"Second","path":"\\Team\\CI"}
    ]}"#;
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[first, second])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("folder", Value::String("\\Team\\CI".to_string()))]),
        runner.clone(),
    )
    .unwrap();

    let activities = feed.poll().await.unwrap();
    assert_eq!(
        activities
            .iter()
            .map(|a| a.title.as_str())
            .collect::<Vec<_>>(),
        ["First", "Second"]
    );
    assert!(activities.iter().all(|a| field_status(a).0 == "not run"));

    let invocations = runner.invocations().await;
    assert!(invocations[3].args.contains(&"path=\\Team\\CI".to_string()));
    assert!(invocations[4]
        .args
        .contains(&"continuationToken=next token".to_string()));
}

#[tokio::test]
async fn exact_folder_with_no_yaml_pipelines_is_empty() {
    let page = r#"{"value":[{"id":2,"name":"Nested","path":"\\Team\\Nested"}]}"#;
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("folder", Value::String("\\Team".to_string()))]),
        Arc::new(StubRunner::new(responses_with_pages(&[page]))),
    )
    .unwrap();
    assert!(feed.poll().await.unwrap().is_empty());
}

#[tokio::test]
async fn missing_id_and_folder_overflow_are_feed_errors() {
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[
        r#"{"value":[{"id":42,"name":"Only","path":"\\"}]}"#,
    ])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42, 73]))]),
        runner,
    )
    .unwrap();
    assert!(feed.poll().await.unwrap_err().to_string().contains("73"));

    let definitions = (1..=21)
        .map(|id| format!(r#"{{"id":{id},"name":"P{id}","path":"\\Team"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let page = format!(r#"{{"value":[{definitions}]}}"#);
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[&page])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("folder", Value::String("\\Team".to_string()))]),
        runner,
    )
    .unwrap();
    assert!(feed
        .poll()
        .await
        .unwrap_err()
        .to_string()
        .contains("more than 20"));
}

#[tokio::test]
async fn rejects_malformed_and_non_progressing_pages() {
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[r#"{"count":0}"#])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("folder", Value::String("\\Team".to_string()))]),
        runner,
    )
    .unwrap();
    assert!(feed
        .poll()
        .await
        .unwrap_err()
        .to_string()
        .contains("parsing"));

    for page in [
        r#"{"value":[{"id":42,"name":"Missing"}]}"#,
        r#"{"value":[{"id":42,"name":"Null","path":null}]}"#,
        r#"{"value":[{"id":42,"name":"Empty","path":""}]}"#,
    ] {
        let feed = AdoPipelinesFeed::from_config_with_runner(
            &config_with([("folder", Value::String("\\Team".to_string()))]),
            Arc::new(StubRunner::new(responses_with_pages(&[page]))),
        )
        .unwrap();
        assert!(feed
            .poll()
            .await
            .unwrap_err()
            .to_string()
            .contains("missing a usable folder path"));
    }

    let repeated = r#"{"value":[],"continuation_token":"same"}"#;
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[repeated, repeated])));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("folder", Value::String("\\Team".to_string()))]),
        runner,
    )
    .unwrap();
    assert!(feed
        .poll()
        .await
        .unwrap_err()
        .to_string()
        .contains("non-progressing"));
}

#[tokio::test]
async fn normalizes_data_request_auth_errors() {
    let mut responses = responses_with_pages(&[]);
    responses.push(Ok(CommandOutput {
        exit_code: Some(1),
        stdout: String::new(),
        stderr: "TF400813: unauthorized".to_string(),
    }));
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42]))]),
        Arc::new(StubRunner::new(responses)),
    )
    .unwrap();
    assert_eq!(
        feed.poll().await.unwrap_err().to_string(),
        AZ_UNAUTHENTICATED_MESSAGE
    );
}

#[tokio::test]
async fn rejects_latest_build_without_an_id() {
    let page =
        r#"{"value":[{"id":42,"name":"API","path":"\\","latestBuild":{"status":"inProgress"}}]}"#;
    let feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42]))]),
        Arc::new(StubRunner::new(responses_with_pages(&[page]))),
    )
    .unwrap();
    assert!(feed
        .poll()
        .await
        .unwrap_err()
        .to_string()
        .contains("latest build is missing its ID"));
}

#[tokio::test]
async fn stable_identity_survives_new_runs_and_renames() {
    let old = r#"{"value":[{"id":42,"name":"Old name","path":"\\","latestBuild":{"id":1,"status":"completed","result":"succeeded"}}]}"#;
    let new = r#"{"value":[{"id":42,"name":"New name","path":"\\","latestBuild":{"id":2,"status":"completed","result":"succeeded"}}]}"#;

    let old_feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42]))]),
        Arc::new(StubRunner::new(responses_with_pages(&[old]))),
    )
    .unwrap();
    let new_feed = AdoPipelinesFeed::from_config_with_runner(
        &config_with([("pipeline_ids", ids(&[42]))]),
        Arc::new(StubRunner::new(responses_with_pages(&[new]))),
    )
    .unwrap();
    let old_activity = old_feed.poll().await.unwrap().remove(0);
    let new_activity = new_feed.poll().await.unwrap().remove(0);

    assert_eq!(old_activity.id, new_activity.id);
    assert_ne!(old_activity.title, new_activity.title);
    assert_eq!(field_status(&old_activity).1, field_status(&new_activity).1);
}

#[test]
fn maps_all_status_contract_branches() {
    let cases = [
        (Some("notStarted"), None, "queued", StatusKind::Waiting),
        (Some("postponed"), None, "queued", StatusKind::Waiting),
        (Some("inProgress"), None, "running", StatusKind::Running),
        (Some("cancelling"), None, "cancelling", StatusKind::Running),
        (
            Some("completed"),
            Some("succeeded"),
            "passing",
            StatusKind::Idle,
        ),
        (
            Some("completed"),
            Some("failed"),
            "failing",
            StatusKind::AttentionNegative,
        ),
        (
            Some("completed"),
            Some("partiallySucceeded"),
            "partially succeeded",
            StatusKind::AttentionNegative,
        ),
        (
            Some("completed"),
            Some("canceled"),
            "cancelled",
            StatusKind::AttentionNegative,
        ),
        (Some("completed"), None, "unknown", StatusKind::Idle),
        (
            Some("mystery"),
            Some("succeeded"),
            "unknown",
            StatusKind::Idle,
        ),
        (None, None, "unknown", StatusKind::Idle),
    ];

    for (status, result, expected_value, expected_kind) in cases {
        let FieldValue::Status { value, kind } = map_run_status(status, result) else {
            panic!("status expected")
        };
        assert_eq!(value, expected_value);
        assert_eq!(kind, expected_kind);
    }
}

#[test]
fn default_interval_and_retention_follow_feed_contract() {
    let mut config = config_with([("pipeline_ids", ids(&[42]))]);
    config.retain = Some(Duration::from_secs(600));
    let feed = AdoPipelinesFeed::from_config(&config).unwrap();
    assert_eq!(feed.interval(), Duration::from_secs(120));
    assert_eq!(feed.retain_for(), Some(Duration::from_secs(600)));
}
