---
sprint: 9
theme: Style System Overhaul
status: pending
---

# Sprint 09 — Style System Overhaul

## Theme

Comprehensive style system cleanup: unified design tokens, shared CSS file, consistent theming (light/dark/system), text-size control, normalized typography (Space Grotesk everywhere), spacing scale, and border-radius scale. Every static style concern addressed in one sprint.

## Decisions

- **Font**: Space Grotesk everywhere, loaded via Google Fonts CDN in all 3 HTML files.
- **Token naming**: Semantic (`--surface`, `--text-primary`, `--status-waiting`, etc.).
- **Theme baseline**: Dark-first in `:root`.
- **Theme control**: Segmented control — labels only (Light / Dark / System). Default: System.
- **Text size control**: Segmented control — labels only (S / M / L / XL). Default: M.
- **Text size mechanism**: Scale root `font-size` (S=12px, M=13px, L=14px, XL=15px). Type scale uses `rem`.
- **Spacing tokens**: Stay in `px` (panels have fixed pixel dimensions).
- **Panel root border-radius**: Standardized to 10px (`--radius-lg`).
- **Granular font sizes**: Snap to nearest type-scale step (accept slight visual shifts).
- **Animations**: Out of scope — `@keyframes` and `prefers-reduced-motion` stay in screen CSS files.

## Task Sequence

1. **01-design-tokens** — Font loading + shared `tokens.css` with all design tokens and theme/text-size mechanism.
2. **02-css-panel** — Normalize menubar panel CSS to use shared tokens.
3. **03-css-main-screen** — Normalize main screen CSS to use shared tokens.
4. **04-css-settings** — Normalize settings CSS to use shared tokens (most complex — 8 component-level light overrides).
5. **05-backend-settings** — Add `theme` and `text_size` fields to `AppSettings`, emit events.
6. **06-frontend-wiring** — Shared `useAppearance` hook, wire all windows.
7. **07-settings-ui** — Theme + text size segmented controls in Settings > General.

Tasks 02–04 are sequential (easiest → hardest). Task 05 can run in parallel with 02–04. Tasks 06–07 depend on 05.

## Notes

- Space Grotesk is currently **not loaded** in the app — only referenced in `settings.css`. The showcases load it via CDN. Task 01 fixes this.
- Settings CSS has **8 component-level** `@media (prefers-color-scheme: light)` overrides beyond the root block. All must be reworked in task 04.
- Panel and main-screen share identical color values — clean extraction confirmed.
- Settings uses a different token naming scheme (`--bg`, `--s`, `--t1`, `--ac`) — task 04 maps these to unified names.
