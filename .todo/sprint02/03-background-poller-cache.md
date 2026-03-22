---
status: pending
---

# Background poller and snapshot cache

## Goal

Introduce a background polling engine that continuously updates feed snapshots in memory, with startup cache seeding and per-feed intervals.

## Acceptance criteria

- [ ] Polling state is separated from one-shot command invocation (dedicated poller/cache state type).
- [ ] On app startup, feeds are polled once to seed cache before regular intervals.
- [ ] Startup seed is best-effort and bounded: initialization should not block forever waiting for slow feeds.
- [ ] Each feed is polled on its configured/default interval in the background.
- [ ] Cached snapshots are exposed for read access without re-polling all feeds on every request.
- [ ] `list_feeds` returns cached snapshots; it must not trigger synchronous poll-all execution.
- [ ] Poll failures preserve last known activities for that feed while surfacing `error`.
- [ ] Poller updates avoid data races and maintain consistent snapshot shape.
- [ ] `just check` passes.

## Notes

- Keep config loading behavior unchanged (startup load only; no file watcher in this sprint).
- Preserve per-feed config-error entries in cache output.

## Relevant files

- `src-tauri/src/feed/mod.rs`
- `src-tauri/src/command.rs`
- `src-tauri/src/main.rs`
