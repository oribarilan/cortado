---
status: deferred
---

# Reconcile existing feed documentation

## Goal

Choose whether to update existing feed contracts or their implementation. The user explicitly deferred this work while adding ADO pipelines.

## Known mismatches

- `github-actions` accepts `repos` and deduplicates returned runs by repository/name; `specs/main.md` documents only `repo` and does not describe this selection behavior.
- The GitHub Actions spec maps cancelled runs to `AttentionNegative`; `src-tauri/src/feed/github_actions.rs` maps them to `Idle`.
- The README's `ado-pr` example uses `org`, `project`, and `repo`; the current implementation and spec require `url`.

## Acceptance criteria

- [ ] Owner chooses the intended contract for each mismatch.
- [ ] Documentation and behavior agree, with tests for any behavior changes.

## Notes

Do not change existing feeds as part of `US-ado-pipelines`. Its cancellation mapping follows the written CI convention without changing GitHub Actions.
