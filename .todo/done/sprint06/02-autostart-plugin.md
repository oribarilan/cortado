---
status: done
---

# 02 — Autostart plugin

## Goal

Integrate `tauri-plugin-autostart` so users can toggle "Start on system startup" from the settings window.

## Acceptance criteria

- [ ] `tauri-plugin-autostart` is added as a Rust dependency
- [ ] `@tauri-apps/plugin-autostart` is added as a JS dependency
- [ ] Plugin is initialized in `main.rs` setup with `MacosLauncher::LaunchAgent`
- [ ] Capabilities include `autostart:allow-enable`, `autostart:allow-disable`, `autostart:allow-is-enabled`
- [ ] Settings UI has a "General" section with a toggle for "Start on system startup"
- [ ] Toggle reads current state via `isEnabled()` on mount
- [ ] Toggle calls `enable()` / `disable()` on change
- [ ] `just check` passes cleanly

## Notes

### Rust setup

```rust
// in main.rs, inside tauri::Builder
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
))
```

### JS usage

```ts
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
```

### Permissions

Add to the settings capability:
```json
"autostart:allow-enable",
"autostart:allow-disable",
"autostart:allow-is-enabled"
```
