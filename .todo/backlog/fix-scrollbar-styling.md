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

## Failed attempt (2026-03-29)

Applied variation A (thin subtle, 6px, transparent track, muted thumb) to `src/shared/tokens.css` using `*::-webkit-scrollbar` selectors with hardcoded oklch values. Both `scrollbar-width: thin` / `scrollbar-color` (standard) and `::-webkit-scrollbar` pseudo-elements (WebKit) were used.

**What was tried:**
1. First attempt used CSS custom properties (`var(--scrollbar-thumb)`) -- did not work because `var()` is silently ignored inside `::-webkit-scrollbar` pseudo-elements in WebKit/WKWebView.
2. Second attempt used hardcoded oklch color literals -- still did not produce a visible scrollbar in the actual Tauri app.

**Possible explanations to investigate next:**
- Tauri's WKWebView may not support `::-webkit-scrollbar` pseudo-elements at all (some WKWebView configurations strip them).
- macOS system-level "Show scroll bars" preference (System Settings > Appearance) may override CSS scrollbar styling in WKWebView.
- The `backdrop-filter` / vibrancy layer on the panel may interfere with scrollbar rendering.
- May need a Tauri plugin or native Swift/ObjC approach to style scrollbars in WKWebView.
- May need to use a JS-based scrollbar library (e.g., OverlayScrollbars, SimpleBar) instead of pure CSS.

## Workflow (for next attempt)

1. Investigate whether WKWebView in Tauri actually respects `::-webkit-scrollbar` -- test with a minimal Tauri app or check Tauri/WebKit issue trackers.
2. If CSS approach is dead, evaluate JS scrollbar libraries (ask user before adding deps).
3. If native approach is needed, look into NSScrollView styling via Tauri's native API access.
4. Test the chosen approach in-app before considering done.
