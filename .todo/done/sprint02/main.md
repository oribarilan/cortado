---
status: done
---

# Sprint 02 — Real polling engine (non-UI)

## Theme

Replace feed stubs with real data collection and add a background polling engine with cached snapshots.
By the end of this sprint, Cortado should continuously refresh tray data from real `github-pr` and `shell` feeds without requiring UI-triggered polling.

## Sequencing

```
01-shell-execution ───────────────┐
                                  ├──> 03-background-poller-cache ──> 04-tray-refresh-loop
02-github-pr-gh-cli ──────────────┤
                                  ├──> 05-field-overrides
                                  └──> 07-cli-dependency-checks

03-background-poller-cache ───────────────────────────────────────────┐
05-field-overrides ───────────────────────────────────────────────────┼──> 06-feed-tests
07-cli-dependency-checks ─────────────────────────────────────────────┘
```

- Tasks 01 and 02 can run in parallel.
- Task 03 depends on 01 and 02 (real polling behavior in place).
- Task 04 depends on 03 (tray refresh uses poller cache).
- Task 05 can run after 01/02 and in parallel with 03/04.
- Task 07 depends on 02 and can run in parallel with 03/04/05.
- Task 06 lands last and verifies all core behavior (especially 03 + 05 + 07 integration).

## Cross-task notes

- **Scope boundary**: This sprint is intentionally backend/non-UI only while panel UI is being reworked between sprints.
- **Auth model**: GitHub feeds must use `gh` CLI (`gh auth login`), with no token fields in `feeds.toml`.
- **CLI dependency UX**: This sprint defines consistent detection and error messaging for missing external dependencies (`gh`, and future `ado-pr` requirements around `az` + `azure-devops` extension).
- **ADO naming**: Use `ado-pr` (not `azdo-pr`) for Azure DevOps PR feed naming going forward.
- **ADO checks scope**: For future `ado-pr` implementation, checks/build policy status may be deferred initially (no N+1 policy calls in first pass).
- **Startup behavior**: Seed feed cache at startup (initial poll) before steady-state intervals.
- **Read-path contract**: `list_feeds` and tray refresh paths should consume cached snapshots; they should not synchronously repoll all feeds.
- **Intervals**: Respect configured `interval`; fallback defaults remain `github-pr = 120s`, `shell = 30s`.
- **Error handling**: Poll failures are surfaced per-feed and should preserve last known activities when available.
- **Config loading**: Continue loading `~/.config/cortado/feeds.toml` once at startup (hot-reload remains backlog).
- **Dependencies**: Prefer existing crates. If tokio process support needs feature changes, use existing `tokio` dependency rather than adding new crates.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-shell-execution.md` | Replace shell stub with real command execution and typed output mapping |
| 02 | `02-github-pr-gh-cli.md` | Replace GitHub stub with real `gh pr list --json ...` polling |
| 03 | `03-background-poller-cache.md` | Add per-feed background polling, startup seed, and cached snapshots |
| 04 | `04-tray-refresh-loop.md` | Refresh tray menu from poller updates and keep manual refresh path |
| 05 | `05-field-overrides.md` | Enforce field overrides (`visible`, `label`) consistently |
| 07 | `07-cli-dependency-checks.md` | Standardize external dependency detection and user-facing errors |
| 06 | `06-feed-tests.md` | Add unit tests for parsing, mapping, overrides, and stale-on-error behavior |
