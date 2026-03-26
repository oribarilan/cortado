---
status: pending
---

# Optional — Digest delivery preset for notifications

## Goal

Add a "Digest" delivery preset that collects notification events across a configurable time window and sends a single summary notification.

## Context

Sprint 07 ships Immediate and Grouped delivery presets. Digest was deferred to reduce complexity — it requires a background timer, event buffer, and flush logic.

## Design

- **Digest** delivery preset: collect changes for a configurable window (default: 5 minutes), then send one summary notification.
- Requires a `digest_window_secs: u64` setting in `NotificationSettings`.
- Background task using `tokio::time::interval` or `tokio::time::sleep` to flush the buffer.
- Summary format: `{total_count} changes across {feed_count} feeds`

## Acceptance criteria

- [ ] `DeliveryPreset::Digest` variant added
- [ ] `digest_window_secs` config field added (default: 300)
- [ ] Background buffer collects `StatusChangeEvent`s across feeds
- [ ] Timer flushes buffer and sends summary notification
- [ ] Settings UI updated with Digest option and time window input
- [ ] Unit tests for buffer + flush logic
- [ ] `just check` passes
