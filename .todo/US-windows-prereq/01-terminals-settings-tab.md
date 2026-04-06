---
status: done
---

# Task: Redesign "Agents" settings tab as "Terminals"

## Goal

Rename the "Agents" settings tab to "Terminals" and redesign it to be modular and OS-aware. Each supported terminal app (and tmux) gets its own expandable row in a flat list, with details revealed on expand.

## Acceptance criteria

- [ ] Tab renamed from "Agents" to "Terminals"
- [ ] All platform-relevant terminals shown in a flat list, each as an expandable row: Ghostty, iTerm2, Terminal.app, WezTerm, Kitty (macOS); Windows Terminal, PowerShell, cmd.exe (Windows -- future)
- [ ] tmux has its own expandable row (cross-platform, shown on both macOS and Windows)
- [ ] Expanding a terminal row reveals its details: version (if available), capabilities, and any settings specific to that terminal
- [ ] macOS-specific features (Ghostty AppleScript tab switching, Accessibility permissions) are shown only within the relevant terminal's expanded section, not as standalone settings
- [ ] Accessibility permission status/prompt is shown contextually within Ghostty's expanded section (since it's the terminal that requires it for AppleScript)
- [ ] The list is inherently OS-aware: on each platform, only platform-relevant terminals appear. No need for separate "hide on Windows" logic -- the terminal catalog is platform-filtered.
- [ ] Existing functionality preserved: Ghostty scriptable status badge, tmux support toggle, accessibility permission check -- all still work, just reorganized into their respective terminal sections
- [ ] `just check` passes

## Notes

- This redesign makes `US-windows/07-settings-platform-compat` much simpler. Instead of hiding macOS-only sections with `#[cfg]` checks, the terminal catalog is naturally platform-filtered.
- Use a `TERMINAL_CATALOG` similar to `FEED_CATALOG` in `src/shared/feedTypes.ts` -- a single data structure defining each terminal's metadata. Suggested shape:
  ```ts
  type CatalogTerminal = {
    id: string;              // e.g., "ghostty"
    name: string;            // e.g., "Ghostty"
    platform: "macos" | "windows" | "all";
    bundleId?: string;       // macOS bundle ID, e.g., "com.mitchellh.ghostty"
    description?: string;    // Brief description shown in the row
  }
  ```
  Terminal-specific settings (toggles, status badges) are rendered by the component based on terminal `id`, not encoded in the catalog.
- The backend already has bundle IDs for all 5 macOS terminals in `terminal_focus/terminals/mod.rs` -- keep these in sync with the frontend catalog.
- Windows terminals (Windows Terminal, PowerShell, cmd.exe) can be added to the catalog now as placeholders -- they'll light up when `US-windows` lands terminal focus for Windows.
- The current Tauri command for capabilities is `get_focus_capabilities` (command.rs:262), not `check_focus_caps`.

## Showcase

See `showcases/terminals-tab-showcase.html` -- approved direction: disclosure rows with no icons, no detection gating.

## Related files

- `src/settings/SettingsApp.tsx` (current "Agents" tab implementation, lines 1288-1431)
- `src-tauri/src/terminal_focus/terminals/mod.rs` (terminal strategy registry with bundle IDs)
- `src-tauri/src/command.rs` (`get_focus_capabilities`, line 262)
- `src/shared/feedTypes.ts` (pattern reference for catalog approach)
