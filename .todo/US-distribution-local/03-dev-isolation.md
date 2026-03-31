---
status: pending
---

# Dev / Release Isolation

## Goal

Ensure three build variants can coexist cleanly using **two identities** (dev and production):

| Variant | Identity | How it runs |
|---------|----------|-------------|
| `just dev` (unpacked) | **Dev** | Hot-reload, from source |
| `just build` (local DMG) | **Dev** | Packaged DMG for testing |
| Production release | **Production** | Signed, notarized, from CI |

Dev and production can run **side-by-side** (different bundle IDs). Two dev instances (packaged + unpacked) are **prevented** by the single-instance plugin.

## Collision points

| Resource | Current value | Risk |
|----------|---------------|------|
| Bundle ID | `com.cortado.app` | macOS treats both builds as the same app (notifications, Gatekeeper, tray icon grouping). Should be `sh.oribi.cortado` for release. |
| Config dir | `~/.config/cortado/` | Dev and release share `feeds.toml` and `settings.toml` — edits in one affect the other |
| App name | `Cortado` | Both show as "Cortado" in Activity Monitor, Dock, tray — impossible to distinguish |
| Tray icon ID | `"tray"` | Process-local, no collision across separate processes |
| Window labels | `"main"`, `"main-screen"`, `"settings"` | Process-local, no collision |
| Autostart | `tauri_plugin_autostart` | Dev build could register itself as the login item, replacing the release build |
| Global hotkey | `tauri_plugin_global_shortcut` | Dev build could steal the hotkey from the release build |
| Multiple dev instances | N/A | `just dev` and installed dev DMG share the same bundle ID — if both run, the second steals the global hotkey from the first |

## Acceptance criteria

- [ ] Dev builds use a distinct bundle ID (`sh.oribi.cortado.dev`)
- [ ] Dev builds use a separate config directory (`~/.config/cortado-dev/`)
- [ ] Dev builds show a distinct app name ("Cortado Dev") so they're visually distinguishable in Activity Monitor and tray
- [ ] Dev builds do NOT register autostart (or use a separate launch agent name)
- [ ] Dev builds do NOT register the global hotkey (avoids stealing it from the release build)
- [ ] Dev builds show a clear visual indicator (e.g., colored "DEV" badge in the panel header or footer, tinted panel border, distinct tray icon) so the developer always knows which instance they're looking at
- [ ] `tauri-plugin-single-instance` prevents running two dev instances simultaneously (packaged DMG + `just dev`). Second instance focuses the existing one.
- [ ] Release builds continue using `sh.oribi.cortado` and `~/.config/cortado/`
- [ ] The mechanism is automatic — developers don't need to manually toggle anything; `just dev` and `just build` use dev identity, production release uses production identity
- [ ] Document the isolation setup in `CONTRIBUTING.md`

## Implementation approach

Tauri supports build-time config overrides. Options:

**Option A: Cargo feature flag**
- Add a `dev` feature to `Cargo.toml`
- In Rust code, check `#[cfg(feature = "dev")]` to switch config dir and app name
- `just dev` passes `--features dev`; release builds don't
- Bundle ID override may need a separate `tauri.conf.json` or Tauri's `--config` flag

**Option B: Tauri config overlay**
- Create `src-tauri/tauri.dev.conf.json` with dev-specific overrides (identifier, productName)
- `just dev` passes `--config tauri.dev.conf.json` to `tauri dev`
- Config dir switch still needs a Rust-side mechanism (env var or feature flag)

**Option C: Environment variable**
- Set `CORTADO_DEV=1` in `just dev`
- Rust code checks this at runtime to switch config dir and app behavior
- Simpler but less compile-time safe

Recommend **Option B + env var hybrid**: Tauri config overlay for bundle ID / app name, env var for config dir path in Rust.

### Single-instance plugin

Add `tauri-plugin-single-instance` to prevent two dev instances from running simultaneously. Since `just dev` and `just build` DMG share the same bundle ID (`sh.oribi.cortado.dev`), the plugin detects the collision and focuses the existing instance instead of launching a second one.

```rust
// main.rs
.plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
    // Focus existing window
}))
```

This applies to both dev and production builds — no duplicate production instances either.

## Relevant files

- `src-tauri/tauri.conf.json` — `identifier`, `productName`
- `src-tauri/tauri.dev.conf.json` (to create)
- `src-tauri/Cargo.toml` — add `tauri-plugin-single-instance` dependency
- `src-tauri/src/app_settings.rs` — `CONFIG_DIR` constant
- `src-tauri/src/feed/config.rs` — `CONFIG_DIR` constant
- `src-tauri/src/main.rs` — autostart plugin init, single-instance plugin init
- `Justfile` — `dev` and `build` commands
- `CONTRIBUTING.md` — document the setup
