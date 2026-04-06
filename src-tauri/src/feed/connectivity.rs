use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{watch, Mutex, Notify};

/// All network feeds must fail this many consecutive times before triggering a connectivity check.
const FAILURE_THRESHOLD: u32 = 2;

/// Initial delay between recovery pings when offline.
const BACKOFF_INITIAL: Duration = Duration::from_secs(5);

/// Maximum delay between recovery pings.
const BACKOFF_CAP: Duration = Duration::from_secs(120);

/// Manages connectivity detection and offline state.
///
/// Tracks consecutive poll failures per network feed. When all network feeds
/// reach the failure threshold, triggers a connectivity ping. If the ping fails,
/// the app enters offline mode. Auto-recovery via exponential backoff pinging.
///
/// The failure tracking is intentionally coarse -- any poll error (network,
/// auth, parsing) increments the counter. This is safe because the actual
/// connectivity decision is made by `ping_ok`, not by error classification.
/// Non-network errors may trigger an unnecessary ping, but the ping will
/// succeed and reset counters without entering offline mode.
pub struct ConnectivityManager {
    /// Number of network-based feeds in the registry.
    network_feed_count: usize,
    /// Consecutive failure count per feed name.
    failure_counts: Mutex<HashMap<String, u32>>,
    /// Offline state: true when the app considers itself disconnected.
    offline_tx: watch::Sender<bool>,
    offline_rx: watch::Receiver<bool>,
    /// Wakes the checker task for an immediate connectivity test.
    check_trigger: Notify,
}

impl ConnectivityManager {
    /// Creates a new manager. `network_feed_count` is the number of feeds that require
    /// network connectivity (github-pr, github-actions, ado-pr, http-health).
    pub fn new(network_feed_count: usize) -> Arc<Self> {
        let (offline_tx, offline_rx) = watch::channel(false);
        Arc::new(Self {
            network_feed_count,
            failure_counts: Mutex::new(HashMap::new()),
            offline_tx,
            offline_rx,
            check_trigger: Notify::new(),
        })
    }

    /// Returns true if the app is currently offline.
    pub fn is_offline(&self) -> bool {
        *self.offline_rx.borrow()
    }

    /// Records a poll failure for a network feed. May trigger a connectivity check
    /// if all network feeds have reached the failure threshold.
    pub async fn record_failure(&self, feed_name: &str) {
        if self.network_feed_count == 0 {
            return;
        }
        let mut counts = self.failure_counts.lock().await;
        let count = counts.entry(feed_name.to_string()).or_insert(0);
        *count += 1;

        // Check if ALL network feeds have failed enough times.
        let failing_count = counts.values().filter(|c| **c >= FAILURE_THRESHOLD).count();
        if failing_count >= self.network_feed_count {
            self.check_trigger.notify_one();
        }
    }

    /// Records a successful poll for a network feed, resetting its failure counter.
    pub async fn record_success(&self, feed_name: &str) {
        let mut counts = self.failure_counts.lock().await;
        counts.insert(feed_name.to_string(), 0);
    }

    /// Triggers an immediate connectivity check (used by the "Retry" button).
    pub fn trigger_retry(&self) {
        self.check_trigger.notify_one();
    }

    /// Runs the connectivity checker loop. Call this once via `tokio::spawn`.
    pub async fn run_checker(self: Arc<Self>) {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        loop {
            // Wait for a trigger (failure threshold reached or manual retry).
            self.check_trigger.notified().await;

            // Quick connectivity check.
            if self.ping_ok(&client).await {
                // Transient failure -- reset counters and stay online.
                self.failure_counts.lock().await.clear();
                let _ = self.offline_tx.send(false);
                continue;
            }

            // Confirmed offline -- enter offline mode.
            let _ = self.offline_tx.send(true);

            // Exponential backoff recovery loop.
            let mut delay = BACKOFF_INITIAL;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = self.check_trigger.notified() => {},
                }

                if self.ping_ok(&client).await {
                    // Back online.
                    self.failure_counts.lock().await.clear();
                    let _ = self.offline_tx.send(false);
                    break;
                }

                delay = (delay * 2).min(BACKOFF_CAP);
            }
        }
    }

    /// Lightweight connectivity probe. Expects HTTP 204 from Google's generate_204
    /// endpoint. Checks status code to avoid false positives from captive portals
    /// that return 200 with a login page.
    async fn ping_ok(&self, client: &reqwest::Client) -> bool {
        match client
            .head("https://clients3.google.com/generate_204")
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_online() {
        let mgr = ConnectivityManager::new(3);
        assert!(!mgr.is_offline());
    }

    #[tokio::test]
    async fn record_failure_below_threshold_stays_online() {
        let mgr = ConnectivityManager::new(2);
        // One failure for one feed -- below threshold for triggering check.
        mgr.record_failure("feed-a").await;
        assert!(!mgr.is_offline());
    }

    #[tokio::test]
    async fn record_success_resets_counter() {
        let mgr = ConnectivityManager::new(1);
        mgr.record_failure("feed-a").await;
        mgr.record_success("feed-a").await;
        // After reset, a single failure should not re-trigger.
        mgr.record_failure("feed-a").await;
        assert!(!mgr.is_offline());
    }

    #[tokio::test]
    async fn zero_network_feeds_ignores_failures() {
        let mgr = ConnectivityManager::new(0);
        mgr.record_failure("feed-a").await;
        mgr.record_failure("feed-a").await;
        mgr.record_failure("feed-a").await;
        assert!(!mgr.is_offline());
    }

    #[test]
    fn trigger_retry_does_not_panic() {
        let mgr = ConnectivityManager::new(1);
        mgr.trigger_retry();
    }
}
