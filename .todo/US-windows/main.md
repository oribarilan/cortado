# US-windows: Cross-Platform Support (Windows)

## Theme

Add Windows support to Cortado without regressing macOS. The app is deeply macOS-native today — NSPanel swizzling, AppleScript terminal focus, `open`/`defaults`/`osascript` shell commands, vibrancy effects, and macOS-only Cargo dependencies. This story introduces `#[cfg(target_os)]` guards and Windows-native alternatives so the same codebase compiles and runs on both platforms.

## Guiding Principles

1. **macOS is not at risk.** Every change wraps existing macOS code behind `cfg(target_os = "macos")` — no behavioral changes to macOS paths.
2. **Cfg-dispatch over abstraction.** Use `#[cfg]`-gated module pairs (e.g., `panel_macos.rs` / `panel_windows.rs` with a thin dispatcher) rather than trait abstractions. There are exactly two platforms; a trait is premature.
3. **Graceful degradation.** Features without a Windows equivalent (e.g., terminal focus via AppleScript) are disabled at compile time with clear "not supported on this platform" messaging rather than crashing.
4. **Compile-gate everything.** The project must compile on both `x86_64-pc-windows-msvc` and the existing macOS targets after each task. Use `cargo check --target` locally as a smoke test; CI is the gate.
5. **Simplify where platforms diverge.** Don't replicate macOS's NSPanel popover UX on Windows — use idiomatic Windows patterns (e.g., the main screen panel may serve as the primary UI on Windows, with the menubar popup as a simpler overlay).

## Sequencing

Tasks are ordered by dependency. **Task 00 must be completed first** — it resolves design decisions that affect tasks 01-09. Tasks 01-03 are foundational and must be done next (in order). Tasks 04-08 can be parallelized after the foundation, **except** task 04 also depends on task 02.

```
00-decisions                 ─── Resolve all open design decisions FIRST
01-cargo-platform-deps       ─┐
02-platform-window-mgmt       │ Foundation (sequential)
03-platform-shell-commands    ─┘
04-tray-icon-platform          ─── Depends on 02 + 03
05-terminal-focus-platform      ─┐
06-frontend-platform-compat    │ Feature parity (parallelizable after 01-03)
07-settings-platform-compat    │
08-autostart-and-activation    ─┘
09-build-and-ci               ─── After all features
10-spec-and-docs              ─── Final
```

## Key Decisions (**Resolved**)

All decisions are recorded in `00-decisions.md` (status: done). Summary:

| # | Decision | Chosen |
|---|----------|--------|
| D1 | Config directory | macOS keeps XDG (`~/.config/`), Windows uses `%APPDATA%`. Proper `$XDG_CONFIG_HOME` support is a prereq. |
| D2 | Window management | Skip menubar popup on Windows. Tray click shows main screen panel. Tray icon still shows status dots. |
| D3 | Vibrancy | Mica/Acrylic on Windows 11, solid fallback on Windows 10. |
| D4 | URL/file opening | `open` crate. |
| D5 | CLI detection | `which` crate. |
| D6 | PID liveness | `windows-sys` with `OpenProcess`. |
| D7 | macOS-only settings | Superseded by `US-windows-prereq/01-terminals-settings-tab` redesign. |
| D8 | Copilot hooks | Support Windows. Details deferred to implementation on a Windows machine. |
| D9 | WebView2 | Tauri NSIS default bootstrapper. |

## Prerequisites

**`US-windows-prereq` must be completed before starting this story:**

1. **`01-terminals-settings-tab`** — Redesign "Agents" tab as "Terminals" with modular, OS-aware, per-terminal sections. Unblocks task 07.
2. **`02-xdg-config-home`** — Proper `$XDG_CONFIG_HOME` support on macOS. Unblocks task 01.

## Cross-Cutting Concerns

- **PATH resolution**: macOS resolves via `$SHELL -l -c`. Windows skips it entirely — packaged Windows apps inherit the full user PATH from Explorer/Start.
- **Config location**: `app_env.rs` currently hardcodes `~/.config/cortado/` via `dirs::home_dir().join(".config")`. This must change to `dirs::config_dir()` which returns `~/Library/Application Support` on macOS and `%APPDATA%` on Windows. **However**, since macOS users already have configs at `~/.config/cortado/`, we need a migration strategy or keep the macOS path as-is and only use `dirs::config_dir()` on Windows. Resolve in task 01.
- **File paths**: Forward slashes work in Tauri/Rust on Windows, but display paths should use native separators.
- **Keyboard shortcuts**: `⌘` → `Ctrl` on Windows. `⌥` → `Alt`. Frontend must detect platform and render accordingly.
- **`which` vs `where`**: `settings_config.rs` uses `Command::new("which")` to check for CLI tools. Windows needs `where.exe` or the `which` crate.
- **`RunEvent::Reopen`**: macOS-only (dock click). Windows equivalent is single-instance second-launch or tray icon double-click.
- **Copilot plugin hooks**: The copilot plugin install uses `#[cfg(unix)]` for `.sh` hook file permissions. Windows needs `.bat`/`.ps1` hooks or a different permission strategy.

## macOS-Specific Code Inventory (Reference)

### Category 1: NSPanel / Window Management (~600 lines)
- `src-tauri/src/fns.rs` — Menubar panel: NSPanel swizzling, delegates, vibrancy, positioning, toggle
- `src-tauri/src/main_screen.rs` — Main screen panel: same pattern
- `src-tauri/src/command.rs` — `init_panel`, `init_main_screen_panel`, `hide_menubar_panel`, `hide_all_panels`
- `tauri_nspanel::ManagerExt` adds `get_webview_panel()` — called in command.rs, fns.rs, main_screen.rs (~8 call sites)

### Category 2: Tray Icon (~200 lines)
- `src-tauri/src/tray_icon.rs` — Template mode, dark mode via `defaults`, icon compositing with menubar-colored ring
- `src-tauri/src/panel.rs` — `icon_as_template(true)`, tray click → `monitor::get_monitor_with_cursor()` → `fns::toggle_menubar_panel()`

### Category 3: Terminal Focus (~1,400 lines, entirely macOS)
- `src-tauri/src/terminal_focus/` — AppleScript for Ghostty, Terminal.app, iTerm2, WezTerm, Kitty
- `src-tauri/src/terminal_focus/pid_ancestry.rs` — Process tree walking via `ps` (POSIX-only)
- `AXIsProcessTrusted()` FFI for Accessibility permission check

### Category 4: System Commands (~20 call sites)
- `Command::new("open")` — 6 call sites (URLs, Finder reveal)
- `Command::new("osascript")` — ~12 call sites (terminal focus)
- `Command::new("defaults")` — 1 call site (dark mode detection)
- `Command::new("which")` — 1 call site (CLI tool detection)
- `$SHELL -l -c` PATH resolution — 1 call site

### Category 5: Cargo Dependencies (macOS-only)
- `tauri-nspanel` — NSPanel swizzling
- `monitor`, `popover`, `menubar`, `system-notification` — tauri-toolkit crates
- `macos-private-api` Tauri feature (no-op on Windows — verified)
- `MacosLauncher::LaunchAgent` for autostart

### Category 6: App Lifecycle (macOS-only)
- `RunEvent::Reopen` in `main.rs:214` — dock click shows panel (no-op on Windows)
- `ActivationPolicy::Accessory` in `main.rs:148` — no Dock icon (macOS-only Tauri API)
- Config path: `app_env.rs` hardcodes `~/.config/cortado` instead of using `dirs::config_dir()`

### Category 7: Frontend (~50 locations)
- Hardcoded ⌘ shortcuts and macOS modifier symbols in `formatShortcut()`
- `invoke("init_panel")`, `invoke("init_main_screen_panel")` — NSPanel init
- `x-apple.systempreferences:` deep links
- Ghostty/Accessibility settings sections (macOS-only concepts)
- `-apple-system`, `BlinkMacSystemFont` font stack
- `backdrop-filter: blur()` for vibrancy (works cross-platform but tuning may differ)

### Category 8: Build/Config/CI
- `tauri.conf.json` — `targets: ["dmg", "app"]`, `macOSPrivateApi: true`
- CI workflows (`.github/workflows/ci.yml`, `ci-build.yml`) — macOS-only runners
- `latest.json` updater manifest — only `darwin-aarch64` platform
- Apple signing and notarization pipeline
- `.icns` icon files (`.ico` already exists)
- `#[cfg(unix)]` in copilot plugin install for file permissions

### Category 9: Already Cross-Platform (no work needed)
- File watchers (`notify::RecommendedWatcher`) — auto-selects FSEvents/ReadDirectoryChanges
- Icons — both `.icns` and `.ico` present
- `build.rs` — no platform logic
- Frontend config paths — provided from backend commands, not hardcoded
- Tauri capabilities — no platform-scoped permissions
- `tauri_plugin_notification` — cross-platform
- `tauri_plugin_updater` — cross-platform (needs Windows artifacts in `latest.json`)
