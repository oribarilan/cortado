# Plan: Status-Priority Session Deduplication

## Problem

When multiple sessions share the same CWD (e.g., two OpenCode instances in the same repo/branch), the current `deduplicate_sessions()` in `feed.rs` picks the winner by most recent `last_active_at`. This means a session with "question" status (attention needed) can be hidden by one that's merely "working" but wrote a file more recently.

## Solution

Change the dedup winner selection to prioritize **status urgency** over recency. Attention-needed statuses should always surface over active work, which surfaces over idle.

## Changes

### 1. Add `status_priority()` function (`feed.rs`)

New helper mapping `SessionStatus` to a numeric priority:

```rust
fn status_priority(status: SessionStatus) -> u8 {
    match status {
        SessionStatus::Question | SessionStatus::Approval => 2,
        SessionStatus::Working => 1,
        SessionStatus::Idle | SessionStatus::Unknown => 0,
    }
}
```

### 2. Update `deduplicate_sessions()` (`feed.rs:207-232`)

Replace the `last_active_at` comparison with status-priority-first logic:

```rust
fn deduplicate_sessions(sessions: Vec<SessionInfo>) -> Vec<SessionInfo> {
    let mut best_by_cwd: HashMap<String, SessionInfo> = HashMap::new();

    for session in sessions {
        let key = session.cwd.clone();

        let dominated = match best_by_cwd.get(&key) {
            None => false,
            Some(existing) => {
                let existing_prio = status_priority(existing.status);
                let session_prio = status_priority(session.status);

                if existing_prio != session_prio {
                    existing_prio > session_prio
                } else {
                    existing.last_active_at >= session.last_active_at
                }
            }
        };

        if !dominated {
            best_by_cwd.insert(key, session);
        }
    }

    best_by_cwd.into_values().collect()
}
```

### 3. Use stable CWD-derived activity ID (`feed.rs:316`)

Currently `activity.id = session.id`. When the dedup winner changes (e.g., session A gets a question, session B was previously winning), the activity ID flips — causing UI jank (detail pane losing focus).

Fix: when consolidating, derive a stable ID from the CWD. Since this only matters when there are multiple sessions per CWD, we can always use a CWD-based ID:

In `session_to_activity()`, change `id: session.id.clone()` to use a CWD-based hash:

```rust
// Stable ID from CWD — survives dedup winner changes.
use std::hash::{Hash, Hasher};
let mut hasher = std::collections::hash_map::DefaultHasher::new();
session.cwd.hash(&mut hasher);
let stable_id = format!("harness-{:x}", hasher.finish());
```

**Wait — this changes behavior for single-session cases too.** The activity ID would no longer match the session ID, which could break `find_session()` lookups. Better approach: only use the CWD-derived ID when there were duplicates. But `session_to_activity` doesn't know if dedup happened.

**Simpler approach:** In `deduplicate_sessions`, when a CWD key already exists (i.e., we're replacing), set the winner's `id` to a CWD-derived stable value. Single-session CWDs keep their original session ID.

```rust
fn deduplicate_sessions(sessions: Vec<SessionInfo>) -> Vec<SessionInfo> {
    let mut best_by_cwd: HashMap<String, SessionInfo> = HashMap::new();
    let mut had_duplicate: HashSet<String> = HashSet::new();

    for session in sessions {
        let key = session.cwd.clone();

        match best_by_cwd.get(&key) {
            None => {
                best_by_cwd.insert(key, session);
            }
            Some(existing) => {
                had_duplicate.insert(key.clone());
                let existing_prio = status_priority(existing.status);
                let session_prio = status_priority(session.status);
                let replace = if existing_prio != session_prio {
                    session_prio > existing_prio
                } else {
                    session.last_active_at > existing.last_active_at
                };
                if replace {
                    best_by_cwd.insert(key, session);
                }
            }
        }
    }

    // For CWDs that had duplicates, use a stable CWD-derived ID
    // so the activity row doesn't jump when the winner changes.
    best_by_cwd
        .into_iter()
        .map(|(cwd, mut session)| {
            if had_duplicate.contains(&cwd) {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                cwd.hash(&mut hasher);
                session.id = format!("harness-{:x}", hasher.finish());
            }
            session
        })
        .collect()
}
```

### 4. Update `find_session()` impact

`find_session()` looks up by `session.id` — but the cached sessions already have the stable ID applied (caching happens after dedup). The `focus_session` command passes the activity ID from the frontend, which will also be the stable ID. So this should work without changes.

### 5. Update tests

- **`deduplicate_keeps_most_recent_per_cwd`** — rename to `deduplicate_prefers_higher_status_priority`. Test that a Question session beats a Working session regardless of `last_active_at`.
- **`deduplicate_no_last_active_keeps_last_seen`** — keep but note it tests same-priority tiebreaking.
- Add **`deduplicate_attention_beats_working`** — Question + Working same CWD → Question wins.
- Add **`deduplicate_working_beats_idle`** — Working + Idle same CWD → Working wins.
- Add **`deduplicate_same_priority_uses_recency`** — Two Working sessions → most recent wins.
- Add **`deduplicate_stable_id_for_duplicates`** — verify the ID is CWD-derived when dedup happened.
- Add **`deduplicate_single_session_keeps_original_id`** — verify no-dedup case keeps session ID.
- Existing tests for different CWDs, empty list, single session remain unchanged.

## Files modified

| File | Change |
|------|--------|
| `src-tauri/src/feed/harness/feed.rs` | Add `status_priority()`, rewrite `deduplicate_sessions()`, update tests |

## Verification

`just check` — must pass cleanly.
