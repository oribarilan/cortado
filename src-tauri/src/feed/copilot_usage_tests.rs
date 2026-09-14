use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::Mutex;
use toml::{Table, Value};

use crate::{
    app_settings::FeedNotifyOverride,
    feed::{
        config::FeedConfig,
        github_common::GH_MISSING_MESSAGE,
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldValue, StatusKind,
    },
};

use super::{CopilotUsageFeed, DEFAULT_DETAILS_URL};

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

#[test]
fn config_requires_account_and_reference_amount() {
    let mut config = base_config();
    config.type_specific.remove("account");
    assert!(build_error(&config).to_string().contains("account"));

    let mut config = base_config();
    config.type_specific.remove("reference_amount_usd");
    assert!(build_error(&config)
        .to_string()
        .contains("reference_amount_usd"));
}

#[test]
fn config_validates_numeric_boundaries_and_url() {
    for value in [0.0, -1.0] {
        let mut config = base_config();
        config
            .type_specific
            .insert("reference_amount_usd".to_string(), Value::Float(value));
        assert!(build(&config).is_err());
    }

    for value in [0.0, 100.1] {
        let mut config = base_config();
        config
            .type_specific
            .insert("attention_at_percent".to_string(), Value::Float(value));
        assert!(build(&config).is_err());
    }

    let mut config = base_config();
    config.type_specific.insert(
        "details_url".to_string(),
        Value::String("http://example.com".to_string()),
    );
    assert!(build_error(&config).to_string().contains("HTTPS"));

    config.type_specific.insert(
        "details_url".to_string(),
        Value::String("https://user:secret@example.com".to_string()),
    );
    assert!(build_error(&config)
        .to_string()
        .contains("without embedded credentials"));
}

#[test]
fn config_applies_defaults_and_override() {
    let feed = build(&base_config()).expect("valid config");
    assert_eq!(feed.attention_at_percent, 80.0);
    assert_eq!(feed.details_url, DEFAULT_DETAILS_URL);
    assert_eq!(feed.interval, Duration::from_secs(120));

    let mut config = base_config();
    config.type_specific.insert(
        "details_url".to_string(),
        Value::String("https://example.com/usage".to_string()),
    );
    config
        .type_specific
        .insert("attention_at_percent".to_string(), Value::Integer(90));
    let feed = build(&config).expect("valid overrides");
    assert_eq!(feed.details_url, "https://example.com/usage");
    assert_eq!(feed.attention_at_percent, 90.0);
}

#[tokio::test]
async fn poll_selects_account_without_switching_and_hides_token() {
    let runner = StubRunner::new(vec![ok("secret-token\n"), ok(quota_fixture(160_000.0))]);
    let feed = CopilotUsageFeed::from_config_with_runner(&base_config(), Arc::new(runner.clone()))
        .expect("valid feed");

    let activities = feed.poll().await.expect("successful poll");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].title, "octocat");
    assert_eq!(activities[0].id, DEFAULT_DETAILS_URL);
    assert_status(&activities[0], "80.0% used", StatusKind::AttentionNegative);

    let invocations = runner.invocations().await;
    assert_eq!(invocations.len(), 2);
    assert_eq!(
        invocations[0].display(),
        "gh auth token --hostname github.com --user octocat"
    );
    assert!(invocations[1]
        .display()
        .contains("gh api --hostname github.com /copilot_internal/user"));
    assert!(!format!("{:?}", invocations[1]).contains("secret-token"));
    assert!(invocations
        .iter()
        .all(|invocation| !invocation.display().contains("auth switch")));
}

#[tokio::test]
async fn threshold_is_idle_below_and_attention_at_boundary() {
    let below = feed_with_responses(vec![ok("token"), ok(quota_fixture(159_999.0))]);
    let activity = below.poll().await.unwrap().remove(0);
    assert_status(&activity, "80.0% used", StatusKind::Idle);

    let boundary = feed_with_responses(vec![ok("token"), ok(quota_fixture(160_000.0))]);
    let activity = boundary.poll().await.unwrap().remove(0);
    assert_status(&activity, "80.0% used", StatusKind::AttentionNegative);

    let over = feed_with_responses(vec![ok("token"), ok(quota_fixture(250_000.0))]);
    let activity = over.poll().await.unwrap().remove(0);
    assert_status(&activity, "125.0% used", StatusKind::AttentionNegative);
}

#[tokio::test]
async fn poll_maps_fields_and_reset_fallback() {
    let feed = feed_with_responses(vec![ok("token"), ok(quota_fixture(12_345.0))]);
    let activity = feed.poll().await.unwrap().remove(0);

    assert_text(&activity, "nominal_usage", "$123.45");
    assert_text(&activity, "reference_amount", "$2000.00");
    assert_text(&activity, "reset", "2026-10-01T00:00:00.000Z");
    let credits = activity
        .fields
        .iter()
        .find(|field| field.name == "credits_used")
        .unwrap();
    assert!(matches!(credits.value, FieldValue::Number { value } if value == 12_345.0));
}

#[tokio::test]
async fn poll_falls_back_to_user_endpoint_when_login_missing() {
    let response = quota_fixture(100.0).replace("\"login\":\"octocat\",", "");
    let runner = StubRunner::new(vec![
        ok("token"),
        ok(response),
        ok(r#"{"login":"octocat"}"#),
    ]);
    let feed = CopilotUsageFeed::from_config_with_runner(&base_config(), Arc::new(runner.clone()))
        .unwrap();

    feed.poll().await.expect("fallback identity succeeds");
    let invocations = runner.invocations().await;
    assert_eq!(invocations.len(), 3);
    assert!(invocations[2].display().contains("/user"));
}

#[tokio::test]
async fn poll_rejects_login_mismatch_legacy_units_and_missing_usage() {
    let mismatch = quota_fixture(100.0).replace("octocat", "other-user");
    let error = feed_with_responses(vec![ok("token"), ok(mismatch)])
        .poll()
        .await
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("expected GitHub account `octocat`"));

    let legacy = quota_fixture(100.0).replace(
        "\"token_based_billing\":true",
        "\"token_based_billing\":false",
    );
    let error = feed_with_responses(vec![ok("token"), ok(legacy)])
        .poll()
        .await
        .unwrap_err();
    assert!(error.to_string().contains("legacy request units"));

    let missing = quota_fixture(100.0).replace("\"credits_used\":100", "");
    let error = feed_with_responses(vec![ok("token"), ok(missing)])
        .poll()
        .await
        .unwrap_err();
    assert!(error.to_string().contains("usable Copilot AI credit"));

    let error = feed_with_responses(vec![ok("token"), ok(quota_fixture(-1.0))])
        .poll()
        .await
        .unwrap_err();
    assert!(error.to_string().contains("usable Copilot AI credit"));
}

#[tokio::test]
async fn poll_reports_api_and_json_failures_without_exposing_token() {
    let api_failure = feed_with_responses(vec![
        ok("secret-token"),
        Ok(CommandOutput {
            exit_code: Some(1),
            stdout: String::new(),
            stderr: "service unavailable".to_string(),
        }),
    ]);
    let error = api_failure.poll().await.unwrap_err().to_string();
    assert!(error.contains("service unavailable"));
    assert!(!error.contains("secret-token"));

    let malformed = feed_with_responses(vec![ok("secret-token"), ok("not json")]);
    let error = malformed.poll().await.unwrap_err().to_string();
    assert!(error.contains("failed parsing Copilot usage response"));
    assert!(!error.contains("secret-token"));
}

#[tokio::test]
#[ignore = "requires a read-only authenticated GitHub account"]
async fn live_poll_uses_selected_gh_account() {
    let account = std::env::var("CORTADO_TEST_GITHUB_ACCOUNT")
        .expect("set CORTADO_TEST_GITHUB_ACCOUNT to a gh-authenticated github.com login");
    let mut config = base_config();
    config
        .type_specific
        .insert("account".to_string(), Value::String(account.clone()));

    let feed = CopilotUsageFeed::from_config(&config).expect("live config should build");
    let activity = feed
        .poll()
        .await
        .expect("live read-only poll should succeed")
        .remove(0);

    assert!(activity.title.eq_ignore_ascii_case(&account));
    assert!(activity.fields.iter().any(|field| field.name == "usage"));
}

#[tokio::test]
async fn poll_reports_missing_gh_and_account_auth() {
    let missing = feed_with_responses(vec![Err(CommandError::NotFound {
        program: "gh".to_string(),
    })]);
    assert_eq!(
        missing.poll().await.unwrap_err().to_string(),
        GH_MISSING_MESSAGE
    );

    let unauthenticated = feed_with_responses(vec![Ok(CommandOutput {
        exit_code: Some(1),
        stdout: String::new(),
        stderr: "no oauth token found".to_string(),
    })]);
    let error = unauthenticated.poll().await.unwrap_err().to_string();
    assert!(error.contains("GitHub account `octocat` authenticated in `gh`"));
    assert!(error.contains("gh auth login --hostname github.com"));
}

fn base_config() -> FeedConfig {
    let mut type_specific = Table::new();
    type_specific.insert("account".to_string(), Value::String("octocat".to_string()));
    type_specific.insert("reference_amount_usd".to_string(), Value::Integer(2_000));
    FeedConfig {
        name: "Copilot usage".to_string(),
        feed_type: "copilot-usage".to_string(),
        interval: None,
        retain: None,
        notify: FeedNotifyOverride::Global,
        type_specific,
        field_overrides: HashMap::new(),
    }
}

fn build(config: &FeedConfig) -> anyhow::Result<CopilotUsageFeed> {
    CopilotUsageFeed::from_config_with_runner(config, Arc::new(StubRunner::new(Vec::new())))
}

fn build_error(config: &FeedConfig) -> anyhow::Error {
    match build(config) {
        Ok(_) => panic!("config should fail"),
        Err(error) => error,
    }
}

fn feed_with_responses(
    responses: Vec<std::result::Result<CommandOutput, CommandError>>,
) -> CopilotUsageFeed {
    CopilotUsageFeed::from_config_with_runner(&base_config(), Arc::new(StubRunner::new(responses)))
        .unwrap()
}

fn ok(stdout: impl Into<String>) -> std::result::Result<CommandOutput, CommandError> {
    Ok(CommandOutput {
        exit_code: Some(0),
        stdout: stdout.into(),
        stderr: String::new(),
    })
}

fn quota_fixture(credits_used: f64) -> String {
    format!(
        r#"{{"login":"octocat","token_based_billing":true,"quota_reset_date":"2026-10-01","quota_reset_date_utc":"2026-10-01T00:00:00.000Z","quota_snapshots":{{"premium_interactions":{{"credits_used":{credits_used}}}}}}}"#
    )
}

fn assert_status(activity: &crate::feed::Activity, value: &str, kind: StatusKind) {
    let field = activity
        .fields
        .iter()
        .find(|field| field.name == "usage")
        .unwrap();
    assert!(
        matches!(&field.value, FieldValue::Status { value: actual, kind: actual_kind } if actual == value && *actual_kind == kind)
    );
}

fn assert_text(activity: &crate::feed::Activity, name: &str, value: &str) {
    let field = activity
        .fields
        .iter()
        .find(|field| field.name == name)
        .unwrap();
    assert!(matches!(&field.value, FieldValue::Text { value: actual } if actual == value));
}
