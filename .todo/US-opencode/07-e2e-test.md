---
status: done
---

# End-to-end validation

## Goal

Verify the full pipeline works: OpenCode plugin writes state, Cortado discovers and displays sessions.

## Acceptance criteria

- [ ] Install the plugin in an OpenCode project (local plugin path)
- [ ] Configure `opencode-session` feed in `feeds.toml`
- [ ] Start an OpenCode session -- verify it appears in Cortado tray/panel
- [ ] Verify status updates in near-real-time (working -> idle transitions, via FSEvents)
- [ ] End the OpenCode session -- verify the activity disappears from Cortado
- [ ] Kill OpenCode process (simulate crash) -- verify Cortado detects stale PID and cleans up
- [ ] Verify multiple simultaneous OpenCode instances are tracked correctly
- [ ] Verify `just check` still passes (including plugin)

## Notes

This is a manual validation step. Requires both OpenCode and Cortado running locally.

If OpenCode is not installed, validate the Cortado side by manually writing interchange JSON files to `~/.config/cortado/harness/` and verifying Cortado picks them up correctly. This tests the GenericProvider and FSEvents watcher without needing the plugin.

### Multi-instance test

Run two OpenCode sessions in different project directories simultaneously. Both should appear as separate activities in Cortado. Ending one should not affect the other.
