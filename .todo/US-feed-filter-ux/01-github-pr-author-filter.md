---
status: done
---

# GitHub PR author filter UX

## Goal

Make the GitHub PR author filter intuitive with a 3-option UI control instead of a plain text input. The most common case ("show my PRs") should be a single click. "Show all" should be equally easy. And when filtering by a specific user, the expected format should be obvious.

## Context

**Current behavior:**
- Field: `user` (label "Author filter"), rendered as a plain text input.
- Backend (`feed/github_pr.rs`): trims the value, defaults to `@me` if empty. Passed as `--author <value>` to `gh pr list`.
- Valid values: any GitHub **username** (e.g., `octocat`) or the literal `@me` (resolved by `gh` CLI to the authenticated user). **Email addresses are NOT supported** by `gh pr list --author`.
- `gh pr list` without `--author` returns PRs from **all authors**.

**New config semantics (see `main.md`):**
- Empty / omitted `user` → no filter (all authors). Backend omits `--author`.
- `@me` → current authenticated user.
- Any other value → specific GitHub username passed as `--author <value>`.

## Acceptance criteria

- [ ] The author filter renders as a 3-option control: **All** / **Me** / **Specific user**.
- [ ] "All" stores empty string. "Me" stores `@me`. "Specific user" shows a text input.
- [ ] The text input has ghost text indicating the expected format: a GitHub username (e.g., `octocat`).
- [ ] Backend change: when `user` is empty/omitted, omit the `--author` flag entirely (remove the default-to-`@me` fallback).
- [ ] The `FeedTypeField` type is extended (or a new field kind is introduced) to support this 3-option + conditional input pattern.
- [ ] The settings form rendering supports the new field kind.
- [ ] The pattern is generic enough to reuse in tasks 02 and 03.
- [ ] Existing tests updated; new tests cover: empty → no `--author`, `@me` → `--author @me`, username → `--author <username>`.

## Related files

- `src/shared/feedTypes.ts` -- `FeedTypeField` type and `github-pr` field definitions
- `src/settings/SettingsApp.tsx` -- settings form field rendering
- `src-tauri/src/feed/github_pr.rs` -- backend filter logic (lines 63-70: remove default-to-`@me`)

## Notes

- Whatever UI control is chosen (segmented control, radio group, dropdown) should be consistent with the UX design spec in `specs/ux_design.md`.
- Default selection for new feeds should be "Me" (most common use case).
