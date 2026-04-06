---
status: done
---

# Settings hot-reload: analysis and complexity assessment

## Goal

Assess whether hot-reloading `settings.toml` is worth pursuing alongside feeds hot-reload, or whether the current live-settings mechanism is sufficient. Identify gaps and evaluate the cost of closing them.

## Current State Audit

### What's already live (no restart needed)

| Setting | Mechanism | Latency |
|---------|-----------|---------|
| `notifications.enabled` | Read from `Arc<RwLock>` each poll cycle | Next poll (~30-120s) |
| `notifications.mode` | Same | Next poll |
| `notifications.delivery` | Same | Next poll |
| `notifications.notify_new_activities` | Same | Next poll |
| `notifications.notify_removed_activities` | Same | Next poll |
| `general.theme` | `appearance-changed` event → all windows | Instant |
| `general.text_size` | `appearance-changed` event → all windows | Instant |
| `general.global_hotkey` | `set_global_hotkey` re-registers at runtime | Instant |
| `panel.show_priority_section` | Pull-on-show (re-fetched when panel opens) | Next panel open |
| `panel.hide_empty_feeds` | Pull-on-show in MainScreen; **bug: tray reads once** | See gap below |
| `focus.tmux_enabled` | Read from state on each `focus_session` call | Instant |
| `focus.accessibility_enabled` | Same | Instant |

### What requires restart

| Setting | Why | Impact |
|---------|-----|--------|
| `general.show_menubar` | Tray icon creation is startup-only (`main.rs:142`) | Low -- one-time setup. Users rarely toggle this. |

### Gaps (not restart-required, but not fully live)

| Gap | Description | Severity |
|-----|-------------|----------|
| `hide_empty_feeds` in tray | `App.tsx:159` reads it once at bootstrap, never re-fetches | Low -- standalone bugfix (add pull-on-show or listen for settings-changed event) |
| External `settings.toml` edits | "Open in editor" button exists (`app_settings.rs:289`), but edits are not detected | Low -- rare workflow, and the GUI is the primary editing path |

## Approach A: File Watcher on settings.toml

### Description

Add a `notify` watcher on `settings.toml`. On change:
1. Re-parse the file.
2. Compare to current in-memory state.
3. Update `AppSettingsState` RwLock.
4. Emit `appearance-changed` (and/or a broader `settings-changed`) event.

### Pros

- Handles external edits (text editor, scripting, dotfiles sync).
- Consistent with feeds hot-reload if implemented.
- Single source of truth -- file is always authoritative.

### Cons

- **Race condition with GUI saves.** The GUI writes to the same file. The watcher would fire on GUI-initiated writes, triggering a redundant reload. Need to either: (a) suppress watcher events during GUI saves (fragile), or (b) make the reload idempotent (compare parsed result to current state -- skip if identical).
- **Merge conflicts.** If the GUI has unsaved local state and an external edit arrives, which wins? The GUI currently holds settings in local `useState` and writes the full object on every change. An external edit would silently overwrite any in-flight GUI state.
- **Partial writes.** Not all editors use atomic write (write-to-temp + rename). Vim does; VS Code doesn't always. Could read a truncated file.
- **Complexity for low impact.** Very few users edit settings.toml by hand.

### Complexity: Medium

The watcher itself is simple (reuse harness_watcher pattern). The complications are in handling the race condition with GUI saves and partial writes.

## Approach B: Broader Event Propagation (Extend Current Pattern)

### Description

Don't add a file watcher. Instead, extend the existing `appearance-changed` event pattern to cover all settings:

1. Rename or extend to `settings-changed` with the full `AppSettings` payload.
2. All windows listen and update their local state.
3. The tray re-reads `hide_empty_feeds` on this event (or on panel-show).

### Pros

- No file watcher complexity.
- No race conditions -- the GUI is the only writer.
- Simple: one event, all consumers update.
- Already partially implemented (just extend `appearance-changed`).

### Cons

- Doesn't handle external edits (but this is rare and low-impact).
- Doesn't change the backend flow -- just improves frontend consistency.

### Complexity: Low

Extend the existing event, add one listener in `App.tsx` for tray settings. Minor change.

## Approach C: Do Nothing (Fix Bugs Only)

### Description

Don't pursue settings hot-reload as a feature. Instead:

1. Fix the `hide_empty_feeds` bug in tray (add pull-on-show, matching main screen's pattern for `show_priority_section`).
2. Accept that `show_menubar` requires restart (document it).
3. Accept that external `settings.toml` edits require restart (document it).

### Pros

- Zero complexity.
- No risk of regressions.
- The existing system already works well for 95% of use cases.

### Cons

- External edits still require restart.
- `show_menubar` still requires restart.

### Complexity: Trivial

One bugfix (`hide_empty_feeds`), zero architecture changes.

## Comparison

| Criterion | A: File Watcher | B: Extend Events | C: Do Nothing |
|-----------|:-:|:-:|:-:|
| Handles external edits | Yes | No | No |
| Implementation complexity | Medium | Low | Trivial |
| Race condition risk | Yes | None | None |
| GUI save conflicts | Yes | None | None |
| Fixes `hide_empty_feeds` | Indirectly | Yes | Yes (standalone fix) |
| Makes `show_menubar` live | Possible (hard) | No | No |

## Updated Recommendation (given Approach F from task 01)

**If Approach F (self-restart) is adopted for feeds, settings change detection becomes trivially worthwhile.**

### Rationale

Approach F already proposes watching `feeds.toml` for changes and surfacing a "Restart to apply changes" activity. Extending this to also watch `settings.toml` is near-zero marginal effort -- it's one additional file path in the same watcher. And since the response to a change is "restart the whole app" (not hot-reload specific settings), all the complexity concerns from Approach A evaporate:

- **No race conditions with GUI saves.** The watcher doesn't try to reload settings into memory -- it just sets a "changed" flag. The GUI still writes and updates the RwLock as before. The watcher's flag only matters for *external* edits (text editor, dotfiles sync).
- **No merge conflicts.** No partial reload, no state reconciliation. The restart picks up whatever is on disk.
- **No partial writes.** The debounce handles editor save patterns. The actual file is only read at startup (after restart).
- **`show_menubar` works.** Since the whole app restarts, the tray icon creation runs again with the new value. This is the one setting that genuinely required restart -- and now it gets it for free.

### What this looks like

1. `ConfigChangeTracker` (or a `notify` watcher) watches both `feeds.toml` and `settings.toml`.
2. When either file changes, the synthetic "Configuration" activity appears: "Config changed. Click to restart and apply."
3. The user clicks (tray) or presses Enter (panel) to restart.
4. All settings -- including `show_menubar`, external edits, everything -- take effect.

### What remains out of scope

- **The `hide_empty_feeds` tray bug** is still a standalone bugfix, not related to restart-based change detection. It should be fixed separately (add pull-on-show in `App.tsx`).
- **GUI save → immediate effect** is already working for most settings. The restart path is only for external edits and the `show_menubar` edge case.

### Previous recommendation (preserved for reference)

The original recommendation was **Approach C (Do Nothing)**, which remains correct if the feeds approach were hot-reload (Approaches A-E). With those approaches, settings file watching adds complexity for low payoff. But with Approach F, the marginal cost is essentially zero.

## Notes

- `show_menubar` is the one setting that genuinely needs restart. With Approach F, it gets restart for free -- no special handling needed.
- The "whole-object save" pattern in the frontend (`SettingsApp.tsx` assembles full `AppSettings` on every save) is a design choice that makes partial updates impossible via the current Tauri command. This is fine -- Approach F doesn't require partial updates.
- The GUI save → watcher fire → "restart needed" false positive is handled by comparing the in-memory state to the on-disk state. If the GUI just wrote what's already in memory, the fingerprint matches and no restart prompt appears. Alternatively, suppress the watcher for a short window after GUI saves.
