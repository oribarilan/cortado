---
status: done
---

# ADO PR creator filter UX

## Goal

Apply the same 3-option author filter pattern from task 01 to the Azure DevOps PR creator filter. Same UX, different format guidance (email instead of username).

## Context

**Current behavior:**
- Field: `user` (label "Creator filter"), rendered as a plain text input.
- Backend (`feed/ado_pr.rs`): trims the value, defaults to `"me"` if empty. Passed as `--creator <value>` to `az repos pr list`.
- Valid values: `"me"` (current authenticated user) or an Azure DevOps identity. **Email/UPN is the preferred format** -- display names can be ambiguous and trigger an error.
- `az repos pr list` without `--creator` returns PRs from **all authors**.

**New config semantics (see `main.md`):**
- Empty / omitted `user` → no filter (all creators). Backend omits `--creator`.
- `me` → current authenticated user.
- Any other value → specific identity passed as `--creator <value>`.

## Acceptance criteria

- [ ] The creator filter uses the same 3-option control from task 01: **All** / **Me** / **Specific user**.
- [ ] "All" stores empty string. "Me" stores `me`. "Specific user" shows a text input.
- [ ] The text input has ghost text indicating the expected format: an **email address** (e.g., `user@org.com`).
- [ ] Backend change: when `user` is empty/omitted, omit the `--creator` flag (remove default-to-`me`).
- [ ] Default selection for new feeds is "Me".

## Dependencies

- Task 01 (establishes the shared UX pattern and field type extension).

## Related files

- `src/shared/feedTypes.ts` -- `ado-pr` field definitions
- `src/settings/SettingsApp.tsx` -- settings form field rendering (reuses pattern from task 01)
- `src-tauri/src/feed/ado_pr.rs` -- backend filter logic (remove default-to-`me`)
