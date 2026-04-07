---
status: pending
---

# Report Issue button and hotkey

## Goal

Surface a "Report Issue" action in both the tray and the panel so users can file a bug in one click/keystroke.

## Context

Even with a great issue template, users need to know where to go. A visible entry point in the tray menu and a keyboard shortcut in the panel removes friction.

**Value delivered**: Users can reach the bug report form from anywhere in the app without hunting for a GitHub URL.

## Related Files

- `src/App.tsx` (tray UI)
- `src/main-screen/` (panel UI)
- `src-tauri/src/command.rs` (if opening URL via Tauri command)

## Dependencies

- 01-github-issue-template (the URL needs to exist)

## Acceptance Criteria

- [ ] Tray: a "Report Issue" item in the tray menu (bottom section, near settings)
- [ ] Panel: a keyboard shortcut that opens the issue page (document in shortcuts)
- [ ] Both open the GitHub new-issue URL with the bug report template pre-selected
- [ ] URL opens in the user's default browser (not in-app)
- [ ] Works when the app is offline (just opens the URL -- browser handles the rest)

## Scope Estimate

Small

## Notes

The URL format for a pre-selected template is:
`https://github.com/<owner>/<repo>/issues/new?template=bug-report.yml`

Consider whether the Cortado version can be pre-filled via a query param (`&cortado_version=0.10.0`). GitHub YAML forms support default values and URL-based pre-fill via field IDs.
