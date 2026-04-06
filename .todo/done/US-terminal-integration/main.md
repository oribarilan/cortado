---
status: done
---

# Terminal Integration

## Theme

Build a robust, extensible terminal focus system that works across different terminal emulators and tmux configurations. Each terminal gets its own focus strategy, auto-detected by bundle ID.

## Spec

See `specs/terminal_integration.md` for the full architecture, API contracts, and per-terminal strategy details.

## Sequencing

```
01-refactor-strategies ──────┐
                             │
02-terminal-app ─────────────┤ (can parallel with 03-05)
                             │
03-iterm2 ───────────────────┤ (can parallel with 02, 04-05)
                             │
04-wezterm ──────────────────┤ (can parallel with 02-03, 05)
                             │
05-kitty ────────────────────┤ (can parallel with 02-04)
                             │
06-tests ────────────────────┘
```

Task 01 is prerequisite -- restructures existing code into the extensible pattern. Tasks 02-05 are independent terminal implementations. Task 06 adds comprehensive tests across all strategies.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-refactor-strategies.md` | Restructure into `terminals/` module with strategy registry |
| 02 | `02-terminal-app.md` | macOS Terminal.app: AppleScript TTY matching |
| 03 | `03-iterm2.md` | iTerm2: AppleScript TTY matching |
| 04 | `04-wezterm.md` | WezTerm: CLI-based pane focusing |
| 05 | `05-kitty.md` | kitty: remote control PID matching |
| 06 | `06-tests.md` | Comprehensive unit + integration tests |

## Cross-cutting notes

- **`KNOWN_TERMINALS`** in `pid_ancestry.rs` already includes process names for all planned terminals. No changes needed for detection -- only for focus strategies.
- **`escape_applescript()`** in `mod.rs` is shared by Ghostty, Terminal.app, and iTerm2. Already handles quotes and backslashes.
- **Settings UI**: the Agents tab currently shows tmux toggle, Ghostty detection, and accessibility. As strategies are added, consider showing detection status per terminal (auto-detected, no user config needed). Not a blocker -- can be done after all strategies ship.
- **`focus_label` field**: the "Open in {app}" button label is built from cached `FocusInfo` (app name, tmux status). New strategies don't need to change this -- the label comes from terminal detection, not the strategy.
- **E2E tests**: `src-tauri/src/terminal_focus/e2e.rs` has API verification tests for each terminal. As strategies are implemented, extend these to test actual focus behavior (open tab → defocus → run strategy → assert refocused).
- **Bundle IDs**: Warp is `dev.warp.Warp-Stable`. Alacritty is `org.alacritty`. Both already in `KNOWN_TERMINALS`. iTerm2 may appear as `iTerm.app` (not `iTerm2.app`) on some systems -- the AppleScript app name should handle both.
- **README**: after implementation, update the terminal support table. tmux and Ghostty are stable -- all other terminal strategies should be marked **(experimental)** until validated by broader usage.

## Out of scope (researched -- no viable API)

### Alacritty

- **Bundle ID**: `org.alacritty`
- **AppleScript**: None. Maintainers explicitly rejected adding it ([issue #2638](https://github.com/alacritty/alacritty/issues/2638)).
- **Tabs**: Not supported. Alacritty is intentionally single-window; tabs/splits are deferred to tmux or a window manager.
- **IPC**: Limited -- `alacritty msg` over Unix socket supports `create-window`, `config`, `get-config`. No window focus/listing.
- **Strategy**: App activation only. With tmux, the tmux strategy handles pane switching regardless.

### Warp

- **Bundle ID**: `dev.warp.Warp-Stable`
- **AppleScript**: Only generic `activate` works. No scripting dictionary (`sdef` returns error -192). Cannot enumerate windows, tabs, or sessions.
- **CLI**: No `warp` CLI for tab/pane management. Feature requested ([issue #3959](https://github.com/warpdotdev/Warp/issues/3959)).
- **Session focus**: Not possible. URI scheme (`warp://action/new_tab`) only creates new tabs -- cannot focus existing ones ([issue #8611](https://github.com/warpdotdev/Warp/issues/8611)).
- **Strategy**: App activation only.
