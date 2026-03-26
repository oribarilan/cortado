---
status: pending
---

# 10 — Integration testing & edge cases

## Goal

Comprehensive testing of the full notification pipeline, including edge cases and error conditions.

## Test scenarios

### Core flow
- [ ] Status kind change triggers notification (happy path)
- [ ] New activity triggers notification when toggle is on
- [ ] Removed activity triggers notification when toggle is on
- [ ] No notification fires when global toggle is off
- [ ] No notification fires for feed with `notify = false`

### Notification modes
- [ ] `All` mode: any rollup kind change triggers notification
- [ ] `EscalationOnly` mode: only higher-priority transitions trigger (Idle→AttentionNegative: yes, AttentionNegative→Idle: no)
- [ ] `SpecificKinds` mode: only configured destination kinds trigger

### Delivery presets
- [ ] `Immediate`: one notification per change event
- [ ] `Grouped`: multiple changes in one poll → single notification per feed

### Edge cases
- [ ] Startup suppression: no notifications during `seed_startup_best_effort`
- [ ] Rapid status flapping (kind changes back and forth quickly): each change is a separate event
- [ ] Retained activities: status frozen at Idle, no spurious notifications
- [ ] Errored feeds: poll errors don't trigger status change notifications
- [ ] Empty feed (no activities): no spurious notifications
- [ ] Feed added/removed while app is running: no crash
- [ ] Notification permission denied: handled gracefully, no crash, user informed

### Config changes
- [ ] Changing notification mode takes effect on next poll
- [ ] Changing delivery preset takes effect on next poll
- [ ] Toggling per-feed notify takes effect after config reload

## Acceptance criteria

- [ ] All test scenarios above have corresponding unit or integration tests
- [ ] No warnings from `just check`
- [ ] Manual verification of OS notification appearance on macOS

## Notes

- Unit tests cover the diff engine and filtering logic in isolation.
- Integration tests verify the full pipeline from poll result to notification dispatch.
- Manual testing is needed for actual OS notification appearance, permission prompts, and click actions.
