---
status: pending
---

# Fix scrollbar styling (tray + settings)

## Problem

The tray and settings views used to have a sleek, dark scrollbar that matched the app's style. At some point it regressed to a default OS/browser scrollbar -- light-colored, vanilla-looking, and visually out of place even in dark theme.

## Goal

Add custom scrollbar styling that matches the app's dark theme. Ensure it looks correct in tray, main-screen, and settings views.

## Findings

**No custom scrollbar CSS has ever existed in this repo.** Git history has zero matches for `::-webkit-scrollbar`, `scrollbar-width`, or `scrollbar-color`. The "nice dark scrollbar" was likely a previous macOS/WebKit default that changed with a Tauri or WebKit update.

### Scrollable containers that need styling

| View        | Selector          | File                              |
|-------------|-------------------|-----------------------------------|
| Tray/panel  | `.panel-content`  | `src/styles.css`                  |
| Main-screen | `.ms-list`        | `src/main-screen/main-screen.css` |
| Main-screen | `.ms-detail`      | `src/main-screen/main-screen.css` |
| Settings    | `.settings-main`  | `src/settings/settings.css`       |

### Design tokens (from `src/shared/tokens.css`)

The app uses oklch colors in a dark theme:
- `--surface`: `oklch(22% 0.008 250 / 0.93)` (dark blue-gray)
- `--surface-inset`: `oklch(20% 0.012 250)` (slightly darker)
- `--text-tertiary`: `oklch(56% 0.01 250)` (muted gray)
- `--border`: `oklch(33% 0.01 250 / 0.78)`
- `--hover`: `oklch(100% 0 0 / 0.08)` (subtle white overlay)

### Research: modern CSS approaches

Two viable approaches, both should be used together for compatibility:

**1. Standard properties (Firefox, Chrome 121+, Edge, Safari)**
```css
scrollbar-width: thin;
scrollbar-color: <thumb> <track>;
```

**2. WebKit pseudo-elements (Chrome, Safari, older WebKit)**
```css
::-webkit-scrollbar { width: Npx; }
::-webkit-scrollbar-thumb { background: ...; border-radius: ...; }
::-webkit-scrollbar-track { background: ...; }
```

Since Tauri uses WebKit on macOS, the `::-webkit-scrollbar` approach is likely the primary one. The standard properties should be included as well for forward-compatibility.

## Workflow

1. **Reproduce** -- build a standalone HTML page that mimics the app's dark theme (using the actual design tokens) with scrollable content, showing the default ugly scrollbar.
2. **Variations** -- create multiple scrollbar styling alternatives in the HTML page:
   - A: Thin subtle (near-invisible track, muted thumb)
   - B: Standard dark (visible track, contrasting thumb)
   - C: Overlay-style (transparent track, thumb appears on hover only)
   - D: Accent-tinted (thumb uses `--accent` color family)
3. **Test with Playwright** -- open the HTML page, take screenshots of each variation to verify rendering.
4. **Showcase** -- use the `showcase` skill to present the alternatives side by side for user to pick.
5. **Implement** -- apply the chosen style to `src/shared/tokens.css` (so it's shared across all views) targeting the scrollable containers.
