---
status: done
---

# Panel empty state UI

## Goal

Replace the current "No feeds configured. Add feeds in Settings." text in the panel with a rich, welcoming empty state that includes feed type discovery and deep links to Settings.

## Context

The panel (main-screen, 840x480) currently shows a single line of text when zero feeds are configured. The new empty state should use the split layout (list + detail panes) to show a welcome message with CTA on the left and clickable feed types on the right.

See `showcases/empty-state-showcase.html` variant C2 for the visual reference.

## Acceptance criteria

- [ ] List pane shows: welcome headline, brief feed explanation, CTA button ("+ Add your first feed"), secondary TOML link, hotkey hint
- [ ] Detail pane shows: feed types list with icons and one-line descriptions (all 6 types)
- [ ] CTA button opens Settings to the Feeds section add-feed flow (uses deep-link from task 02)
- [ ] Each feed type card is clickable and opens Settings with that type pre-selected (uses deep-link from task 02)
- [ ] Same welcome copy shown regardless of whether this is first launch or re-entry (user deleted all feeds)
- [ ] Empty state disappears and normal feed list appears as soon as a feed exists (reactive via `feeds-updated` event)
- [ ] Styling matches Cortado design tokens (colors, typography, spacing, radii)
- [ ] Light and dark theme support
- [ ] `prefers-reduced-motion` coverage for any animations

## Layout spec (from showcase C2)

```
+---------------------------+-------------------+
| [list pane]               | [detail pane]     |
|                           |                   |
|     (coffee icon)         | FEED TYPES        |
|  Welcome to Cortado       |                   |
|  A feed tracks a data     | * GitHub PR       |
|  source and surfaces...   | * GitHub Actions   |
|                           | * Azure DevOps PR |
|  [+ Add your first feed]  | * HTTP Health     |
|  or edit feeds.toml       | * Shell           |
|                           | * Copilot Session |
+---------------------------+-------------------+
| kbd hints         v0.x.x  ⚙                  |
+-----------------------------------------------+
```

## Notes

- The existing `ms-empty-state` div in `MainScreenApp.tsx` (lines 334-337) is the replacement target.
- Feed type data (names, icons, descriptions) can be hardcoded in the component -- it mirrors the `FEED_TYPE_LABELS` in SettingsApp but with descriptions added.
- The "or edit ~/.config/cortado/feeds.toml" link should be a plain text button (not a real link -- it's informational).
- No new CSS file needed -- add styles to `main-screen.css`.

## Relevant files

- `src/main-screen/MainScreenApp.tsx` -- replace empty state rendering
- `src/main-screen/main-screen.css` -- empty state styles
- `src-tauri/src/command.rs` -- `open_settings` with deep-link params (from task 02)
