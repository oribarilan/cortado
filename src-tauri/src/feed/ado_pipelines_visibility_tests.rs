use std::{sync::Arc, time::Duration};

use serde_json::{json, Value as JsonValue};
use toml::Value;

use super::{config_error, config_with, ids, responses_with_pages, AdoPipelinesFeed, StubRunner};
use crate::{
    app_settings::NotificationMode,
    feed::{
        config::FieldOverride, runtime::build_snapshot_for_feed, Feed, FeedSnapshotCache,
        StatusKind,
    },
    notification::{
        change_detection::{detect_changes, ChangeType},
        dispatch::matches_mode,
    },
};

const FINISH_MS: u64 = 1_767_326_645_000;

fn page(build: JsonValue) -> String {
    json!({"value": [{"id": 42, "name": "API", "path": "\\Team", "latestBuild": build}]})
        .to_string()
}

#[test]
fn passing_window_defaults_and_accepts_nonnegative_durations() {
    for (raw, expected) in [
        (None, 3_600_000),
        (Some(""), 3_600_000),
        (Some("  "), 3_600_000),
        (Some("0s"), 0),
        (Some("30m"), 1_800_000),
        (Some(" 1.5h "), 5_400_000),
        (Some("0.5s"), 500),
    ] {
        let mut config = config_with([("pipeline_ids", ids(&[42]))]);
        if let Some(raw) = raw {
            config
                .type_specific
                .insert("show_passing_for".into(), Value::String(raw.into()));
        }
        let feed = AdoPipelinesFeed::from_config(&config).unwrap();
        assert_eq!(feed.show_passing_for.as_millis(), expected, "{raw:?}");
    }
}

#[test]
fn rejects_invalid_passing_windows_with_actionable_context() {
    for value in [
        Value::Integer(0),
        Value::Float(1.5),
        Value::Boolean(false),
        Value::Array(vec![]),
        Value::String("-1h".into()),
        Value::String("later".into()),
        Value::String("9223372036854775807s".into()),
    ] {
        let config = config_with([("pipeline_ids", ids(&[42])), ("show_passing_for", value)]);
        let error = config_error(&config);
        assert!(
            error.contains("feed `Team CI` (type ado-pipelines)"),
            "{error}"
        );
        assert!(error.contains("show_passing_for"), "{error}");
    }
}

#[tokio::test]
async fn deadline_uses_finish_time_and_survives_hidden_status_fields() {
    let response = page(json!({
        "id": 7, "status": "completed", "result": "succeeded",
        "queueTime": "2026-01-02T03:04:05Z", "finishTime": "2026-01-02T04:04:05Z"
    }));
    let mut config = config_with([("pipeline_ids", ids(&[42]))]);
    config.field_overrides.insert(
        "status".into(),
        FieldOverride {
            visible: Some(false),
            label: Some("Checks".into()),
        },
    );
    let runner = Arc::new(StubRunner::new(responses_with_pages(&[&response])));
    let feed = AdoPipelinesFeed::from_config_with_runner(&config, runner.clone()).unwrap();
    let activities = feed.poll().await.unwrap();
    let activity = &activities[0];
    assert_eq!(activity.visible_until, Some(FINISH_MS + 3_600_000));
    assert_eq!(activity.sort_ts, Some(FINISH_MS - 3_600_000));
    assert!(!activity.fields.iter().any(|field| field.name == "status"));
    assert_eq!(
        serde_json::to_value(activity).unwrap()["visible_until"],
        FINISH_MS + 3_600_000
    );
    assert_eq!(runner.invocations().await.len(), 4, "no extra ADO requests");
}

#[tokio::test]
async fn lifecycle_states_and_finish_time_fallbacks_are_conservative() {
    for (build, expected) in [
        (JsonValue::Null, Some(0)),
        (json!({"id": 1, "status": "notStarted"}), None),
        (json!({"id": 1, "status": "postponed"}), None),
        (
            json!({"id": 1, "status": "inProgress", "result": "succeeded"}),
            None,
        ),
        (json!({"id": 1, "status": "cancelling"}), None),
        (
            json!({"id": 1, "status": "completed", "result": "failed"}),
            None,
        ),
        (
            json!({"id": 1, "status": "completed", "result": "partiallySucceeded"}),
            None,
        ),
        (
            json!({"id": 1, "status": "completed", "result": "canceled"}),
            None,
        ),
        (json!({"id": 1, "status": "completed"}), None),
        (
            json!({"id": 1, "status": "unexpected", "result": "succeeded"}),
            None,
        ),
        (json!({"id": 1}), None),
        (
            json!({"id": 1, "status": "completed", "result": "succeeded", "queueTime": "2020-01-01T00:00:00Z"}),
            None,
        ),
        (
            json!({"id": 1, "status": "completed", "result": "succeeded", "finishTime": "invalid"}),
            None,
        ),
        (
            json!({"id": 1, "status": "completed", "result": "succeeded", "finishTime": "1960-01-01T00:00:00Z"}),
            None,
        ),
        (
            json!({"id": 1, "status": "COMPLETED", "result": "SUCCEEDED", "finishTime": "2026-01-02T05:04:05+01:00"}),
            Some(FINISH_MS + 3_600_000),
        ),
    ] {
        let response = page(build.clone());
        let feed = AdoPipelinesFeed::from_config_with_runner(
            &config_with([("pipeline_ids", ids(&[42]))]),
            Arc::new(StubRunner::new(responses_with_pages(&[&response]))),
        )
        .unwrap();
        let activity = feed.poll().await.unwrap().remove(0);
        assert_eq!(activity.visible_until, expected, "{build}");
        if expected.is_none() {
            assert!(serde_json::to_value(activity)
                .unwrap()
                .get("visible_until")
                .is_none());
        }
    }
}

#[tokio::test]
async fn zero_window_hides_success_even_without_finish_time_or_with_future_time() {
    for finish in [
        JsonValue::Null,
        json!("invalid"),
        json!("2099-01-01T00:00:00Z"),
    ] {
        let response = page(json!({
            "id": 1, "status": "completed", "result": "succeeded", "finishTime": finish,
        }));
        let feed = AdoPipelinesFeed::from_config_with_runner(
            &config_with([
                ("pipeline_ids", ids(&[42])),
                ("show_passing_for", Value::String("0s".into())),
            ]),
            Arc::new(StubRunner::new(responses_with_pages(&[&response]))),
        )
        .unwrap();
        assert_eq!(feed.poll().await.unwrap()[0].visible_until, Some(0));
    }
}

#[tokio::test]
async fn hidden_pipelines_stay_active_and_notify_recovery_without_synthetic_removals() {
    let failed = page(json!({"id": 1, "status": "completed", "result": "failed"}));
    let passed = page(json!({
        "id": 2, "status": "completed", "result": "succeeded", "finishTime": "2000-01-01T00:00:00Z",
    }));
    let empty = r#"{"value":[]}"#;
    let responses = [&failed[..], &passed, &passed, &failed, empty]
        .into_iter()
        .flat_map(|response| responses_with_pages(&[response]))
        .collect();
    let mut config = config_with([("folder", Value::String("\\Team".into()))]);
    config.retain = Some(Duration::from_secs(3600));
    let feed =
        AdoPipelinesFeed::from_config_with_runner(&config, Arc::new(StubRunner::new(responses)))
            .unwrap();
    let cache = FeedSnapshotCache::default();
    let failing = build_snapshot_for_feed(&cache, &feed).await;
    cache.upsert(failing.clone()).await;
    let passing = build_snapshot_for_feed(&cache, &feed).await;
    assert_eq!(passing.activities.len(), 1);
    assert_eq!(passing.activities[0].id, failing.activities[0].id);
    assert!(!passing.activities[0].retained);
    assert!(passing.activities[0].retained_at_unix_ms.is_none());
    assert!(passing.activities[0].visible_until.unwrap() < passing.last_refreshed.unwrap());
    let events = detect_changes(&failing, &passing);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].change_type, ChangeType::KindChanged);
    assert_eq!(events[0].new_kind, Some(StatusKind::Idle));
    assert!(matches_mode(&NotificationMode::WorthKnowing, &events[0]));
    cache.upsert(passing.clone()).await;

    let still_passing = build_snapshot_for_feed(&cache, &feed).await;
    assert_eq!(still_passing.activities.len(), 1);
    assert!(!still_passing.activities[0].retained);
    assert!(detect_changes(&passing, &still_passing).is_empty());
    cache.upsert(still_passing.clone()).await;

    let failing_again = build_snapshot_for_feed(&cache, &feed).await;
    assert!(failing_again.activities[0].visible_until.is_none());
    let events = detect_changes(&still_passing, &failing_again);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].change_type, ChangeType::KindChanged);
    assert_eq!(events[0].new_kind, Some(StatusKind::AttentionNegative));
    cache.upsert(failing_again.clone()).await;

    let removed = build_snapshot_for_feed(&cache, &feed).await;
    assert_eq!(removed.activities.len(), 1);
    assert!(
        removed.activities[0].retained,
        "only leaving the folder starts retention"
    );
    assert!(detect_changes(&failing_again, &removed).is_empty());
}
