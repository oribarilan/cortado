# US: Feed Filter UX

## Theme

Improve the usability of author/actor filters and workflow filters across all feed types. Currently every filter is a plain text input with minimal guidance -- users don't know what format to use, and the common "just show my stuff" case requires typing a magic string instead of clicking a toggle.

## Scope

Three feed types are affected:

1. **GitHub PRs** -- author filter (`user` field)
2. **ADO PRs** -- creator filter (`user` field)
3. **GitHub Actions** -- actor filter (`user` field) + workflow filter (`workflow` field)

## Design decisions

### Config representation (all feed types)
- Empty / omitted `user` = no filter (show all). This is a change from the current behavior where empty defaults to "me".
- `@me` (GitHub) / `me` (ADO) = current authenticated user.
- Any other value = specific user (GitHub username or ADO email).

### UI control
A 3-option selector (not plain text): **All** / **Me** / **Specific user**.
- "All" → stores empty string in config.
- "Me" → stores `@me` (GitHub) or `me` (ADO).
- "Specific user" → shows a text input with ghost text indicating the expected format (GitHub username for GitHub feeds, email for ADO).

### Actions "Only mine"
`gh run list` doesn't support `@me`, so "Me" for Actions resolves the user's GitHub username via `gh api user --jq .login` in the settings UI (one-time API call when the user selects "Me"). The resolved username is stored as the config value -- under the hood it's just a specific user. TOML-only users must enter their username manually.

## Cross-cutting concerns

- The `FeedTypeField` type in `src/shared/feedTypes.ts` currently has no concept of field *kind* (text vs toggle vs select). All tasks will likely need to extend this type or introduce a new field-rendering pattern in the settings form.
- The settings form in `src/settings/SettingsApp.tsx` renders all fields uniformly as `<input type="text">`. The new 3-option control needs rendering support there.
- The shared UX pattern (3-option selector + conditional text input) should be extracted so all three tasks stay consistent.

## Sequencing

Tasks 01 and 02 are very similar (author filter for GitHub PRs and ADO PRs). Do 01 first to establish the pattern, then 02 follows it. Task 03 builds on the same pattern but adds the workflow filter concern and the username resolution for "Me".

All three tasks are shippable independently.
