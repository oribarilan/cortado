---
status: pending
---

# Optional: logging standards in AGENTS.md

## Goal

Define logging conventions and add them to AGENTS.md so agents produce consistent, useful logs.

## Notes

- Log levels: when to use `trace`, `debug`, `info`, `warn`, `error`.
- Structured logging vs plain strings.
- What to log: state transitions, external calls, errors, config loading. What *not* to log: secrets, tokens, PII.
- Rust: `tracing` crate is the de facto standard. Decide whether to adopt it.
- Frontend: `console.log` vs a lightweight logging lib.
- Performance: logging should not be a bottleneck (avoid formatting in hot paths unless the level is enabled).
