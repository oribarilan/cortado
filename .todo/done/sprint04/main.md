---
status: done
---

# Sprint 04 -- Retained activities + duration-string config

## Theme

Introduce a generic retained-activity lifecycle primitive across feeds and migrate config timing fields to string-based durations parsed with `jiff`.

## Sequencing

```
01-spec-contract ─────────────────────────────────────┐
                                                      ├──> 04-runtime-retention-logic ───> 05-tray-retained-rendering ─┐
02-duration-config-parsing ───────────────────────────┘                                                            │
                                                                                                                     ├──> 06-retention-tests
03-feed-retain-plumbing ──────────────────────────────────────────────────────────────────────────────────────────────┘

07-migrate-personal-config  (can run after 01/02)
```

- Task 01 comes first so spec remains source-of-truth before implementation.
- Task 02 updates parser contract to string durations (no integer backward compatibility).
- Task 03 wires `retain` through feed config/feed runtime interfaces.
- Task 04 implements core runtime retention lifecycle.
- Task 05 adds retained UI rendering semantics (hollow dot, ordering).
- Task 06 locks behavior with tests.
- Task 07 migrates local dev config (`~/.config/cortado/feeds.toml`) to new duration-string shape.

Status snapshot:

- ✅ Done: 01 spec contract, 02 duration parsing, 03 retain plumbing, 04 runtime retention, 05 tray rendering, 06 tests, 07 personal config migration

## Cross-task notes

- **Duration parsing library**: use `jiff` for duration-string parsing.
- **No integer backward compatibility**: `interval = 60` is invalid in sprint04; use `interval = "60s"`.
- **Retention config**: `retain` is optional; omitted means no retention.
- **Terminology**: use **Retained Activity** (not "inactive").
- **UI semantics**: retained items are prefixed with hollow dot (`◦`) and listed after active items.
- **Persistence**: out of scope for sprint04 (in-memory only). Backlog item exists at `.todo/backlog/optional-retained-activity-persistence.md`.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-spec-contract.md` | Update spec for duration strings, retained activity lifecycle, and tray semantics |
| 02 | `02-duration-config-parsing.md` | Parse `interval` as required duration string via `jiff` (no integer fallback) |
| 03 | `03-feed-retain-plumbing.md` | Add optional per-feed `retain` duration config and expose to runtime |
| 04 | `04-runtime-retention-logic.md` | Retain disappeared activities for configured duration and expire deterministically |
| 05 | `05-tray-retained-rendering.md` | Render retained activities with hollow dot and post-active ordering |
| 06 | `06-retention-tests.md` | Add deterministic parser/runtime/tray behavior tests |
| 07 | `07-migrate-personal-config.md` | Migrate local `~/.config/cortado/feeds.toml` to duration-string format |

## Outcome

- Sprint04 implementation scope completed end-to-end (duration-string parsing + retained-activity lifecycle + tray rendering + tests).
- Persistence of retained activities across restarts remains explicitly deferred in backlog:
  - `.todo/backlog/optional-retained-activity-persistence.md`
