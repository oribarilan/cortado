---
status: done
---

# Background poller and snapshot cache

## Goal

Introduce a background polling engine that continuously updates feed snapshots in memory, with startup cache seeding and per-feed intervals.

## Acceptance criteria

- [x] Polling state is separated from one-shot command invocation (dedicated poller/cache state type).
- [x] On app startup, feeds are polled once to seed cache before regular intervals.
- [x] Startup seed is best-effort and bounded: initialization should not block forever waiting for slow feeds.
- [x] Each feed is polled on its configured/default interval in the background.
- [x] Cached snapshots are exposed for read access without re-polling all feeds on every request.
- [x] `list_feeds` returns cached snapshots; it must not trigger synchronous poll-all execution.
- [x] Poll failures preserve last known activities for that feed while surfacing `error`.
- [x] Poller updates avoid data races and maintain consistent snapshot shape.
- [x] `just check` passes.

## Notes

- Keep config loading behavior unchanged (startup load only; no file watcher in this sprint).
- Preserve per-feed config-error entries in cache output.

## Relevant files

- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/command.rs`
- `src-tauri/src/main.rs`
