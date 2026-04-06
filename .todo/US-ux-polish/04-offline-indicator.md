---
status: pending
---

# Offline detection and subtle connectivity indicator

## Goal

Detect offline state and show a single, subtle connectivity indicator in the panel and tray — instead of N separate feed errors. The indicator should be unobtrusive and disappear automatically when connectivity is restored.

## Design ideas

- A small status line in the panel footer (e.g., "Offline" with a muted icon) that replaces or sits alongside the existing footer content.
- In the tray, a single top-level item like "Offline -- feeds paused" instead of per-feed errors.
- Feeds should suppress their individual poll errors while offline, or at least de-emphasize them.
- When connectivity returns, the indicator disappears and feeds resume polling normally.

## Open questions

- How to detect offline? Options: periodic ping, infer from all feeds failing simultaneously, or use macOS network reachability APIs.
- Should feeds pause polling while offline to avoid wasted work, or keep polling and suppress errors?
- Should there be a manual "retry now" action?
- Interaction with the inline error experience task (`inline-error-experience.md`) — offline indicator should take precedence over per-feed errors.

## Acceptance criteria

- [ ] App detects when there is no internet connectivity
- [ ] Panel shows a single subtle offline indicator instead of per-feed errors
- [ ] Tray shows a single offline indicator instead of per-feed errors
- [ ] Individual feed errors are suppressed or de-emphasized while offline
- [ ] Indicator disappears automatically when connectivity is restored
- [ ] Feeds resume normal polling when back online
- [ ] `just check` passes

## Relevant files

- `src/styles.css` — panel footer (`.panel-footer`)
- `src/main-screen/main-screen.css` — main-screen footer (`.ms-footer`)
- `src-tauri/src/tray.rs` — tray menu construction
- `src-tauri/src/feed/runtime.rs` — poll loop, error handling
