---
status: pending
---

# Hot-reload feed config changes without restart

## Problem

When the user edits feed configuration (adds, removes, or modifies feeds in the TOML config), they must restart the app for changes to take effect.

## Goal

Detect feed config file changes at runtime and apply them live — start new feeds, stop removed feeds, and update modified feeds — without restarting the app. Settings (appearance, hotkey, etc.) are out of scope for this task.

## Notes

- The feed config file is already watched for changes (`src-tauri/src/feed/config.rs` has a `ChangeTracker`). Check if this is wired up to trigger a re-read.
- Key challenge: reconciling the running feed set with the new config. Need to diff old vs new and decide what to start/stop/restart.
- Modified feeds should ideally preserve their last-known activities during the transition rather than showing a blank slate.
- The tray and panel should update immediately to reflect the new feed set.

## Relevant files

- `src-tauri/src/feed/config.rs` — config parsing, `ChangeTracker`
- `src-tauri/src/feed/runtime.rs` — feed poll loop, feed lifecycle
- `src-tauri/src/feed/mod.rs` — feed registry, core types
