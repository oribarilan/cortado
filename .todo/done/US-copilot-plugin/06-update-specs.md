---
status: done
---

# Update specs, docs, and changelog

## Goal

Update all specification and documentation files to reflect the new Copilot CLI extension-based session tracking.

## Acceptance criteria

- [ ] `specs/feeds.md`:
  - `copilot-session` section rewritten: now uses interchange format via Cortado extension (not native file reading)
  - Remove references to `workspace.yaml`, `events.jsonl`, lock files, status inference
  - Document the extension's event-to-status mapping
  - Reference the extension system (`~/.copilot/extensions/cortado/extension.mjs`)
  - Update the architecture diagram (CopilotProvider -> GenericProvider("copilot"))
- [ ] `specs/main.md`:
  - `copilot-session` field definitions updated if any changed
  - Config examples remain valid
  - Default interval stays `30s`
- [ ] `specs/glossary.md`:
  - Update Harness definition to mention Copilot CLI extension alongside OpenCode plugin
  - Ensure `CopilotProvider` references are removed (it no longer exists)
- [ ] `specs/harness-interchange.md`:
  - Add Copilot as a concrete example alongside OpenCode
  - Note `process.ppid` PID strategy for child-process extensions
- [ ] `README.md`:
  - Update copilot-session entry in feed types table
  - Note that Cortado extension must be installed
- [ ] `CHANGELOG.md`:
  - User-facing entry, e.g.: "Copilot: streamlined session tracking with an installable extension -- more accurate status, including notifications when Copilot asks a question"
- [ ] `AGENTS.md`:
  - Update "Adding a new feed type" section if the copilot changes affected the pattern
  - `plugins/copilot/` mentioned in directory structure

## Notes

Keep changelog entry user-facing and brief per AGENTS.md conventions. No implementation details like "replaced CopilotProvider with GenericProvider" -- describe what the user sees.
