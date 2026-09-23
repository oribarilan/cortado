---
status: done
---

# Hide uninteresting pipeline results

## Acceptance criteria
- [x] Backend validates a nonnegative duration string, defaults to 1h, and exposes UI-only visibility deadlines based on finish time independently of field overrides.
- [x] Both UIs apply the same filter, refresh expiry without polling, preserve access to hidden pipelines, and do not hide feed errors.
- [x] Settings supports creating/editing the passing window using existing feed save feedback.
- [x] Tests establish exact cutoff/zero behavior, unknown and missing-time safety, All pipelines, non-ADO isolation, and no synthetic removals/retention/recovery-notification loss.
- [x] Docs updated and all required checks pass; independent review findings addressed.

## Verification plan
- Parent owns behavior and integration evidence: mocked ADO responses + Rust lifecycle/change-detection regressions, pure Vitest visibility/config tests, component rendering checks where supported.
- Independent reviewer owns correctness review of the final diff; no duplicate full-suite runs.
- Parent runs `just check`, `pnpm test:ts`, and a frontend production build. No real ADO credentials or user config changes.
- Chromium automation was available through an existing local Playwright installation; no dependency added. Native macOS integration and real ADO access were not exercised.

## Results
- `just check`: 581 Rust tests passed, 10 existing ignored tests; 22 plugin tests passed; no warnings.
- `pnpm test:ts`: 63 passed. `pnpm build` and `git diff --check`: passed.
- Rust fixtures cover finish-vs-queue timestamps, invalid/zero/default windows, state mapping, hidden status fields, serialization, Settings save validation/round-trip, and full lifecycle/change detection through recovery and actual selection removal.
- Vitest covers exact deadline boundaries, All pipelines, unchanged source snapshots, non-ADO isolation, errors, retention metadata, header reachability, and accessible toggle markup.
- Ad-hoc browser checks (`/tmp/cortado-pipeline-browser.mjs`, `/tmp/cortado-pipeline-settings-browser.mjs`) passed on the production frontend with mocked IPC. Tested timer expiry without calls, window-show refresh, hidden-only feeds, keyboard Enter/Space, toggle-to-list navigation via arrows/j/k, reopening then Enter, and Settings create/edit/zero/reset/save/error feedback.
- Independent review: `1911e47d-edc3-4933-b691-ffaa0f561767`, child `d15eecc6-2d8d-4c8f-9bf3-a2019632ec2e`; no remaining issues. Fixed two findings during review: reject invalid ADO config before Save writes, and clear native toggle focus when list navigation or reopening takes over.

## Implementation
- `Activity.visible_until` is optional UI metadata; other feed constructors set it to None.
- ADO polling remains unchanged apart from parsing finish time and computing deadlines.
- Shared frontend visibility logic/hook/toggle drives both views using the existing 30-second timer.
- Feed Settings uses catalog metadata and existing save/error feedback; clearing the field restores 1h.
- Specs, README, and changelog updated. No app installed.
