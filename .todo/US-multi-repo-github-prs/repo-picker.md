# repo-picker

## Context

GitHub feeds now support multiple repos (via `multi-repo` task). The repo field is still a manual text input. A repo picker that lists the user's repos would streamline the common case.

**Value delivered**: Users can select repos from a list instead of typing `owner/repo` strings manually.

## Related Files

- `src/shared/feedTypes.ts` — feed catalog, field definitions
- `src/settings/SettingsApp.tsx` — feed edit form, field rendering
- `src-tauri/src/command.rs` — Tauri commands

## Dependencies

- `multi-repo.md` (backend must support `repos` array before the picker can populate it)

## Design

### Repo field UX

Two-mode repo field via segmented control ("My repos" / "Any repo"):

1. **"My repos" mode** (default when adding): A searchable, multi-select list of repos the user contributes to. Fetched via `gh repo list --json nameWithOwner,description --limit 200`. Sorted by recency. Unselected items show no checkbox — only selected items show a filled teal checkbox.

2. **"Any repo" mode**: A text input for `owner/repo` with an Add button (Enter also works). Added repos appear as removable chips/tags below the input (with x to remove). Input clears after adding. Supports adding multiple arbitrary repos.

Both modes' selections combine into the feed's `repos` list. Selections persist when switching between tabs. Duplicates are silently prevented (selecting a repo in "My repos" that's already a chip in "Any repo" is a no-op, and vice versa). A summary at the bottom shows the count (e.g., "Watching 5 repos").

### Edit flow

When editing an existing feed, the repo field shows the current repos as editable chips (removable) and allows adding more via the same picker/manual modes.

### User filter

The user filter keeps its current 3-segment control: All / Me / User (manual text input). No "Contributors" mode — with multi-repo feeds, scoping contributors to a specific repo adds complexity without enough value.

### Backend: new Tauri command

`list_github_repos` — runs `gh repo list --json nameWithOwner,description --limit 200` and returns the parsed list. Includes repos where the user is owner or org member. The 200 limit covers the vast majority of users; for users in large orgs, the "Any repo" manual entry mode serves as fallback.

### Repo list display

Show the first page of results sorted by recency (most recently pushed first). Search queries against all repos, not just the visible page.

### Caching

Cache the repo list in frontend state for the duration the settings window is open. No persistent cache.

### Loading and error robustness

- `gh` not installed → show dependency banner (already exists)
- Not authenticated → show "Run `gh auth login` first" message
- API error → show inline error with "Enter repo manually instead" fallback link
- Loading state → show spinner with "Cancel" link that switches to "Any repo" mode

## Acceptance Criteria

- [ ] Feed edit form shows a repo mode selector: "My repos" vs "Any repo"
- [ ] "My repos" mode fetches and displays a searchable list of the user's repos
- [ ] User can select multiple repos from the list
- [ ] "Any repo" mode shows a text input with Add button; added repos appear as removable chips
- [ ] Selections from both modes persist across tab switches and combine into the feed's `repos`
- [ ] Unselected items in "My repos" list show no checkbox; only selected items show a filled checkbox
- [ ] A new Tauri command `list_github_repos` exists and returns repo list via `gh` CLI
- [ ] Repo list is cached in frontend state while settings window is open
- [ ] Loading state shows spinner with Cancel link that falls back to "Any repo" mode
- [ ] Error states (gh missing, auth failure, API error) show inline error with manual entry fallback
- [ ] Editing an existing feed shows current repos as editable chips with ability to add more
- [ ] `just check` passes

## Verification

- **Ad-hoc**: Open settings → add GitHub PR feed → verify "My repos" mode shows repos → select 3 → switch to "Any repo" → add 1 manually → save → verify config has `repos` with 4 entries
- **Ad-hoc**: Edit existing multi-repo feed → verify chips shown → remove one → add one → save → verify updated
- **Ad-hoc**: Disconnect network → verify error message and manual fallback works
