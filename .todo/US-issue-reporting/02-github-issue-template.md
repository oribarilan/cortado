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

- None (references the log path as a static string; doesn't require logging to be implemented)

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
  - Log files (optional file upload, with hint: "Find logs at ~/Library/Logs/Cortado/")
  - Any additional context (optional textarea)
- [ ] Template is simple -- no more than ~9 fields total
- [ ] Template renders correctly on GitHub (verify by pushing and viewing /issues/new)

## Scope Estimate

Small

## Notes

Use GitHub's YAML form schema (not the older Markdown template). YAML forms render as structured fields in the browser, which makes filling them in easier and produces cleaner issues.

The log files field should include a description explaining where to find logs (`~/Library/Logs/Cortado/`) and that they help diagnose issues. GitHub YAML forms don't support file upload fields directly -- use a textarea with a description that tells users to drag-and-drop log files into the field.
