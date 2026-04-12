use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::watch;

use crate::app_settings::FeedNotifyOverride;
use crate::notification;

use super::harness::feed::HarnessFeed;
use super::runtime::{build_snapshot_for_feed, bump_update_counter, NotificationContext};
use super::{Feed, FeedSnapshotCache};

const DEBOUNCE_DELAY: Duration = Duration::from_millis(200);
const FALLBACK_INTERVAL: Duration = Duration::from_secs(60);

/// Spawns a file watcher for a harness feed.
///
/// Watches the given directories for file changes and triggers an immediate
/// re-poll with a short debounce window. Falls back to a 60s timer to ensure
/// robustness if the watcher misses events.
pub fn spawn_harness_watcher(
    feed: Arc<HarnessFeed>,
    watch_paths: Vec<PathBuf>,
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    notify_ctx: Option<NotificationContext>,
) {
    tokio::spawn(async move {
        harness_watch_loop(feed, watch_paths, cache, update_tx, notify_ctx).await;
    });
}

async fn harness_watch_loop(
    feed: Arc<HarnessFeed>,
    watch_paths: Vec<PathBuf>,
    cache: FeedSnapshotCache,
    update_tx: watch::Sender<u64>,
    notify_ctx: Option<NotificationContext>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    // Set up the file watcher.
    // Keep watcher alive -- it's dropped when this task ends.
    let _watcher = setup_watcher(tx, &watch_paths);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(FALLBACK_INTERVAL) => {
                // Fallback timer -- re-poll even if watcher missed events.
            }
            _ = rx.recv() => {
                // File change detected. Debounce: drain any queued events
                // and wait a short window for more to settle.
                tokio::time::sleep(DEBOUNCE_DELAY).await;
                while rx.try_recv().is_ok() {}
            }
        }

        // Re-poll the feed and dispatch notifications before upserting.
        let snapshot = build_snapshot_for_feed(&cache, feed.as_ref()).await;

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

fn setup_watcher(
    tx: tokio::sync::mpsc::Sender<()>,
    watch_paths: &[PathBuf],
) -> Option<RecommendedWatcher> {
    let watcher = RecommendedWatcher::new(
        move |_event: Result<notify::Event, notify::Error>| {
            let _ = tx.blocking_send(());
        },
        Config::default(),
    );

    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[harness-watcher] failed to create watcher: {e}");
            return None;
        }
    };

    for path in watch_paths {
        if path.exists() {
            if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                eprintln!("[harness-watcher] failed to watch {}: {e}", path.display());
            }
        }
        // If the directory doesn't exist yet, that's OK -- the fallback timer
        // will catch sessions. We could re-try watching periodically, but
        // the directory is typically created by the first plugin that runs.
    }

    Some(watcher)
}
