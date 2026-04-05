---
status: pending
---

# Task: Platform-conditional Cargo dependencies and compilation foundation

## Goal

Make the Rust codebase compile on both macOS and Windows targets by gating macOS-only dependencies behind `#[cfg(target_os = "macos")]`, fixing the config directory path strategy, deciding on Windows crate dependencies, and adding a Windows CI smoke test.

## Acceptance criteria

- [ ] macOS-only Cargo dependencies are gated behind `[target.'cfg(target_os = "macos")'.dependencies]`: `tauri-nspanel`, `monitor`, `popover`, `menubar`, `system-notification`
- [ ] Windows-only dependencies added under `[target.'cfg(target_os = "windows")'.dependencies]` — decide on: `winreg` (registry access for dark mode), `open` crate (URL/file opening), and whether `windows-sys` is needed for PID checks
- [ ] `macos-private-api` Tauri feature: verify it's a no-op on Windows (it should be — Tauri gates it internally). Document the verification in this task.
- [ ] Config directory path strategy resolved: `app_env.rs` currently hardcodes `~/.config/cortado` via `dirs::home_dir().join(".config")`. Either: (a) switch to `dirs::config_dir()` on all platforms (but this changes macOS from `~/.config/` to `~/Library/Application Support/` — breaking change), or (b) keep `~/.config/` on macOS via cfg, use `dirs::config_dir()` on Windows. Decide and implement.
- [ ] All macOS-only module files (`fns.rs`, `main_screen.rs`, `terminal_focus/`) are gated with `#[cfg(target_os = "macos")]` at the module declaration level in `main.rs`
- [ ] Stub modules or conditional compilation added so `main.rs` compiles on Windows (functions called from `command.rs` etc. need Windows stubs or cfg-gated call sites)
- [ ] `libc::kill(pid, 0)` call sites (2) have Windows alternatives or are cfg-gated
- [ ] Harness config path: `feed/harness/generic.rs:40` and `feed/harness/e2e.rs:39` hardcode `.config/cortado/harness` instead of using `app_env::config_dir()`. Refactor to use `app_env::config_dir().join("harness")` so the config directory strategy applies uniformly
- [ ] `app_env.rs` test assertion (`dir.ends_with(".config/cortado")` at line 48) updated to be platform-aware after config directory strategy is resolved
- [ ] Doc comments referencing `~/.config/cortado/` (`app_settings.rs:33,194,214`, `generic.rs:17`) updated to say "config directory" generically
- [ ] `cargo check --target x86_64-pc-windows-msvc` passes locally (best-effort; CI is the real gate)
- [ ] `cargo check` on macOS still passes with no warnings
- [ ] `just check` passes on macOS
- [ ] CI workflow updated to add a `cargo check --target x86_64-pc-windows-msvc` job on `windows-latest` runner (fast feedback — just check, not build)

## Notes

- The `macos-private-api` Cargo feature (line 15 of `Cargo.toml`) gates macOS-only Tauri code paths. On Windows, these paths aren't compiled. The `macOSPrivateApi: true` in `tauri.conf.json` is similarly ignored at runtime on non-macOS. Verify explicitly.
- `notify` crate (file watching) is already cross-platform — no changes needed.
- `dirs` crate is already cross-platform — `config_dir()` returns `%APPDATA%` on Windows. But the app deliberately uses `~/.config/` on macOS (XDG-style), not `~/Library/Application Support/`. Changing this on macOS would break existing users.
- For PID checks: `sysinfo` crate or `windows-sys::Win32::System::Threading::OpenProcess` are both options. `sysinfo` is heavier but cross-platform.
- The `cargo-clippy` feature workaround relates to macOS `objc`/`sel_impl` macros — may not be needed on Windows.
- `libc` crate itself compiles on Windows, but `kill()` is POSIX-only. Check for any other `libc` calls beyond the two known sites.

## Related files

- `src-tauri/Cargo.toml`
- `src-tauri/src/main.rs` (module declarations, plugin init)
- `src-tauri/src/app_env.rs` (config directory path — hardcodes `~/.config/`)
- `src-tauri/src/fns.rs` (macOS-only, needs module-level gating)
- `src-tauri/src/main_screen.rs` (macOS-only, needs module-level gating)
- `src-tauri/src/terminal_focus/` (macOS-only, needs module-level gating)
- `src-tauri/src/feed/harness/generic.rs:40` (hardcodes `.config/cortado/harness` — should use `app_env::config_dir()`)
- `src-tauri/src/feed/harness/e2e.rs:39` (same)
- `src-tauri/src/feed/harness/generic.rs:188` (`libc::kill`)
- `src-tauri/src/feed/harness/e2e.rs:106` (`libc::kill`)
- `src-tauri/src/app_settings.rs:33,194,214` (doc comments with hardcoded path)
- `.github/workflows/ci.yml` (add Windows check job)
- `.github/workflows/ci-build.yml` (add Windows check job)
