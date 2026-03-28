# UX Design — Style System

This document captures the design decisions, motivation, and reasoning behind Cortado's style system.

## Design Token System

### Why semantic tokens

Token names describe their *purpose*, not their visual value. `--text-primary` says "this is the main readable text color" — it doesn't say "this is 92% lightness oklch." This lets us:

- Swap palettes (themes) without touching component CSS.
- Reason about intent: "should this label be `--text-secondary` or `--text-tertiary`?" is a UX question, not a color question.
- Add new themes later (e.g., high-contrast) by only editing `tokens.css`.

### Why a shared `tokens.css`

Before sprint 09, the three CSS files (`styles.css`, `main-screen.css`, `settings.css`) each defined their own color tokens with different names. Panel screens used `--panel-text`, settings used `--t1`. The values were mostly identical but triple-maintained. Extracting them into one file eliminates drift and makes the palette a single source of truth.

### Token inventory

| Category | Tokens |
|----------|--------|
| Surfaces | `--surface`, `--surface-raised`, `--surface-inset` |
| Text | `--text-primary`, `--text-secondary`, `--text-tertiary` |
| Chrome | `--border`, `--separator`, `--hover`, `--expanded-bg` |
| Status | `--status-attention-negative`, `--status-waiting`, `--status-running`, `--status-attention-positive`, `--status-idle` |
| Accent | `--accent`, `--accent-dim`, `--accent-soft`, `--danger` |
| Type scale | `--text-2xs` (~9px), `--text-xs` (~10px), `--text-sm` (~11px), `--text-base` (~13px), `--text-lg` (~14px) |
| Weights | `--font-normal` (400), `--font-medium` (500), `--font-semibold` (600) |
| Spacing | `--space-1` (2px) through `--space-9` (24px) |
| Radii | `--radius-sm` (3px), `--radius-md` (5px), `--radius-lg` (10px), `--radius-full` (999px) |

## Typography

### Why Space Grotesk

Space Grotesk is a proportional sans-serif with a geometric, technical feel that matches Cortado's developer-tool identity. It's distinctive enough to give the app personality without sacrificing readability at small sizes. The fallback stack (`SF Pro Text`, system-ui) ensures graceful degradation.

### Type scale

The scale uses `rem` units so all text sizes scale proportionally when the user changes the text-size setting. Steps were chosen to cover the range of sizes used across the app (9px–14px at default) with minimal visual disruption when snapping existing hardcoded sizes to the nearest step.

## Color Palette

### Dark-first baseline

`:root` defines dark-mode colors. Light-mode colors are applied via `[data-theme="light"]` and `@media (prefers-color-scheme: light)` (when not forced dark). Rationale: two of three original CSS files already defaulted dark, and macOS developer tools are predominantly dark-themed.

### Surface hierarchy

- `--surface` — the main background of any screen or panel.
- `--surface-raised` — elements sitting above the surface (sidebar, cards). Slightly lighter in dark mode, slightly darker in light mode.
- `--surface-inset` — elements recessed into raised surfaces (inner card areas, input backgrounds).

### Status colors

Status colors map directly to the `StatusKind` enum defined in `specs/status.md`. They are used for status dots, chips, field values, and notification badges. Each color is tuned for readability against both dark and light surfaces.

### Accent colors

The accent hue (oklch hue ~178, teal) is used for interactive elements: links, active states, toggles, and focus rings. Three levels of intensity:

- `--accent` — full strength, for text and active indicators.
- `--accent-dim` — medium, for button backgrounds and borders.
- `--accent-soft` — subtle, for selected-state backgrounds.

## Theme System

### Mechanism

A `data-theme` attribute on `<html>` controls the active theme:

- `data-theme="system"` (or absent) — CSS media queries control light/dark based on OS preference.
- `data-theme="dark"` — always dark, overrides OS setting.
- `data-theme="light"` — always light, overrides OS setting.

The CSS uses `:root` for dark tokens, `@media (prefers-color-scheme: light)` on `:root:not([data-theme="dark"])` for system-responsive light, and `[data-theme="light"]` for forced light.

### Cross-window propagation

When the user changes theme or text size in Settings, the backend emits an `appearance-changed` event. All windows listen for this event via the `useAppearance` hook and update their `data-theme` and `data-text-size` attributes immediately — no restart required.

## Text Size Scaling

### Approach

Text size is controlled by scaling the root `font-size` on `<html>`:

| Level | Root size |
|-------|-----------|
| S | 12px |
| M (default) | 13px |
| L | 14px |
| XL | 15px |

Because the type scale uses `rem`, all text sizes scale proportionally. Spacing tokens remain in `px` because panel dimensions are fixed pixel values and shouldn't shift with text size.

### Why root font-size scaling

Considered per-token multipliers but rejected them: they require touching every token, risk spacing mismatches, and add maintenance burden. Root scaling is one line of CSS per level and naturally scales everything defined in `rem`.

## Spacing Scale

A numeric scale (`--space-1` through `--space-9`) covering 2px to 24px. The steps were derived from an audit of all padding, margin, and gap values in the codebase, covering the most common values with clean multiples. Values that don't map to a scale step (3px, 5px, 7px, etc.) remain as hardcoded `px` — forcing everything onto the scale would distort layout.

## Border Radius Scale

Four levels covering the range of roundness in the app:

- `--radius-sm` (3px) — buttons, input fields, inline elements.
- `--radius-md` (5px) — cards, panels, form controls.
- `--radius-lg` (10px) — root panel containers (standardized from 10px/12px to 10px).
- `--radius-full` (999px) — pills, dots, fully rounded elements.

The panel root radius was standardized to 10px (previously 10px for menubar, 12px for main screen) for visual consistency.
