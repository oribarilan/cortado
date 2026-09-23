use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use toml::Value;

use crate::feed::{
    config::{FeedConfig, FieldOverride},
    dependency::{classify_dependency_result, DependencyCheck},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    github_common::{
        looks_like_gh_auth_error, non_zero_exit_context, GH_COMMAND_TIMEOUT, GH_MISSING_MESSAGE,
    },
    process::{CommandInvocation, CommandOutput, ProcessRunner, TokioProcessRunner},
    Activity, Feed, Field, FieldDefinition, FieldType, FieldValue, StatusKind,
};

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const DEFAULT_ATTENTION_PERCENT: f64 = 80.0;
const AI_CREDIT_USD: f64 = 0.01;
const DEFAULT_DETAILS_URL: &str = "https://github.com/settings/copilot";

/// Experimental feed that tracks one GitHub account's Copilot AI credit usage.
pub struct CopilotUsageFeed {
    name: String,
    account: String,
    reference_amount_usd: f64,
    attention_at_percent: f64,
    details_url: String,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
}

impl CopilotUsageFeed {
    /// Builds a Copilot usage feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds a Copilot usage feed with an injected process runner.
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let account = required_non_blank_string(config, "account")?;
        let reference_amount_usd = required_number(config, "reference_amount_usd")?;
        if !reference_amount_usd.is_finite() || reference_amount_usd <= 0.0 {
            bail!(
                "feed `{}` (type copilot-usage): `reference_amount_usd` must be a finite number greater than zero",
                config.name
            );
        }

        let attention_at_percent =
            optional_number(config, "attention_at_percent")?.unwrap_or(DEFAULT_ATTENTION_PERCENT);
        if !attention_at_percent.is_finite() || !(1.0..=100.0).contains(&attention_at_percent) {
            bail!(
                "feed `{}` (type copilot-usage): `attention_at_percent` must be a finite number from 1 through 100",
                config.name
            );
        }

        let details_url = optional_non_blank_string(config, "details_url")?
            .unwrap_or_else(|| DEFAULT_DETAILS_URL.to_string());
        validate_details_url(config, &details_url)?;

        Ok(Self {
            name: config.name.clone(),
            account,
            reference_amount_usd,
            attention_at_percent,
            details_url,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            process_runner,
        })
    }

    async fn resolve_account_token(&self) -> Result<String> {
        let invocation = CommandInvocation::new(
            "gh",
            [
                "auth",
                "token",
                "--hostname",
                "github.com",
                "--user",
                self.account.as_str(),
            ],
            GH_COMMAND_TIMEOUT,
        )
        .without_environment("GH_TOKEN")
        .without_environment("GITHUB_TOKEN");
        let display = invocation.display();

        match classify_dependency_result(&display, self.process_runner.run(invocation).await) {
            DependencyCheck::MissingBinary => bail!(GH_MISSING_MESSAGE),
            DependencyCheck::Healthy(output) => {
                let token = output.stdout.trim();
                if token.is_empty() {
                    bail!(self.account_auth_error());
                }
                Ok(token.to_string())
            }
            DependencyCheck::InvocationError(_) => bail!(self.account_auth_error()),
        }
    }

    async fn run_account_api(&self, token: &str, path: &str) -> Result<CommandOutput> {
        let invocation = CommandInvocation::new(
            "gh",
            [
                "api",
                "--hostname",
                "github.com",
                path,
                "-H",
                "Accept: application/vnd.github+json",
            ],
            GH_COMMAND_TIMEOUT,
        )
        .with_environment("GH_TOKEN", token)
        .without_environment("GITHUB_TOKEN");
        let display = invocation.display();
        let output = self
            .process_runner
            .run(invocation)
            .await
            .map_err(|error| anyhow!("failed invoking `{display}`: {error}"))?;

        if !output.succeeded() {
            if looks_like_gh_auth_error(&output.stdout, &output.stderr) {
                bail!(self.account_auth_error());
            }
            bail!(
                "`{display}` failed with {}",
                non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
            );
        }

        Ok(output)
    }

    async fn verified_login(&self, token: &str, response_login: Option<&str>) -> Result<String> {
        let login = match response_login
            .map(str::trim)
            .filter(|login| !login.is_empty())
        {
            Some(login) => login.to_string(),
            None => {
                let output = self.run_account_api(token, "/user").await?;
                let identity: GithubIdentity = serde_json::from_str(&output.stdout)
                    .map_err(|error| anyhow!("failed parsing GitHub account response: {error}"))?;
                identity.login
            }
        };

        if !login.eq_ignore_ascii_case(&self.account) {
            bail!(
                "Copilot usage feed expected GitHub account `{}`, but GitHub returned `{login}`",
                self.account
            );
        }

        Ok(login)
    }

    fn account_auth_error(&self) -> String {
        format!(
            "Copilot usage feed requires GitHub account `{}` authenticated in `gh`. Run `gh auth login --hostname github.com` and retry.",
            self.account
        )
    }

    fn activity_from_response(
        &self,
        response: CopilotQuotaResponse,
        verified_login: String,
    ) -> Result<Activity> {
        if response.token_based_billing != Some(true) {
            bail!(
                "Copilot usage for `{}` uses legacy request units; nominal USD usage is unavailable",
                self.account
            );
        }

        let credits_used = response
            .quota_snapshots
            .and_then(|snapshots| snapshots.premium_interactions)
            .and_then(|snapshot| snapshot.credits_used)
            .filter(|credits| credits.is_finite() && *credits >= 0.0)
            .ok_or_else(|| {
                anyhow!(
                    "GitHub did not return usable Copilot AI credit consumption for `{}`",
                    self.account
                )
            })?;

        let nominal_usage_usd = credits_used * AI_CREDIT_USD;
        let utilization_percent = nominal_usage_usd / self.reference_amount_usd * 100.0;
        if !nominal_usage_usd.is_finite() || !utilization_percent.is_finite() {
            bail!(
                "GitHub returned Copilot AI credit consumption too large to calculate for `{}`",
                self.account
            );
        }
        let kind = if utilization_percent >= self.attention_at_percent {
            StatusKind::AttentionNegative
        } else {
            StatusKind::Idle
        };
        let reset = response
            .quota_reset_date_utc
            .or(response.quota_reset_date)
            .unwrap_or_else(|| "unknown".to_string());

        let fields = apply_activity_overrides(
            vec![
                Field {
                    name: "usage".to_string(),
                    label: "Usage".to_string(),
                    value: FieldValue::Status {
                        value: format!("{utilization_percent:.1}% used"),
                        kind,
                    },
                },
                Field {
                    name: "nominal_usage".to_string(),
                    label: "Nominal Usage".to_string(),
                    value: FieldValue::Text {
                        value: format!("${nominal_usage_usd:.2}"),
                    },
                },
                Field {
                    name: "reference_amount".to_string(),
                    label: "Reference Amount".to_string(),
                    value: FieldValue::Text {
                        value: format!("${:.2}", self.reference_amount_usd),
                    },
                },
                Field {
                    name: "credits_used".to_string(),
                    label: "AI Credits Used".to_string(),
                    value: FieldValue::Number {
                        value: credits_used,
                    },
                },
                Field {
                    name: "reset".to_string(),
                    label: "Resets".to_string(),
                    value: FieldValue::Text { value: reset },
                },
            ],
            &HashMap::new(),
            &self.config_overrides,
        );

        Ok(Activity {
            id: self.details_url.clone(),
            title: verified_login,
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            visible_until: None,
            action: None,
        })
    }
}

#[async_trait::async_trait]
impl Feed for CopilotUsageFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "copilot-usage"
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
        let token = self.resolve_account_token().await?;
        let output = self
            .run_account_api(&token, "/copilot_internal/user")
            .await?;
        let response: CopilotQuotaResponse = serde_json::from_str(&output.stdout)
            .map_err(|error| anyhow!("failed parsing Copilot usage response: {error}"))?;
        let verified_login = self
            .verified_login(&token, response.login.as_deref())
            .await?;

        Ok(vec![self.activity_from_response(response, verified_login)?])
    }
}

fn base_field_definitions() -> Vec<FieldDefinition> {
    vec![
        FieldDefinition {
            name: "usage".to_string(),
            label: "Usage".to_string(),
            field_type: FieldType::Status,
            description: "Nominal utilization of the configured reference amount".to_string(),
        },
        FieldDefinition {
            name: "nominal_usage".to_string(),
            label: "Nominal Usage".to_string(),
            field_type: FieldType::Text,
            description: "AI credits converted at GitHub's published per-credit rate".to_string(),
        },
        FieldDefinition {
            name: "reference_amount".to_string(),
            label: "Reference Amount".to_string(),
            field_type: FieldType::Text,
            description: "Configured USD comparison amount".to_string(),
        },
        FieldDefinition {
            name: "credits_used".to_string(),
            label: "AI Credits Used".to_string(),
            field_type: FieldType::Number,
            description: "Individual AI credits reported by GitHub".to_string(),
        },
        FieldDefinition {
            name: "reset".to_string(),
            label: "Resets".to_string(),
            field_type: FieldType::Text,
            description: "Quota reset date reported by GitHub".to_string(),
        },
    ]
}

fn required_non_blank_string(config: &FeedConfig, key: &str) -> Result<String> {
    optional_non_blank_string(config, key)?.ok_or_else(|| {
        anyhow!(
            "feed `{}` (type copilot-usage) is missing required non-empty `{key}` string",
            config.name
        )
    })
}

fn optional_non_blank_string(config: &FeedConfig, key: &str) -> Result<Option<String>> {
    let Some(value) = config.type_specific.get(key) else {
        return Ok(None);
    };
    let value = value.as_str().ok_or_else(|| {
        anyhow!(
            "feed `{}` (type copilot-usage): `{key}` must be a string",
            config.name
        )
    })?;
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(value.to_string()))
}

fn required_number(config: &FeedConfig, key: &str) -> Result<f64> {
    optional_number(config, key)?.ok_or_else(|| {
        anyhow!(
            "feed `{}` (type copilot-usage) is missing required numeric `{key}`",
            config.name
        )
    })
}

fn optional_number(config: &FeedConfig, key: &str) -> Result<Option<f64>> {
    let Some(value) = config.type_specific.get(key) else {
        return Ok(None);
    };
    match value {
        Value::Integer(value) => Ok(Some(*value as f64)),
        Value::Float(value) => Ok(Some(*value)),
        _ => bail!(
            "feed `{}` (type copilot-usage): `{key}` must be a number",
            config.name
        ),
    }
}

fn validate_details_url(config: &FeedConfig, value: &str) -> Result<()> {
    let invalid_url = || {
        anyhow!(
            "feed `{}` (type copilot-usage): `details_url` must be a valid HTTPS URL without embedded credentials",
            config.name
        )
    };
    let url = reqwest::Url::parse(value).map_err(|_| invalid_url())?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid_url());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CopilotQuotaResponse {
    login: Option<String>,
    token_based_billing: Option<bool>,
    quota_reset_date: Option<String>,
    quota_reset_date_utc: Option<String>,
    quota_snapshots: Option<QuotaSnapshots>,
}

#[derive(Debug, Deserialize)]
struct QuotaSnapshots {
    premium_interactions: Option<QuotaSnapshot>,
}

#[derive(Debug, Deserialize)]
struct QuotaSnapshot {
    credits_used: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct GithubIdentity {
    login: String,
}

#[cfg(test)]
#[path = "copilot_usage_tests.rs"]
mod tests;
