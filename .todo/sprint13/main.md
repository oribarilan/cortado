---
status: pending
---

# Sprint 13 -- Harness Session Feed

## Theme

Add a `copilot-session` feed type that tracks GitHub Copilot CLI sessions as activities, built on a generic **harness** abstraction. A coding harness is a terminal-based AI coding agent (Copilot CLI, Claude Code, etc.). The architecture separates the generic feed behavior (`HarnessFeed`) from harness-specific session discovery (`HarnessProvider` trait), so adding new harnesses later requires only a new provider implementation.

## Data source

The Copilot CLI writes session data to `~/.copilot/session-state/<session-id>/`:

- **`workspace.yaml`** — session metadata: id, cwd, repo, branch, summary, timestamps, host_type.
- **`events.jsonl`** — chronological event stream. Each line is a JSON object with `type`, `data`, `timestamp`.
- **`inuse.<PID>.lock`** — lock file present while the owning Copilot CLI process is alive. Contains the PID as text. Removed on clean shutdown.

### Active session detection

The lock file is the authoritative signal for "session is open":

1. Scan `~/.copilot/session-state/*/inuse.*.lock` files.
2. Extract the PID from the filename (or file contents — both contain it).
3. Check if the PID is still alive (`kill(pid, 0)` on Unix).
4. If alive → session is active. If dead → stale lock, skip.

This approach is precise (no heuristics) and cheap (glob + a few syscalls).

A single Copilot CLI process can own multiple sessions (via `/resume`). The feed shows **all** sessions with live locks to maximize recall.

### Key event types in events.jsonl

| Event type                | Meaning                                   |
|---------------------------|-------------------------------------------|
| `session.start`           | Session created or resumed                |
| `user.message`            | User submitted a prompt                   |
| `assistant.turn_start`    | Agent began processing                    |
| `assistant.message`       | Agent response (may contain tool requests)|
| `tool.execution_start`    | Tool executing                            |
| `tool.execution_complete` | Tool finished                             |
| `assistant.turn_end`      | Agent finished its turn                   |
| `abort`                   | User aborted the current operation        |
| `session.shutdown`        | Session ended cleanly                     |

### Status inference from last event

| Last event(s)                                        | Status value    | StatusKind        |
|------------------------------------------------------|-----------------|-------------------|
| `assistant.turn_start` / `tool.execution_start`      | `working`       | Running           |
| `assistant.message` with `ask_user` tool request     | `question`      | AttentionPositive |
| `assistant.message` with pending tool approval        | `approval`      | AttentionPositive |
| `user.message` (agent hasn't started responding yet) | `working`       | Running           |
| `assistant.message` with no tool requests             | `idle`          | Idle              |
| `assistant.turn_end` / `tool.execution_complete`     | `idle`          | Idle              |
| `session.shutdown`                                   | `ended`         | Idle              |
| `abort`                                              | `ended`         | Idle              |
| No events.jsonl / unparseable                        | `unknown`       | Idle              |

Note: `session.shutdown` and `abort` rows are documentation-only — sessions with no lock file are not discovered, so these statuses are unreachable in practice.

## Sequencing

```
01-session-state-reader ──────┐
                              │
02-copilot-session-feed ──────┤
                              │
03-focus-resolver-infra ──────┤
                              │
04-strategy-tmux ─────────────┤ (can parallel with 07)
                              │
05-strategy-terminal-script ──┤ (can parallel with 06, 07)
                              │
06-strategy-accessibility ────┤ (can parallel with 05, 07)
                              │
07-focus-settings-ui ─────────┘
```

- Tasks 01-03 are sequential (each depends on the previous).
- Task 04 (tmux strategy) depends on 03 (resolver infra).
- Tasks 05 and 06 are independent strategies — can be done in parallel, each depends on 03.
- Task 07 (settings UI) depends on 03 (needs the capability query), but can be built in parallel with 04-06.

### Sprint 13 scope vs future

Tasks 01-04 and 07 are the sprint 13 core. Tasks 05 and 06 are stretch goals — the resolver stubs them as `NotApplicable` until implemented, so they can ship in a follow-up without blocking.

## Tasks

| # | File | Summary |
|---|------|---------|
| 01 | `01-session-state-reader.md` | `HarnessProvider` trait + `CopilotProvider` (session discovery, status inference) |
| 02 | `02-copilot-session-feed.md` | `HarnessFeed` — generic Feed impl over any provider. Registered as `copilot-session`. |
| 03 | `03-focus-terminal-macos.md` | `TerminalFocusResolver` module, PID ancestry walk, frontend integration |
| 04 | `04-strategy-tmux.md` | tmux pane switching strategy |
| 05 | `05-strategy-terminal-script.md` | Per-terminal AppleScript strategy (Terminal.app, iTerm2, Ghostty) |
| 06 | `06-strategy-accessibility.md` | Accessibility API (AXRaise) strategy |
| 07 | `07-focus-settings-ui.md` | Settings section showing focus capabilities and enabling accessibility |

## Cross-cutting notes

- This feed has **no external CLI dependency** — it reads local files only.
- **Harness** = terminal-based AI coding agent (Copilot CLI, Claude Code, etc.). The `HarnessProvider` trait abstracts session discovery; `HarnessFeed` is the generic Feed impl. Adding a new harness = new provider file + registration in `instantiate_feed()`.
- Keep parsing resilient: unknown event types and missing/malformed fields should degrade gracefully to `unknown`/`Idle`, never panic or error the whole feed.
- The `~/.copilot/session-state/` directory can contain 2500+ directories, but only a handful have lock files (~12 observed). The glob for `inuse.*.lock` is fast.
- Cap activities at 20 (consistent with other feeds).
- `workspace.yaml` is a YAML file (~460 bytes). `events.jsonl` can be large (up to 15MB) but we only need the last line (`tail`-equivalent).
- The entire poll cycle should complete in <50ms.
- The "focus terminal" action (task 03) is macOS-only. Windows support is tracked in `.todo/backlog/optional-focus-terminal-windows.md`.
