---
status: pending
---

# Loading animations for feed refresh

## Goal
Add loading/refresh animations to both the main window and the menubar panel when feeds are loading.

## Scenarios
- **Startup** -- feeds loading for the first time (no existing data yet).
- **Manual refresh** -- user clicks a refresh button.
- **Refresh interval** -- automatic background refresh while existing data is already displayed.

## Notes
- Background refresh (when data is already showing) should be very subtle -- possibly just a small indicator. Might not want it at all; needs visual experimentation first.
- Startup / manual refresh can be more prominent since the user is actively waiting.
- Applies to both the main app window and the menubar panel.
