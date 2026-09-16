---
status: done
---

# US: Azure DevOps YAML pipelines

## Goal

Track a selected group of Azure DevOps YAML pipelines in one feed, using explicit IDs or one exact folder. Each Activity shows one pipeline's latest run.

## Decisions

- User approved YAML-only support, exact folders, stable pipeline Activities, and rejecting selections over 20.
- Review found arbitrary-host credential routing through Azure CLI. User confirmed hosted ADO only and requested alignment with existing ADO feeds. Share hosted organization URL validation between `ado-pr` and `ado-pipelines`, rejecting invalid destinations before CLI calls; custom Server URLs are no longer supported.
- Worktree: `/Users/orbarila/repos/personal/cortado-ado-pipelines`, branch `add/ado-pipelines`, based on freshly fetched `origin/main` at `a7d97b18f99f6db135846e76a802fc158e1d75e0`.
- User approved making the already-locked `percent-encoding` 2.3.1 crate a direct dependency for URL decoding. No new dependency versions or JavaScript packages were introduced.
- No live organization queries or source edits to the original checkout.
- Existing stories remain deferred for this slice. Configurable activity limits stay in `US-activity-limits`.
- User explicitly deferred unrelated GitHub Actions spec drift and the outdated ADO PR README example. See the separate backlog note.

## Sequence

One deliverable: `01-ado-pipelines.md` includes the feed, Settings, documentation, and tests. Implement the approved contract in `specs/main.md`, then independently review and validate it.

## Definition of done

- [x] The feed discovers only the selected YAML pipelines and polls their latest runs in bulk, including exact-folder and pagination safeguards.
- [x] Settings preserves selectors through editing and TOML roundtrips; activities open the latest run.
- [x] Deterministic tests establish selection, errors, lifecycle, and notification behavior without network access.
- [x] Independent review is addressed, `just check` and TypeScript tests pass, and the parent inspects the final diff.

## Evidence path

Use the existing injected `ProcessRunner` with synthetic REST responses and exact argv assertions. This directly tests the CLI boundary without credentials or service variability. Cover all status branches, multi-page folders, YAML filtering, IDs spanning repositories, missing IDs, overflow, never-run pipelines, malformed responses, and errors. A focused snapshot/notification test must show that changing a run ID does not replace the pipeline Activity.

Use existing Vitest tests for catalog/list parsing and URL-opening behavior, Rust tests for TOML/JSON persistence, and type checking/building for the Settings renderer. Parent owns final `just check`, `pnpm test:ts`, and a frontend production build. Worker owns focused tests; reviewer owns independent contract/security/regression analysis. Repeat only checks invalidated by later fixes.

No live Azure DevOps organization was queried. Mocked response tests establish Cortado's behavior against the documented API contract, not live service behavior or signed-in access. Settings behavior was checked through composed helper tests, DTO roundtrips, type checking, and the production frontend build; no interactive app smoke test was performed.

## Final verification

- `just check`: passed without warnings, including 567 Rust tests (10 existing ignored tests) and 22 OpenCode tests.
- `pnpm test:ts`: 41 tests passed.
- `pnpm build` and `git diff --check`: passed.
- Two independent review rounds found six defects. All were fixed; the parent reproduced the final URL regressions with failing tests before correcting them and rerunning validation.
- After validation, the owner authorized committing, pushing, and opening a PR from `add/ado-pipelines`. No merge is authorized. The original checkout's pre-existing deleted symlink was preserved.
