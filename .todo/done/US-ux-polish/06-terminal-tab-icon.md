---
status: done
---

# Better icon for Settings Terminals tab

## Problem

The Terminals tab in Settings uses `▸` (a right-pointing triangle) as its nav icon. This doesn't convey "terminal" -- it looks like a generic play/expand symbol. Other tabs use more meaningful icons.

## Goal

Replace the `▸` icon with a terminal-appropriate symbol.

## Decision

Use `>_` as plain text -- classic terminal prompt. Stays consistent with the Unicode text approach used by other tabs (`⚙` `◉` `♪`).

## Options

~~- `>_` as text (classic terminal prompt)~~
- ~~A Unicode symbol like `⌨` (keyboard) or similar~~
- ~~A small inline SVG matching the style of other nav icons~~

## Relevant files

- `src/settings/SettingsApp.tsx` -- line ~952, `<span className="settings-nav-icon">▸</span> Terminals`

## Scope Estimate

Tiny
