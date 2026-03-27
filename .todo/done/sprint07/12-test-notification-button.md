---
status: done
---

# 12 -- Test notification button

## Goal

Add a "Send test notification" button to the Notifications settings tab so users can verify their notification setup works.

## Alternatives considered

### A. Frontend-only via JS API (Recommended)

Use `@tauri-apps/plugin-notification`'s JS `sendNotification()` API directly from the settings UI. No new Tauri command needed.

```ts
import { sendNotification } from "@tauri-apps/plugin-notification";
sendNotification({ title: "Cortado", body: "Notifications are working!" });
```

**Pros:** Zero backend changes. Simple. Immediate feedback.
**Cons:** Doesn't exercise the Rust dispatch pipeline.

### B. Backend command that fires through the dispatch pipeline

Add a `test_notification` Tauri command that calls the same `send_notification()` used by the real pipeline.

**Pros:** Proves the full pipeline works end-to-end.
**Cons:** More code. The dispatch pipeline needs an AppHandle, which the settings window already has via the plugin. Overkill for a "does it work?" check.

### C. Both: JS for quick test, Rust command for pipeline test

**Pros:** Comprehensive.
**Cons:** Two buttons is confusing UX. Over-engineered.

## Recommendation

**Option A** — frontend-only via JS API. It directly answers the user's question ("are notifications working on my machine?") with minimal code. If the notification shows up, the plugin + OS permissions are correctly configured.

## Acceptance criteria

- [ ] "Send test notification" button in the Notifications tab
- [ ] Button disabled when master toggle is off
- [ ] Fires a notification with title "Cortado" and body "Test notification -- notifications are working!"
- [ ] Handles permission-denied gracefully (show inline error, no crash)
- [ ] `just check` passes

## Relevant files

- `src/settings/SettingsApp.tsx` -- add button after permission status section
