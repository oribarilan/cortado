---
status: done
---

# Offline detection and per-feed disconnected state

## Goal

Detect when the network is down, pause network-based feed polling, and show a clear "Disconnected" state on each affected feed. Local feeds (file-watching/harness) are unaffected. A single "Retry" action in the footer lets the user trigger an immediate connectivity check.

## Design decisions

### Detection mechanism
Hybrid: infer from feed failures, then confirm with a lightweight ping.

1. Track consecutive poll failures per network-based feed. Harness/file-watching feeds are excluded (they don't use the network).
2. When **all** network-based feeds have **2 consecutive failures**, trigger a connectivity check.
3. Connectivity check: `HEAD https://clients3.google.com/generate_204` with exponential backoff (e.g., 5s → 10s → 20s → 40s → 80s → cap at 120s).
4. If the ping fails, the app enters offline state. If it succeeds, the failure was transient -- reset failure counters and resume normally.

### Offline behavior
- **Pause polling**: stop network feed poll timers while offline. Resume when connectivity is restored.
- **Per-feed display (Variant C -- status in header)**: the feed header itself carries the indicator -- feed name dims to `--text-tertiary` and a muted "disconnected" label replaces the count badge (right-aligned). The content area below is empty (no text, no banner). Local feeds (copilot, opencode, future file-watching) continue working normally and are visually unaffected.
- **Retry button**:
  - Tray panel: a `footer-row` with accent-colored text: "↻ Retry connection"
  - Main screen: footer hints area replaced with "No connection · Retry" where "Retry" is a teal link
- **Auto-recovery**: the exponential backoff ping continues in the background. When it succeeds, stop pinging, resume feed polling, restore normal feed headers.
- The ping mechanism only runs while offline -- it stops as soon as connectivity is confirmed and doesn't restart until the next time all feeds fail again.

### Per-feed errors vs. disconnected
- While offline, per-feed errors are replaced with the header-level "disconnected" label -- no raw error messages like "DNS resolution failed".
- When back online, if a specific feed still fails, its individual error re-appears normally.

### Design reference
- Variant C (status in header) was selected. Design decisions are captured above.

## Acceptance criteria

- [ ] App detects when there is no internet connectivity (2 consecutive failures across all network feeds → ping check)
- [ ] Network feed headers dim and show a "disconnected" label replacing the count badge while offline
- [ ] Local/file-watching feeds are unaffected
- [ ] A single "Retry" action in the panel footer triggers connectivity check + re-poll
- [ ] Disconnected state clears automatically when connectivity is restored
- [ ] Network feeds resume normal polling when back online
- [ ] `just check` passes

## Relevant files

- `src/styles.css` -- panel footer (`.panel-footer`)
- `src/main-screen/main-screen.css` -- main-screen footer (`.ms-footer`)
- `src-tauri/src/tray_icon.rs` -- tray icon rendering (status dot)
- `src-tauri/src/feed/runtime.rs` -- poll loop, error handling
