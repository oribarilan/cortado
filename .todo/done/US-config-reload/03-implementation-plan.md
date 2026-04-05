---
status: done
---

# Implementation plan

## Goal

Concrete implementation steps for config change detection + self-restart, based on the analysis in tasks 01 and 02.

This plan assumes **Approach F (Self-Restart)** from task 01 and **settings detection included** (per updated task 02). The approach: watch both `feeds.toml` and `settings.toml` for changes, surface a "restart to apply" activity in the UI, and let the user trigger `app_handle.restart()`.

## Prerequisites

- Task 01 (feeds analysis) reviewed and Approach F confirmed. ✓
- Task 02 (settings analysis) reviewed and inclusion confirmed. ✓

## Decisions (from review)

- **FeedAction enum**: Keep. Add `FeedAction` enum to `FeedSnapshot` (Option A from Step 2). Typed and explicit.
- **Change detection**: Use `notify` crate (already a dependency) with 500ms debounce, not poll-based `ConfigChangeTracker`. Sub-second detection vs 30-120s with polling.
- **False positive suppression**: Content comparison is the primary guard (parse on-disk config, compare to running state). No baseline reset needed. For feeds: on-disk ≠ startup config → prompt (correct — feeds aren't reloaded). For settings: on-disk = in-memory RwLock → no prompt (correct — GUI saves update both).
- **Config validation before prompting**: On file change, parse the new config. If invalid TOML → show `AttentionNegative` status (config error, no restart option). If valid and differs from running config → show `AttentionPositive` status (restart prompt). If valid but identical → no prompt.
- **Settings GUI toast**: Keep the toast in `SettingsApp.tsx` but update text. Feed saves: "Saved (Restart Required)". Settings saves: "Saved (Changes Applied)". The Configuration feed provides the actionable restart mechanism; the toast gives immediate save feedback.
- **hide_empty_feeds tray bug**: Separate task 04 in this story.
- **Naming**: Synthetic feed renamed to "Cortado Config" (prefix groups app-internal feeds: "Cortado Updates", "Cortado Config"). Config change activity uses real `Activity` with `FeedAction::RestartApp` on the activity (not the feed).

## Architecture Overview

```
feeds.toml ─┐
             ├─> notify watcher (500ms debounce)
settings.toml┘         │
                       ├─> parse & compare to running config
                       │         │
                       │    valid & differs ──> "Cortado Configuration" feed
                       │                        with actionable restart activity
                       │    invalid ───────────> config error warning (no restart)
                       │    identical ─────────> no prompt
                       │
                       ├─> Tray: click activity → restart_app command
                       └─> Panel: Enter on activity → restart_app command
```

The flow replaces the existing poll-based `ConfigChangeTracker` with a `notify` file watcher for sub-second detection. On change, it parses and compares to the running config before surfacing any prompt. The main additions are:
1. `notify` watcher on both config files with debounce + content comparison.
2. Synthetic "Cortado Configuration" feed with actionable `FeedAction::RestartApp`.
3. A `restart_app` Tauri command.

## Implementation Steps

### Step 1: File watcher on feeds.toml and settings.toml

**Files:** new `feed/config_watcher.rs` (or extend `feed/config.rs`)

Replace the poll-based `ConfigChangeTracker` (mtime + file size on each refresh tick) with a `notify`-based file watcher. The `notify` crate is already a dependency — reuse the `RecommendedWatcher` pattern from `harness_watcher.rs`.

Changes:
- Create a `ConfigWatcher` that watches both `feeds.toml` and `settings.toml` using `notify::RecommendedWatcher`.
- Debounce FS events by 500ms (handles editor save patterns, partial writes, vim's delete-then-create).
- On debounced event: parse the changed file, compare parsed result to the running config (see Step 5). If different and valid → surface restart prompt. If invalid → surface error. If identical → ignore.
- Expose `which_changed()` → `{feeds: bool, settings: bool}` so the UI message can be specific ("Feed config changed" vs "Settings changed").
- Use `app_settings::settings_path()` for the settings file path (already exists in `app_settings.rs:190`).
- Remove or deprecate the poll-based `ConfigChangeTracker`. Its role is fully replaced by the watcher + content comparison.

Complexity: Low-Medium. The watcher pattern is established in `harness_watcher.rs`. The new piece is content comparison on change (Step 5).

### Step 2: Make the synthetic Cortado Configuration feed actionable

**Files:** `feed/mod.rs`, `ui_snapshot.rs`

Currently the "Configuration" feed is injected as a `FeedSnapshot` with `error: Some("Config file changed...")` and no activities. This step renames it to "Cortado Configuration" and makes it actionable.

**Option A: Add an `action` field to `FeedSnapshot`**

Add an optional `action: Option<FeedAction>` enum field to `FeedSnapshot`:
```rust
#[derive(Debug, Clone, Serialize)]
pub enum FeedAction {
    RestartApp,
}
```
The frontend checks for `action` and renders the feed as clickable. On click/Enter, it invokes the `restart_app` command.

Pros: Clean, typed, extensible (could add other actions later).
Cons: Adds a field to every `FeedSnapshot` (but it's `Option`, so zero cost for normal feeds).

**Option B: Use a special activity instead of an error**

Instead of setting `error: Some(...)`, add an `Activity` to the feed with a well-known field (e.g., `action: "restart"`) that the frontend recognizes:
```rust
FeedSnapshot {
    name: "Cortado Configuration",
    activities: vec![Activity {
        title: "Config changed. Click to restart and apply.",
        fields: vec![Field { name: "action", value: "restart", ... }],
        ...
    }],
    error: None,
    ...
}
```

Pros: Works within existing data model, no schema change.
Cons: Using a magic field name for control flow is fragile.

**Recommendation:** Option A. A typed `FeedAction` enum is cleaner and less fragile.

Also in this step: rename the synthetic feed from "Configuration" to "Cortado Configuration" (consistent with "Cortado Updates" prefix for app-internal feeds). Update the feed name in `ui_snapshot.rs` where the snapshot is constructed.

### Step 3: Add restart_app Tauri command

**Files:** `command.rs`, `main.rs` (register command)

A one-line command:
```rust
#[tauri::command]
pub fn restart_app(app_handle: AppHandle) {
    app_handle.restart();
}
```

Register it in `main.rs`'s `invoke_handler`. This is the same call used in `install_update` (`command.rs:291`).

Complexity: Trivial.

### Step 4: Frontend — make Configuration feed clickable

**Files:** Tray (`App.tsx`), Panel (`main-screen/` components)

When the "Configuration" feed appears with `action: RestartApp`:

**Tray:**
- Render the activity with a visual affordance indicating it's clickable (e.g., underline, hover highlight, or a small restart icon).
- On click: `invoke("restart_app")`.

**Panel:**
- Render the activity in the detail pane with a "Restart" button or make Enter invoke restart.
- On Enter: `invoke("restart_app")`.

The message text should be clear and actionable:
- "Config changed. Restart to apply." (with a visual restart affordance)
- Or: "feeds.toml changed. Restart to apply." / "Settings changed. Restart to apply." (if `which_changed()` is implemented)

**TypeScript types:** Add `FeedAction` to `shared/types.ts` (e.g., `action?: "RestartApp"`) and update frontend components to check for it.

### Step 5: Suppress false positives via content comparison

**Files:** `feed/config_watcher.rs` (or wherever the watcher callback lives), `app_settings.rs`

When the watcher fires (from any source — external edit or GUI save), the response is always the same: parse the changed file and compare to the running config. This single mechanism handles all false positive scenarios without needing baseline resets or suppress flags.

**For feeds.toml:**
- Parse the new file into `Vec<FeedConfig>`.
- Compare to the feed configs loaded at startup (the "running" config).
- If different → prompt restart. This is correct: feeds are not hot-reloaded, so the running feeds still use the startup config.
- If identical → ignore. Handles no-op saves and editors that rewrite without changing content.
- If parse fails → show config error warning (no restart prompt).

**For settings.toml:**
- Parse the new file into `AppSettings`.
- Compare to the current in-memory `AppSettings` (from the `Arc<RwLock>`).
- If different → prompt restart. This covers external edits where the in-memory state doesn't reflect the file.
- If identical → ignore. This correctly handles GUI saves: the GUI updates both the file and the RwLock, so they match → no false prompt.

**Known limitation:** `show_menubar` — when changed via GUI, the in-memory state IS updated but the tray icon isn't recreated. The comparison shows file = in-memory → no restart prompt, even though restart is actually needed. Accepted: this setting is toggled very rarely, and the user can manually restart.

**Why not baseline reset?** With `notify` delivering events in milliseconds, a baseline-reset approach has a real race window (watcher fires before reset). Content comparison eliminates this entirely — the comparison result is always correct regardless of timing.

Complexity: Low. Requires `PartialEq` on `FeedConfig` and `AppSettings` (or serialize-and-compare). The startup feed config must be retained for comparison (store `Vec<FeedConfig>` alongside `FeedRegistry`).

### Step 6: Update spec, rename synthetic feed, and update toast text

**Files:** `specs/main.md`, `SettingsApp.tsx`, `ui_snapshot.rs`

- Update `specs/main.md` "Config loading" section (line 117-120) to describe the new behavior: "Config changes are detected automatically via file watching. A 'restart to apply' prompt appears in the Cortado Configuration feed when feeds.toml or settings.toml changes on disk."
- Rename synthetic feed from "Configuration" to "Cortado Configuration" in `ui_snapshot.rs` (if not already done in Step 2).
- Update the Settings GUI toast text in `SettingsApp.tsx`:
  - **Feed saves:** "Saved (Restart Required)" — feeds are not hot-reloaded, so the user must restart.
  - **Settings saves:** "Saved (Changes Applied)" — most settings take effect immediately via the RwLock / event system.
- Remove any stale "Restart Cortado to apply changes" messaging that implies the *only* way to restart is manually quitting and relaunching.

## Edge Cases

| Edge case | Handling |
|-----------|---------|
| Parse error in changed config | Don't show restart prompt — the restart would fail to load bad config. Instead, show a "Config error" warning with the parse error details. (This is the current behavior for startup parse errors.) |
| Rapid successive edits | 500ms debounce in `notify` watcher absorbs rapid FS events from editors. |
| Config file deleted | Ignore — keep running with current config. Don't prompt restart. |
| Config file recreated (vim pattern) | `notify` with debounce handles the delete-then-create pattern. The debounce window captures both events and only the final state is compared. |
| GUI save triggers watcher | Content comparison handles this: for feeds, on-disk ≠ startup config → prompt restart (correct). For settings, on-disk = in-memory → no prompt (correct). No suppress flag or baseline reset needed. |
| Both files changed simultaneously | Show one "Config changed" prompt (not two). The watcher callback checks both files and produces a single synthetic feed snapshot. |
| `show_menubar` changed via GUI | In-memory is updated but tray isn't recreated. Content comparison shows match → no restart prompt. Accepted limitation — very rare toggle. |

## Testing Plan

- **Unit tests:** Content comparison logic — feed config differs from startup → changed. Settings config matches in-memory → not changed. Invalid TOML → error, not restart.
- **Unit tests:** `inject_config_warning_snapshot` — test actionable activity with `FeedAction::RestartApp` is included when changed, absent when not. Feed name is "Cortado Configuration".
- **Manual test:** Edit `feeds.toml` in vim while app is running → verify "Cortado Configuration" feed appears with restart activity within ~1s → click → app restarts with new config.
- **Manual test:** Edit `settings.toml` externally → same flow.
- **Manual test:** Save feeds in Settings GUI → verify toast shows "Saved (Restart Required)" → verify "Cortado Configuration" feed appears with restart prompt.
- **Manual test:** Save settings in Settings GUI → verify toast shows "Saved (Changes Applied)" → verify no restart prompt appears (content comparison matches in-memory).
- **Manual test:** Break config (invalid TOML) → verify error shown, no restart prompt.

## Open Questions

- **Message specificity:** Should the prompt say which file changed ("Feed config changed" vs "Settings changed"), or just "Config changed"? The former is more informative; the latter is simpler.
- **Auto-restart option:** Should there be a setting to auto-restart on config change (skip the user prompt)? Probably not — it's surprising behavior and could disrupt monitoring. But worth noting as a future option.
- **Upgrade path to hot-reload:** If the restart UX proves insufficient, the file watching infrastructure and actionable feed pattern built here can be reused. The `ConfigChangeTracker` would trigger a `reload_feeds()` function instead of a restart prompt.
