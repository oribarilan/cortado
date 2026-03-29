use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use jiff::SignedDuration;
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 60;
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_EXPECTED_STATUS: u16 = 200;

/// HTTP method used for health checks.
#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Head,
}

/// Result of a single health check request.
struct HealthCheckResult {
    /// HTTP status code, if a response was received.
    status_code: Option<u16>,
    /// Elapsed time in milliseconds.
    elapsed_ms: f64,
    /// Error message, if the request failed entirely.
    error: Option<String>,
}

/// Abstraction over HTTP health checking for testability.
#[async_trait]
trait HealthChecker: Send + Sync {
    async fn check(&self, url: &str, method: Method, timeout: Duration) -> HealthCheckResult;
}

/// Real implementation using `reqwest`.
struct ReqwestHealthChecker {
    client: reqwest::Client,
}

#[async_trait]
impl HealthChecker for ReqwestHealthChecker {
    async fn check(&self, url: &str, method: Method, timeout: Duration) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let request = match method {
            Method::Get => self.client.get(url),
            Method::Head => self.client.head(url),
        };

        let result = request.timeout(timeout).send().await;
        let elapsed_ms = start.elapsed().as_millis() as f64;

        match result {
            Ok(response) => HealthCheckResult {
                status_code: Some(response.status().as_u16()),
                elapsed_ms,
                error: None,
            },
            Err(err) => HealthCheckResult {
                status_code: None,
                elapsed_ms,
                error: Some(err.to_string()),
            },
        }
    }
}

/// Feed that monitors HTTP endpoint availability.
pub struct HttpHealthFeed {
    name: String,
    url: String,
    method: Method,
    timeout: Duration,
    expected_status: u16,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    checker: Arc<dyn HealthChecker>,
}

impl std::fmt::Debug for HttpHealthFeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpHealthFeed")
            .field("name", &self.name)
            .field("url", &self.url)
            .field("method", &self.method)
            .field("timeout", &self.timeout)
            .field("expected_status", &self.expected_status)
            .field("interval", &self.interval)
            .finish_non_exhaustive()
    }
}

impl HttpHealthFeed {
    /// Builds an HTTP health feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("cortado")
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        let checker = Arc::new(ReqwestHealthChecker { client });
        Self::from_config_with_checker(config, checker)
    }

    /// Builds an HTTP health feed with an injected health checker (used by tests).
    fn from_config_with_checker(
        config: &FeedConfig,
        checker: Arc<dyn HealthChecker>,
    ) -> Result<Self> {
        let url = config
            .type_specific
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type http-health) is missing required `url` string",
                    config.name
                )
            })?
            .trim()
            .to_string();

        if url.is_empty() {
            bail!(
                "feed `{}` (type http-health) requires non-empty `url`",
                config.name
            );
        }

        // Validate URL structure (must have scheme and host).
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!(
                "feed `{}` (type http-health) has invalid URL: must start with http:// or https://",
                config.name
            );
        }

        let method = match config
            .type_specific
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("GET")
        {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            other => bail!(
                "feed `{}` (type http-health) has invalid method `{}`: must be GET or HEAD",
                config.name,
                other
            ),
        };

        let timeout = if let Some(val) = config.type_specific.get("timeout").and_then(Value::as_str)
        {
            let parsed = val.trim().parse::<SignedDuration>().map_err(|e| {
                anyhow!(
                    "feed `{}` (type http-health) has invalid timeout `{}`: {}",
                    config.name,
                    val,
                    e
                )
            })?;

            if parsed.is_zero() || parsed.is_negative() {
                bail!(
                    "feed `{}` (type http-health) has invalid timeout `{}`: must be positive",
                    config.name,
                    val
                );
            }

            parsed.unsigned_abs()
        } else {
            Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)
        };

        let expected_status = if let Some(val) = config.type_specific.get("expected_status") {
            let status = val.as_integer().ok_or_else(|| {
                anyhow!(
                    "feed `{}` (type http-health) has invalid expected_status: must be an integer",
                    config.name,
                )
            })?;

            if !(100..=599).contains(&status) {
                bail!(
                    "feed `{}` (type http-health) has invalid expected_status `{}`: must be 100-599",
                    config.name,
                    status
                );
            }

            status as u16
        } else {
            DEFAULT_EXPECTED_STATUS
        };

        Ok(Self {
            name: config.name.clone(),
            url,
            method,
            timeout,
            expected_status,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            checker,
        })
    }
}

#[async_trait]
impl Feed for HttpHealthFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "http-health"
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
        let result = self
            .checker
            .check(&self.url, self.method, self.timeout)
            .await;

        let (status_value, status_kind, status_code_value) =
            match (&result.error, result.status_code) {
                (Some(_), _) => ("down".to_string(), StatusKind::AttentionNegative, 0.0),
                (None, Some(code)) if code == self.expected_status => {
                    ("healthy".to_string(), StatusKind::Idle, f64::from(code))
                }
                (None, Some(code)) => (
                    "unhealthy".to_string(),
                    StatusKind::AttentionNegative,
                    f64::from(code),
                ),
                (None, None) => ("down".to_string(), StatusKind::AttentionNegative, 0.0),
            };

        let fields = apply_activity_overrides(
            vec![
                Field {
                    name: "status".to_string(),
                    label: "Status".to_string(),
                    value: FieldValue::Status {
                        value: status_value,
                        kind: status_kind,
                    },
                },
                Field {
                    name: "response_time".to_string(),
                    label: "Response Time".to_string(),
                    value: FieldValue::Number {
                        value: result.elapsed_ms,
                    },
                },
                Field {
                    name: "status_code".to_string(),
                    label: "Status Code".to_string(),
                    value: FieldValue::Number {
                        value: status_code_value,
                    },
                },
            ],
            &HashMap::new(),
            &self.config_overrides,
        );

        Ok(vec![Activity {
            id: self.url.clone(),
            title: url_display_title(&self.url),
            fields,
            retained: false,
            retained_at_unix_ms: None,
        }])
    }
}

fn base_field_definitions() -> Vec<FieldDefinition> {
    vec![
        FieldDefinition {
            name: "status".to_string(),
            label: "Status".to_string(),
            field_type: FieldType::Status,
            description: "Endpoint health status".to_string(),
        },
        FieldDefinition {
            name: "response_time".to_string(),
            label: "Response Time".to_string(),
            field_type: FieldType::Number,
            description: "Response time in milliseconds".to_string(),
        },
        FieldDefinition {
            name: "status_code".to_string(),
            label: "Status Code".to_string(),
            field_type: FieldType::Number,
            description: "HTTP response status code".to_string(),
        },
    ]
}

/// Strips the URL scheme and trailing slash for a compact display title.
fn url_display_title(url: &str) -> String {
    let stripped = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    stripped.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use toml::Table;

    use crate::feed::{
        config::{FeedConfig, FieldOverride},
        Feed, FieldValue, StatusKind,
    };

    use super::{url_display_title, HealthCheckResult, HealthChecker, HttpHealthFeed, Method};

    /// Test health checker that returns a pre-configured result.
    struct MockHealthChecker {
        result: Mutex<HealthCheckResult>,
    }

    impl MockHealthChecker {
        fn new(result: HealthCheckResult) -> Arc<Self> {
            Arc::new(Self {
                result: Mutex::new(result),
            })
        }
    }

    #[async_trait::async_trait]
    impl HealthChecker for MockHealthChecker {
        async fn check(
            &self,
            _url: &str,
            _method: Method,
            _timeout: Duration,
        ) -> HealthCheckResult {
            let guard = self.result.lock().expect("mock lock poisoned");
            HealthCheckResult {
                status_code: guard.status_code,
                elapsed_ms: guard.elapsed_ms,
                error: guard.error.clone(),
            }
        }
    }

    fn base_config(name: &str) -> FeedConfig {
        FeedConfig {
            name: name.to_string(),
            feed_type: "http-health".to_string(),
            interval: None,
            retain: None,
            notify: None,
            type_specific: Table::new(),
            field_overrides: HashMap::new(),
        }
    }

    fn config_with_url(name: &str, url: &str) -> FeedConfig {
        let mut config = base_config(name);
        config
            .type_specific
            .insert("url".to_string(), toml::Value::String(url.to_string()));
        config
    }

    // --- URL display title ---

    #[test]
    fn url_display_title_strips_https() {
        assert_eq!(
            url_display_title("https://api.example.com/health"),
            "api.example.com/health"
        );
    }

    #[test]
    fn url_display_title_strips_http() {
        assert_eq!(
            url_display_title("http://localhost:8080/status"),
            "localhost:8080/status"
        );
    }

    #[test]
    fn url_display_title_strips_trailing_slash() {
        assert_eq!(url_display_title("https://example.com/"), "example.com");
    }

    #[test]
    fn url_display_title_no_scheme() {
        assert_eq!(url_display_title("example.com"), "example.com");
    }

    #[test]
    fn url_display_title_bare_domain_with_trailing_slash() {
        assert_eq!(url_display_title("https://example.com/"), "example.com");
    }

    // --- Config validation ---

    #[test]
    fn missing_url_errors() {
        let config = base_config("test");
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("missing url should fail");
        assert!(err.to_string().contains("missing required `url` string"));
    }

    #[test]
    fn empty_url_errors() {
        let mut config = base_config("test");
        config
            .type_specific
            .insert("url".to_string(), toml::Value::String("  ".to_string()));
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("empty url should fail");
        assert!(err.to_string().contains("requires non-empty `url`"));
    }

    #[test]
    fn invalid_url_scheme_errors() {
        let config = config_with_url("test", "ftp://example.com");
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("ftp url should fail");
        assert!(err.to_string().contains("has invalid URL"));
    }

    #[test]
    fn invalid_method_errors() {
        let mut config = config_with_url("test", "https://example.com");
        config.type_specific.insert(
            "method".to_string(),
            toml::Value::String("POST".to_string()),
        );
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("POST method should fail");
        assert!(err.to_string().contains("has invalid method `POST`"));
        assert!(err.to_string().contains("must be GET or HEAD"));
    }

    #[test]
    fn invalid_expected_status_too_low_errors() {
        let mut config = config_with_url("test", "https://example.com");
        config
            .type_specific
            .insert("expected_status".to_string(), toml::Value::Integer(99));
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("status 99 should fail");
        assert!(err.to_string().contains("must be 100-599"));
    }

    #[test]
    fn invalid_expected_status_too_high_errors() {
        let mut config = config_with_url("test", "https://example.com");
        config
            .type_specific
            .insert("expected_status".to_string(), toml::Value::Integer(600));
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("status 600 should fail");
        assert!(err.to_string().contains("must be 100-599"));
    }

    #[test]
    fn invalid_timeout_errors() {
        let mut config = config_with_url("test", "https://example.com");
        config.type_specific.insert(
            "timeout".to_string(),
            toml::Value::String("invalid".to_string()),
        );
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("invalid timeout should fail");
        assert!(err.to_string().contains("has invalid timeout"));
    }

    #[test]
    fn zero_timeout_errors() {
        let mut config = config_with_url("test", "https://example.com");
        config
            .type_specific
            .insert("timeout".to_string(), toml::Value::String("0s".to_string()));
        let err = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect_err("zero timeout should fail");
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn valid_config_all_defaults() {
        let config = config_with_url("health", "https://example.com/health");
        let feed = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect("valid config should succeed");

        assert_eq!(feed.name(), "health");
        assert_eq!(feed.feed_type(), "http-health");
        assert_eq!(feed.interval(), Duration::from_secs(60));
        assert!(feed.retain_for().is_none());
        assert_eq!(feed.provided_fields().len(), 3);
    }

    #[test]
    fn valid_config_all_options() {
        let mut config = config_with_url("api", "https://api.example.com/status");
        config.interval = Some(Duration::from_secs(30));
        config.retain = Some(Duration::from_secs(3600));
        config.type_specific.insert(
            "method".to_string(),
            toml::Value::String("HEAD".to_string()),
        );
        config
            .type_specific
            .insert("timeout".to_string(), toml::Value::String("5s".to_string()));
        config
            .type_specific
            .insert("expected_status".to_string(), toml::Value::Integer(204));

        let feed = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(204),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect("fully specified config should succeed");

        assert_eq!(feed.name(), "api");
        assert_eq!(feed.interval(), Duration::from_secs(30));
        assert_eq!(feed.retain_for(), Some(Duration::from_secs(3600)));
    }

    // --- Poll / status mapping ---

    #[tokio::test]
    async fn poll_healthy_response() {
        let config = config_with_url("test", "https://example.com/health");
        let checker = MockHealthChecker::new(HealthCheckResult {
            status_code: Some(200),
            elapsed_ms: 42.0,
            error: None,
        });

        let feed = HttpHealthFeed::from_config_with_checker(&config, checker)
            .expect("config should parse");
        let activities = feed.poll().await.expect("poll should succeed");

        assert_eq!(activities.len(), 1);
        let activity = &activities[0];
        assert_eq!(activity.id, "https://example.com/health");
        assert_eq!(activity.title, "example.com/health");

        let status_field = activity.fields.iter().find(|f| f.name == "status").unwrap();
        match &status_field.value {
            FieldValue::Status { value, kind } => {
                assert_eq!(value, "healthy");
                assert_eq!(*kind, StatusKind::Idle);
            }
            other => panic!("expected Status field, got {other:?}"),
        }

        let time_field = activity
            .fields
            .iter()
            .find(|f| f.name == "response_time")
            .unwrap();
        match &time_field.value {
            FieldValue::Number { value } => assert_eq!(*value, 42.0),
            other => panic!("expected Number field, got {other:?}"),
        }

        let code_field = activity
            .fields
            .iter()
            .find(|f| f.name == "status_code")
            .unwrap();
        match &code_field.value {
            FieldValue::Number { value } => assert_eq!(*value, 200.0),
            other => panic!("expected Number field, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn poll_unhealthy_wrong_status() {
        let config = config_with_url("test", "https://example.com/health");
        let checker = MockHealthChecker::new(HealthCheckResult {
            status_code: Some(503),
            elapsed_ms: 10.0,
            error: None,
        });

        let feed = HttpHealthFeed::from_config_with_checker(&config, checker)
            .expect("config should parse");
        let activities = feed.poll().await.expect("poll should succeed");

        let status_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "status")
            .unwrap();
        match &status_field.value {
            FieldValue::Status { value, kind } => {
                assert_eq!(value, "unhealthy");
                assert_eq!(*kind, StatusKind::AttentionNegative);
            }
            other => panic!("expected Status field, got {other:?}"),
        }

        let code_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "status_code")
            .unwrap();
        match &code_field.value {
            FieldValue::Number { value } => assert_eq!(*value, 503.0),
            other => panic!("expected Number field, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn poll_down_on_error() {
        let config = config_with_url("test", "https://example.com/health");
        let checker = MockHealthChecker::new(HealthCheckResult {
            status_code: None,
            elapsed_ms: 5000.0,
            error: Some("connection refused".to_string()),
        });

        let feed = HttpHealthFeed::from_config_with_checker(&config, checker)
            .expect("config should parse");
        let activities = feed.poll().await.expect("poll should succeed");

        let status_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "status")
            .unwrap();
        match &status_field.value {
            FieldValue::Status { value, kind } => {
                assert_eq!(value, "down");
                assert_eq!(*kind, StatusKind::AttentionNegative);
            }
            other => panic!("expected Status field, got {other:?}"),
        }

        let code_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "status_code")
            .unwrap();
        match &code_field.value {
            FieldValue::Number { value } => assert_eq!(*value, 0.0),
            other => panic!("expected Number field, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn poll_custom_expected_status() {
        let mut config = config_with_url("test", "https://example.com/health");
        config
            .type_specific
            .insert("expected_status".to_string(), toml::Value::Integer(204));

        let checker = MockHealthChecker::new(HealthCheckResult {
            status_code: Some(204),
            elapsed_ms: 15.0,
            error: None,
        });

        let feed = HttpHealthFeed::from_config_with_checker(&config, checker)
            .expect("config should parse");
        let activities = feed.poll().await.expect("poll should succeed");

        let status_field = activities[0]
            .fields
            .iter()
            .find(|f| f.name == "status")
            .unwrap();
        match &status_field.value {
            FieldValue::Status { value, kind } => {
                assert_eq!(value, "healthy");
                assert_eq!(*kind, StatusKind::Idle);
            }
            other => panic!("expected Status field, got {other:?}"),
        }
    }

    // --- Field overrides ---

    #[test]
    fn field_overrides_apply_to_definitions() {
        let mut config = config_with_url("test", "https://example.com/health");
        config.field_overrides.insert(
            "status".to_string(),
            FieldOverride {
                visible: None,
                label: Some("Health".to_string()),
            },
        );

        let feed = HttpHealthFeed::from_config_with_checker(
            &config,
            MockHealthChecker::new(HealthCheckResult {
                status_code: Some(200),
                elapsed_ms: 0.0,
                error: None,
            }),
        )
        .expect("config should parse");

        let defs = feed.provided_fields();
        let status_def = defs.iter().find(|d| d.name == "status").unwrap();
        assert_eq!(status_def.label, "Health");
    }

    #[tokio::test]
    async fn field_overrides_hide_field_in_poll() {
        let mut config = config_with_url("test", "https://example.com/health");
        config.field_overrides.insert(
            "response_time".to_string(),
            FieldOverride {
                visible: Some(false),
                label: None,
            },
        );

        let checker = MockHealthChecker::new(HealthCheckResult {
            status_code: Some(200),
            elapsed_ms: 42.0,
            error: None,
        });

        let feed = HttpHealthFeed::from_config_with_checker(&config, checker)
            .expect("config should parse");
        let activities = feed.poll().await.expect("poll should succeed");

        assert!(activities[0]
            .fields
            .iter()
            .all(|f| f.name != "response_time"));
    }
}
