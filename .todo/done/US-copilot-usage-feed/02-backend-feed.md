---
status: done
---

# Implement the backend feed

## Goal

Poll the selected account safely, parse only required quota data, and map nominal usage to one Cortado Activity.

## Acceptance criteria

- [x] Process invocations support per-command environment overrides/removals without exposing secret values in debug or display output.
- [x] `copilot-usage` validates all type-specific config values.
- [x] Polling resolves the selected account token without changing the active gh account.
- [x] Polling verifies the returned login and parses only allowlisted quota fields.
- [x] Token-based AI credits map to nominal usage and deterministic status fields.
- [x] Missing CLI/auth, API failures, legacy units, malformed data, and login mismatches return actionable feed-level errors.
- [x] The feed is registered as a network feed and in the feed factory.
- [x] Unit tests cover config boundaries, secret handling, invocation flow, parsing, threshold boundaries, and errors.
