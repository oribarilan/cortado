---
status: done
---

# Task: Resolve preliminary design decisions

## Goal

Close all open design decisions that block or influence the implementation of tasks 01-09. Each decision below is self-contained with options, trade-offs, and a recommendation. Resolve them before starting implementation.

## Decisions

### D1: Config directory strategy

**Context:** `app_env.rs` hardcodes `~/.config/cortado/` via `dirs::home_dir().join(".config")`. This path doesn't exist on Windows (`%USERPROFILE%\.config\cortado` is non-standard). Additionally, `feed/harness/generic.rs:40` and `feed/harness/e2e.rs:39` duplicate this path construction instead of using `app_env::config_dir()`.

**Chosen: (a) -- macOS keeps XDG (`~/.config/cortado/`), Windows uses `%APPDATA%\cortado`.** `app_env.rs` uses `#[cfg]` to pick the right base. Harness code is updated to use `app_env::config_dir()` instead of hardcoding. Additionally, proper `$XDG_CONFIG_HOME` support on macOS is a separate prerequisite task (see `US-windows-prereq`).

**Affects:** Task 01

---

### D2: Windows window management approach

**Context:** The entire UX identity of Cortado is the NSPanel floating panel. NSPanel has no Windows equivalent. This is the highest-impact architectural decision in the story.

**Chosen: (c) -- Skip menubar popup on Windows.** On tray click, show only the main screen panel. The menubar popup is a macOS idiom; on Windows, tray apps show a single main window. Tray icon still reflects global roll-up state via status dot compositing (covered by task 04).

**Affects:** Task 02, Task 04

---

### D3: Windows vibrancy

**Context:** macOS uses `NSVisualEffectView` for native blur/vibrancy. Tauri v2 supports Mica/Acrylic on Windows 11 via `set_effects()`. Windows 10 has no equivalent.

**Chosen: (a) -- Mica/Acrylic on Windows 11, solid background fallback on Windows 10.**

**Affects:** Task 02

---

### D4: URL/file opening crate

**Context:** 6 call sites use `Command::new("open")`. Need cross-platform replacement.

**Chosen: (a) -- `open` crate.** Well-maintained, widely used, handles macOS/Windows/Linux. One-line replacement per call site.

**Affects:** Task 03

---

### D5: CLI tool detection

**Context:** `settings_config.rs:256` uses `Command::new("which")` to check if a binary is installed. `which` is Unix-only; Windows uses `where.exe`.

**Chosen: (b) -- `which` crate.** Cross-platform executable detection. Tiny, well-maintained, removes platform conditionals entirely.

**Affects:** Task 03

---

### D6: PID liveness check

**Context:** `libc::kill(pid, 0)` at 2 call sites checks if a process is alive. POSIX-only.

**Chosen: (a) -- `windows-sys` crate with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`.** Lightweight, minimal dependency. `#[cfg]`-gated alongside the existing `libc::kill` on macOS.

**Affects:** Task 03

---

### D7: macOS-only settings sections on Windows

**Context:** Ghostty tab switching and Accessibility permission sections in Settings are macOS-only concepts. They need to be hidden or explained on Windows.

**Chosen: Superseded.** The "Agents" settings tab will be redesigned as a prerequisite (see `US-windows-prereq`). The new "Terminals" tab will be modular per-terminal, OS-aware, and only show installed apps -- making this decision moot.

**Affects:** Task 07 (blocked by `US-windows-prereq`)

---

### D8: Copilot plugin hooks on Windows

**Context:** The copilot plugin install creates `cortado-hook.sh` and sets Unix permissions (`chmod 755`). Windows doesn't use `.sh` files or Unix permissions.

**Chosen: Support Windows hooks.** Copilot CLI works on Windows. Implementation details (`.bat` vs `.ps1` vs other) to be explored when implementing on a Windows machine. Task exists; specifics deferred.

**Affects:** Task 09

---

### D9: WebView2 runtime bootstrapping

**Context:** Windows requires Microsoft Edge WebView2 Runtime. Tauri's NSIS installer template can auto-download it, but this needs explicit configuration.

**Chosen: (a) -- Tauri NSIS default bootstrapper.** Auto-downloads WebView2 if missing. Well-tested, standard approach.

**Affects:** Task 09

## Acceptance criteria

- [x] Each decision above has a chosen option recorded
- [x] Any decisions that spawn new requirements have been added to the relevant task files
- [ ] `main.md` "Key Decisions to Make" section is updated to reflect resolved decisions
