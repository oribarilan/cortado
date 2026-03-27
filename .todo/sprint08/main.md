---
sprint: 8
theme: Main Screen — Floating Keyboard-Centric Panel
status: pending
---

# Sprint 08 — Main Screen

## Theme

Build a floating, keyboard-centric main screen that coexists with the existing menubar panel. Opened via a global hotkey (⌘+Shift+Space), it presents a split-panel UI: compact activity list on the left, live detail pane on the right. Activities are navigated with arrow keys, Enter opens the selected activity's URL, Esc dismisses.

Design reference: **Variant 1 (Clean Split)** from `showcases/main-screen-split-drilldown-showcase.html`, with an optional **"Needs Attention" priority section** (Variant 5) that can be toggled on/off.

## Decisions

- **Coexists with menubar panel** — both remain accessible. Tray click opens the menubar dropdown; ⌘+Shift+Space opens the main screen.
- **Separate Tauri window** — own HTML entrypoint (`main-screen.html`), own React root, own NSPanel lifecycle. Shares types/components with the menubar panel via imports.
- **Global hotkey**: ⌘+Shift+Space (toggle — press again to hide).
- **Panel positioning**: Centered on the active monitor.
- **Filter bar**: Deferred to backlog. With priority section + small activity count, ↑↓ is sufficient.
- **Priority section**: "Needs Attention" section at the top of the list showing cross-feed attention-negative items. On by default. Toggleable. Stored in app settings.

## Task Sequence

> Note: task 04 was removed during pre-sprint planning and the gap was kept intentionally to avoid renumbering.

Tasks 01–03 are foundational and sequential. Tasks 05–07 and 10 can be parallelized after 03. Task 08 depends on all prior. Task 09 is independent and can be done anytime.

1. **01-window-setup** — Create the main-screen Tauri window, HTML entrypoint, NSPanel conversion, and global hotkey registration. Includes: adding `tauri-plugin-global-shortcut` dep, `main-screen.html` entrypoint, Vite multi-page input, Tauri window config, NSPanel conversion in a new `main_screen.rs` module (the existing NSPanel logic in `fns.rs` is hardcoded to the `"main"` window — extract shared helpers or create parallel logic). Also: verify two-NSPanel coexistence early — ensure resign-key delegates are scoped to their own window label and don't interfere with each other.

2. **02-list-pane** — Build the left-side activity list. Feed-grouped sections with compact rows (dot + title). Arrow-key navigation with focus tracking. Enter opens the activity URL. Extract shared TypeScript types (`FeedSnapshot`, `Activity`, `Field`, `StatusKind`) and utility functions (`deriveActivityKind`, `supportsOpen`, etc.) from `src/App.tsx` into `src/shared/` so both panels can use them.

3. **03-detail-pane** — Build the right-side detail pane. Shows feed label, title, status chip, field rows, and "Open" link for the focused activity. Updates live as keyboard focus moves through the list.

4. **05-priority-section** — Add the optional "Needs Attention" cross-feed section at the top of the list. Shows activities with `AttentionNegative` status from any feed, with a feed hint label. Toggle stored in app settings (`main_screen.show_priority_section`), accessible from Settings (General section). Includes Rust-side schema changes to `AppSettings` and new Tauri commands to read/write the setting.

5. **06-styling-polish** — Match the Clean Split showcase styling. OKLCH color tokens, vibrancy, blur, dark/light mode, status colors, focus outlines. Ensure visual consistency with the menubar panel's design language.

6. **07-panel-lifecycle** — Polish panel behavior: reset state on show (scroll to top, focus first item), hide on space change, handle multi-monitor correctly, ensure no Dock icon flash.

7. **08-integration** — End-to-end testing. Verify coexistence with menubar panel, hotkey toggle, keyboard nav, priority section toggle, app-mode toggle, open activity, Esc dismiss. Run `just check`.

8. **09-spec-update** — Update `specs/main.md` to document the main screen: hotkey, behavior, priority section, app mode, coexistence with menubar panel.

9. **10-app-mode** — Make the menubar (tray icon + menubar panel) optional via a `show_menubar` setting. App launch/reopen (double-click, Spotlight, `open -a`) always opens the main screen. Settings accessible from the main screen footer. Global hotkey always registered regardless of menubar setting. Includes Rust-side schema changes to `AppSettings` and new Tauri commands to read/write the setting.

## Implementation Notes (from review)

- **`tauri-plugin-global-shortcut`** is NOT currently a dependency — must be added to `Cargo.toml` and registered in the plugin builder chain in `main.rs`.
- **`monitor`** from `tauri-toolkit` IS already available — use for centering on active monitor.
- **NSPanel logic** lives in `fns.rs` (not `panel.rs`). It's hardcoded to the `"main"` window label. Needs refactoring or parallel implementation for the main-screen window.
- **`panel.rs`** handles tray icon/menu wiring, not NSPanel conversion.
- **Vite multi-page** is already set up (`index.html` + `settings.html`). Adding a third input is straightforward.
- **Settings window pattern** (`settings.html` → `src/settings/main.tsx` → `SettingsApp.tsx` + CSS) should be followed for the main-screen.
- **Shared types** don't exist yet — `FeedSnapshot`/`Activity`/`StatusKind` types are duplicated across `App.tsx` and `SettingsApp.tsx`. Task 02 should extract these.
