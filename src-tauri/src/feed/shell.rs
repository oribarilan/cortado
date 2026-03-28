use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    process::{CommandError, CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 30;
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

/// Feed that executes a shell command and exposes one typed output field.
pub struct ShellFeed {
    name: String,
    command: String,
    field_name: String,
    field_type: FieldType,
    interval: Duration,
    retain_for: Option<Duration>,
    explicit_overrides: HashMap<String, FieldOverride>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
}

impl ShellFeed {
    /// Builds a shell feed from parsed feed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds a shell feed with an injected process runner (used by tests).
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let command = config
            .type_specific
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type shell) is missing required `command` string",
                    config.name
                )
            })?
            .trim()
            .to_string();

        if command.is_empty() {
            bail!(
                "feed `{}` (type shell) requires non-empty `command`",
                config.name
            );
        }

        let field_name = config
            .type_specific
            .get("field_name")
            .and_then(Value::as_str)
            .unwrap_or("output")
            .trim()
            .to_string();

        if field_name.is_empty() {
            bail!(
                "feed `{}` (type shell) requires non-empty `field_name` when provided",
                config.name
            );
        }

        let field_type = parse_field_type(config)?;
        let explicit_label = config
            .type_specific
            .get("label")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .map(ToString::to_string);

        let explicit_overrides = HashMap::from([(
            field_name.clone(),
            FieldOverride {
                visible: None,
                label: explicit_label,
            },
        )]);

        Ok(Self {
            name: config.name.clone(),
            command,
            field_name,
            field_type,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            explicit_overrides,
            config_overrides: config.field_overrides.clone(),
            process_runner,
        })
    }

    fn base_field_definition(&self) -> FieldDefinition {
        FieldDefinition {
            name: self.field_name.clone(),
            label: default_label_for_field(&self.field_name),
            field_type: self.field_type.clone(),
            description: format!("Output of command: {}", self.command),
        }
    }
}

#[async_trait::async_trait]
impl Feed for ShellFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "shell"
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    fn retain_for(&self) -> Option<Duration> {
        self.retain_for
    }

    fn provided_fields(&self) -> Vec<FieldDefinition> {
        apply_definition_overrides(
            vec![self.base_field_definition()],
            &self.explicit_overrides,
            &self.config_overrides,
        )
    }

    async fn poll(&self) -> Result<Vec<Activity>> {
        let invocation = CommandInvocation::shell(self.command.clone(), COMMAND_TIMEOUT);
        let command_display = invocation.display();

        let output = self
            .process_runner
            .run(invocation)
            .await
            .map_err(shell_process_error)?;

        if !output.succeeded() {
            bail!(
                "shell command `{command_display}` failed with {}",
                format_exit_context(output.exit_code, &output.stderr)
            );
        }

        let value = map_stdout_to_field_value(&output.stdout, &self.field_type)?;

        let fields = apply_activity_overrides(
            vec![Field {
                name: self.field_name.clone(),
                label: default_label_for_field(&self.field_name),
                value,
            }],
            &self.explicit_overrides,
            &self.config_overrides,
        );

        Ok(vec![Activity {
            id: format!("shell:{}", self.name),
            title: self.command.clone(),
            fields,
            retained: false,
            retained_at_unix_ms: None,
        }])
    }
}

fn parse_field_type(config: &FeedConfig) -> Result<FieldType> {
    match config
        .type_specific
        .get("field_type")
        .and_then(Value::as_str)
        .unwrap_or("text")
    {
        "text" => Ok(FieldType::Text),
        "status" => Ok(FieldType::Status),
        "number" => Ok(FieldType::Number),
        "url" => Ok(FieldType::Url),
        other => Err(anyhow!(
            "feed `{}` (type shell) has invalid `field_type` `{other}`; expected one of text|status|number|url",
            config.name
        )),
    }
}

fn default_label_for_field(field_name: &str) -> String {
    if field_name == "output" {
        "Output".to_string()
    } else {
        field_name.to_string()
    }
}

fn shell_process_error(error: CommandError) -> anyhow::Error {
    match error {
        CommandError::NotFound { .. } => {
            anyhow!("shell feed failed to start `sh`. Ensure a POSIX shell is available on PATH.")
        }
        CommandError::Spawn { message, .. } => {
            anyhow!("shell feed command spawn failed: {message}")
        }
        CommandError::Timeout {
            command, timeout, ..
        } => anyhow!(
            "shell command `{command}` timed out after {}s",
            timeout.as_secs()
        ),
    }
}

fn format_exit_context(exit_code: Option<i32>, stderr: &str) -> String {
    let status = match exit_code {
        Some(code) => format!("exit code {code}"),
        None => "unknown exit status".to_string(),
    };

    let stderr = stderr.trim();

    if stderr.is_empty() {
        status
    } else {
        format!("{status}: {stderr}")
    }
}

fn map_stdout_to_field_value(stdout: &str, field_type: &FieldType) -> Result<FieldValue> {
    let trimmed = stdout.trim();

    match field_type {
        FieldType::Text => Ok(FieldValue::Text {
            value: trimmed.to_string(),
        }),
        FieldType::Number => {
            let value = trimmed.parse::<f64>().map_err(|err| {
                anyhow!("failed parsing number from shell output `{trimmed}`: {err}")
            })?;

            Ok(FieldValue::Number { value })
        }
        FieldType::Url => Ok(FieldValue::Url {
            value: trimmed.to_string(),
        }),
        FieldType::Status => Ok(FieldValue::Status {
            value: trimmed.to_string(),
            kind: status_kind_from_output(trimmed),
        }),
    }
}

fn status_kind_from_output(raw: &str) -> StatusKind {
    let normalized = raw.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "ok" | "pass" | "passing" | "success" | "healthy" => StatusKind::AttentionPositive,
        "warn" | "warning" => StatusKind::AttentionNegative,
        "err" | "error" | "fail" | "failing" | "critical" => StatusKind::AttentionNegative,
        "pending" | "running" | "in_progress" => StatusKind::Running,
        _ => StatusKind::Idle,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use toml::{Table, Value};

    use crate::feed::{
        config::{FeedConfig, FieldOverride},
        process::{CommandError, CommandInvocation, CommandOutput, ProcessRunner},
        Feed, FieldType, FieldValue, StatusKind,
    };

    use super::{map_stdout_to_field_value, status_kind_from_output, ShellFeed, COMMAND_TIMEOUT};

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
    async fn poll_executes_sh_c_and_maps_text_output() {
        let runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(0),
            stdout: " hello\n".to_string(),
            stderr: String::new(),
        })]));

        let feed = ShellFeed::from_config_with_runner(&base_config(), runner.clone())
            .expect("feed should build");

        let activities = feed.poll().await.expect("poll should succeed");
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].id, "shell:Disk usage");

        let field = &activities[0].fields[0];
        assert_eq!(field.name, "output");

        let FieldValue::Text { value } = &field.value else {
            panic!("expected text field");
        };
        assert_eq!(value, "hello");

        let invocations = runner.invocations().await;
        assert_eq!(invocations.len(), 1);
        assert_eq!(invocations[0].program, "sh");
        assert_eq!(invocations[0].args, vec!["-c", "printf hello"]);
        assert_eq!(invocations[0].timeout, COMMAND_TIMEOUT);
    }

    #[tokio::test]
    async fn poll_maps_status_severity_case_insensitive() {
        let runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(0),
            stdout: "FAILING\n".to_string(),
            stderr: String::new(),
        })]));

        let mut config = base_config();
        config.type_specific.insert(
            "field_type".to_string(),
            Value::String("status".to_string()),
        );

        let feed = ShellFeed::from_config_with_runner(&config, runner).expect("builds");
        let activities = feed.poll().await.expect("polls");

        let FieldValue::Status { value, kind } = &activities[0].fields[0].value else {
            panic!("expected status field");
        };

        assert_eq!(value, "FAILING");
        assert!(matches!(kind, StatusKind::AttentionNegative));
    }

    #[tokio::test]
    async fn poll_maps_number_or_returns_parse_error() {
        let runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(0),
            stdout: "12.5\n".to_string(),
            stderr: String::new(),
        })]));

        let mut config = base_config();
        config.type_specific.insert(
            "field_type".to_string(),
            Value::String("number".to_string()),
        );

        let feed = ShellFeed::from_config_with_runner(&config, runner).expect("builds");
        let activities = feed.poll().await.expect("polls");

        let FieldValue::Number { value } = activities[0].fields[0].value else {
            panic!("expected number field");
        };
        assert_eq!(value, 12.5);

        let failing_runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(0),
            stdout: "nan-nope\n".to_string(),
            stderr: String::new(),
        })]));

        let failing_feed =
            ShellFeed::from_config_with_runner(&config, failing_runner).expect("builds");

        let error = failing_feed
            .poll()
            .await
            .expect_err("number parse should fail");
        assert!(error.to_string().contains("failed parsing number"));
    }

    #[tokio::test]
    async fn poll_non_zero_exit_contains_stderr() {
        let runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(7),
            stdout: String::new(),
            stderr: "boom".to_string(),
        })]));

        let feed = ShellFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");

        let message = error.to_string();
        assert!(message.contains("exit code 7"));
        assert!(message.contains("boom"));
    }

    #[tokio::test]
    async fn poll_timeout_returns_clear_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::Timeout {
            command: "sh -c sleep 30".to_string(),
            timeout: Duration::from_secs(10),
        })]));

        let feed = ShellFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should timeout");

        assert!(error.to_string().contains("timed out after 10s"));
    }

    #[tokio::test]
    async fn from_config_requires_command_and_parses_supported_field_types() {
        let mut missing_command = base_config();
        missing_command.type_specific.remove("command");

        let error = match ShellFeed::from_config_with_runner(
            &missing_command,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("should require command"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("missing required `command`"));

        for field_type in ["text", "status", "number", "url"] {
            let mut config = base_config();
            config.type_specific.insert(
                "field_type".to_string(),
                Value::String(field_type.to_string()),
            );

            let feed = ShellFeed::from_config_with_runner(
                &config,
                Arc::new(StubRunner::new(vec![Ok(CommandOutput {
                    exit_code: Some(0),
                    stdout: "x".to_string(),
                    stderr: String::new(),
                })])),
            )
            .expect("field type should parse");

            match field_type {
                "text" => assert!(matches!(
                    feed.provided_fields()[0].field_type,
                    FieldType::Text
                )),
                "status" => {
                    assert!(matches!(
                        feed.provided_fields()[0].field_type,
                        FieldType::Status
                    ))
                }
                "number" => {
                    assert!(matches!(
                        feed.provided_fields()[0].field_type,
                        FieldType::Number
                    ))
                }
                "url" => assert!(matches!(
                    feed.provided_fields()[0].field_type,
                    FieldType::Url
                )),
                _ => unreachable!(),
            }
        }

        let mut invalid = base_config();
        invalid
            .type_specific
            .insert("field_type".to_string(), Value::String("bool".to_string()));

        let error = match ShellFeed::from_config_with_runner(
            &invalid,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("unsupported type should fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("invalid `field_type` `bool`"));
    }

    #[tokio::test]
    async fn overrides_hide_fields_and_override_labels() {
        let runner = Arc::new(StubRunner::new(vec![Ok(CommandOutput {
            exit_code: Some(0),
            stdout: "ok".to_string(),
            stderr: String::new(),
        })]));

        let mut config = base_config();
        config.type_specific.insert(
            "label".to_string(),
            Value::String("Explicit Label".to_string()),
        );

        config.field_overrides.insert(
            "output".to_string(),
            FieldOverride {
                visible: Some(false),
                label: Some("Config Label".to_string()),
            },
        );

        let feed = ShellFeed::from_config_with_runner(&config, runner).expect("builds");

        let fields = feed.provided_fields();
        assert_eq!(fields[0].label, "Config Label");

        let activities = feed.poll().await.expect("polls");
        assert!(activities[0].fields.is_empty());
    }

    #[test]
    fn status_mapping_edge_cases() {
        assert!(matches!(
            status_kind_from_output(" ok "),
            StatusKind::AttentionPositive
        ));
        assert!(matches!(
            status_kind_from_output("WARNING"),
            StatusKind::AttentionNegative
        ));
        assert!(matches!(
            status_kind_from_output("critical"),
            StatusKind::AttentionNegative
        ));
        assert!(matches!(
            status_kind_from_output("in_progress"),
            StatusKind::Running
        ));
        assert!(matches!(
            status_kind_from_output("unknown"),
            StatusKind::Idle
        ));
    }

    #[test]
    fn map_stdout_to_field_value_for_url_keeps_trimmed_value() {
        let value = map_stdout_to_field_value(" https://example.com \n", &FieldType::Url)
            .expect("url should map");

        let FieldValue::Url { value } = value else {
            panic!("expected url field");
        };

        assert_eq!(value, "https://example.com");
    }

    #[test]
    fn from_config_rejects_empty_command() {
        let mut config = base_config();
        config
            .type_specific
            .insert("command".to_string(), Value::String("   ".to_string()));

        let err = match ShellFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("blank command should fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("non-empty `command`"));
    }

    #[test]
    fn from_config_rejects_empty_field_name() {
        let mut config = base_config();
        config
            .type_specific
            .insert("field_name".to_string(), Value::String("  ".to_string()));

        let err = match ShellFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(Vec::new())),
        ) {
            Ok(_) => panic!("blank field_name should fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("non-empty `field_name`"));
    }

    #[test]
    fn from_config_custom_field_name_and_label() {
        let mut config = base_config();
        config
            .type_specific
            .insert("field_name".to_string(), Value::String("disk".to_string()));
        config
            .type_specific
            .insert("label".to_string(), Value::String("Disk Usage".to_string()));

        let feed =
            ShellFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("should build");

        let defs = feed.provided_fields();
        assert_eq!(defs[0].name, "disk");
        assert_eq!(defs[0].label, "Disk Usage");
    }

    #[test]
    fn from_config_default_interval_is_30s() {
        let mut config = base_config();
        config.interval = None;

        let feed =
            ShellFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("should build");

        assert_eq!(feed.interval(), Duration::from_secs(30));
    }

    #[test]
    fn from_config_retain_for_is_passed_through() {
        let mut config = base_config();
        config.retain = Some(Duration::from_secs(3600));

        let feed =
            ShellFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(Vec::new())))
                .expect("should build");

        assert_eq!(feed.retain_for(), Some(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn poll_missing_sh_binary_returns_clear_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::NotFound {
            program: "sh".to_string(),
        })]));

        let feed = ShellFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert!(error.to_string().contains("POSIX shell"));
    }

    #[tokio::test]
    async fn poll_spawn_failure_returns_clear_error() {
        let runner = Arc::new(StubRunner::new(vec![Err(CommandError::Spawn {
            program: "sh".to_string(),
            message: "permission denied".to_string(),
        })]));

        let feed = ShellFeed::from_config_with_runner(&base_config(), runner).expect("builds");
        let error = feed.poll().await.expect_err("should fail");
        assert!(error.to_string().contains("spawn failed"));
    }

    fn base_config() -> FeedConfig {
        let mut type_specific = Table::new();
        type_specific.insert(
            "command".to_string(),
            Value::String("printf hello".to_string()),
        );

        FeedConfig {
            name: "Disk usage".to_string(),
            feed_type: "shell".to_string(),
            interval: Some(Duration::from_secs(5)),
            retain: None,
            notify: None,
            type_specific,
            field_overrides: HashMap::new(),
        }
    }
}
