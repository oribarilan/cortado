use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{watch, RwLock};

use crate::feed::{Feed, FeedRegistry, FeedSnapshot};

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

/// Background poller that seeds and continuously refreshes snapshots.
pub struct BackgroundPoller {
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
}

impl BackgroundPoller {
    /// Builds a poller for an existing cache.
    pub fn new(cache: FeedSnapshotCache) -> Self {
        let (update_tx, _) = watch::channel(0_u64);

        Self { cache, update_tx }
    }

    /// Returns a receiver that is notified whenever the cache updates.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.update_tx.subscribe()
    }

    /// Performs a best-effort startup seed poll within a bounded time budget.
    pub async fn seed_startup_best_effort(&self, registry: Arc<FeedRegistry>, budget: Duration) {
        let feeds: Vec<Arc<dyn Feed>> = registry.active_feeds().to_vec();
        seed_cache_best_effort(self.cache.clone(), self.update_tx.clone(), feeds, budget).await;
    }

    /// Spawns recurring per-feed polling loops.
    pub fn start(&self, registry: Arc<FeedRegistry>) {
        let feeds: Vec<Arc<dyn Feed>> = registry.active_feeds().to_vec();

        for feed in feeds {
            let cache = self.cache.clone();
            let update_tx = self.update_tx.clone();

            tokio::spawn(async move {
                poll_feed_loop(cache, update_tx, feed).await;
            });
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
        let _ = update_tx.send(*update_tx.borrow() + 1);
    }
}

async fn poll_feed_loop(
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    feed: Arc<dyn Feed>,
) {
    loop {
        let interval = Duration::from_secs(feed.interval_seconds().max(1));
        tokio::time::sleep(interval).await;

        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;
        cache.upsert(snapshot).await;

        let _ = update_tx.send(*update_tx.borrow() + 1);
    }
}

async fn build_snapshot_for_feed(cache: &FeedSnapshotCache, feed: &dyn Feed) -> FeedSnapshot {
    let baseline =
        cache.list().await.into_iter().find(|snapshot| {
            snapshot.name == feed.name() && snapshot.feed_type == feed.feed_type()
        });

    match feed.poll().await {
        Ok(activities) => FeedSnapshot {
            name: feed.name().to_string(),
            feed_type: feed.feed_type().to_string(),
            activities,
            provided_fields: feed.provided_fields(),
            error: None,
        },
        Err(error) => FeedSnapshot {
            name: feed.name().to_string(),
            feed_type: feed.feed_type().to_string(),
            activities: baseline.map_or_else(Vec::new, |snapshot| snapshot.activities),
            provided_fields: feed.provided_fields(),
            error: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use crate::feed::{Activity, Feed, FeedRegistry, FeedSnapshot, FieldDefinition, FieldType};

    use super::{build_snapshot_for_feed, FeedSnapshotCache};

    struct SequencedFeed {
        name: String,
        feed_type: String,
        interval: u64,
        outcomes: Mutex<Vec<Result<Vec<Activity>>>>,
    }

    #[async_trait]
    impl Feed for SequencedFeed {
        fn name(&self) -> &str {
            &self.name
        }

        fn feed_type(&self) -> &str {
            &self.feed_type
        }

        fn interval_seconds(&self) -> u64 {
            self.interval
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
            feed_type: "shell".to_string(),
            interval: 30,
            outcomes: Mutex::new(vec![Err(anyhow!("poll failed"))]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        cache
            .upsert(FeedSnapshot {
                name: "My feed".to_string(),
                feed_type: "shell".to_string(),
                activities: vec![Activity {
                    id: "1".to_string(),
                    title: "Old activity".to_string(),
                    fields: Vec::new(),
                }],
                provided_fields: Vec::new(),
                error: None,
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
                feed_type: "shell".to_string(),
                activities: Vec::new(),
                provided_fields: Vec::new(),
                error: None,
            })
            .await;

        cache
            .upsert(FeedSnapshot {
                name: "A".to_string(),
                feed_type: "shell".to_string(),
                activities: vec![Activity {
                    id: "2".to_string(),
                    title: "Updated".to_string(),
                    fields: Vec::new(),
                }],
                provided_fields: Vec::new(),
                error: None,
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
            "shell".to_string(),
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
            feed_type: "shell".to_string(),
            interval: 30,
            outcomes: Mutex::new(vec![Ok(vec![Activity {
                id: "new".to_string(),
                title: "Fresh".to_string(),
                fields: Vec::new(),
            }])]),
        });

        let mut registry = FeedRegistry::new();
        registry.register(feed.clone());

        let cache = FeedSnapshotCache::from_registry(&registry);
        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

        assert_eq!(snapshot.activities.len(), 1);
        assert_eq!(snapshot.activities[0].title, "Fresh");
        assert!(snapshot.error.is_none());
    }
}
