---
status: pending
---

# Dev / Release Isolation

## Goal

Ensure that running `just dev` during development does not interfere with an installed release build of Cortado — and vice versa. Both should be able to run side-by-side without colliding.

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

## Acceptance criteria

- [ ] Dev builds use a distinct bundle ID (e.g., `sh.oribi.cortado.dev`)
- [ ] Dev builds use a separate config directory (e.g., `~/.config/cortado-dev/`)
- [ ] Dev builds show a distinct app name (e.g., "Cortado Dev") so they're visually distinguishable in Activity Monitor and tray
- [ ] Dev builds do NOT register autostart (or use a separate launch agent name)
- [ ] Dev builds do NOT register the global hotkey (avoids stealing it from the release build)
- [ ] Dev builds show a clear visual indicator (e.g., colored "DEV" badge in the panel header or footer, tinted panel border, distinct tray icon) so the developer always knows which instance they're looking at
- [ ] Release builds continue using `sh.oribi.cortado` and `~/.config/cortado/`
- [ ] The mechanism is automatic — developers don't need to manually toggle anything; `just dev` uses dev identity, `just build` / release uses production identity
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

## Relevant files

- `src-tauri/tauri.conf.json` — `identifier`, `productName`
- `src-tauri/tauri.dev.conf.json` (to create)
- `src-tauri/src/app_settings.rs` — `CONFIG_DIR` constant
- `src-tauri/src/feed/config.rs` — `CONFIG_DIR` constant
- `src-tauri/src/main.rs` — autostart plugin init
- `Justfile` — `dev` command
- `CONTRIBUTING.md` — document the setup
