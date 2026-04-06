---
status: done
---

# Registry and Tauri wiring

## Goal

Create a `FeedRegistry` that holds active feeds, wire it into the Tauri app as managed state, and expose a `list_feeds` command the frontend can invoke.

## Acceptance criteria

- [ ] `FeedRegistry` struct in `feed/mod.rs` with `register()` and `poll_all()` methods
- [ ] Registry is wrapped in `Arc<Mutex<...>>` (or `Arc<RwLock<...>>`) and added via `app.manage()`
- [ ] `list_feeds` Tauri command returns `Vec<FeedSnapshot>` (JSON-serializable)
- [ ] On startup, config is loaded, feeds are instantiated from config, and registered
- [ ] Unknown feed types in config are included as errored feeds (config error shown in UI), not silently skipped
- [ ] Feeds with bad config (missing required fields) are included as errored feeds with the error message
- [ ] `FeedSnapshot` supports an error state (e.g., `error: Option<String>`) so the frontend can render it
- [ ] If no config file exists, the registry starts empty
- [ ] `just check` passes

## Notes

- The registry construction flow: `load_feeds_config()` → iterate configs → match on `type` to instantiate `GithubPrFeed` or `ShellFeed` → register each.
- `poll_all()` calls `poll()` on each feed and collects `FeedSnapshot` results.
- For now, `list_feeds` does a synchronous poll on invoke (no background loop). Background polling is a sprint 02 concern.
- The `list_feeds` command needs the registry from Tauri state: `State<Arc<Mutex<FeedRegistry>>>`.

## Relevant files

- `src-tauri/src/feed/mod.rs` -- add registry
- `src-tauri/src/command.rs` -- add `list_feeds` command
- `src-tauri/src/main.rs` -- load config, build registry, manage state, register command
