---
status: pending
---

# CLI dependency detection and error UX

## Goal

Define and implement a consistent dependency-check pattern for feed runtimes that rely on external CLIs (current: `gh`; future: `ado-pr` with `az` + `azure-devops` extension), with clear user-facing error messages.

## Acceptance criteria

- [ ] A shared helper exists for command dependency checks (binary existence and basic invocability).
- [ ] Shared helper contract is explicit:
  - binary missing
  - binary present but returns invocation/auth/setup error
  - binary healthy
- [ ] GitHub PR feed surfaces a deterministic error when `gh` is missing, including a concrete install/action hint.
- [ ] GitHub PR feed surfaces a deterministic error when `gh` is present but not authenticated.
- [ ] Dependency errors are represented as feed-level poll errors (not app-global failures) and shown in tray output.
- [ ] Error wording is concise, actionable, and consistent across feeds.
- [ ] GitHub auth-check strategy is deterministic and documented in this task:
  - preferred preflight: `gh auth status`
  - fallback (if preflight omitted): infer unauthenticated state from `gh pr list` stderr/exit and normalize to the same user-facing error
- [ ] User-facing error copy is pinned for current GitHub feed checks:
  - missing binary: ``GitHub feed requires `gh` CLI. Install it from https://cli.github.com/ and run `gh auth login`.``
  - unauthenticated: ``GitHub feed requires `gh` authentication. Run `gh auth login` and retry.``
- [ ] Future-facing rules are documented for `ado-pr` dependency checks:
  - `az` CLI availability
  - `azure-devops` extension availability
  - authentication expectation (PAT/env or logged-in state)
- [ ] `just check` passes.

## Notes

- Keep the implementation scoped to current feeds where applicable; `ado-pr` behavior can be documented/contracted even if implementation is deferred.
- Reuse this pattern for any future feed type with external process dependencies.

## Relevant files

- `src-tauri/src/feed/github_pr.rs`
- `src-tauri/src/feed/mod.rs` (or a new helper module under `src-tauri/src/feed/`)
- `specs/main.md` (only if dependency UX contract needs explicit spec wording)
