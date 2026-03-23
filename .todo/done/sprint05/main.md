---
status: done
---

# Sprint 05 — Azure DevOps PR feed (`ado-pr`) + custom-feed primitive docs

## Theme

Add a curated Azure DevOps PR feed type with robust CLI dependency/auth handling, and document the existing custom-feed primitives users already have.

## Decisions locked

- `ado-pr` required config keys: `org`, `project`, `repo`
- `ado-pr.org` uses full organization URL (for example `https://dev.azure.com/your-org`)
- Auth model for sprint05: **`az login` only** (no PAT mode)
- Initial `ado-pr` fields: `review`, `mergeable`, `draft`, `labels`
- Poll scope: active PRs only (`--status active`) + rely on generic `retain`
- Custom feed work in sprint05: documentation-only (no new primitive implementation)

## Sequencing

```
01-spec-contract ────────────────┐
                                 ├──> 03-ado-pr-feed-implementation ───> 04-registry-wiring ───┐
02-ado-pr-config-parsing ────────┘                                                               │
                                                                                                  ├──> 05-ado-pr-tests
06-custom-feed-primitives-docs ───────────────────────────────────────────────────────────────────┘
```

- Task 01 first to keep spec as source of truth.
- Task 02 + 03 can progress closely; 02 validates config contract, 03 implements feed behavior.
- Task 04 wires type into registry once feed implementation compiles.
- Task 05 adds deterministic tests for dependency/auth + field mapping behavior.
- Task 06 updates user-facing docs about current custom-feed primitives.

## Cross-task notes

- Use the existing process/dependency infrastructure (`process.rs`, `dependency.rs`) as in `github-pr`.
- Force explicit CLI context via `--org`, `--project`, `--repository`, `--detect false`.
- Dependency/auth checks should be explicit and user-facing errors should be actionable.
- No PAT auth support in sprint05; failing auth should instruct `az login`.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-spec-contract.md` | Update spec for `ado-pr` config, auth model, fields, and dependency contract |
| 02 | `02-ado-pr-config-parsing.md` | Add/validate required `org`+`project`+`repo` config handling for `ado-pr` |
| 03 | `03-ado-pr-feed-implementation.md` | Implement Azure DevOps PR polling via `az repos pr list` |
| 04 | `04-registry-wiring.md` | Register `ado-pr` in feed instantiation/dispatch |
| 05 | `05-ado-pr-tests.md` | Add deterministic unit tests for config, dependency/auth, and mapping behavior |
| 06 | `06-custom-feed-primitives-docs.md` | Document current custom-feed primitives and limits (no new primitive) |

## Outcome

- Implemented `ado-pr` curated feed with explicit Azure CLI dependency/auth preflight and deterministic field mapping.
- Added docs for `ado-pr` config plus existing custom-feed primitives (`shell` escape hatch + limits).
