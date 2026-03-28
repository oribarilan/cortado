---
status: done
---

# 05 — Backend: Theme + Text Size Settings

## Goal

Add `theme` and `text_size` fields to `AppSettings` so the frontend can read and persist appearance preferences.

## Acceptance Criteria

- [ ] `AppSettings` has `theme: String` field (default `"system"`, valid: `"system"`, `"light"`, `"dark"`)
- [ ] `AppSettings` has `text_size: String` field (default `"m"`, valid: `"s"`, `"m"`, `"l"`, `"xl"`)
- [ ] Both fields have serde defaults so existing `settings.toml` files without them load without error
- [ ] When `save_settings` is called and theme or text_size changed, emit `appearance-changed` event to all windows
- [ ] Event payload: `{ "theme": "...", "text_size": "..." }`
- [ ] `just check` passes

## Files to Change

- `src-tauri/src/app_settings.rs` — add fields, defaults, serde attributes
- `src-tauri/src/command.rs` (or wherever `save_settings` is implemented) — emit event after save

## Notes

- Consider using enums instead of raw strings for type safety (`Theme::System | Light | Dark`, `TextSize::S | M | L | XL`) with serde rename to lowercase strings for TOML compat.
- The event should be emitted to all windows so menubar panel and panel update immediately when settings change.
