---
status: done
---

# 01 — Settings window infrastructure

## Goal

Establish the multi-window foundation: a settings window that can be opened from the tray menu, with its own frontend entry point, without breaking the existing menubar panel.

## Acceptance criteria

- [ ] `tauri.conf.json` has a `"settings"` window entry with `"create": false`
- [ ] Vite is configured for multi-page: `settings.html` as a second entry point
- [ ] `src/settings/` directory exists with a React root that renders a placeholder "Settings" page
- [ ] `settings.html` loads the settings React app
- [ ] Tray menu has a "Settings..." item (between "Refresh feeds" and separator)
- [ ] Panel footer has a "Settings..." button (alongside "Refresh feeds" and "Quit Cortado")
- [ ] Clicking "Settings..." opens or focuses the settings window via Rust
- [ ] If the settings window is already open, it is shown and focused (no duplicate windows)
- [ ] Settings window is a standard decorated macOS window (not transparent, not NSPanel)
- [ ] Settings window has a reasonable default size (e.g. 640×480) and is resizable
- [ ] A new capability entry grants the settings window necessary permissions
- [ ] `just check` passes cleanly

## Notes

### Tauri config shape

```json
{
  "label": "settings",
  "title": "Cortado Settings",
  "url": "settings.html",
  "create": false,
  "width": 640,
  "height": 480,
  "resizable": true,
  "decorations": true,
  "transparent": false,
  "visible": true,
  "center": true
}
```

### Vite multi-page

In `vite.config.ts`, add `build.rollupOptions.input`:

```ts
build: {
  rollupOptions: {
    input: {
      main: resolve(__dirname, 'index.html'),
      settings: resolve(__dirname, 'settings.html'),
    },
  },
},
```

### Rust window management

Add an `open_settings` Tauri command (in a new `src-tauri/src/settings.rs` or in `command.rs`) that:
1. Checks `app.get_webview_window("settings")`
2. If found: `.show()` + `.set_focus()`
3. If not: `WebviewWindowBuilder::from_config(...)` using the config entry

This must be a `#[tauri::command]` so the panel footer's "Settings..." button can call it via `invoke("open_settings")` from the frontend. The tray menu handler in `panel.rs` also calls this same function.

### Tray menu update

In `panel.rs`, add a `MENU_ID_SETTINGS` item. Handler calls the open function.

### Capability

Create `src-tauri/capabilities/settings.json` scoped to `windows: ["settings"]` with core permissions.
