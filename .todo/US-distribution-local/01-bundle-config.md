---
status: pending
---

# Bundle Config

## Goal

Fix the bundle configuration so Tauri produces a correct, properly-identified DMG.

## Acceptance criteria

- [ ] `src-tauri/tauri.conf.json` version aligned with `src-tauri/Cargo.toml` (use `0.2.0`)
- [ ] Bundle identifier changed from `com.cortado.app` to `sh.oribi.cortado`
- [ ] Bundle targets set to `["dmg", "app"]` (DMG for distribution, app bundle as its input)
- [ ] `just check` passes after changes

## Notes

- Tauri derives the app bundle version from `tauri.conf.json` `version`. The Cargo.toml version is for the Rust crate. They should match but tauri.conf.json is what ends up in the macOS bundle's `Info.plist`.
- `targets: "all"` currently builds DMG, app, and updater artifacts. Narrowing to `["dmg", "app"]` avoids unnecessary artifacts for now.
- The bundle ID (`sh.oribi.cortado`) is used by macOS for app identity — Gatekeeper, notifications, preferences, tray icon grouping. Getting it right now avoids migration pain later.

## Relevant files

- `src-tauri/tauri.conf.json` — `version`, `identifier`, `bundle.targets`
- `src-tauri/Cargo.toml` — `version` in `[package]`
