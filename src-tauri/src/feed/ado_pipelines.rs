use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Result};

use self::model::{
    base_field_definitions, build_browser_url, field, map_run_status, parse_iso_to_unix_ms,
    parse_pipeline_ids, required_string, status_field, validate_folder, validate_organization,
    validate_project, AdoPipelineDefinition, DefinitionsResponse,
};

use crate::feed::{
    ado_common::{
        ensure_az_ready, looks_like_az_auth_error, looks_like_missing_extension,
        non_zero_exit_context, AZ_EXTENSION_MISSING_MESSAGE, AZ_UNAUTHENTICATED_MESSAGE,
    },
    config::{FeedConfig, FieldOverride},
    field_overrides::{apply_activity_overrides, apply_definition_overrides},
    process::{CommandInvocation, ProcessRunner, TokioProcessRunner},
    Activity, Feed, FeedAction, FieldDefinition, FieldValue, StatusKind,
};

mod model;

const DEFAULT_INTERVAL_SECONDS: u64 = 120;
const AZ_POLL_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_PIPELINES: usize = 20;
const MAX_PAGES: usize = 100;
/// Azure's own CLI uses process type 2 when creating YAML definitions.
const YAML_PROCESS_TYPE: u8 = 2;

#[derive(Debug, Clone)]
enum PipelineSelector {
    Ids(Vec<u32>),
    Folder(String),
}

/// Feed that polls the latest run for selected Azure DevOps YAML pipelines.
pub struct AdoPipelinesFeed {
    name: String,
    organization: String,
    project: String,
    selector: PipelineSelector,
    interval: Duration,
    retain_for: Option<Duration>,
    config_overrides: HashMap<String, FieldOverride>,
    process_runner: Arc<dyn ProcessRunner>,
}

impl AdoPipelinesFeed {
    /// Builds an Azure DevOps pipelines feed from parsed config.
    pub fn from_config(config: &FeedConfig) -> Result<Self> {
        Self::from_config_with_runner(config, Arc::new(TokioProcessRunner))
    }

    /// Builds a feed with an injected process runner for deterministic tests.
    pub fn from_config_with_runner(
        config: &FeedConfig,
        process_runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self> {
        let prefix = || format!("feed `{}` (type ado-pipelines)", config.name);
        // Registry and Settings errors use Display, which omits anyhow's cause chain.
        let config_error = |error: anyhow::Error| anyhow!("{}: {error}", prefix());
        let organization = validate_organization(required_string(config, "organization")?)
            .map_err(config_error)?;
        let project =
            validate_project(required_string(config, "project")?).map_err(config_error)?;

        let selector = match (
            config.type_specific.get("pipeline_ids"),
            config.type_specific.get("folder"),
        ) {
            (Some(_), Some(_)) => bail!(
                "{}: specify either `pipeline_ids` or `folder`, not both",
                prefix()
            ),
            (None, None) => bail!(
                "{} is missing required `pipeline_ids` or `folder` selector",
                prefix()
            ),
            (Some(value), None) => {
                PipelineSelector::Ids(parse_pipeline_ids(value, &prefix(), MAX_PIPELINES)?)
            }
            (None, Some(value)) => {
                let folder = value
                    .as_str()
                    .ok_or_else(|| anyhow!("{}: `folder` must be a string", prefix()))?;
                PipelineSelector::Folder(validate_folder(folder).map_err(config_error)?)
            }
        };

        Ok(Self {
            name: config.name.clone(),
            organization,
            project,
            selector,
            interval: config
                .interval
                .unwrap_or(Duration::from_secs(DEFAULT_INTERVAL_SECONDS)),
            retain_for: config.retain,
            config_overrides: config.field_overrides.clone(),
            process_runner,
        })
    }

    async fn fetch_definitions(&self) -> Result<Vec<AdoPipelineDefinition>> {
        let mut definitions = Vec::new();
        let mut continuation_token: Option<String> = None;
        let mut seen_tokens = HashSet::new();
        let mut seen_definition_ids = HashSet::new();

        for _ in 0..MAX_PAGES {
            let invocation = self.definitions_invocation(continuation_token.as_deref());
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
                bail!(
                    "`{command_display}` failed with {}",
                    non_zero_exit_context(output.exit_code, &output.stdout, &output.stderr)
                );
            }

            let page: DefinitionsResponse =
                serde_json::from_str(&output.stdout).map_err(|error| {
                    anyhow!("failed parsing Azure DevOps definitions JSON output: {error}")
                })?;
            for definition in page.value {
                let matches_selector = match &self.selector {
                    PipelineSelector::Ids(ids) => ids.contains(&definition.id),
                    PipelineSelector::Folder(folder) => {
                        let path = definition
                            .path
                            .as_deref()
                            .filter(|path| !path.is_empty())
                            .ok_or_else(|| {
                                anyhow!(
                                    "Azure DevOps pipeline definition {} is missing a usable folder path",
                                    definition.id
                                )
                            })?;
                        path == folder
                    }
                };
                if !matches_selector {
                    continue;
                }
                if !seen_definition_ids.insert(definition.id) {
                    bail!(
                        "Azure DevOps returned duplicate pipeline definition ID {}",
                        definition.id
                    );
                }
                definitions.push(definition);
                if definitions.len() > MAX_PIPELINES {
                    bail!(
                        "selection contains more than {MAX_PIPELINES} YAML pipelines; narrow the selection"
                    );
                }
            }

            let Some(next_token) = page.continuation_token else {
                return Ok(definitions);
            };
            if next_token.trim().is_empty() || !seen_tokens.insert(next_token.clone()) {
                bail!("Azure DevOps definitions pagination returned a non-progressing continuation token");
            }
            continuation_token = Some(next_token);
        }

        bail!("Azure DevOps definitions pagination exceeded {MAX_PAGES} pages")
    }

    fn definitions_invocation(&self, continuation_token: Option<&str>) -> CommandInvocation {
        let mut query = vec![
            format!("processType={YAML_PROCESS_TYPE}"),
            "includeLatestBuilds=true".to_string(),
        ];
        match &self.selector {
            PipelineSelector::Ids(ids) => query.push(format!(
                "definitionIds={}",
                ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
            )),
            PipelineSelector::Folder(folder) => query.push(format!("path={folder}")),
        }
        if let Some(token) = continuation_token {
            query.push(format!("continuationToken={token}"));
        }

        let mut args = vec![
            "devops".to_string(),
            "invoke".to_string(),
            "--area".to_string(),
            "build".to_string(),
            "--resource".to_string(),
            "definitions".to_string(),
            "--route-parameters".to_string(),
            format!("project={}", self.project),
            "--query-parameters".to_string(),
        ];
        args.extend(query);
        args.extend([
            "--organization".to_string(),
            self.organization.clone(),
            "--api-version".to_string(),
            "7.1".to_string(),
            "--http-method".to_string(),
            "GET".to_string(),
            "--output".to_string(),
            "json".to_string(),
        ]);

        CommandInvocation::new("az", args, AZ_POLL_TIMEOUT)
    }

    fn select_definitions(
        &self,
        definitions: Vec<AdoPipelineDefinition>,
    ) -> Result<Vec<AdoPipelineDefinition>> {
        let selected: Vec<AdoPipelineDefinition> = match &self.selector {
            PipelineSelector::Ids(ids) => {
                let expected: HashSet<u32> = ids.iter().copied().collect();
                let mut by_id = HashMap::new();
                for definition in definitions {
                    if expected.contains(&definition.id) {
                        by_id.insert(definition.id, definition);
                    }
                }
                let missing: Vec<String> = ids
                    .iter()
                    .filter(|id| !by_id.contains_key(id))
                    .map(u32::to_string)
                    .collect();
                if !missing.is_empty() {
                    bail!(
                        "configured pipeline IDs were not returned as accessible YAML pipelines: {}",
                        missing.join(", ")
                    );
                }
                ids.iter().filter_map(|id| by_id.remove(id)).collect()
            }
            PipelineSelector::Folder(_) => definitions,
        };

        if selected.len() > MAX_PIPELINES {
            bail!(
                "selection contains {} YAML pipelines; narrow it to at most {MAX_PIPELINES}",
                selected.len()
            );
        }
        Ok(selected)
    }
}

#[async_trait::async_trait]
impl Feed for AdoPipelinesFeed {
    fn name(&self) -> &str {
        &self.name
    }

    fn feed_type(&self) -> &str {
        "ado-pipelines"
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
        ensure_az_ready(self.process_runner.as_ref()).await?;
        self.select_definitions(self.fetch_definitions().await?)?
            .into_iter()
            .map(|definition| self.definition_to_activity(definition))
            .collect()
    }
}

impl AdoPipelinesFeed {
    fn definition_to_activity(&self, definition: AdoPipelineDefinition) -> Result<Activity> {
        let overview_url =
            build_browser_url(&self.organization, &self.project, definition.id, None)?;
        let (status, branch, run, event, link, sort_ts) = match definition.latest_build {
            Some(build) => {
                let build_id = build.id.ok_or_else(|| {
                    anyhow!("pipeline {} latest build is missing its ID", definition.id)
                })?;
                (
                    map_run_status(build.status.as_deref(), build.result.as_deref()),
                    build.source_branch.unwrap_or_default(),
                    build.build_number.unwrap_or_default(),
                    build.reason.unwrap_or_default(),
                    build_browser_url(
                        &self.organization,
                        &self.project,
                        definition.id,
                        Some(build_id),
                    )?,
                    parse_iso_to_unix_ms(build.queue_time.as_deref()),
                )
            }
            None => (
                status_field("not run", StatusKind::Idle),
                String::new(),
                String::new(),
                String::new(),
                overview_url.clone(),
                None,
            ),
        };

        let fields = apply_activity_overrides(
            vec![
                field("status", "Status", status),
                field("branch", "Branch", FieldValue::Text { value: branch }),
                field("run", "Run", FieldValue::Text { value: run }),
                field("event", "Event", FieldValue::Text { value: event }),
                field(
                    "link",
                    "Link",
                    FieldValue::Url {
                        value: link.clone(),
                    },
                ),
            ],
            &HashMap::new(),
            &self.config_overrides,
        );

        Ok(Activity {
            id: format!("ado-pipeline:{overview_url}"),
            title: definition.name,
            fields,
            retained: false,
            retained_at_unix_ms: None,
            sort_ts,
            action: Some(FeedAction::OpenUrl(link)),
        })
    }
}

#[cfg(test)]
#[path = "ado_pipelines_tests.rs"]
mod tests;
