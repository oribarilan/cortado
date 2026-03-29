---
status: pending
---

# Sprint 12 -- New Feed Types

## Theme

Expand cortado's feed ecosystem with three new curated feed types covering different patterns: CLI-backed (GitHub Actions), pure-Rust HTTP (HTTP Health), and API-backed with token auth (Vercel Deployments).

## Sequencing

```
01-github-actions-feed ──────────────────────────────────────────────────┐
                                                                         │
02-http-health-feed (adds reqwest) ──────────────────────────────────────┤
                                                                         │
03-feed-catalog-ui ─────────────────────────────────────────────────────┘
```

- Task 01 and 02 are independent backend tasks and can be done in parallel.
- Task 02 adds `reqwest` to `Cargo.toml`. This dependency will be reused by future API-backed feeds (Vercel, Linear, Sentry, etc.) when they move out of backlog.
- Task 03 is frontend-only and independent of the backend tasks. It can be built in parallel, but should be wired up after at least one new feed type is registered so the multi-type provider flow (GitHub: PR + Actions) can be tested.

## Tasks

| # | File | Feed type | Pattern | Summary |
|---|------|-----------|---------|---------|
| 01 | `01-github-actions-feed.md` | `github-actions` | CLI (`gh`) | Workflow run status. Same CLI/auth as `github-pr`. |
| 02 | `02-http-health-feed.md` | `http-health` | Pure Rust | HTTP endpoint monitoring. Adds `reqwest` dependency. |
| 03 | `03-feed-catalog-ui.md` | — | Frontend | Replace "+ New feed" dropdown with provider grid → feed type catalog. |

## Cross-cutting notes

- Extract shared `gh` preflight logic from `github_pr.rs` when building `github-actions` (task 01).
- `reqwest` (added in task 02) becomes a shared dependency for all future API-backed feeds.
- The `token_env` pattern (introduced in task 03) should be placed in a reusable location for future feeds.
- All feeds must respect existing `interval`, `retain`, `notify`, and `field_overrides` config.
- Each feed needs comprehensive unit tests: config validation, field/status mapping, error handling, field overrides.

## Reference

The full feasibility analysis for 14 candidate feed types (including these 3) lives in `optional-feed-types.md`.
