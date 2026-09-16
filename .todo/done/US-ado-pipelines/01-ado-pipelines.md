---
status: done
---

# Add the Azure DevOps YAML pipelines feed

## Goal

A user can configure a single feed for up to 20 YAML pipelines by IDs or exact folder, then see each pipeline's latest run and open it from Cortado.

## Acceptance criteria

- [x] Config validation enforces hosted organization/project, mutually exclusive selectors, valid integer IDs, exact folders, and the 20-pipeline limit.
- [x] Bulk Build Definitions 7.1 queries use YAML process filtering and latest builds, follow pagination safely, and surface missing IDs, overflow, auth failures, and malformed responses as feed errors.
- [x] Activities keep their identity across runs and renames, map every specified status, display never-run pipelines, expose browser links, and provide recency ordering.
- [x] Shared hosted-URL validation rejects invalid destinations before CLI calls for both ADO feed types; existing dependency preflight behavior stays unchanged.
- [x] Settings creates/edits the new feed with integer-array roundtripping and validation, without an ADO picker or new JavaScript dependencies.
- [x] Synthetic CLI tests, lifecycle/notification coverage, TypeScript tests, and config roundtrip tests pass.
- [x] README and changelog describe the feature and limits; specs match implementation; unrelated drift remains deferred.
- [x] Independent review findings are resolved or explicitly deferred; parent final validation passes with no warnings.

## Relevant files

- `specs/main.md`, `specs/glossary.md`, `specs/ux_design.md`
- `src-tauri/src/feed/ado_pr.rs`, `feed/mod.rs`, `feed/config.rs`, `feed/process.rs`, `feed/runtime.rs`
- `src-tauri/src/settings_config.rs`, `notification/change_detection.rs`
- `src/shared/feedTypes.ts`, `src/shared/utils.ts`, `src/settings/SettingsApp.tsx`
- `README.md`, `CHANGELOG.md`

## Implementation evidence

- API: https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1
- YAML process type is `2`, confirmed by Microsoft's CLI `_create_process_object` in https://github.com/Azure/azure-devops-cli-extension/blob/4d49a4b367fdba19442bfccf07e9516a8a3dd857/azure-devops/azext_devops/dev/pipelines/pipeline_create.py
- `az devops invoke` exposes the continuation header as `continuation_token`; verify the public implementation if necessary. Do not assume `az pipelines list` paginates.
- `includeAllProperties` is not needed: use `processType=2` to avoid fetching full definitions or pipeline variables.
- `src/shared/utils.ts` opens a URL field for non-URL activity IDs. The notification URL helper now does the same, with regression coverage.

## Verification

Worker: focused Rust and Vitest tests plus formatting/type checking as needed. Parent: final `just check`, `pnpm test:ts`, `pnpm build`, diff review, and task housekeeping. Use the existing dependencies linked into the worktree. For Rust build reuse, `CARGO_TARGET_DIR=/Users/orbarila/repos/personal/cortado/src-tauri/target` is available; no simultaneous Rust build is planned in the original checkout.

## Outcome and review

- Implemented shared ADO preflight, bulk paginated YAML polling, stable pipeline Activities, URL fields, and catalog-driven Settings selectors. The original checkout remains untouched.
- Synthetic tests cover selection, exact-folder pagination, empty/overflow/missing selections, malformed responses, status mapping, stable identity, links, timestamps, and authentication errors.
- Review round 1 found incomplete hosted-URL validation, broken folder-only normalization, coercion of malformed ID arrays, and silent omission of missing folder paths. All four were fixed and verified in the second review.
- The user approved hosted-only alignment for `ado-pr` and `ado-pipelines`. Both constructors reject invalid destinations before CLI calls; Settings applies matching rules. Custom Server URLs are no longer supported.
- Review round 2 found double-encoded CLI names and acceptance of bare query/fragment suffixes in Settings. Parent added failing regressions, then fixed both. Decoded CLI names and encoded browser paths remain separate; spaces, Unicode, plus signs, and literal percent escapes are covered.
- User approved using the already-locked `percent-encoding` 2.3.1 as a direct dependency. No dependency versions changed.
- Final parent validation passed: 56 ADO-filtered Rust tests, `just check` (567 Rust tests passed, 10 existing ignored, 22 OpenCode tests passed), `pnpm test:ts` (41 tests), `pnpm build`, and `git diff --check`. Clippy and type checking produced no warnings.
- Parent reviewed the final diff and resolved all scoped review findings. No live organization query or interactive app smoke test was performed. Unrelated documentation drift stays in the deferred backlog note.
- After validation, the owner authorized committing, pushing, and opening a PR. No merge is authorized.
