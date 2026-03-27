---
status: done
---

# 11 -- No-emoji policy and symbol system

## Goal

Ban emoji from the codebase and establish a consistent symbol system using plain Unicode text symbols.

## Context

The codebase currently uses a mix of emoji (🔔) and Unicode symbols (⚙, ◉, ▸, ⚠). Emoji render inconsistently across OS versions and themes, and look out of place in a developer tool. Unicode text symbols are monochrome, theme-aware, and consistent.

## Recommendation: Plain Unicode text symbols (no library)

**Why not SF Symbols?** SF Symbols require an NSImage render pipeline and are not directly usable in web views. Tauri's webview can't reference SF Symbols by name without a native bridge.

**Why not an icon library (lucide, heroicons)?** Adds a dependency, increases bundle size, and the project only uses ~15 symbols total. Overkill.

**Why plain Unicode?** Already in use for most symbols (⚙, ◉, ▸, ⚠, ↗, ✓, ✕). The only outlier is the 🔔 emoji. Replacing it with a Unicode bell character keeps the system consistent and zero-dependency.

### Symbol inventory

| Use | Current | Replacement |
|-----|---------|-------------|
| General nav | ⚙ | ⚙ (keep) |
| Feeds nav | ◉ | ◉ (keep) |
| Notifications nav | 🔔 | ♪ or ◬ or ⏣ |
| Warning | ⚠ | ⚠ (keep) |
| Chevron | ▸ | ▸ (keep) |
| Open external | ↗ | ↗ (keep) |
| Success | ✓ | ✓ (keep) |
| Failure | ✕ | ✕ (keep) |

Best replacement for 🔔: Use `⏣` (benzene ring — geometric, neutral) or simply the text "Notifications" with no icon prefix. Alternatively `▣` (square with fill) to match the geometric style of ⚙ and ◉.

## Acceptance criteria

- [ ] AGENTS.md updated: "No emoji in code or UI. Use plain Unicode text symbols for icons."
- [ ] 🔔 replaced with a non-emoji Unicode symbol in SettingsApp.tsx
- [ ] No emoji characters remain anywhere in `src/`
- [ ] `just check` passes
