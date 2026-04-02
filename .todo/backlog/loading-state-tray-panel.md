---
status: done
---

# Loading state for tray and panel on first launch

## Problem

When the app first loads, the tray and panel show empty/blank content until feeds finish their initial poll. There's no visual indication that data is loading.

## Goal

Add a loading animation (shimmer skeletons or similar) to the tray and panel so users see immediate feedback on first launch while feeds are being fetched.

## Notes

- The panel already has skeleton CSS (`.skeleton`, `.loading-state` in `src/styles.css`) and the main-screen has its own (`.ms-skeleton`, `.ms-loading-state` in `src/main-screen/main-screen.css`). Check if these are wired up or unused.
- The tray (native menu) may need a different approach since it's not HTML-rendered — possibly a "Loading..." menu item or spinner text.
- Consider whether to show skeletons per-feed or a single global loading state.

## Relevant files

- `src/styles.css` — existing skeleton classes
- `src/main-screen/main-screen.css` — existing skeleton classes
- `src/App.tsx` — panel rendering logic
- `src-tauri/src/tray.rs` — tray menu construction
