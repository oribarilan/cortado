# US-issue-reporting: Bug Reports & Issue Filing

## Theme

Make it easy for users to report bugs. Add structured logging to disk so users have something to attach, a GitHub issue template that collects the right context, and a "Report Issue" action in the UI that guides users to file a bug with logs.

## Task Sequencing

Task 01 (log files) is independent. Tasks 02 and 03 are sequential (the template URL must exist before the UI links to it), but neither depends on 01 -- they reference the log path as a static string.

Recommended order: 02 → 03, with 01 in parallel.

1. **Log files** -- add a logging framework that writes to disk so users have logs to share
2. **GitHub issue template** -- create a bug report template that asks for logs (can start before 01)
3. **Report Issue UI** -- surface a "Report Issue" action in tray and panel (depends on 02)
