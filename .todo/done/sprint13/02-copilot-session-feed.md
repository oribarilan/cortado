---
status: done
---

# Harness feed (`HarnessFeed`)

## Goal

Implement the generic `HarnessFeed` — a single `Feed` trait implementation that works with any `HarnessProvider`. The first registered instance uses `CopilotProvider` with feed type `copilot-session`. Future harnesses (Claude Code, etc.) register additional instances with their own providers.

## Architecture

```rust
/// Generic feed that delegates session discovery to a HarnessProvider.
pub struct HarnessFeed {
    name: String,
    provider: Box<dyn HarnessProvider>,
    interval: Duration,
    retain_for: Option<Duration>,
    explicit_overrides: HashMap<String, FieldOverride>,
    config_overrides: HashMap<String, FieldOverride>,
}
```

The `HarnessFeed` implements `Feed` and maps `SessionInfo` (from the provider) into `Activity` (for the UI). All harness feeds share the same fields, status mappings, and activity title format.

## Config

```toml
[[feed]]
name = "copilot sessions"
type = "copilot-session"
```

No type-specific config keys needed for the MVP. The feed type determines which `HarnessProvider` to use.

In `instantiate_feed()`, `"copilot-session"` creates a `HarnessFeed` with `CopilotProvider`. Future types like `"claude-code-session"` would create a `HarnessFeed` with a different provider.

## Auth & preflight

None required. If the session state directory doesn't exist, the provider returns an empty list — the feed shows no activities (not an error).

## Provided fields

| Field    | Type   | Label  | Description                                          |
|----------|--------|--------|------------------------------------------------------|
| `status` | status | Status | Session status (working, idle, question, etc)        |
| `repo`   | text   | Repo   | Repository name (e.g. `oribarilan/cortado`)          |
| `branch` | text   | Branch | Git branch name                                      |

## Status kind mapping

| SessionStatus | Status value  | StatusKind        |
|---------------|---------------|-------------------|
| Working       | `working`     | Running           |
| Question      | `question`    | AttentionPositive |
| Approval      | `approval`    | AttentionPositive |
| Idle          | `idle`        | Idle              |
| Unknown       | `unknown`     | Idle              |

## Activity identity

`SessionInfo.id` (session UUID). Globally unique and stable.

## Activity title

Format: `{short_repo} @ {branch}`

`short_repo` = repo name without owner (e.g., `cortado` from `oribarilan/cortado`). If repo unknown, use last path component of `cwd`. If branch unknown, omit the `@ {branch}` suffix.

## Default interval

`30s`. Local file reads are cheap.

## Acceptance criteria

- [ ] `src-tauri/src/feed/harness/feed.rs` with `HarnessFeed` implementing `Feed`
- [ ] `HarnessFeed::from_config(config, provider)` constructor
- [ ] `poll()` calls `provider.discover_sessions()` and maps to `Vec<Activity>`
- [ ] `poll()` caches last `Vec<SessionInfo>` for `focus_session` lookup
- [ ] All status values mapped to StatusKind per table above
- [ ] Activity title formatted as `{short_repo} @ {branch}`
- [ ] Field overrides supported
- [ ] Registered in `instantiate_feed()` in `mod.rs` as `"copilot-session"` -> `HarnessFeed` with `CopilotProvider`
- [ ] Returns empty activities (not error) when provider finds nothing
- [ ] Feed catalog: new "Coding Agents" provider with `copilot-session` type entry
- [ ] Unit tests: status-to-kind mapping, title formatting, empty sessions, field overrides
- [ ] `specs/main.md` updated with `copilot-session` feed documentation
- [ ] `specs/glossary.md` updated with "Harness" term
- [ ] `just check` passes

## Feed catalog entry

New provider category in `FEED_CATALOG`:

```ts
{
  id: "coding-agents",
  name: "Coding Agents",
  icon: `<svg .../>`,       // TBD — simple terminal/agent icon
  types: [
    {
      feedType: "copilot-session",
      name: "Copilot Sessions",
      description: "Track active GitHub Copilot CLI sessions",
      icon: `<svg .../>`,   // GitHub Copilot icon (plain, no emoji)
      defaultInterval: "30s",
    },
    // Future: claude-code-session, etc.
  ],
}
```

Also update:
- `FeedType` union: add `"copilot-session"`
- `FEED_TYPE_LABELS`: `"copilot-session": "Copilot Session"`
- `FEED_TYPE_FIELDS`: `"copilot-session": []` (no type-specific config keys)
- No `FEED_TYPE_DEPS` entry needed (no external CLI dependency)

## Notes

- The `HarnessFeed` knows nothing about Copilot, YAML, events.jsonl, or lock files. It only knows `SessionInfo`.
- Adding a new harness (e.g., Claude Code) means: write a new provider, register a new feed type in `instantiate_feed()`. Zero changes to `HarnessFeed`.
- `poll()` caches the last `Vec<SessionInfo>` so `focus_session(session_id)` can look up session details between polls. The frontend uses the activity `id` (= session UUID) to call `focus_session`, and the backend resolves it to a `SessionInfo` from the cache.
- No retention (`retain_for: None`). When a session's lock file disappears, the activity vanishes on the next poll.
- `serde-saphyr` dependency is added in task 01.
- Add "Harness" to `specs/glossary.md` during implementation.

## Relevant files

- `src-tauri/src/feed/harness/feed.rs` — new (generic feed impl)
- `src-tauri/src/feed/mod.rs` — register `"copilot-session"` in `instantiate_feed()`
- `specs/main.md` — add feed type documentation
