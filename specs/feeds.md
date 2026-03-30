# Feeds

This document covers feed types beyond the basics in `main.md`. For shared feed concepts (config format, field overrides, retention, intervals), see `specs/main.md`.

## Harness feeds

A **harness** is a terminal-based AI coding agent — GitHub Copilot CLI, Claude Code, or similar. Harness feeds track active coding sessions as activities, showing their status, context, and providing one-click terminal focus.

### Architecture

The harness system separates generic feed behavior from agent-specific session discovery:

```
HarnessProvider (trait)          HarnessFeed (generic Feed impl)
  |                                 |
  |-- CopilotProvider               |-- maps SessionInfo -> Activity
  |-- (future: ClaudeCodeProvider)  |-- caches focus context per session
  '-- ...                           '-- registered in instantiate_feed()
```

**Adding a new harness** requires only a new `HarnessProvider` implementation — zero changes to `HarnessFeed`, the UI, or the config format. The provider discovers sessions and returns `Vec<SessionInfo>`; the feed handles everything else.

### `copilot-session` feed type

Tracks active GitHub Copilot CLI sessions by reading local session state files.

**Data source**: `~/.copilot/session-state/<session-id>/`

| File | Contents |
|------|----------|
| `workspace.yaml` | Session metadata: id, cwd, repo, branch, summary |
| `events.jsonl` | Chronological event stream (we read only the last line) |
| `inuse.<PID>.lock` | Lock file — present while the owning process is alive |

**Active session detection**: Glob `inuse.*.lock` files, extract PID from filename, check liveness via `kill(pid, 0)`. Dead PID = stale lock, skip. No heuristics.

**Config**:

```toml
[[feed]]
name = "copilot sessions"
type = "copilot-session"
```

No type-specific config keys. No external CLI dependency — reads local files only.

**Default interval**: `30s`.

**Provided fields**:

| Field | Type | Description |
|-------|------|-------------|
| `status` | status | Session status: working, question, approval, idle, unknown |
| `summary` | text | Agent-generated session description |
| `last_active` | text | Relative time since last event (e.g., "2m ago") |
| `repo` | text | Repository name (e.g., `oribarilan/cortado`) |
| `branch` | text | Git branch name |
| `focus_label` | text | Terminal focus action label (e.g., "Open in Ghostty (via tmux)") |

**Activity title**: `{short_repo} @ {branch}` — e.g., `cortado @ main`. Falls back to last directory component of `cwd` if repo is unknown.

**Activity identity**: Session UUID from `workspace.yaml`. Globally unique and stable.

**Deduplication**: Multiple sessions can exist for the same working directory (e.g., from Copilot CLI's `/resume`). The feed deduplicates by cwd, keeping only the most recently active session per directory.

### Status inference

Status is inferred from the last event in `events.jsonl`:

| Last event | Status value | StatusKind |
|------------|-------------|------------|
| `assistant.turn_start` / `tool.execution_start` / `user.message` | working | Running |
| `assistant.message` with `ask_user` tool request | question | AttentionPositive |
| `assistant.message` with other tool requests | approval | AttentionPositive |
| `assistant.message` with no tool requests | idle | Idle |
| `assistant.turn_end` / `tool.execution_complete` | idle | Idle |
| No events / unparseable | unknown | Idle |

### Terminal focus

When a user opens a copilot-session activity, Cortado focuses the terminal containing that session rather than opening a URL. The focus system resolves which terminal and strategy to use via a PID ancestry walk.

#### Focus context

On first poll for each session, Cortado walks the process tree from the copilot PID upward to discover:

- **Terminal app** — the GUI application (e.g., Ghostty, iTerm2, Terminal.app) identified via `NSRunningApplication`
- **tmux** — detected by process name in the ancestry chain

This context is cached per session ID for the session's lifetime (the terminal and tmux state don't change while a session is alive).

The context is surfaced as the `focus_label` field — e.g., "Open in Ghostty (via tmux)" — and used as the action button label in the UI.

#### Strategy waterfall

When the user triggers focus, strategies are tried in priority order:

| # | Strategy | Precision | Condition |
|---|----------|-----------|-----------|
| 1 | **tmux pane switching** | Exact pane | tmux detected in ancestry |
| 2 | Terminal-specific scripting | Tab/window | Scriptable terminal (stretch — stubbed) |
| 3 | Accessibility window focus | Window by title | AX permission granted (stretch — stubbed) |
| 4 | **App activation** (fallback) | App-level | Always available |

The first strategy that returns `Focused` wins. Strategies return `NotApplicable` (skip) or `Failed` (try next).

#### tmux strategy

When tmux is detected:

1. `tmux list-panes -a` — find the pane whose PID matches the copilot process (or its ancestor)
2. `tmux list-clients` — find a client, preferring one already attached to the target session
3. `tmux switch-client` + `tmux select-pane` — switch to the exact pane
4. Activate the terminal app

#### App activation fallback

Activates the terminal app via `System Events` AppleScript. Brings the app to front but can't target a specific window — may focus the wrong one if multiple are open.

### Performance

- Session discovery completes in <50ms for ~12 active sessions
- `events.jsonl` is read via reverse-seek (last 64KB) — never reads the full file
- Focus context resolved once per session, cached thereafter
- `~/.copilot/session-state/` can contain 2500+ directories but the lock file glob is fast

### Implementation

```
src-tauri/src/feed/harness/
  mod.rs          # SessionStatus, SessionInfo, HarnessProvider trait
  copilot.rs      # CopilotProvider: lock files, YAML, events.jsonl
  feed.rs         # HarnessFeed: Feed impl, focus context caching

src-tauri/src/terminal_focus/
  mod.rs          # FocusContext, FocusResult, strategy waterfall, capabilities
  pid_ancestry.rs # PID walk, tmux detection, GUI app lookup
  tmux.rs         # tmux pane switching strategy
```
