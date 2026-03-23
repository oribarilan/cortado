---
status: pending
---

# Optional: persist retained activities across restarts

## Goal

Keep retained activities visible after app restart (until their retention duration expires).

## Notes

- Current behavior is in-memory only; retained activities are lost on restart.
- Evaluate persistence options:
  - **JSON state file** (`~/.config/cortado/state.json`): simplest, low overhead, likely sufficient.
  - **SQLite**: stronger querying/concurrency guarantees, likely overkill for this scope.
- If implemented, persist only retained activity state (not full poll cache).
- Writes should be atomic (`.tmp` + rename) and resilient to partial/corrupt files.
- Expired retained activities should be pruned during load/save.
- Requires spec update first because current non-goals still exclude persistent storage beyond config.
