---
status: pending
---

# Better default names for new feeds

## Goal

Auto-populate the feed name with a descriptive default derived from the feed type and its primary config field. The user can always override it.

## Suggested defaults

| Feed type          | Default name pattern           | Example                        |
|--------------------|--------------------------------|--------------------------------|
| `github-pr`        | `{repo} PRs`                   | `facebook/react PRs`           |
| `github-actions`   | `{repo} Actions`               | `my-org/api Actions`           |
| `ado-pr`           | `{project/repo} PRs`           | `myproject/myrepo PRs`         |
| `http-health`      | `{hostname}`                   | `api.example.com`              |
| `copilot-session`  | `Copilot`                      | `Copilot`                      |
| `opencode-session` | `OpenCode`                     | `OpenCode`                     |

## Behavior

- Default is generated when the feed type is selected (or when the primary field is filled in for types that use a repo/url).
- If the user has manually edited the name, don't overwrite it.
- If the user clears the name, regenerate the default.

## Acceptance criteria

- [ ] Selecting a feed type auto-populates the name field with a sensible default
- [ ] For repo/url-based feeds, the default updates when the primary field changes
- [ ] Manual edits are preserved — auto-population stops once the user types
- [ ] Clearing the name field regenerates the default
- [ ] `just check` passes

## Relevant files

- `src/settings/SettingsApp.tsx` — feed form, `name: ""` initial state
- `src/shared/feedTypes.ts` — `FEED_CATALOG` with per-type metadata
