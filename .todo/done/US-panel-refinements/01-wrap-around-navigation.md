---
status: done
---

# Wrap-around keyboard navigation in panel activity list

## Goal

When navigating the panel's activity list with arrow keys (or j/k), pressing Down on the last item should wrap to the first item, and pressing Up on the first item should wrap to the last. This is standard behavior for circular list navigation in macOS utilities.

## Current behavior

In `src/main-screen/MainScreenApp.tsx` (lines 367-378), navigation is clamped:

```ts
// Down: stops at last item
setFocusIndex((i) => Math.min(i + 1, flatList.length - 1));

// Up: stops at first item
setFocusIndex((i) => Math.max(i - 1, 0));
```

## Changes needed

| File | Location | Change |
|------|----------|--------|
| `src/main-screen/MainScreenApp.tsx` | ~line 370 | Down: wrap to 0 when at `flatList.length - 1` |
| `src/main-screen/MainScreenApp.tsx` | ~line 377 | Up: wrap to `flatList.length - 1` when at 0 |

Replace the clamped arithmetic with modular wrap:

```ts
// Down
setFocusIndex((i) => (i + 1) % flatList.length);

// Up
setFocusIndex((i) => (i - 1 + flatList.length) % flatList.length);
```

## Acceptance criteria

- [x] Pressing Down (or j) on the last activity wraps focus to the first activity
- [x] Pressing Up (or k) on the first activity wraps focus to the last activity
- [x] Scroll-into-view still works correctly after wrapping (the list scrolls to the newly focused row)
- [x] Navigation still works normally for all other positions in the list
- [x] `just check` passes
