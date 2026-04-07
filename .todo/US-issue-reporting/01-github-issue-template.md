---
status: pending
---

# GitHub issue template

## Goal

Create a lightweight bug report template that collects just enough context to reproduce and triage issues without being burdensome.

## Context

Users currently have no structured way to report bugs. A GitHub issue template pre-fills the right questions so reporters don't forget critical environment details (Cortado version, terminal app, which part of the UI is affected).

**Value delivered**: Bug reports arrive with consistent, actionable context from day one.

## Related Files

- `.github/ISSUE_TEMPLATE/bug-report.yml` (new)

## Dependencies

- None

## Acceptance Criteria

- [ ] `.github/ISSUE_TEMPLATE/bug-report.yml` exists with YAML form format
- [ ] Template asks for:
  - Description of the bug (required)
  - Steps to reproduce
  - Expected vs. actual behavior
  - Cortado version (text input, required)
  - Affected area: dropdown with Tray / Panel / Settings / General
  - Terminal app and version (text input, optional -- "if relevant to bug")
  - macOS version (text input)
  - Any additional context (optional textarea)
- [ ] Template is simple -- no more than ~8 fields total
- [ ] Template renders correctly on GitHub (verify by pushing and viewing /issues/new)

## Scope Estimate

Small

## Notes

Use GitHub's YAML form schema (not the older Markdown template). YAML forms render as structured fields in the browser, which makes filling them in easier and produces cleaner issues.

Keep it lightweight -- don't ask for logs, screenshots, or config unless the user wants to volunteer them in the free-text "additional context" field.
