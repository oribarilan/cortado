---
status: done
---

# Add Settings and user documentation

## Goal

Make the experimental feed configurable through Settings and document its TOML contract and limitations.

## Acceptance criteria

- [x] The GitHub catalog includes an experimental Copilot Usage feed.
- [x] Settings exposes account, reference amount, attention percentage, optional details URL, and interval.
- [x] Frontend validation matches backend boundaries, including HTTPS-only action overrides.
- [x] The feed form notes the browser-account limitation and nominal-usage semantics.
- [x] Default feed naming includes the selected account.
- [x] README feed tables and configuration reference include `copilot-usage`.
- [x] Frontend catalog/default-name tests cover the new type.
