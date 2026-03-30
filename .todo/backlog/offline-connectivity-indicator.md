---
status: pending
---

# Offline detection and subtle connectivity indicator

## Problem

When there's no internet connection, every feed fails its poll independently and displays its own error. This floods the UI with repetitive "network error" messages that look like feed-specific problems, when the real issue is a single global cause: no connectivity.

## Goal

Detect offline state and show a single, subtle connectivity indicator in the panel and tray — instead of N separate feed errors. The indicator should be unobtrusive (not a banner or modal) and disappear automatically when connectivity is restored.

## Design ideas

- A small status line in the panel footer (e.g., "Offline" with a muted icon) that replaces or sits alongside the existing footer content.
- In the tray, a single top-level item like "Offline -- feeds paused" instead of per-feed errors.
- Feeds should suppress their individual poll errors while offline, or at least de-emphasize them.
- When connectivity returns, the indicator disappears and feeds resume polling normally.

## Questions

- How to detect offline? Options: periodic ping, check on poll failure (if all feeds fail simultaneously, infer offline), or use macOS network reachability APIs via Tauri.
- Should feeds pause polling while offline to avoid wasted work, or keep polling and just suppress errors?
- Should there be a manual "retry now" action?
- Interaction with the inline error experience task (`inline-error-experience.md`) — offline indicator should take precedence over per-feed errors when offline.

## Relevant files

- `src/styles.css` — panel footer (`.panel-footer`)
- `src/main-screen/main-screen.css` — main-screen footer (`.ms-footer`)
- `src-tauri/src/tray.rs` — tray menu construction
- `src-tauri/src/feed/runtime.rs` — poll loop, error handling
