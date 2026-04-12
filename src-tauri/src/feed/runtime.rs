use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tokio::sync::{watch, RwLock};

use crate::app_settings::{AppSettingsState, FeedNotifyOverride};
use crate::feed::connectivity::ConnectivityManager;
use crate::feed::{Activity, Feed, FeedRegistry, FeedSnapshot, StatusKind};
use crate::notification;

const MAX_ACTIVITIES_PER_FEED: usize = 20;

/// In-memory cache of feed snapshots used by commands and tray rendering.
#[derive(Clone, Default)]
pub struct FeedSnapshotCache {
    snapshots: Arc<RwLock<Vec<FeedSnapshot>>>,
}

impl FeedSnapshotCache {
    /// Creates a cache seeded with feed-level defaults and config errors.
    pub fn from_registry(registry: &FeedRegistry) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(registry.initial_snapshots())),
        }
    }

    /// Reads all cached snapshots.
    pub async fn list(&self) -> Vec<FeedSnapshot> {
        self.snapshots.read().await.clone()
    }

    /// Updates one feed snapshot in-place, preserving ordering and config errors.
    pub async fn upsert(&self, snapshot: FeedSnapshot) {
        let mut guard = self.snapshots.write().await;

        if let Some(index) = guard.iter().position(|candidate| {
            candidate.name == snapshot.name && candidate.feed_type == snapshot.feed_type
        }) {
            guard[index] = snapshot;
            return;
        }

        guard.push(snapshot);
    }
}

/// Optional context for dispatching OS notifications from the poll loop.
#[derive(Clone)]
pub struct NotificationContext {
    pub app_handle: tauri::AppHandle,
    pub settings_state: AppSettingsState,
    /// Per-feed notification override: feed name → override setting.
    pub feed_notify_map: Arc<std::collections::HashMap<String, FeedNotifyOverride>>,
}

/// Background poller that seeds and continuously refreshes snapshots.
#[derive(Clone)]
pub struct BackgroundPoller {
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    notify_ctx: Option<NotificationContext>,
    connectivity: Option<Arc<ConnectivityManager>>,
}

impl BackgroundPoller {
    /// Builds a poller for an existing cache.
    pub fn new(cache: FeedSnapshotCache) -> Self {
        let (update_tx, _) = watch::channel(0_u64);

        Self {
            cache,
            update_tx,
            notify_ctx: None,
            connectivity: None,
        }
    }

    /// Attaches notification dispatch context to this poller.
    pub fn with_notifications(mut self, ctx: NotificationContext) -> Self {
        self.notify_ctx = Some(ctx);
        self
    }

    /// Attaches connectivity tracking to this poller.
    pub fn with_connectivity(mut self, mgr: Arc<ConnectivityManager>) -> Self {
        self.connectivity = Some(mgr);
        self
    }

    /// Returns a receiver that is notified whenever the cache updates.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.update_tx.subscribe()
    }

    /// Returns a clone of the internal update sender.
    ///
    /// Used by the config watcher to signal UI refreshes when config files change.
    pub fn update_sender(&self) -> watch::Sender<u64> {
        self.update_tx.clone()
    }

    /// Performs a best-effort startup seed poll within a bounded time budget.
    pub async fn seed_startup_best_effort(&self, registry: Arc<FeedRegistry>, budget: Duration) {
        let feeds: Vec<Arc<dyn Feed>> = registry.active_feeds().to_vec();
        seed_cache_best_effort(self.cache.clone(), self.update_tx.clone(), feeds, budget).await;
    }

    /// Runs an immediate one-shot refresh for all active feeds.
    /// Calls `on_progress(completed, total)` after each feed finishes.
    pub async fn refresh_now(
        &self,
        registry: Arc<FeedRegistry>,
        on_progress: impl Fn(usize, usize),
    ) {
        let feeds: Vec<Arc<dyn Feed>> = registry.active_feeds().to_vec();
        let total = feeds.len();

        if feeds.is_empty() {
            return;
        }

        for (i, feed) in feeds.into_iter().enumerate() {
            let snapshot = build_snapshot_for_feed(&self.cache, feed.as_ref()).await;
            self.cache.upsert(snapshot).await;
            on_progress(i + 1, total);
        }

        bump_update_counter(&self.update_tx);
    }

    /// Spawns recurring per-feed polling loops.
    pub fn start(&self, registry: Arc<FeedRegistry>) {
        let feeds: Vec<Arc<dyn Feed>> = registry.active_feeds().to_vec();

        for feed in feeds {
            let cache = self.cache.clone();
            let update_tx = self.update_tx.clone();
            let notify_ctx = self.notify_ctx.clone();
            let connectivity = self.connectivity.clone();

            tokio::spawn(async move {
                poll_feed_loop(cache, update_tx, feed, notify_ctx, connectivity).await;
            });
        }

        // Spawn file watchers for harness feeds (additive to the poll loop).
        for (feed, paths) in registry.harness_watch_info() {
            super::harness_watcher::spawn_harness_watcher(
                feed,
                paths,
                self.cache.clone(),
                self.update_tx.clone(),
                self.notify_ctx.clone(),
            );
        }
    }
}

async fn seed_cache_best_effort(
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    feeds: Vec<Arc<dyn Feed>>,
    budget: Duration,
) {
    if feeds.is_empty() {
        return;
    }

    let start = Instant::now();
    let mut seeded = false;

    for feed in feeds {
        let elapsed = start.elapsed();
        if elapsed >= budget {
            break;
        }

        let remaining = budget - elapsed;
        let poll_future = build_snapshot_for_feed(&cache, feed.as_ref());

        if let Ok(snapshot) = tokio::time::timeout(remaining, poll_future).await {
            cache.upsert(snapshot).await;
            seeded = true;
        }
    }

    if seeded {
        bump_update_counter(&update_tx);
    }
}

async fn poll_feed_loop(
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    feed: Arc<dyn Feed>,
    notify_ctx: Option<NotificationContext>,
    connectivity: Option<Arc<ConnectivityManager>>,
) {
    loop {
        let interval = feed.interval().max(Duration::from_secs(1));
        tokio::time::sleep(interval).await;

        // Skip polling for network feeds while offline.
        let is_network = super::is_network_feed_type(feed.feed_type());
        if let Some(ref cm) = connectivity {
            if is_network && cm.is_offline() {
                continue;
            }
        }

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        // Track network feed poll outcomes for connectivity detection.
        // Any error type (network, auth, parsing) increments the failure counter.
        // This is intentionally coarse -- the connectivity manager's ping check
        // is the actual arbiter of online/offline state, so false triggers from
        // non-network errors are harmless (the ping succeeds and resets counters).
        if let Some(ref cm) = connectivity {
            if is_network {
                if snapshot.error.is_some() {
                    cm.record_failure(feed.name()).await;
                } else {
                    cm.record_success(feed.name()).await;
                }
            }
        }

        // Dispatch notifications before upserting (prev snapshot is still in cache).
        if let Some(ref ctx) = notify_ctx {
            let prev = cache
                .list()
                .await
                .into_iter()
                .find(|s| s.name == feed.name() && s.feed_type == feed.feed_type());

            if let Some(prev) = prev {
                let feed_override = ctx
                    .feed_notify_map
                    .get(feed.name())
                    .cloned()
                    .unwrap_or(FeedNotifyOverride::Global);

                notification::dispatch::process_feed_update(
                    &ctx.app_handle,
                    &ctx.settings_state,
                    &prev,
                    &snapshot,
                    &feed_override,
                )
                .await;
            }
        }

        cache.upsert(snapshot).await;

        bump_update_counter(&update_tx);
    }
}

pub fn bump_update_counter(update_tx: &watch::Sender<u64>) {
    let next = (*update_tx.borrow()).wrapping_add(1);
    let _ = update_tx.send(next);
}

pub(crate) async fn build_snapshot_for_feed(
    cache: &FeedSnapshotCache,
    feed: &dyn Feed,
) -> FeedSnapshot {
    let baseline =
        cache.list().await.into_iter().find(|snapshot| {
            snapshot.name == feed.name() && snapshot.feed_type == feed.feed_type()
        });

    // Preserve the previous last_refreshed for error paths (poll failed, data is stale).
    let baseline_last_refreshed = baseline.as_ref().and_then(|s| s.last_refreshed);

    match feed.poll().await {
        Ok(mut activities) => {
            for activity in &mut activities {
                activity.retained = false;
            }

            let retained = retained_activities_from_baseline(
                baseline.as_ref(),
                &activities,
                feed.retain_for(),
            );
            activities.extend(retained);
            activities.sort_by(|a, b| {
                a.retained
                    .cmp(&b.retained)
                    .then_with(|| {
                        let a_priority = StatusKind::rollup_for_activity(a).priority();
                        let b_priority = StatusKind::rollup_for_activity(b).priority();
                        b_priority.cmp(&a_priority)
                    })
                    .then_with(|| {
                        // Within the same kind, most recently active first.
                        b.sort_ts.cmp(&a.sort_ts)
                    })
            });
            activities.truncate(MAX_ACTIVITIES_PER_FEED);

            FeedSnapshot {
                name: feed.name().to_string(),
                feed_type: feed.feed_type().to_string(),
                activities,
                provided_fields: feed.provided_fields(),
                error: None,
                hide_when_empty: feed.hide_when_empty(),
                last_refreshed: Some(unix_epoch_millis_now()),
                is_disconnected: false,
            }
        }
        Err(error) => FeedSnapshot {
            name: feed.name().to_string(),
            feed_type: feed.feed_type().to_string(),
            activities: baseline.map_or_else(Vec::new, |snapshot| snapshot.activities),
            provided_fields: feed.provided_fields(),
            error: Some(error.to_string()),
            hide_when_empty: feed.hide_when_empty(),
            last_refreshed: baseline_last_refreshed,
            is_disconnected: false,
        },
    }
}

fn retained_activities_from_baseline(
    baseline: Option<&FeedSnapshot>,
    active_activities: &[Activity],
    retain_for: Option<Duration>,
) -> Vec<Activity> {
    let Some(retain_for) = retain_for else {
        return Vec::new();
    };

    let Some(baseline) = baseline else {
        return Vec::new();
    };

    let active_ids: HashSet<&str> = active_activities
        .iter()
        .map(|activity| activity.id.as_str())
        .collect();

    let now_unix_ms = unix_epoch_millis_now();
    let mut retained = Vec::new();

    for activity in &baseline.activities {
        if active_ids.contains(activity.id.as_str()) {
            continue;
        }

        let mut retained_activity = activity.clone();
        let first_retained_at = retained_activity.retained_at_unix_ms.unwrap_or(now_unix_ms);

        let retained_age_ms = now_unix_ms.saturating_sub(first_retained_at);
        if Duration::from_millis(retained_age_ms) >= retain_for {
            continue;
        }

        retained_activity.retained_at_unix_ms = Some(first_retained_at);
        retained_activity.retained = true;
        retained.push(retained_activity);
    }

    retained
}

fn unix_epoch_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use crate::feed::{Activity, Feed, FeedRegistry, FeedSnapshot, FieldDefinition, FieldType};

    use super::{build_snapshot_for_feed, BackgroundPoller, FeedSnapshotCache};

    struct SequencedFeed {
        name: String,
        feed_type: String,
        interval: Duration,
        retain_for: Option<Duration>,
        outcomes: Mutex<Vec<Result<Vec<Activity>>>>,
    }

    fn activity(id: &str, title: &str) -> Activity {
        Activity {
            id: id.to_string(),
            title: title.to_string(),
            fields: Vec::new(),
            retained: false,
            retained_at_unix_ms: None,
            sort_ts: None,
            action: None,
        }
    }

    #[async_trait]
    impl Feed for SequencedFeed {
        fn name(&self) -> &str {
            &self.name
        }

        fn feed_type(&self) -> &str {
            &self.feed_type
        }

        fn interval(&self) -> Duration {
            self.interval
        }

        fn retain_for(&self) -> Option<Duration> {
            self.retain_for
        }

        fn provided_fields(&self) -> Vec<FieldDefinition> {
            vec![FieldDefinition {
                name: "status".to_string(),
                label: "Status".to_string(),
                field_type: FieldType::Status,
                description: "Status field".to_string(),
            }]
        }

        async fn poll(&self) -> Result<Vec<Activity>> {
            let mut outcomes = self.outcomes.lock().await;

            if outcomes.is_empty() {
                return Ok(Vec::new());
            }

            outcomes.remove(0)
        }
    }

    #[tokio::test]
    async fn build_snapshot_for_feed_keeps_stale_activities_on_error() {
        let feed = Arc::new(SequencedFeed {
            name: "My feed".to_string(),
            feed_type: "test".to_string(),
            interval: Duration::from_secs(30),
            retain_for: None,
            outcomes: Mutex::new(vec![Err(anyhow!("poll failed"))]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "My feed".to_string(),
                feed_type: "test".to_string(),
                activities: vec![activity("1", "Old activity")],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        assert_eq!(snapshot.activities.len(), 1);
        assert_eq!(snapshot.activities[0].title, "Old activity");
        assert_eq!(snapshot.error.as_deref(), Some("poll failed"));
    }

    #[tokio::test]
    async fn cache_upsert_replaces_existing_snapshot() {
        let cache = FeedSnapshotCache::default();

        cache
            .upsert(FeedSnapshot {
                name: "A".to_string(),
                feed_type: "test".to_string(),
                activities: Vec::new(),
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        cache
            .upsert(FeedSnapshot {
                name: "A".to_string(),
                feed_type: "test".to_string(),
                activities: vec![activity("2", "Updated")],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshots = cache.list().await;
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].activities.len(), 1);
        assert_eq!(snapshots[0].activities[0].title, "Updated");
    }

    #[tokio::test]
    async fn cache_from_registry_seeds_config_error_entries() {
        let mut registry = FeedRegistry::new();
        registry.register_error(
            "Broken".to_string(),
            "test".to_string(),
            "invalid config".to_string(),
        );

        let cache = FeedSnapshotCache::from_registry(&registry);
        let snapshots = cache.list().await;

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].name, "Broken");
        assert_eq!(snapshots[0].error.as_deref(), Some("invalid config"));
    }

    #[tokio::test]
    async fn build_snapshot_for_feed_success_replaces_activities_and_clears_error() {
        let feed = Arc::new(SequencedFeed {
            name: "My feed".to_string(),
            feed_type: "test".to_string(),
            interval: Duration::from_secs(30),
            retain_for: None,
            outcomes: Mutex::new(vec![Ok(vec![activity("new", "Fresh")])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        assert_eq!(snapshot.activities.len(), 1);
        assert_eq!(snapshot.activities[0].title, "Fresh");
        assert!(snapshot.error.is_none());
    }

    #[tokio::test]
    async fn refresh_now_updates_cache_and_notifies() {
        let feed = Arc::new(SequencedFeed {
            name: "My feed".to_string(),
            feed_type: "test".to_string(),
            interval: Duration::from_secs(30),
            retain_for: None,
            outcomes: Mutex::new(vec![Ok(vec![activity("fresh-1", "Fresh activity")])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed);
        let registry = Arc::new(registry);

        let cache = FeedSnapshotCache::from_registry(registry.as_ref());
        let poller = BackgroundPoller::new(cache.clone());
        let updates = poller.subscribe();

        poller.refresh_now(registry.clone(), |_, _| {}).await;

        assert!(*updates.borrow() > 0);

        let snapshots = cache.list().await;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.name == "My feed" && snapshot.feed_type == "test")
            .expect("snapshot should exist");

        assert_eq!(snapshot.activities.len(), 1);
        assert_eq!(snapshot.activities[0].title, "Fresh activity");
        assert!(snapshot.error.is_none());
    }

    #[tokio::test]
    async fn retention_marks_disappeared_activity_as_retained() {
        let feed = Arc::new(SequencedFeed {
            name: "Retain feed".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Duration::from_secs(30),
            retain_for: Some(Duration::from_secs(3600)),
            outcomes: Mutex::new(vec![Ok(vec![activity("A", "Still open")])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "Retain feed".to_string(),
                feed_type: "github-pr".to_string(),
                activities: vec![activity("A", "Still open"), activity("B", "Just merged")],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        assert_eq!(snapshot.activities.len(), 2);
        assert_eq!(snapshot.activities[0].id, "A");
        assert!(!snapshot.activities[0].retained);

        assert_eq!(snapshot.activities[1].id, "B");
        assert!(snapshot.activities[1].retained);
        assert!(snapshot.activities[1].retained_at_unix_ms.is_some());
    }

    #[tokio::test]
    async fn retention_none_drops_disappeared_activity() {
        let feed = Arc::new(SequencedFeed {
            name: "No retain feed".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Duration::from_secs(30),
            retain_for: None,
            outcomes: Mutex::new(vec![Ok(vec![activity("A", "Still open")])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "No retain feed".to_string(),
                feed_type: "github-pr".to_string(),
                activities: vec![activity("A", "Still open"), activity("B", "Closed")],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        assert_eq!(snapshot.activities.len(), 1);
        assert_eq!(snapshot.activities[0].id, "A");
    }

    #[tokio::test]
    async fn retention_expires_old_retained_activity() {
        let now_ms = super::unix_epoch_millis_now();
        let old_ms = now_ms.saturating_sub(Duration::from_secs(600).as_millis() as u64);

        let feed = Arc::new(SequencedFeed {
            name: "Expire feed".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Duration::from_secs(30),
            retain_for: Some(Duration::from_secs(300)),
            outcomes: Mutex::new(vec![Ok(vec![])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "Expire feed".to_string(),
                feed_type: "github-pr".to_string(),
                activities: vec![Activity {
                    retained: true,
                    retained_at_unix_ms: Some(old_ms),
                    ..activity("gone", "Gone")
                }],
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;
        assert!(snapshot.activities.is_empty());
    }

    #[tokio::test]
    async fn activity_list_is_capped_to_twenty_with_active_first() {
        let active: Vec<Activity> = (0..20)
            .map(|index| activity(&format!("active-{index}"), &format!("Active {index}")))
            .collect();

        let feed = Arc::new(SequencedFeed {
            name: "Cap feed".to_string(),
            feed_type: "github-pr".to_string(),
            interval: Duration::from_secs(30),
            retain_for: Some(Duration::from_secs(3600)),
            outcomes: Mutex::new(vec![Ok(active.clone())]),
        });

        let mut baseline = active;
        baseline.push(activity("retained-a", "Retained A"));
        baseline.push(activity("retained-b", "Retained B"));

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "Cap feed".to_string(),
                feed_type: "github-pr".to_string(),
                activities: baseline,
                provided_fields: Vec::new(),
                error: None,
                hide_when_empty: false,
                last_refreshed: None,
                is_disconnected: false,
            })
            .await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;
        assert_eq!(snapshot.activities.len(), 20);
        assert!(snapshot
            .activities
            .iter()
            .all(|activity| !activity.retained));
    }
}
