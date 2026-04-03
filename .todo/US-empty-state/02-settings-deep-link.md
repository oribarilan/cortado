---
status: done
---

# Settings deep-link support

## Goal

Allow the Settings window to be opened with a target section and optional pre-selected feed type, so external surfaces (like the empty state) can deep-link into the add-feed flow.

## Context

Currently `open_settings` takes no parameters and always opens to the "General" section. The add-feed flow in Settings is driven by internal React state (`catalogStep`, `editingFeed`, `isNewFeed`). There's no way to trigger it from outside.

## Acceptance criteria

- [ ] `open_settings` Tauri command accepts an optional payload (e.g., `{ section: "feeds", feed_type: "github-pr" }`)
- [ ] When Settings opens with `section: "feeds"`, it navigates directly to the Feeds section
- [ ] When Settings opens with `feed_type`, it additionally auto-starts the add-feed flow with that type pre-selected (calls `selectFeedType` equivalent)
- [ ] When opened with no payload, behavior is unchanged (opens to General)
- [ ] If Settings is already open, receiving a deep-link navigates to the requested section/flow

## Implementation approach

1. **Backend (`command.rs`):** Add optional `section` and `feed_type` params to `open_settings`. After showing the window, emit a Tauri event (e.g., `settings-navigate`) with the payload.
2. **Frontend (`SettingsApp.tsx`):** Listen for the `settings-navigate` event. On receive, call `setSection("feeds")` and optionally `selectFeedType(feedType)`.
3. **Edge case:** If Settings is already open and showing a different section, the event handler should still navigate (not ignore).

## Relevant files

- `src-tauri/src/command.rs` — `open_settings` command
- `src/settings/SettingsApp.tsx` — section state, `startAdd()`, `selectFeedType()`
- `src/main-screen/MainScreenApp.tsx` — will invoke `open_settings` with params (in task 03)
