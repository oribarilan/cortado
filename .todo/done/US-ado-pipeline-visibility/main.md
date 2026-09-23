---
status: done
---

# ADO pipeline visibility

## Goal
Reduce passing-pipeline noise without changing polling, activity identity, retention, or notifications.

## Definition of done
- [x] Passing pipelines hide one hour after completion by default; configurable through Settings and `show_passing_for` (`"0s"` hides immediately).
- [x] Problems, queued/running/cancelling runs, unknown states, and feed errors remain visible. Never-run pipelines are hidden.
- [x] Tray and Panel offer a per-feed All pipelines toggle, including when every pipeline is hidden.
- [x] Deterministic Rust/Vitest regressions cover config, completion-time cutoffs, presentation, and unchanged lifecycle/notifications.
- [x] Specs and README describe the behavior; `just check` and `pnpm test:ts` pass without warnings.

## Task order
1. `01-visibility.md` implements and verifies the complete change.

## Decisions
- Visibility is presentation-only; the backend continues tracking all selected pipelines.
- A passing run with a missing/invalid completion timestamp remains visible, except with `show_passing_for = "0s"`.
- Use the existing 30-second UI refresh cadence and refresh on window opening; no extra ADO calls or dependencies.
- All pipelines is a per-window, per-feed view toggle, not a saved setting.
- Settings validates ADO pipeline configuration with the existing feed constructor before writing; no CLI requests are made during validation.
- Existing unrelated deletion at `videos/.opencode/skills/remotion-best-practices` is left untouched.

## Completion evidence
- `just check`: clean; 581 Rust tests passed, 10 existing ignored tests; 22 plugin tests passed.
- `pnpm test:ts`: 63 tests passed. `pnpm build` and `git diff --check` passed.
- Chromium checks with mocked IPC passed for both views and Settings, including timed expiry, keyboard activation/navigation, reopen, hidden-only snapshots, errors, saved values, and default/reset behavior.
- Independent review completed with no outstanding findings. No live ADO or native macOS smoke test; no app installed.
