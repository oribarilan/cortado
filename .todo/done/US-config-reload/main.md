# US-config-reload: Config Change Detection + Self-Restart

## Theme

Cortado currently requires a manual full app restart to pick up changes to `feeds.toml`. The spec acknowledges this limitation explicitly ("Hot-reload may be added later" -- `specs/main.md:118`). This story investigates approaches for applying config changes -- ranging from true hot-reload to simpler restart-based patterns -- and recommends the simplest viable solution.

## Current State

### Feeds

- `feeds.toml` is loaded once at startup (`main.rs:51`).
- Config is parsed into `Vec<FeedConfig>`, instantiated into an **immutable** `Arc<FeedRegistry>` (`main.rs:66`).
- Each feed gets a dedicated tokio task in `poll_feed_loop()` that runs forever (`runtime.rs:177`).
- Harness feeds additionally get FSEvents-based file watchers (`harness_watcher.rs`).
- A `ConfigChangeTracker` fingerprints `feeds.toml` (mtime + file size) and injects a synthetic "Configuration" feed warning when changes are detected, instructing the user to restart (`ui_snapshot.rs:7-68`).
- The `notify` crate (v7) is already a dependency, used by harness watchers.

### Settings

- `settings.toml` is loaded once at startup into `AppSettingsState` (`Arc<RwLock<AppSettings>>`).
- The GUI's `save_settings` command writes to disk, updates the in-memory RwLock, and emits `appearance-changed` for theme/text-size.
- **Already live:** All notification settings (read from RwLock each poll), theme, text size, global hotkey, focus settings, `panel.show_priority_section` (pull-on-show).
- **Not live:** `show_menubar` (tray creation is startup-only), `hide_empty_feeds` (tray reads once).
- **No file watcher** on settings.toml -- external edits (e.g., via "Open in editor") are not detected until restart.

## Recommended Approach: Self-Restart (Approach F)

After analyzing six approaches for feed config changes (five hot-reload variants + self-restart), the recommendation is **Approach F: detect config changes and let the user trigger a self-restart**.

### Why not hot-reload?

Hot-reload (Approaches A-E in task 01) solves a problem that happens infrequently -- config changes -- with significant architectural complexity: mutable registries, per-feed task cancellation, diff logic, state transfer, and new error surfaces. The simplest hot-reload variant (Approach E) still requires refactoring `FeedRegistry`, `BackgroundPoller`, and the feed runtime.

### Why self-restart?

- **Dramatically simpler.** Zero changes to feed runtime architecture. The main work is extending change detection and making the existing "Configuration" warning actionable.
- **Covers both feeds and settings.** Since the whole app restarts, all config changes take effect -- including `show_menubar` and external `settings.toml` edits.
- **Already proven.** `app_handle.restart()` is used by the updater (`command.rs:291`).
- **User-initiated.** The user decides when to restart, so half-finished edits don't cause surprises.
- **Upgradeable.** If the restart UX proves insufficient, hot-reload can be added later. The file watching infrastructure built here is reusable.

### Trade-offs accepted

- ~1-2s restart window with no monitoring (acceptable for a menubar utility).
- All in-memory state is lost (retained activities, notification baselines). Existing startup suppression prevents notification spam.
- User must take an action (click/Enter) to apply changes (arguably a feature, not a bug).

## Design Constraints

- **Performance:** Watching config files via FSEvents is negligible. The restart itself is ~1-2s.
- **Error tolerance:** A broken config edit should not trigger a restart prompt -- only valid changes that differ from the running config.
- **Atomicity:** External editors may produce multiple rapid FS events -- debouncing is essential.
- **False positive suppression:** GUI saves should not trigger a "restart needed" prompt since the in-memory state already reflects the change.
- **notify crate:** Already a dependency. Reuse the same watcher pattern from `harness_watcher.rs`.

## Task Sequencing

```
01-feeds-hot-reload ──> 03-implementation-plan
02-settings-hot-reload ─┘
```

Tasks 01 and 02 are analysis tasks (alternatives, pros/cons). Task 03 synthesizes findings into a concrete implementation plan for Approach F.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-feeds-hot-reload.md` | Analyze feed config change approaches: six alternatives (A-F), pros/cons, recommends F |
| 02 | `02-settings-hot-reload.md` | Analyze settings hot-reload: current gaps, confirms settings detection is free with Approach F |
| 03 | `03-implementation-plan.md` | Concrete implementation plan for self-restart |
| 04 | `04-fix-hide-empty-feeds-tray.md` | Fix `hide_empty_feeds` not updating in tray (standalone bugfix) |
