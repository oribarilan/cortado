---
status: pending
---

# Log files

## Goal

The app currently logs only via `eprintln!` to stderr, which is invisible in packaged builds (no terminal attached). Add structured logging to rotating log files so users can attach them to bug reports.

## Current state

62 `eprintln!` calls across 15 files. No logging framework, no log files on disk. When the app misbehaves in a packaged build, there's no diagnostic output to look at.

## Design

### Framework

Use `tauri-plugin-log` -- it integrates with Rust's `log` crate and writes to the Tauri app log directory (`~/Library/Logs/Cortado/` on macOS). It supports log rotation and level filtering out of the box.

### Migration

Replace `eprintln!` calls with appropriate `log` macros:

- `eprintln!("failed ...")` → `log::warn!("...")` or `log::error!("...")`
- `eprintln!("focus: ...")` → `log::debug!("...")`
- Informational startup messages → `log::info!("...")`

### Log rotation

Configure rotation to keep logs bounded (e.g., 5 files, 2 MB each = 10 MB max). Old logs rotate out automatically.

### Log level

Default level: `info` in production, `debug` in dev builds. Consider making the level configurable in settings later, but not in this task.

### Log location

`tauri-plugin-log` writes to the platform app log directory:
- macOS: `~/Library/Logs/Cortado/`
- Dev: `~/Library/Logs/Cortado Dev/` (follows the dev identity)

### On/off toggle

Add a `local_logs` boolean to `settings.toml` under `[general]`. Default: `true` (logging enabled). When `false`, the file target is omitted from the plugin builder (no log files written to disk).

This is a **restart-required setting** -- `tauri-plugin-log` configures targets at init time, not at runtime. The Settings UI should indicate that changing this requires a restart.

When logging is disabled, existing log files are left in place (not deleted) -- the user may still want to attach older logs to an issue.

### Boot order

The `local_logs` setting must be read **before** Tauri plugin registration (in `main()`, before `.plugin(...)` calls). This means reading `settings.toml` directly via `app_settings` or a lightweight TOML parse, not via a Tauri command. This is the same pattern used for `resolve_login_shell_path()`.

### Log content policy

Logs must not contain secrets (API tokens, passwords) or PII beyond what the user explicitly configured (repo names, usernames). Feed poll results should log activity counts and errors, not full API response bodies. When in doubt, log at `debug` level so it's excluded from production logs by default.

## Acceptance criteria

- [ ] `tauri-plugin-log` and `log` are added as dependencies
- [ ] Log files are written to `~/Library/Logs/Cortado/`
- [ ] All existing `eprintln!` calls are replaced with `log::` macros at appropriate levels
- [ ] Log rotation is configured (e.g., `KeepSome(5)` at 2 MB per file = 10 MB max)
- [ ] Dev builds log at debug level, production at info level
- [ ] `[general] local_logs = true|false` setting controls whether log files are written (default: `true`)
- [ ] Settings UI has a toggle for local logs under General, with a "requires restart" note
- [ ] Logs do not contain secrets or full API response bodies
- [ ] Verify: launch the app, trigger some feeds, check that log files appear and contain useful output
- [ ] Verify: disable local logs, restart, confirm no new log files are written

## Related files

- `src-tauri/Cargo.toml` -- add dependency
- `src-tauri/src/main.rs` -- plugin registration
- `src-tauri/src/app_settings.rs` -- add `local_logs` field
- `src/settings/SettingsApp.tsx` -- add toggle in General section
- All 15 files with `eprintln!` calls (see `grep -r eprintln src-tauri/src/`)

## Notes

- Add `log` as a direct dependency alongside `tauri-plugin-log`. The plugin re-exports `log` but the idiomatic approach is `log = "0.4"` in `Cargo.toml` for the macros.
- The frontend (`console.log`/`console.error`) can also be routed to the same log file via `TargetKind::Webview`, but that's optional for this task.
- Use `level_for()` to suppress noisy deps (e.g., `level_for("hyper", log::LevelFilter::Warn)`).
