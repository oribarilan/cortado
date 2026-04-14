---
status: pending
---

# Update feed: open action in panel

## Goal

The auto-update feed currently supports "Install update" and "Update plugin" actions in both tray and panel, but there's no way to *open* the update -- e.g., view the GitHub release page or the plugin's changelog. Other feeds support opening via Enter (panel) or click (tray) because their activity IDs are URLs.

Add an "open" action so users can view what's in the update before installing.

## Design

### Backend (`cortado_update.rs`)

The auto-update feed currently sets `action: None` on all activities. Activities need a URL field so the frontend can open them:

- **App update**: Add a `url`-typed field (or set the activity ID to the release URL). The release URL follows the pattern `https://github.com/oribarilan/cortado/releases/tag/v{version}`.
- **Plugin updates**: Add a URL field pointing to the relevant plugin page or changelog, if available. If no meaningful URL exists, these can remain non-openable.

The simplest approach: change the activity `id` for app updates to the release URL (e.g., `https://github.com/oribarilan/cortado/releases/tag/v0.14.0`). This makes `supportsOpen()` return the URL automatically with no frontend changes.

For plugin updates, add a field with `field_type: "url"` pointing to the plugin's source.

### Frontend

If the activity ID is a URL, `supportsOpen()` already works. The panel's Enter handler checks `supportsUpdate` first (for the install action), then `supportsOpen`. We may want to add a secondary action (e.g., Shift+Enter to open, Enter to install) or show both buttons in the detail pane.

Consider: should Enter install (current behavior) and a separate key open the release page? Or should we show both actions in the detail pane like tray does?

## Acceptance criteria

- [ ] App update activities have a URL pointing to the GitHub release page
- [ ] Panel shows an "open release" action (in addition to the install action)
- [ ] Tray shows an "open release" action (in addition to the install action)
- [ ] Plugin update activities have a URL if a meaningful target exists
- [ ] `supportsOpen()` returns the correct URL for update activities

## Related files

- `src-tauri/src/feed/cortado_update.rs` -- update feed implementation
- `src/main-screen/MainScreenApp.tsx` -- panel Enter handler, DetailPane
- `src/App.tsx` -- tray detail rendering, update buttons
- `src/shared/utils.ts` -- `supportsOpen()`, `supportsUpdate()`
