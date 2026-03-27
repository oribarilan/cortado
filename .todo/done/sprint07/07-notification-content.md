---
status: done
---

# 07 — Notification content formatting

## Goal

Design and implement the notification title/body text for all delivery modes. Content should be concise, informative, and use human-friendly language.

## Content templates

### Single activity (Immediate mode, or Grouped with 1 change)

**New activity:**
- Title: `{feed_name}`
- Body: `New: {activity_title}`

**Removed activity:**
- Title: `{feed_name}`
- Body: `Gone: {activity_title}`

**Status change:**
- Title: `{feed_name}`
- Body: `{activity_title} — {human_kind_name}`

### Grouped (multiple changes in one feed)

- Title: `{feed_name}`
- Body: `{count} activities changed` followed by first 2-3 activity titles

## Human-friendly kind names

| StatusKind | Human name |
|-----------|-----------|
| AttentionNegative | needs attention |
| AttentionPositive | ready to go |
| Waiting | waiting |
| Running | in progress |
| Idle | idle |

## Acceptance criteria

- [ ] Formatter function(s) that produce title + body from `StatusChangeEvent`(s)
- [ ] Single and grouped templates implemented
- [ ] Human-friendly kind names (not raw enum names)
- [ ] Activity titles are truncated if too long (macOS notification body has practical limits)
- [ ] Unit tests for each template variant
- [ ] `just check` passes

## Notes

- macOS notifications support a title, subtitle, and body. Consider using subtitle for the activity title in single-change notifications.
- Keep text short — notifications are glanceable, not detailed.
- The status value (e.g., "approved", "failing") might be more useful than the kind name in some cases. Consider showing both: "PR #42 — checks failing" vs "PR #42 — needs attention". Decide during implementation.

## Relevant files

- Task 05 output — `StatusChangeEvent` types
- Task 06 output — dispatch pipeline (consumes formatted content)
