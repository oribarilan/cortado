---
status: pending
---

# Feed Sort Order

## Goal

Let users reorder feeds in the settings feed list. The chosen order persists to `feeds.toml` and controls display order in both the tray and panel.

## Context

Feed order is currently determined by `[[feed]]` array order in `feeds.toml`. There is no `order`/`position` field and no reorder UI. The order chain is: TOML file order → feed registry → snapshot cache → tray/panel. Because `save_feeds_config` already serializes the `Vec<FeedConfigDto>` in array order, reordering the array before saving is enough to persist the new order — no backend schema changes needed.

## Approach

Add drag-and-drop reordering (or up/down move buttons) to the feed list in settings (`src/settings/SettingsApp.tsx`, the `{/* ===== FEED LIST ===== */}` section around line 1710). When the user reorders feeds, update the `feeds` state array order and save it. The backend already preserves array order through the full chain.

## Acceptance criteria

- [ ] Feed list in settings supports reordering (drag-and-drop or move up/down buttons)
- [ ] Reordered feeds persist to `feeds.toml` after save
- [ ] Tray displays feeds in the user-chosen order
- [ ] Panel displays feeds in the user-chosen order
- [ ] Adding a new feed appends it to the end of the list
- [ ] Deleting a feed preserves the relative order of remaining feeds

## Related files

- `src/settings/SettingsApp.tsx` — feed list UI, `saveFeed()`, `loadFeeds()`
- `src-tauri/src/settings_config.rs` — `FeedConfigDto`, `save_feeds_config`, `dto_to_toml_document()`
- `src-tauri/src/feed/config.rs` — `FeedConfig`, `parse_feeds_config_toml()`
- `src-tauri/src/feed/mod.rs` — `build_feed_registry_from_configs()`
- `src-tauri/src/feed/runtime.rs` — `FeedSnapshotCache::upsert()`
- `src/App.tsx` — tray feed rendering
- `src/main-screen/MainScreenApp.tsx` — panel feed rendering

## Scope Estimate

Medium
