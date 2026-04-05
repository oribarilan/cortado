---
status: done
---

# Task: Investigate terminal detection on macOS

## Goal

Determine the best approach for detecting which terminal emulators are installed on macOS, so the Terminals settings tab can show detection status in each terminal's expanded view.

## Acceptance criteria

- [ ] Investigate feasible detection methods for macOS `.app` bundles: Launch Services API (`LSCopyApplicationURLsForBundleIdentifier`), known-path checks (`/Applications/*.app`), `mdfind` queries, or other approaches
- [ ] Evaluate trade-offs: reliability, performance, edge cases (Homebrew cask vs drag-install, multiple versions, sandboxing)
- [ ] For CLI tools (tmux): confirm that the existing `which`-based check (via resolved login shell PATH) is sufficient, or propose an alternative
- [ ] Write up a recommendation in this file's Notes section: which method to use, what the API looks like, any caveats
- [ ] Implement a `get_installed_terminals()` Tauri command (or extend `get_focus_capabilities()`) that returns detection status for each terminal in the catalog
- [ ] Unit test the detection logic (at least for the "not found" path)
- [ ] `just check` passes

## Notes

**Existing bundle IDs** (from `terminal_focus/terminals/mod.rs`):
- Ghostty: `com.mitchellh.ghostty`
- Terminal.app: `com.apple.Terminal`
- iTerm2: `com.googlecode.iterm2`
- WezTerm: `com.github.wez.wezterm`
- Kitty: `net.kovidgoyal.kitty`

**Existing detection**: tmux is already checked via `tmux -V` in `get_capabilities()`. Ghostty version is checked similarly. But there's no general "is this app installed?" check.

**Constraints**:
- Must work in a packaged `.app` (minimal PATH — see AGENTS.md gotchas)
- Must be fast enough to call from settings UI without visible delay
- Should not require Accessibility permissions just to check if an app exists

## Related files

- `src-tauri/src/terminal_focus/terminals/mod.rs` (bundle IDs)
- `src-tauri/src/terminal_focus/mod.rs` (`get_capabilities()`)
- `src-tauri/src/command.rs` (`get_focus_capabilities`)
