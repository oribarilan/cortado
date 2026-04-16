# repo-picker

## Context

Adding a GitHub feed currently requires typing `owner/repo` manually. Most users want to watch repos they already contribute to. A multi-select repo picker that lists their repos would streamline the common case dramatically — and we can still offer manual entry for arbitrary repos.

**Value delivered**: Users can set up GitHub feeds in seconds by selecting from their own repos instead of remembering and typing exact `owner/repo` strings.

## Related Files

- `src/shared/feedTypes.ts` — feed catalog, field definitions
- `src/settings/SettingsApp.tsx` — feed edit form, field rendering
- `src-tauri/src/command.rs` — Tauri commands
- `src-tauri/src/feed/config.rs` — feed config parsing
- `src-tauri/src/feed/github_pr.rs` — GitHub PR feed
- `src-tauri/src/feed/github_actions.rs` — GitHub Actions feed

## Dependencies

- None

## Design Exploration

### Repo field UX

**Current**: Single text input for `owner/repo`.

**Proposed**: Two-mode repo field:

1. **"My repos" mode** (default): A searchable, multi-select list of repos the user contributes to. Fetched via `gh repo list --json nameWithOwner,description --limit 100` (includes repos where user is owner/collaborator/org member). User checks the repos they want to watch. Each selected repo becomes a separate feed (since feeds are per-repo today).

2. **"Any repo" mode**: Falls back to the current manual text input for `owner/repo`. For watching public repos or repos not in the user's list.

A segmented control or toggle switches between the two modes.

### Multi-select → multiple feeds

Since each feed config maps to exactly one repo, selecting 3 repos in the picker should create 3 separate feed entries. The UI should make this clear — e.g., "3 feeds will be created" confirmation. The name for each feed auto-generates from the `defaultNamePattern` (e.g., `"{repo} PRs"`).

**Save flow impact**: The current save flow handles one feed at a time. Multi-select requires creating N feed configs in a single save action. This means `save_feeds_config` (or the frontend state around it) needs to handle batch creation — inserting multiple `[[feed]]` entries into the TOML and updating the feed list state accordingly.

### Edit flow

The multi-select picker only applies when **adding** new feeds. When **editing** an existing feed, the repo field should show the current repo as a read-only label or single-value display (not a multi-select picker) — the user is editing one feed, not creating new ones. To change the repo on an existing feed, the user edits the text value directly (or deletes and recreates).

### Backend: new Tauri command

A new command `list_github_repos` that runs `gh repo list --json nameWithOwner,description --limit 200` and returns the parsed list. Includes repos where the user is owner or org member. Collaborator repos (added to someone else's repo) require a separate `gh api /user/repos?affiliation=collaborator` call — include these too for completeness.

### Shared settings for multi-select

When the user selects multiple repos, all created feeds inherit the settings from the current form (interval, user filter, retain, notifications). Users can edit individual feeds afterward if needed.

### Repo list display

Show the first page of results sorted by recency (most recently pushed first). Search queries against all repos, not just the visible page. This keeps the initial list manageable while still providing access to everything.

### Caching

Cache the repo list in frontend state for the duration the settings window is open. No persistent cache — repos can change, and the call is fast enough (~1-2s).

### Error handling

- `gh` not installed → show dependency banner (already exists)
- Not authenticated → show "Run `gh auth login` first" message
- API error → show inline error, fall back to manual input

## Acceptance Criteria

- [ ] GitHub feed type edit form shows a repo mode selector: "My repos" vs "Any repo"
- [ ] "My repos" mode fetches and displays a searchable list of the user's repos (owner, collaborator, org member)
- [ ] User can select multiple repos; each selection will create a separate feed
- [ ] "Any repo" mode shows the current manual `owner/repo` text input
- [ ] A new Tauri command `list_github_repos` exists and returns repo list via `gh` CLI
- [ ] Repo list is cached in frontend state while settings window is open
- [ ] Error states (gh missing, auth failure, API error) are handled gracefully with user-visible messages
- [ ] Editing an existing feed shows the repo as a single value (not multi-select)
- [ ] Save flow handles batch creation of multiple feeds from multi-select
- [ ] Existing feeds with manually-entered repos continue to work unchanged
- [ ] `just check` passes

## Verification

- **Ad-hoc**: Open settings → add GitHub PR feed → verify "My repos" mode shows repos → select 2 → verify 2 feeds created with correct names → switch to "Any repo" → verify manual input works
- **Ad-hoc**: Disconnect network or use invalid token → verify error message appears and manual fallback works

## Notes

- `gh repo list` returns repos where user has push access by default. Collaborator repos need a separate `gh api /user/repos?affiliation=collaborator` call.
- Org-scoped listing (e.g., `gh repo list <org>`) could be a follow-up.
- The repo picker component is shared between GitHub PR and GitHub Actions feed types — both use the same `repo` field.
