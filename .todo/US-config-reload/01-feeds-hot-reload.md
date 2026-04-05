---
status: done
---

# Feed hot-reload: alternatives analysis

## Goal

Evaluate implementation approaches for hot-reloading feed configuration. Determine how to detect config changes, reconcile old and new state, and manage feed task lifecycles — without restarting the app.

## Current Architecture (relevant details)

### Data structures

- `FeedConfig` — parsed TOML entry (name, type, interval, retain, type_specific fields, field overrides). Defined in `feed/config.rs:24-32`.
- `FeedRegistry` — holds `Vec<Arc<dyn Feed>>` + `Vec<FeedSnapshot>` for config errors. Wrapped in `Arc<FeedRegistry>` (immutable). Defined in `feed/mod.rs:227-321`.
- `FeedSnapshotCache` — `Arc<RwLock<Vec<FeedSnapshot>>>` of latest snapshots. The runtime upserts snapshots here. Defined in `feed/runtime.rs:17-47`.
- `BackgroundPoller` — owns the cache, a `watch::Sender<u64>` update counter, and optional `NotificationContext`. Spawns one tokio task per feed. Defined in `feed/runtime.rs:59-64`.

### Feed identity

Feeds are identified by `name` (unique within config). The feed name is the key used in `FeedSnapshotCache::upsert()`. Renaming a feed = removing old + adding new.

### Per-feed state (what would be lost on restart)

| State | Where | Impact of loss |
|-------|-------|----------------|
| Retained activities | In-memory in `FeedSnapshot` | Activities that disappeared < retain window are lost. Minor — they'll re-appear if still active, or naturally expire. |
| Notification baseline | `NotificationTracker` per-feed map | First poll after reload would be treated as "seed" (no notifications). Need startup suppression logic. |
| Poll timer phase | tokio `sleep()` in `poll_feed_loop` | Feed re-polls immediately or after a full interval. Minor. |

### Existing change detection

`ConfigChangeTracker` (`config.rs:34-83`) already fingerprints `feeds.toml` by mtime + file size on every refresh loop tick. Currently used only to inject a "restart required" warning.

### Existing file watching

The `notify` crate (v7) is already used for harness directory watching (`harness_watcher.rs`). The pattern is well-established: `RecommendedWatcher` → mpsc channel → debounce → act.

## Approach A: Full Registry Rebuild (Stop-the-World)

### Description

On config change, tear down everything and rebuild from scratch:

1. Watch `feeds.toml` with `notify` (or continue using `ConfigChangeTracker` fingerprinting).
2. On change: parse new config.
3. Cancel all existing feed poll tasks (via `CancellationToken` or `AbortHandle`).
4. Build a new `FeedRegistry` from the new config.
5. Replace `Arc<FeedRegistry>` (swap into `ArcSwap` or `RwLock<Arc<FeedRegistry>>`).
6. Clear `FeedSnapshotCache`.
7. Seed + start new poll loops.

### Pros

- **Simple.** No diffing logic. The reload path is essentially the same as startup.
- **Complete.** Guaranteed to pick up any config change — type changes, name changes, field changes, everything.
- **Easy to test.** One code path: load → build → start.
- **Low risk of stale state.** No zombie tasks, no orphaned feeds.

### Cons

- **Loses all in-memory state.** Retained activities, notification baselines, accumulated snapshots — all gone.
- **Brief monitoring gap.** Between teardown and re-seed, there's a window (potentially seconds) where no data is available. The UI would flash empty.
- **Unnecessary work.** If only one feed changed out of ten, all ten re-poll.
- **Notification spam risk.** First poll after rebuild would see all activities as "new" unless startup suppression is applied (which it already is — `seed_startup_best_effort` sets is_seed=true).
- **Harness watcher churn.** FSEvents watchers for harness feeds would be torn down and recreated.

### Complexity: Low

Most of the code already exists. The main additions are: making `Arc<FeedRegistry>` swappable, adding task cancellation, and wiring up the file watcher.

---

## Approach B: Differential Update (Diff & Patch)

### Description

On config change, diff old and new configs and apply targeted changes:

1. Watch `feeds.toml` with `notify`.
2. On change: parse new config.
3. Diff against current config by feed name:
   - **Removed feeds:** Cancel their poll tasks, remove from registry, remove snapshots.
   - **Added feeds:** Instantiate, add to registry, seed + start poll task.
   - **Modified feeds:** Cancel old task, re-instantiate with new config, start new task. Optionally preserve retained activities and notification baseline.
   - **Unchanged feeds:** Leave alone entirely.
4. Update the registry in-place (requires `RwLock` or similar).

### Feed change detection

A feed is "modified" when any config field differs from the current version. Simplest: serialize `FeedConfig` and compare (or derive `PartialEq`). Alternatively, compare a config fingerprint/hash.

### Pros

- **Minimal disruption.** Unchanged feeds keep polling, keep their state, keep their notification baselines.
- **No monitoring gap** for unchanged feeds.
- **Efficient.** Only re-polls feeds that actually changed.
- **Better UX.** User edits one feed, only that feed refreshes. Others remain stable.

### Cons

- **Diffing complexity.** Need to define equality for `FeedConfig`, handle edge cases:
  - Feed renamed (old name gone, new name appears) — is it a rename or remove+add? Must treat as remove+add since name is identity.
  - Feed type changed but name kept — must fully re-instantiate.
  - Only interval changed — could just update the sleep duration without re-instantiating the feed, but that's another layer of granularity.
- **Mutable registry.** `FeedRegistry` must become mutable (`RwLock<FeedRegistry>` or similar). Every reader needs to handle the lock.
- **Task lifecycle management.** Need per-feed `CancellationToken` or `AbortHandle` to stop individual tasks.
- **State transfer edge cases.** Preserving retained activities across a feed re-instantiation requires matching activities by ID, which may change if the feed type's identity scheme differs.
- **More test surface.** Need to test: add, remove, modify, rename, type-change, error-to-valid, valid-to-error transitions.

### Complexity: Medium-High

Significant refactoring of `FeedRegistry` (immutable → mutable), `BackgroundPoller` (static start → dynamic add/remove), and the runtime. But the logic is straightforward — it's bookkeeping, not algorithmic complexity.

---

## Approach C: Reactor Pattern (Message-Based)

### Description

Model the feed system as a reactor that receives commands:

1. A central `FeedManager` actor owns all feed state.
2. It receives messages: `AddFeed(config)`, `RemoveFeed(name)`, `UpdateFeed(name, config)`, `ReloadAll`.
3. The config watcher sends messages to the reactor.
4. The Settings GUI also sends messages directly (bypassing the file).
5. Each feed is an independent task with its own channel for control messages (pause, resume, update interval, stop).

### Pros

- **Clean architecture.** Message-passing is composable and testable.
- **Unified API.** Both file watcher and GUI use the same commands.
- **Fine-grained control.** Can pause/resume individual feeds, update intervals without restart.
- **Future-proof.** Easy to add new operations (reorder, disable, etc.).

### Cons

- **Major refactor.** The current architecture has no actor/message layer. This would be a ground-up rewrite of the feed runtime.
- **Over-engineered.** Cortado has ~6 feed types and a simple lifecycle. The reactor pattern adds indirection and complexity that doesn't pay off at this scale.
- **Debugging overhead.** Message-based systems are harder to trace than direct function calls.
- **Latency.** Message passing adds a hop vs direct mutation (negligible in practice, but architecturally unnecessary).

### Complexity: High

This is essentially a rewrite of the feed runtime. Not justified by the current requirements.

---

## Approach D: Hybrid — Full Rebuild with State Preservation

### Description

A pragmatic middle ground: rebuild the registry from scratch (like Approach A), but preserve transferable state:

1. Watch `feeds.toml` with `notify`.
2. On change: parse new config.
3. If parse fails: log error, emit UI warning with parse error details, keep running with current config.
4. If parse succeeds:
   a. Snapshot current state: retained activities per feed name, notification baselines per feed name.
   b. Cancel all poll tasks.
   c. Build new registry.
   d. Restore retained activities and notification baselines for feeds that still exist with the same name and type.
   e. Seed new/changed feeds, start all poll loops.
   f. Emit `feeds-updated` to refresh UI.

### Pros

- **Simple as Approach A** — still a full rebuild, same code path as startup.
- **Preserves important state** — retained activities and notification baselines survive for unchanged feeds.
- **Clean error handling** — bad config = keep running, show error. No partial state.
- **Low risk** — full rebuild means no zombie tasks or orphaned state.

### Cons

- **Still re-polls unchanged feeds.** All feeds re-seed, even if they didn't change. This is unnecessary network traffic and API calls.
- **Brief UI disruption.** Snapshots are cleared during rebuild. Could mitigate by keeping old snapshots until new ones arrive.
- **State preservation is imperfect.** If a feed's type changes but name stays, retained activities from the old type might not make sense for the new type. Need to match on name+type.

### Complexity: Low-Medium

Similar to Approach A with a small state-snapshot/restore layer on top.

---

## Approach E: Differential with Simplified Scope

### Description

Like Approach B, but with a pragmatic simplification: don't try to preserve state for modified feeds. Only distinguish three cases:

1. **Unchanged** (config identical): leave alone entirely.
2. **Removed** (name gone): cancel task, remove from registry, remove snapshot.
3. **Added or Modified** (new name, or name exists but config differs): cancel old task (if exists), instantiate new feed, seed + start.

No state transfer for modified feeds. No rename detection. No partial updates (e.g., just interval).

### Pros

- **Clean three-way split.** Simple to reason about.
- **Preserves state for unchanged feeds.** The common case (editing one feed) leaves all others untouched.
- **No UI disruption for unchanged feeds.**
- **Efficient.** Doesn't re-poll unchanged feeds.
- **Manageable complexity.** Diffing by name + config equality is straightforward.

### Cons

- **Mutable registry** still required (same as Approach B).
- **Per-feed task cancellation** still required.
- **Modified feeds lose state** (retained activities, notification baseline). Acceptable — the user just changed the config, so a fresh start for that feed makes sense.
- **Config equality** needs a reliable comparison. Can derive `PartialEq` on `FeedConfig` or hash it.

### Complexity: Medium

Less than full Approach B (no state transfer, no rename detection), but requires the same structural changes to `FeedRegistry` and `BackgroundPoller`.

---

## Approach F: Detect Changes + Self-Restart

### Description

Instead of hot-reloading feeds at runtime, detect config file changes and let the user trigger a full app restart:

1. Watch both `feeds.toml` and `settings.toml` with `notify` (or extend the existing `ConfigChangeTracker` to cover both files).
2. On change: surface a clickable "Restart to apply changes" activity in the existing synthetic "Configuration" feed (already exists — `ui_snapshot.rs:7-68`).
3. In the tray: clicking the activity calls `app_handle.restart()`.
4. In the panel: pressing Enter on the activity calls `app_handle.restart()`.
5. The app restarts, picks up the new config through the normal startup path.

### How restart works

`app_handle.restart()` is already used in the updater (`command.rs:291`). It's a Tauri built-in via `tauri-plugin-process` — it exits the current process and relaunches the same binary. On macOS this is effectively instantaneous for a lightweight app like Cortado.

### Pros

- **Dramatically simpler.** No mutable registry, no per-feed task cancellation, no diffing, no state transfer, no reload handler. Zero changes to `FeedRegistry`, `BackgroundPoller`, or the feed runtime.
- **Covers both feeds AND settings.** Since the entire app restarts, settings changes (including `show_menubar`) are also picked up. No need for a separate settings hot-reload analysis.
- **Zero risk of stale state.** A fresh startup is guaranteed clean — no zombie tasks, no orphaned watchers, no half-applied config.
- **Already proven.** `app_handle.restart()` is battle-tested in the updater flow.
- **Minimal code changes.** Extend the existing `ConfigChangeTracker` to watch `settings.toml`, make the synthetic "Configuration" activity clickable, wire click to `restart()`.
- **User stays in control.** The user decides when to restart, so an accidental half-finished edit doesn't immediately disrupt monitoring.

### Cons

- **Brief interruption.** The app disappears and reappears. On macOS, restart takes ~1-2s. During that window, no monitoring occurs.
- **All state is lost.** Retained activities, notification baselines, poll timer phases — all reset. First poll after restart suppresses notifications (existing startup behavior), so no notification spam.
- **Not fully "hot".** The user must take an action (click/Enter). This is arguably a pro (no surprise disruption), but it's not seamless.
- **Restart required for any change.** Even a trivial interval change requires full restart. With hot-reload (Approaches B/E), only the changed feed would restart.

### Complexity: Very Low

The simplest approach by far. The main pieces are:
1. Extend `ConfigChangeTracker` to also fingerprint `settings.toml` (or add a `notify` watcher for both files).
2. Make the synthetic "Configuration" activity actionable (clickable → restart).
3. Add a `restart_app` Tauri command (one line: `app_handle.restart()`).

### Why this may be the right choice

The fundamental question is: **how often do users change feed config?** If the answer is "rarely, and usually a batch of changes at once" (which matches the current UX — users edit config in settings, then want it applied), then a single restart is a better UX than watching individual feeds flicker through reload cycles. The user edits, clicks restart, and gets a clean slate.

Hot-reload (Approaches B/E) is more elegant but introduces significant complexity for a scenario that happens infrequently. The restart approach can always be upgraded to hot-reload later if the UX proves insufficient.

---

## Comparison Matrix

| Criterion | A: Full Rebuild | B: Full Diff | C: Reactor | D: Rebuild + State | E: Simple Diff | F: Self-Restart |
|-----------|:-:|:-:|:-:|:-:|:-:|:-:|
| Implementation complexity | Low | High | Very High | Low-Med | Medium | **Very Low** |
| Unchanged feed disruption | Yes (all repoll) | None | None | Yes (all repoll) | None | Yes (full restart) |
| State preservation | None | Full | Full | Partial | Unchanged only | None |
| Error handling simplicity | Simple | Complex | Complex | Simple | Moderate | **Trivial** |
| Monitoring gap | Yes | No | No | Yes (brief) | No | Yes (~1-2s) |
| Unnecessary API calls | All feeds | None | None | All feeds | None | All feeds |
| Future extensibility | Low | High | Very High | Low | Medium | Low |
| Test surface | Small | Large | Very Large | Small-Med | Medium | **Tiny** |
| Covers settings too | No | No | No | No | No | **Yes** |
| User-initiated | No | No | No | No | No | **Yes** |
| New deps / refactoring | Minor | Significant | Major | Minor | Moderate | **None** |

## Recommendation

**Approach F (Detect Changes + Self-Restart)** is the recommended approach.

The core argument: hot-reload (Approaches A-E) solves a problem that happens infrequently (config changes) with significant architectural complexity. Approach F delivers the same end-user value — "I edited config, now it works" — with a fraction of the implementation cost. The ~1-2s restart is imperceptible for a menubar utility app.

Key advantages over the previous recommendation (Approach E):
- **No registry refactoring.** `FeedRegistry` stays immutable `Arc<FeedRegistry>`.
- **No per-feed task management.** No `AbortHandle`, no `start_feed`/`stop_feed`.
- **No diff logic.** No `PartialEq` derivation, no three-way split.
- **Covers settings for free.** Since the whole app restarts, `show_menubar`, external `settings.toml` edits, and all other settings gaps are automatically resolved.
- **Already proven.** The same `app_handle.restart()` call is used by the updater.

**Fallback:** If users find the full restart disruptive (unlikely for a menubar app), Approach E remains a viable upgrade path. The file watching infrastructure built for Approach F would be reusable.

## Notes

- The `notify` crate is already a dependency — no new deps needed.
- `app_handle.restart()` is already used in `install_update` (`command.rs:291`) — proven in production.
- The existing synthetic "Configuration" feed (`ui_snapshot.rs`) already surfaces change detection — just needs to become actionable.
- Startup suppression logic already exists in `seed_startup_best_effort()` — no notification spam after restart.
- The Settings GUI's "restart required" message (`SettingsApp.tsx`) becomes accurate and actionable rather than a dead-end warning.
