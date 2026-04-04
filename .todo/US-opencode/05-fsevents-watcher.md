---
status: done
---

# Add FSEvents-based file watching for harness feeds

## Goal

Replace timer-based polling with filesystem event watching for harness feeds. Instead of sleeping for 30s between polls, watch the session state directory for changes and re-poll immediately when files are created, modified, or deleted.

This benefits all harness feeds (currently Copilot, soon OpenCode) and reduces status update latency from 30s to ~100ms.

## Acceptance criteria

- [ ] `notify` crate added to `Cargo.toml` (requires user approval -- well-maintained, 10M+ downloads, de facto standard)
- [ ] Harness feeds use `notify::RecommendedWatcher` to watch their session state directory
- [ ] On file change event, the feed re-polls immediately
- [ ] A fallback timer (60s) ensures re-poll even if watcher misses events
- [ ] Non-harness feeds (github-pr, ado-pr, http-health, etc.) continue using timer-based polling unchanged
- [ ] Watcher handles missing directories gracefully (directory may not exist at startup)
- [ ] Watcher handles directory creation/deletion (re-watch if directory appears)
- [ ] `just check` passes

## Notes

### Architecture

The `Feed` trait or `HarnessFeed` could expose a method like `fn watch_paths(&self) -> Option<Vec<PathBuf>>` that returns directories to watch. The `poll_feed_loop` (or equivalent) would check this:
- If watch paths are returned: set up a `notify::RecommendedWatcher` + fallback timer, `tokio::select!` between them
- If `None`: use the existing interval-only loop

### Implementation sketch

The `notify` crate's `RecommendedWatcher` takes a callback-based event handler. Bridge to tokio with a `tokio::sync::mpsc` channel:

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};

// In the feed poll loop for harness feeds:
let (tx, mut rx) = tokio::sync::mpsc::channel(16);
let mut watcher = RecommendedWatcher::new(
    move |_event| { let _ = tx.blocking_send(()); },
    Config::default(),
)?;
watcher.watch(&harness_dir, RecursiveMode::NonRecursive)?;

loop {
    tokio::select! {
        _ = tokio::time::sleep(fallback_interval) => { /* fallback re-poll */ }
        _ = rx.recv() => { /* file changed, re-poll */ }
    }
    feed.poll().await;
}
```

**Debouncing:** FSEvents can fire multiple events for a single atomic write (the temp file creation, the rename). Add a short debounce window (~200ms) after receiving the first event before polling -- drain any queued events during the window. This prevents redundant back-to-back polls.

### Scope

This task is independent of the OpenCode-specific work. It improves the infrastructure for all harness feeds. It can be done in parallel with tasks 02-04.

### Why `notify`

The `notify` crate uses FSEvents on macOS (kqueue as fallback). It's the de facto standard for file watching in Rust, with 10M+ downloads and active maintenance. Per AGENTS.md, new dependencies require user approval.

### Copilot benefit

Once this is in place, the Copilot provider also benefits from near-instant detection of session state changes (currently on a 30s poll cycle).
