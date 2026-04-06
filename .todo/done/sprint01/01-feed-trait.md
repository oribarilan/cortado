---
status: done
---

# Feed trait and core types

## Goal

Define the foundational Rust types for the feed system and add the new crate dependencies. After this task, other tasks can import and implement the `Feed` trait.

## Acceptance criteria

- [ ] `src-tauri/src/feed/mod.rs` exists with: `Feed` trait, `Activity`, `Field`, `FieldValue`, `FieldDefinition`, `FeedSnapshot` types
- [ ] `FieldValue` enum has variants: `Text`, `Status` (with severity), `Number`, `Url`
- [ ] `Feed` trait has: `name()`, `feed_type()`, `provided_fields()`, `poll()` (async)
- [ ] `FeedSnapshot` is serializable (Serde) for sending to frontend, includes `error: Option<String>` for config/poll errors
- [ ] Rust deps added to `Cargo.toml`: `toml`, `async-trait`, `anyhow`, `dirs`
- [ ] `mod feed;` declared in `main.rs`
- [ ] `just check` passes

## Notes

- The `Feed` trait uses `async_trait` for async poll support with dyn dispatch.
- `StatusKind` enum: `success`, `warning`, `error`, `pending`, `neutral`.
- Keep types minimal -- don't add registry, config, or implementations here.
- All types that cross the Tauri boundary need `Serialize`. Types only used in Rust need `Deserialize` too if they come from config.

## Relevant files

- `src-tauri/Cargo.toml` -- add deps
- `src-tauri/src/feed/mod.rs` -- new file
- `src-tauri/src/main.rs` -- add `mod feed`
