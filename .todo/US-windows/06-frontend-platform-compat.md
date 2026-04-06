---
status: pending
---

# Task: Frontend platform compatibility

## Goal

Make the React frontend platform-aware so it renders correct keyboard shortcuts, modifier symbols, and platform-appropriate UI on both macOS and Windows.

## Acceptance criteria

- [ ] Platform detection utility: a `getPlatform()` or `isMacOS()` helper using Tauri's `@tauri-apps/plugin-os` `platform()` function (or `navigator.platform` as fallback). Created in `src/shared/utils.ts` or a new `src/shared/platform.ts`.
- [ ] `formatShortcut()` in `SettingsApp.tsx` renders platform-appropriate symbols: macOS uses `⌘⌃⌥⇧`; Windows uses `Ctrl`, `Alt`, `Shift`, `Win`
- [ ] `keyEventToShortcut()` maps `e.metaKey` to `"super"` on macOS (unchanged), `e.ctrlKey` to primary modifier on Windows
- [ ] Hardcoded `⌘⇧Space` hotkey hint in `MainScreenApp.tsx:97` is platform-conditional: `Ctrl+Shift+Space` on Windows
- [ ] `⌘Q` and `⌘,` keyboard handlers in `MainScreenApp.tsx:343-375` use `e.metaKey` on macOS (unchanged), `e.ctrlKey` on Windows
- [ ] Font stack in `tokens.css` includes Windows system fonts: add `"Segoe UI Variable", "Segoe UI"` to the stack alongside existing macOS fonts
- [ ] `-webkit-font-smoothing` and `-moz-osx-font-smoothing` remain (they're no-ops on Windows, no harm)
- [ ] All `invoke()` calls to panel commands (`init_panel`, `init_main_screen_panel`) continue working -- backend handles the platform switch (from task 02)
- [ ] Config file path shown in empty state (`MainScreenApp.tsx:94` shows `~/.config/cortado/feeds.toml`) uses backend-provided path instead of hardcoded Unix path
- [ ] Harness feed descriptions in `feedTypes.ts:226,256` hardcode `~/.config/cortado/harness/` -- replace with platform-neutral text or a backend-provided path
- [ ] Both platforms build the frontend cleanly (`pnpm build`)

## Notes

- Tauri v2 provides `platform()` from `@tauri-apps/plugin-os` for runtime OS detection. Requires adding the plugin dependency if not already present.
- The keyboard shortcut rendering is the most visible cross-platform difference for users.
- CSS `-webkit-appearance: none` resets work on Windows Chromium (Edge/WebView2) too.
- Consider creating a `usePlatform()` React hook for ergonomic platform checks.
- The `x-apple.systempreferences:` deep links are covered in task 07 (Settings UI).
- `backdrop-filter: blur()` works in WebView2 (Chromium-based) -- no changes needed.

## Dependencies

- Task 02 (platform window management -- so backend invoke calls route correctly)

## Related files

- `src/settings/SettingsApp.tsx:192-222` (`formatShortcut`, `keyEventToShortcut`)
- `src/main-screen/MainScreenApp.tsx:94` (hardcoded config path)
- `src/main-screen/MainScreenApp.tsx:97` (hotkey hint)
- `src/main-screen/MainScreenApp.tsx:343-375` (keyboard handlers)
- `src/shared/tokens.css:78` (font stack)
- `src/shared/feedTypes.ts:226,256` (hardcoded `~/.config/cortado/harness/` in descriptions)
- `src/shared/utils.ts` (or new `platform.ts`)
- `src/App.tsx:158` (`invoke("init_panel")`)
- `src/main-screen/MainScreenApp.tsx:243` (`invoke("init_main_screen_panel")`)
