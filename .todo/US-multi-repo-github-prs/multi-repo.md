# multi-repo

## Context

GitHub feeds currently support a single `repo` per feed. Watching 3 repos means creating 3 separate feeds. Supporting multiple repos per feed simplifies configuration and enables the upcoming repo picker UI.

**Value delivered**: Users can watch multiple repos in a single feed, reducing feed clutter and simplifying management.

## Related Files

- `src-tauri/src/feed/config.rs` — feed config parsing (`repo` field)
- `src-tauri/src/feed/github_pr.rs` — GitHub PR feed (polls single repo today)
- `src-tauri/src/feed/github_actions.rs` — GitHub Actions feed (polls single repo today)
- `src-tauri/src/settings_config.rs` — feed config serialization/deserialization
- `src/shared/feedTypes.ts` — feed catalog field definitions
- `src/shared/types.ts` — frontend types

## Dependencies

- None

## Design

### Config format

```toml
# New format
[[feed]]
name = "My PRs"
type = "github-pr"
repos = ["myorg/api", "myorg/frontend", "personal/dotfiles"]

# Old format still supported (backward compat)
[[feed]]
name = "Legacy PR"
type = "github-pr"
repo = "myorg/api"
```

Both `repo` (single string) and `repos` (array) are accepted. Internally, `repo` is normalized to a single-element `repos` vec. If both are present, that's a config parse error.

### Backend changes

- `config.rs`: Parse `repos` as `Vec<String>`, fall back to `repo` as `vec![single]`. At least one repo required.
- `github_pr.rs`: `poll()` iterates over `repos`, runs `gh pr list` for each, and merges results. Activity IDs always include the repo prefix (e.g., `owner/name:pr:42`), regardless of how many repos are configured. This means existing single-repo feeds will get a one-time state reset on upgrade (read/seen state lost), but avoids a worse problem: conditional namespacing would break all activity state whenever a user adds a second repo to an existing feed.
- `github_actions.rs`: Same pattern — iterate, poll, merge, always-namespaced IDs.
- `settings_config.rs`: Always serialize as `repos = [...]` array. Read both `repo` and `repos` formats.

### Feed naming

- 1 repo → auto-name from `defaultNamePattern` (e.g., `"{repo} PRs"`)
- 2+ repos → generic default like `"GitHub PRs"` (user can rename)

Frontend `defaultNamePattern` logic needs adjustment for the multi-repo case (this is a minor frontend change that belongs here since it's tied to the model).

### Activity display

Activities from different repos show which repo they belong to. This likely means adding the repo as a field or subtitle on the activity. The exact display is a frontend concern but the backend must include repo info in the activity data.

### Scope

GitHub-only. ADO PR feed stays single-repo.

## Acceptance Criteria

- [ ] `config.rs` parses `repos = [...]` array field; falls back to `repo = "..."` as single-element vec
- [ ] Error if both `repo` and `repos` are present in the same feed config
- [ ] At least one repo is required; empty `repos = []` is an error
- [ ] `github_pr.rs` polls all repos and merges activities; activity IDs always include repo prefix (e.g., `owner/name:pr:42`)
- [ ] `github_actions.rs` polls all repos and merges activities; same always-namespaced IDs
- [ ] `settings_config.rs` serializes as `repos = [...]`; reads both old and new formats
- [ ] Activities include repo info so the frontend can display which repo they belong to
- [ ] Existing feeds with single `repo` field continue to work unchanged
- [ ] Frontend types and feed catalog updated for `repos` array field
- [ ] `just check` passes

## Verification

- **Automated**: Unit tests for config parsing (single `repo`, `repos` array, both present → error, empty array → error)
- **Automated**: Unit tests for multi-repo poll merging and activity ID uniqueness
- **Ad-hoc**: Create a feed with `repos = ["owner/a", "owner/b"]` in TOML → verify both repos' activities appear
- **Ad-hoc**: Existing config with `repo = "owner/a"` → verify it loads and works
