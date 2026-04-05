# Feeds

This document covers feed types beyond the basics in `main.md`. For shared feed concepts (config format, field overrides, retention, intervals), see `specs/main.md`.

## Cortado Update feed

A **built-in** feed that checks for new Cortado versions and surfaces update availability as a standard activity.

### Architecture

Unlike user-configured feeds, the update feed is always registered and not parsed from `feeds.toml`. It is added to the feed registry in `main.rs` after loading user-configured feeds.

```
CortadoUpdateFeed (implements Feed trait)
  |
  |- poll(): fetches latest.json from GitHub Releases via reqwest
  |- compares remote version with current app version (semver)
  |- returns Vec<Activity> — one activity if update available, empty if current
  |- activity has StatusKind::AttentionPositive
  |- frontend "Install update" button triggers install_update Tauri command
  |- install_update uses tauri-plugin-updater to download, verify, install, restart
```

### Feed type: `cortado-update`

**Data source**: `latest.json` from `https://github.com/oribarilan/cortado/releases/latest/download/latest.json`

**Default interval**: `6h` (21600s).

**Provided fields**:

| Field | Type | Description |
|-------|------|-------------|
| `status` | status | "update available" with `AttentionPositive` kind |
| `version` | text | Available version (e.g., "v0.5.0") |
| `notes` | text | Release notes from `latest.json` |

**Activity title**: `Cortado vX.Y.Z available`

**Activity identity**: `cortado-update-vX.Y.Z` — unique per version.

**Behavior**:
- When app is up to date: feed produces no activities (hidden from view).
- When update available: one activity with `AttentionPositive` status. No dismiss — stays visible until installed or app restarts at the new version.
- Expanding the activity shows release notes and an "Install update" action button.
- Clicking "Install update" triggers the Tauri updater plugin to download, verify signature, install, and restart the app.

### Implementation

```
src-tauri/src/feed/cortado_update.rs  # Feed implementation
src-tauri/src/command.rs              # install_update Tauri command
src-tauri/src/main.rs                 # Built-in feed registration
src/App.tsx                           # Frontend update button rendering
src/shared/utils.ts                   # supportsUpdate() helper
```

## Harness feeds

A **harness** is a terminal-based AI coding agent — GitHub Copilot CLI, Claude Code, or similar. Harness feeds track active coding sessions as activities, showing their status, context, and providing one-click terminal focus.

### Architecture

The harness system separates generic feed behavior from agent-specific session discovery:

```
HarnessProvider (trait)          HarnessFeed (generic Feed impl)
  |                                 |
  |-- GenericProvider("copilot")    |-- maps SessionInfo -> Activity
  |-- GenericProvider("opencode")   |-- caches focus context per session
  '-- GenericProvider("...")        '-- registered in instantiate_harness_feed()
```

**Adding a new harness** requires only a new `HarnessProvider` implementation — zero changes to `HarnessFeed`, the UI, or the config format. The provider discovers sessions and returns `Vec<SessionInfo>`; the feed handles everything else. All current harness feeds use `GenericProvider` backed by the interchange format; agent-specific logic lives in plugins that write the interchange files.

### `copilot-session` feed type

Tracks active GitHub Copilot CLI sessions via the generic harness interchange format. Session state is written by the **Cortado plugin** for Copilot CLI, which uses the Copilot CLI plugin system (hooks-based) to write interchange files consumed by `GenericProvider("copilot")`.

**Data source**: `~/.config/cortado/harness/<pid>.json` (files written by the Cortado Copilot CLI plugin)

**Plugin**: Installed via `copilot plugin install` into `~/.copilot/installed-plugins/`. The plugin uses shell-based hooks (`hooks.json`) to write interchange files. Cortado offers a one-click install button when you add this feed type.

**Config**:

```toml
[[feed]]
name = "Copilot"
type = "copilot-session"
```

No type-specific config keys. Default interval: 30s, with near-instant FSEvents-based detection.

**Provided fields**: same as `opencode-session` (status, summary, last_active, repo, branch, focus_app, focus_has_tmux).

**Activity title**: `{short_repo} @ {branch}` (same format as other harness feeds).

**How it works**: The Cortado plugin registers with Copilot CLI via `copilot plugin install` and defines shell-based hooks in `hooks.json`. Each hook invocation writes session state to the harness directory as an interchange file. The `GenericProvider("copilot")` reads these files, checks PID liveness, and returns active sessions. FSEvents watching triggers near-instant re-polls when files change.

See `specs/harness-interchange.md` for the full interchange format specification.

#### Plugin hook-to-status mapping

The Cortado plugin maps Copilot CLI hooks to interchange statuses:

| Hook | Condition | Interchange status |
|------|-----------|-------------------|
| `sessionStart` | file doesn't exist yet | working |
| `userPromptSubmitted` | | working |
| `preToolUse` | `toolName` is `ask_user` | question |
| `preToolUse` | other tools | working |
| `postToolUse` | | working |
| `sessionEnd` | | (deletes the interchange file) |

### Terminal focus

When a user opens a harness feed activity (any coding agent session), Cortado focuses the terminal containing that session rather than opening a URL. The focus system resolves which terminal and strategy to use via a PID ancestry walk.

Focus eligibility is detected by the presence of a `focus_app` field on the activity — any harness feed that provides a PID gets focus support automatically.

#### Focus context

On first poll for each session, Cortado walks the process tree from the session PID upward to discover:

- **Terminal app** — the GUI application (e.g., Ghostty, iTerm2, Terminal.app) identified via `NSRunningApplication`
- **tmux** — detected by process name in the ancestry chain

This context is cached per session ID for the session's lifetime (the terminal and tmux state don't change while a session is alive).

The context is surfaced as the `focus_label` field — e.g., "Open in Ghostty (via tmux)" — and used as the action button label in the UI.

#### Two-phase focus

When the user triggers focus, the system runs two phases:

**Phase 1 — tmux pre-step** (if tmux detected and enabled): navigates to the exact pane within tmux. Does not activate the terminal app.

**Phase 2 — terminal strategy waterfall**: tries terminal-specific strategies by bundle ID, then falls back to app activation.

| # | Strategy | Precision | Condition |
|---|----------|-----------|-----------|
| 1 | Terminal-specific scripting | Tab/window | Scriptable terminal (Ghostty, iTerm2, etc.) |
| 2 | Accessibility window focus | Window by title | AX permission granted (stretch — stubbed) |
| 3 | **App activation** (fallback) | App-level | Always available |

The first strategy that returns `Focused` wins. Strategies return `NotApplicable` (skip) or `Failed` (try next).

See `specs/terminal_integration.md` for the full architecture, supported terminals, and integration details.

#### tmux strategy

When tmux is detected:

1. `tmux list-panes -a` — find the pane whose PID matches the session process (or its ancestor)
2. `tmux list-clients` — find a client, preferring one already attached to the target session
3. `tmux switch-client` + `tmux select-pane` — switch to the exact pane
4. Activate the terminal app

#### App activation fallback

Activates the terminal app via `System Events` AppleScript. Brings the app to front but can't target a specific window — may focus the wrong one if multiple are open.

### Implementation

```
src-tauri/src/feed/harness/
  mod.rs          # SessionStatus, SessionInfo, HarnessProvider trait
  generic.rs      # GenericProvider: reads interchange format JSON files
  feed.rs         # HarnessFeed: Feed impl, focus context caching

src-tauri/src/feed/
  harness_watcher.rs  # FSEvents-based file watching for harness feeds

plugins/copilot/      # Cortado plugin for Copilot CLI (hooks-based)

src-tauri/src/terminal_focus/
  mod.rs          # FocusContext, FocusResult, two-phase focus_terminal(), capabilities
  pid_ancestry.rs # PID walk, tmux detection, GUI app lookup
  tmux.rs         # Phase 1: tmux pane navigation pre-step
```

### `opencode-session` feed type

Tracks active OpenCode coding sessions via the generic harness interchange format.

**Data source**: `~/.config/cortado/harness/<pid>.json` (files written by the `cortado-opencode` plugin)

**Config**:

```toml
[[feed]]
name = "OpenCode"
type = "opencode-session"
```

No type-specific config fields. Default interval: 30s, with near-instant FSEvents-based detection.

**Provided fields**: same as `copilot-session` (status, summary, last_active, repo, branch, focus_app, focus_has_tmux).

**Activity title**: `{short_repo} @ {branch}` (same format as other harness feeds).

**How it works**: The `cortado-opencode` OpenCode plugin listens to `session.status` events and writes session state to the interchange directory. The `GenericProvider("opencode")` reads these files, checks PID liveness, and returns active sessions. FSEvents watching triggers near-instant re-polls when files change.

See `specs/harness-interchange.md` for the full interchange format specification.
