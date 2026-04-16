# user-picker

## Context

The user filter field currently offers All / Me / manual username entry. For teams, it would be useful to pick from known collaborators or org members rather than typing usernames manually. This mirrors the repo-picker pattern.

**Value delivered**: Users can select team members from a list instead of remembering GitHub usernames.

## Related Files

- `src/shared/feedTypes.ts` — `user-filter` field kind definition
- `src/settings/SettingsApp.tsx` — `UserFilterField` component (lines 207-322)
- `src-tauri/src/command.rs` — Tauri commands

## Dependencies

- `repo-picker.md` (the repo must be known before we can list collaborators for it)

## Design Exploration

### Feasibility analysis

Listing users is repo-scoped (collaborators) or org-scoped (members). Unlike repo listing which is user-global, user listing depends on context:

- **Repo collaborators**: `gh api repos/{owner}/{repo}/collaborators --jq '.[].login'` — requires admin access on private repos, works on public repos.
- **Org members**: `gh api orgs/{org}/members --jq '.[].login'` — requires org membership.
- **Recent actors**: Could parse from `gh pr list` or `gh run list` output — always available but limited.

### Proposed approach

Enhance the existing "User" mode (third segment) to show a dropdown/autocomplete of suggestions:

1. Keep the current 3-segment control: All / Me / User
2. When "User" is selected, instead of a plain text input, show a combo box (text input + dropdown)
3. Dropdown is populated with recent contributors fetched from the repo (e.g., recent PR authors via `gh pr list --json author --limit 50`)
4. User can still type any username (the combo box accepts free text)

This is lower-risk than a full picker since it's additive — the free text input still works exactly as before.

### Backend: new Tauri command

`list_repo_contributors { repo: String }` — runs `gh pr list --repo {repo} --json author --limit 50` and returns deduplicated usernames. This piggybacks on data already available without special permissions.

### When repo isn't known yet

If the user hasn't selected a repo yet, the user field falls back to plain text input (no suggestions). The suggestions populate once a repo is selected.

## Acceptance Criteria

- [ ] "User" segment in user-filter field shows a combo box with autocomplete suggestions
- [ ] Suggestions are populated from recent PR authors / run actors for the selected repo
- [ ] Free text entry still works (user can type any username)
- [ ] When no repo is selected yet, the field falls back to plain text input
- [ ] A new Tauri command `list_repo_contributors` exists and returns deduplicated usernames
- [ ] Error states are handled gracefully (no suggestions shown on error, input still works)
- [ ] `just check` passes

## Verification

- **Ad-hoc**: Add GitHub PR feed → select a repo → click "User" segment → verify dropdown shows recent contributors → type a custom username → verify it's accepted

## Notes

- **This task is optional/stretch.** The collaborators API requires admin access on private repos, and the recent-PR-authors fallback gives a limited, potentially stale list. If the UX doesn't feel valuable enough given these constraints, defer and keep the current plain text input.
- The combo box pattern could be reused for other fields in the future (e.g., branch, workflow).
