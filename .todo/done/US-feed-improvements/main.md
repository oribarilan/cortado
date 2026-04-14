# US: Feed Improvements

## Theme

Two independent quality-of-life improvements to the feed system:

1. **Update feed: open action in panel** -- The auto-update feed's "Install update" / "Update plugin" actions work in both tray and panel, but the panel has no way to *open* the update (e.g., view the release on GitHub). Add an "open" action so pressing Enter navigates to the release page, matching the pattern of other feeds.

2. **Feed uniqueness / deduplication** -- Some feed types return historical runs that are redundant. For example, GitHub Actions returns the last N runs of each workflow file, but usually only the latest run per workflow matters. Add a uniqueness mechanism so feeds can deduplicate activities by a grouping key, keeping only the most recent per group.

## Sequencing

Tasks are independent and can be done in parallel.

- Task 01 is frontend + backend (update feed needs a URL in its activity, panel needs to handle it).
- Task 02 is primarily backend (dedup logic in the feed layer) with no frontend changes expected.
