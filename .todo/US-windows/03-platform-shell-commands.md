---
status: pending
---

# Task: Platform shell commands and system integration

## Goal

Replace macOS-specific shell commands (`open`, `osascript`, `defaults`, `which`) with cross-platform alternatives or `#[cfg]`-gated implementations so the app functions correctly on both macOS and Windows.

## Approach

Use the `open` crate for URL/file opening (1 line per call site) and inline `#[cfg]` guards for platform-specific logic. Don't build a `platform::shell` abstraction module — it's over-engineered for the number of call sites.

## Acceptance criteria

- [ ] URL/file opening: all 6 `Command::new("open")` call sites replaced with `open::that()` or `open::that_detached()` (cross-platform crate, or use Tauri's `tauri_plugin_shell::ShellExt` if preferred — decide)
- [ ] Reveal in file manager: macOS `open -R` call sites (2) → `#[cfg]` guard: macOS keeps `open -R`, Windows uses `Command::new("explorer").arg("/select,").arg(path)`
- [ ] Dark mode detection: `tray_icon.rs:is_macos_dark_mode()` → `is_dark_mode()` with `#[cfg]` body: macOS keeps `defaults read`, Windows reads `HKCU\...\Personalize\AppsUseLightTheme` via `winreg` crate
- [ ] CLI tool detection: `Command::new("which")` in `settings_config.rs:256` → `#[cfg]`: macOS keeps `which`, Windows uses `where.exe` (or use the `which` crate for cross-platform resolution)
- [ ] PATH resolution: `main.rs:32-47` gated with `#[cfg(target_os = "macos")]`. Windows skips PATH resolution entirely (packaged Windows apps inherit full PATH from Explorer/Start Menu).
- [ ] `open_notification_settings` command: macOS opens `x-apple.systempreferences:` URL; Windows opens `ms-settings:notifications` via `open::that()`
- [ ] PID liveness check: `libc::kill(pid, 0)` at 2 call sites → `#[cfg]`: macOS keeps `libc::kill`, Windows uses `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)` via `windows-sys` or the `sysinfo` crate
- [ ] macOS behavior unchanged — all existing implementations preserved behind `#[cfg(target_os = "macos")]`
- [ ] Both targets compile cleanly

## Notes

- The `open` crate (https://crates.io/crates/open) is well-maintained, widely used, and handles cross-platform URL/file opening. It uses `open` on macOS, `start` on Windows, `xdg-open` on Linux. This is simpler than hand-rolling `cmd /c start "" "url"`.
- `start` on Windows requires `cmd /c start "" "url"` for URLs with special characters — the `open` crate handles this.
- `explorer /select,"path"` reveals a file in Explorer (equivalent to `open -R` in Finder).
- Windows dark mode detection: read `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme` (0 = dark, 1 = light).
- The `which` crate (https://crates.io/crates/which) provides cross-platform executable detection. May be cleaner than `#[cfg]`-gating `which` vs `where.exe`.
- `osascript` calls are entirely within `terminal_focus/` — already gated behind macOS by task 05. No action needed here.

## Related files

- `src-tauri/src/main.rs:32-47` (PATH resolution)
- `src-tauri/src/command.rs:97-108` (`open_activity` → `Command::new("open")`)
- `src-tauri/src/command.rs:167-181` (`open_notification_settings`)
- `src-tauri/src/settings_config.rs:219,240` (`open`/`open -R`)
- `src-tauri/src/settings_config.rs:256` (`Command::new("which")`)
- `src-tauri/src/app_settings.rs:301,322` (`open`/`open -R`)
- `src-tauri/src/tray_icon.rs:187-194` (`is_macos_dark_mode`)
- `src-tauri/src/feed/harness/generic.rs:188` (`libc::kill`)
- `src-tauri/src/feed/harness/e2e.rs:106` (`libc::kill`)
