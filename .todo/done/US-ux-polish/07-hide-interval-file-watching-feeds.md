---
status: done
---

# Hide interval for file-watching feed types

## Problem

Harness-based feeds (Copilot CLI, OpenCode, and future file-watching feeds) don't use poll intervals -- they work via file watching. The interval is already hidden in the feed editor form (via `hideInterval` in `FEED_CATALOG`), but the feed card in the feed list still shows the interval badge (e.g., "↻ 5m"), which is confusing.

## Goal

Don't display the interval on feed cards for feed types that have `hideInterval: true`.

## Change

In `src/settings/SettingsApp.tsx` (~line 1964), the interval display:

```tsx
{feed.interval && (
  <span className="feed-card-detail">
    <span className="feed-card-detail-icon">↻</span> {feed.interval}
  </span>
)}
```

Should also check `hideInterval`:

```tsx
{feed.interval && !findFeedType(feed.type)?.hideInterval && (
  ...
)}
```

## Acceptance criteria

- [ ] Feed cards for harness-based feeds (copilot-session, opencode-session) don't show an interval
- [ ] Feed cards for poll-based feeds (github-pr, github-actions, etc.) still show their interval
- [ ] `just check` passes

## Scope Estimate

Tiny
