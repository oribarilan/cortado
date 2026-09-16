use std::sync::Arc;

use serde_json::json;
use toml::Value;

use super::{config_with, ids, responses_with_pages, AdoPipelinesFeed, StubRunner};
use crate::{
    feed::{
        build_feed_registry_from_configs, config::FieldOverride, Feed, FeedSnapshot,
        RegistryBuildMode,
    },
    notification::change_detection::detect_changes,
    settings_config::{test_feed, FeedConfigDto},
};

#[tokio::test]
async fn hidden_link_preserves_open_action_and_notification_url_across_runs() {
    let overview = "https://dev.azure.com/acme/Platform%20Team/_build?definitionId=42";
    let mut previous = FeedSnapshot {
        name: "Team CI".to_string(),
        feed_type: "ado-pipelines".to_string(),
        activities: Vec::new(),
        provided_fields: Vec::new(),
        error: None,
        hide_when_empty: false,
        last_refreshed: None,
        is_disconnected: false,
    };
    for (latest_build, expected_url) in [
        (serde_json::Value::Null, overview),
        (
            json!({"id": 1, "status": "inProgress"}),
            "https://dev.azure.com/acme/Platform%20Team/_build/results?buildId=1&view=results",
        ),
        (
            json!({"id": 2, "status": "completed", "result": "failed"}),
            "https://dev.azure.com/acme/Platform%20Team/_build/results?buildId=2&view=results",
        ),
    ] {
        let page =
            json!({"value": [{"id": 42, "name": "API", "latestBuild": latest_build}]}).to_string();
        let mut config = config_with([("pipeline_ids", ids(&[42]))]);
        config.field_overrides.insert(
            "link".to_string(),
            FieldOverride {
                visible: Some(false),
                label: None,
            },
        );
        let feed = AdoPipelinesFeed::from_config_with_runner(
            &config,
            Arc::new(StubRunner::new(responses_with_pages(&[&page]))),
        )
        .unwrap();
        let activities = feed.poll().await.unwrap();
        let activity = &activities[0];
        assert_eq!(activity.id, format!("ado-pipeline:{overview}"));
        assert!(!activity.fields.iter().any(|field| field.name == "link"));
        // Assert the wire representation consumed by supportsOpen in both UIs.
        assert_eq!(
            serde_json::to_value(activity).unwrap()["action"],
            json!({"open_url": expected_url})
        );

        let current = FeedSnapshot {
            activities,
            ..previous.clone()
        };
        let events = detect_changes(&previous, &current);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].activity_url.as_deref(), Some(expected_url));
        previous = current;
    }
}

#[tokio::test]
async fn validation_causes_reach_registry_and_settings_test() {
    for (key, value, expected) in [
        (
            "organization",
            "https://example.com/acme",
            "hosted Azure DevOps organization root",
        ),
        ("project", "", "`project` must not be empty"),
        (
            "folder",
            "Team",
            "`folder` must be an exact Azure DevOps folder path",
        ),
    ] {
        let mut config = config_with([("pipeline_ids", ids(&[42]))]);
        if key == "folder" {
            config.type_specific.remove("pipeline_ids");
        }
        config
            .type_specific
            .insert(key.to_string(), Value::String(value.to_string()));
        let dto: FeedConfigDto = serde_json::from_value(json!({
            "name": config.name,
            "type": config.feed_type,
            "type_specific": config.type_specific,
        }))
        .unwrap();
        let registry =
            build_feed_registry_from_configs(vec![config], RegistryBuildMode::Tolerant).unwrap();
        let error = registry.initial_snapshots()[0].error.clone().unwrap();
        assert!(
            error.contains("feed `Team CI` (type ado-pipelines)"),
            "{error}"
        );
        assert!(error.contains(expected), "{error}");
        let result = test_feed(dto).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains(expected));
    }
}
