---
status: done
---

# Make the hidden count the disclosure control

## Acceptance criteria
- [x] One shared text button shows the hidden count when collapsed and Show less when expanded, with accessible names and expanded state.
- [x] Nothing-hidden normal views and other feed types have no control; expanded feeds remain reachable after live updates.
- [x] Tray and Panel empty-state copy, Settings help, README, and spec describe the new control.
- [x] Existing Vitest coverage is updated for the new markup and expanded-empty state.
- [x] Browser checks cover both windows, click/Enter/Space, live updates, focus, no extra IPC, and no-hidden/hidden-only/error states.
- [x] `just check`, `pnpm test:ts`, `pnpm build`, and `git diff --check` pass; independent review findings are addressed.

## Verification plan
Parent owns validation. Use existing Vitest tests for deterministic state/markup cases and the production frontend with mocked Tauri IPC for interaction checks. Run the required repository checks once after implementation. A read-only reviewer checks the final diff. No live Azure DevOps calls or installed-app changes are needed for this frontend-only change.

## Progress
- Created branch `update/pipeline-hidden-control` from released `v0.19.0` (`8891686`).
- Replaced the count/button wrapper with one text button using `aria-expanded`, contextual accessible labels, and decorative chevrons. Removed unused count styling; no new animations.
- Expanded empty feeds remain visible until Show less is selected; no backend or saved-state changes.

## Verification results
- New contract tests failed before implementation, then passed. `pnpm test:ts`: 70 passed.
- `just check`: passed without warnings; 581 Rust tests passed, 10 existing tests ignored, and 22 plugin tests passed. `pnpm build` and `git diff --check` passed.
- `/tmp/cortado-hidden-control/browser.mjs`: passed against both production frontend entrypoints with mocked Tauri IPC. Covered mouse/Enter/Space, visible focus, per-feed/window independence, expanded-empty live updates, hidden-only/error/no-hidden states, normal Panel keyboard navigation, reopening, and no IPC on disclosure.
- Inspected screenshots in light/dark themes and XL text; controls fit both headers. Reduced-motion mode also exercised; no new animations.
- The browser tool's installed Chrome failed to start twice. Used already-installed headless Chromium/Playwright for parent-owned checks; no dependencies were added.
- Independent reviewer found no issues: workflow `6d1e79f5-90c9-46bf-8a5b-3d71eb6e7771`, child `04335001-75f3-4d60-82f8-039dcbfb3236`.
- No live Azure DevOps, native macOS WebView, or screen-reader smoke test was performed.
