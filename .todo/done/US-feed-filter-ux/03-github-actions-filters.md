---
status: done
---

# GitHub Actions filter UX

## Goal

Improve the GitHub Actions actor and workflow filter UX. Apply the 3-option actor pattern (with username auto-resolution for "Me"), and add clear guidance for the workflow and branch filters.

## Context

**Current behavior:**
- Actor field (config key `user`): plain text input, placeholder `@me`. Backend stores as `Option<String>` -- empty = no filter (all actors). Passed as `--user <value>` to `gh run list`.
  - **`@me` does NOT work** with `gh run list`. Only GitHub usernames are accepted. The current `@me` placeholder is a bug.
  - Email does NOT work either.
- Workflow field: plain text input, placeholder `ci.yml`. Empty = all workflows.
- Branch field: plain text input, placeholder `main`. Empty = all branches.

**New config semantics for actor (see `main.md`):**
- Empty / omitted `user` → no filter (all actors). Already works this way.
- Specific GitHub username → passed as `--user <value>`.
- No `@me` equivalent exists in `gh run list`.

**"Me" resolution strategy:**
When the user selects "Me" in the settings UI, the frontend calls `gh api user --jq .login` to resolve their GitHub username. The resolved username is stored in config as a regular specific-user value. This is a one-time API call in the settings UI. TOML-only users must enter their username manually.

## Acceptance criteria

- [ ] The actor filter uses the 3-option control: **All** / **Me** / **Specific user**.
- [ ] "Me" triggers a `gh api user --jq .login` call in the settings UI to resolve the username. The resolved username is stored in config (identical to "Specific user" under the hood).
- [ ] If the username resolution fails (e.g., not authenticated), show an inline error and don't save.
- [ ] The actor text input (for "Specific user") has ghost text: a GitHub username (e.g., `octocat`).
- [ ] The misleading `@me` placeholder is removed.
- [ ] The workflow filter has hint text explaining: (a) expected format is a workflow filename (e.g., `ci.yml`), and (b) leaving it empty shows runs from all workflows.
- [ ] The branch filter has hint text explaining that leaving it empty shows runs from all branches.
- [ ] Backend behavior is unchanged (actor is already `Option<String>`, no default-to-self logic).

## Dependencies

- Task 01 (establishes the shared UX pattern for the 3-option control).

## Related files

- `src/shared/feedTypes.ts` -- `github-actions` field definitions (remove `@me` placeholder)
- `src/settings/SettingsApp.tsx` -- settings form rendering + username resolution logic
- `src-tauri/src/feed/github_actions.rs` -- backend filter logic (no changes needed)

## Notes

- The username resolution could be done via a Tauri command that shells out to `gh api user --jq .login`. This keeps the logic in Rust and avoids exposing shell access to the frontend.
- The `event` config key is parsed by the backend but intentionally not exposed in the UI. Out of scope.
