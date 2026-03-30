---
status: pending
---

# Comprehensive terminal focus tests

## Goal

Add thorough unit and integration tests for the terminal focus system, covering the waterfall logic, PID ancestry resolution, tmux client mapping, and each terminal strategy.

## Test areas

### Unit tests

- **Waterfall logic**: strategy ordering, enabled/disabled, first-match-wins, all-fail error
- **PID ancestry**: parent PID resolution, process name matching, known terminal detection
- **tmux client resolution**: correct session mapping, multi-client scenarios, no-client fallback
- **Focus context building**: with/without tmux, with/without terminal detected
- **Each terminal strategy**: bundle ID gating, `NotApplicable` for wrong terminal, AppleScript/CLI output parsing

### Integration tests (with mock data)

- **Full waterfall with mock strategies**: verify correct strategy is called based on context
- **tmux output parsing**: realistic `list-panes`, `list-clients` output
- **Session deduplication**: multiple sessions same CWD, most recent wins
- **Focus info resolution**: correct app name and tmux detection per session

### Edge cases

- Copilot process dead between poll and focus (stale session cache)
- tmux not installed but tmux_enabled is true
- Terminal process name variations (e.g., `ghostty` vs `Ghostty`)
- Multiple terminals open simultaneously
- No terminal detected (fallback to app activation)

## Acceptance criteria

- [ ] Waterfall unit tests with mock strategies
- [ ] PID ancestry tests for all known terminals
- [ ] tmux client resolution tests (correct session, multi-client)
- [ ] Each terminal strategy has bundle ID gating test
- [ ] Integration test: full flow from SessionInfo to FocusResult
- [ ] Edge case coverage
- [ ] `just check` passes
