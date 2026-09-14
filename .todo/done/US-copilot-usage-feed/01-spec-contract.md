---
status: done
---

# Specify the Copilot usage feed

## Goal

Record the approved feed, authentication, threshold, field, action, and error contracts before implementation.

## Acceptance criteria

- [x] `specs/main.md` lists `copilot-usage`, its default interval, config keys, and external CLI requirement.
- [x] `specs/feeds.md` defines the endpoint, Activity shape, field mappings, threshold behavior, action URL, and failure behavior.
- [x] `specs/glossary.md` defines nominal usage without conflating it with billed spend.
- [x] The contract labels the feed experimental and limits initial support to `github.com`.

## Notes

The endpoint is undocumented even though first-party clients use it. Missing or legacy-unit data must not be presented as zero or healthy.
