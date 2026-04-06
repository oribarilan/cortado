---
status: done
---

# Task: Proper XDG_CONFIG_HOME support on macOS

## Goal

Support the `$XDG_CONFIG_HOME` environment variable on macOS, falling back to `~/.config/` when unset. This is standard XDG Base Directory compliance -- currently the app hardcodes `~/.config/` without checking the env var.

## Acceptance criteria

- [ ] `app_env.rs` on macOS: check `$XDG_CONFIG_HOME` first. If set, use `$XDG_CONFIG_HOME/cortado`. If unset, fall back to `~/.config/cortado/` (current behavior).
- [ ] Windows behavior unchanged: uses `dirs::config_dir()` → `%APPDATA%\cortado` (no XDG on Windows)
- [ ] Unit test: verify `XDG_CONFIG_HOME` override works
- [ ] Unit test: verify fallback to `~/.config/cortado/` when unset
- [ ] Doc comment on `config_dir()` documents the resolution order
- [ ] `just check` passes

## Notes

- This is a small change in `app_env.rs:init()` -- check `std::env::var("XDG_CONFIG_HOME")` before falling back to `dirs::home_dir().join(".config")`.
- The `dirs` crate's `config_dir()` on macOS returns `~/Library/Application Support/` (Apple convention), NOT `~/.config/` (XDG convention). We deliberately use XDG on macOS, so we can't just switch to `dirs::config_dir()`.
- On Windows, `dirs::config_dir()` returns `%APPDATA%` which is correct. No XDG needed.
- This must land before `US-windows/01-cargo-platform-deps` adds the `#[cfg]` split for config directory, so the macOS path is clean.

## Related files

- `src-tauri/src/app_env.rs` (config directory resolution)
