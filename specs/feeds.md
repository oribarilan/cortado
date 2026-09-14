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
  |- returns Vec<Activity> -- one activity if update available, empty if current
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

**Activity identity**: `cortado-update-vX.Y.Z` -- unique per version.

**Behavior**:
- When app is up to date: feed produces no activities (hidden from view).
- When update available: one activity with `AttentionPositive` status. No dismiss -- stays visible until installed or app restarts at the new version.
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

## Copilot usage feed

An experimental GitHub feed that tracks one account's Copilot AI credit consumption. It is intended for ordinary users and does not require organization or enterprise billing-admin access.

### Feed type: `copilot-usage`

**Data source**: the undocumented `GET https://api.github.com/copilot_internal/user` endpoint used by first-party GitHub clients.

**Support boundary**: `github.com` accounts only. GHE.com and GitHub Enterprise Server are not supported initially.

**Default interval**: `120s`.

**Authentication**:

1. The required `account` config value names a GitHub login already authenticated in `gh`.
2. Cortado resolves that account with `gh auth token --hostname github.com --user <account>`.
3. The token is passed only through the quota request's child-process environment. It must never appear in command display strings, debug output, errors, config, or logs.
4. Cortado must not call `gh auth switch` or otherwise change the globally active account.
5. The quota response's `login` must match `account` case-insensitively. A mismatch is a poll error.

The endpoint is undocumented and may change without notice. Settings and documentation must label the feed experimental.

**Config**:

```toml
[[feed]]
name = "Copilot usage"
type = "copilot-usage"
account = "octocat"
reference_amount_usd = 2000
attention_at_percent = 80
# details_url = "https://example.com/copilot-consumption"
interval = "120s"
```

| Key | Required | Default | Contract |
|-----|----------|---------|----------|
| `account` | Yes | | Non-empty GitHub login authenticated for `github.com` in `gh` |
| `reference_amount_usd` | Yes | | Finite number greater than zero; comparison value only |
| `attention_at_percent` | No | `80` | Finite number from 1 through 100 |
| `details_url` | No | `https://github.com/settings/copilot` | HTTPS Activity action URL without embedded credentials |

The Settings form must note: "Links open with your browser's active GitHub session, which may differ from the GitHub account selected above."

### Usage calculation and status

The feed reads `quota_snapshots.premium_interactions`. It must require:

- top-level `token_based_billing = true`
- a finite, non-negative `credits_used`
- a response `login` matching the configured account

Nominal usage and utilization are calculated as:

```text
nominal_usage_usd = credits_used * 0.01
utilization_percent = nominal_usage_usd / reference_amount_usd * 100
```

The $0.01 conversion is GitHub's published value for one AI credit at the time of implementation. Recheck the published rate when maintaining this feed. Nominal Usage is not billed spend, invoice charges, an enforced budget, or an employer's internal cost.

Status mapping:

- `utilization_percent < attention_at_percent` -> `"<percent>% used"` (`Idle`)
- `utilization_percent >= attention_at_percent` -> `"<percent>% used"` (`AttentionNegative`)

Utilization above 100% is valid and remains `AttentionNegative`. Notifications follow the normal Status Kind transition model, so the initial threshold crossing notifies according to user settings; later numeric increases within `AttentionNegative` do not.

### Activity contract

The feed always returns one Activity after a successful poll.

**Activity title**: the verified GitHub login.

**Activity action**: open `details_url`, or GitHub Copilot settings when no override is configured. The browser may be signed into a different GitHub account; Cortado does not attempt to switch browser sessions.

**Provided fields**:

| Field | Type | Description |
|-------|------|-------------|
| `usage` | status | Utilization percentage and threshold-derived Status Kind |
| `nominal_usage` | text | AI credits converted at $0.01 per credit, formatted as USD |
| `reference_amount` | text | Configured USD comparison amount |
| `credits_used` | number | Individual AI credits reported directly by GitHub |
| `reset` | text | `quota_reset_date_utc`, falling back to `quota_reset_date` or `unknown` |

Use `credits_used` directly. Do not derive total consumption from `entitlement - quota_remaining`; authenticated research found those values can diverge.

### Failure behavior

Failures are feed-level poll errors and preserve previously cached Activities through the normal runtime behavior.

- Missing `gh`: use the canonical GitHub CLI dependency error.
- Selected account unavailable: identify the account and tell the user to authenticate it with `gh auth login --hostname github.com`.
- HTTP/API or malformed JSON failure: provide concise request/parsing context without including raw successful responses or credentials.
- Login mismatch: identify the expected and returned logins.
- `token_based_billing` false or absent: report that legacy request units cannot be converted to nominal USD.
- Missing, negative, invalid, or unrepresentably large `credits_used`: report that GitHub did not return usable AI credit consumption.

Parse only the allowlisted fields needed by this contract. The raw endpoint response includes unrelated account, organization, analytics, and service metadata that must not be retained or logged.

### Implementation

```text
src-tauri/src/feed/copilot_usage.rs  # Feed implementation and response parsing
src-tauri/src/feed/process.rs        # Secret-safe per-command environment support
src/shared/feedTypes.ts              # Experimental Settings catalog entry
```

## Harness feeds

A **harness** is a terminal-based AI coding agent -- GitHub Copilot CLI, Claude Code, or similar. Harness feeds track active coding sessions as activities, showing their status, context, and providing one-click terminal focus.

### Architecture

The harness system separates generic feed behavior from agent-specific session discovery:

```
HarnessProvider (trait)          HarnessFeed (generic Feed impl)
  |                                 |
  |-- GenericProvider("copilot")    |-- maps SessionInfo -> Activity
  |-- GenericProvider("opencode")   |-- caches focus context per session
  '-- GenericProvider("...")        '-- registered in instantiate_harness_feed()
```

**Adding a new harness** requires only a new `HarnessProvider` implementation -- zero changes to `HarnessFeed`, the UI, or the config format. The provider discovers sessions and returns `Vec<SessionInfo>`; the feed handles everything else. All current harness feeds use `GenericProvider` backed by the interchange format; agent-specific logic lives in plugins that write the interchange files.

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
| `postToolUse` | current status is not `question` | working |
| `postToolUse` | current status is `question` | (no write -- preserves question) |
| `sessionEnd` | | idle |

**Hook ordering quirks**: Copilot CLI batches tool requests. When `ask_user` is called alongside `report_intent`, the hooks fire as: `preToolUse(report_intent)` -> `preToolUse(ask_user)` -> `postToolUse(report_intent)` -> (user answers) -> `postToolUse(ask_user)`. Without protection, `postToolUse(report_intent)` would overwrite the `question` status set by `preToolUse(ask_user)`. The plugin reads the current file status and refuses to overwrite `question` in `postToolUse`.

In prompt mode (`-p`), `userPromptSubmitted` fires before `sessionStart`. The plugin skips `sessionStart` when a file already exists.

`sessionEnd` writes `idle` instead of deleting the file, matching OpenCode behavior. GenericProvider's PID liveness check cleans up the file once the copilot process exits.

### Terminal focus

When a user opens a harness feed activity (any coding agent session), Cortado focuses the terminal containing that session rather than opening a URL. The focus system resolves which terminal and strategy to use via a PID ancestry walk.

Focus eligibility is detected by the presence of a `focus_app` field on the activity -- any harness feed that provides a PID gets focus support automatically.

#### Focus context

On first poll for each session, Cortado walks the process tree from the session PID upward to discover:

- **Terminal app** -- the GUI application (e.g., Ghostty, iTerm2, Terminal.app) identified via `NSRunningApplication`
- **tmux** -- detected by process name in the ancestry chain

This context is cached per session ID for the session's lifetime (the terminal and tmux state don't change while a session is alive).

The context is surfaced as the `focus_label` field -- e.g., "Open in Ghostty (via tmux)" -- and used as the action button label in the UI.

#### Two-phase focus

When the user triggers focus, the system runs two phases:

**Phase 1 -- tmux pre-step** (if tmux detected and enabled): navigates to the exact pane within tmux. Does not activate the terminal app.

**Phase 2 -- terminal strategy waterfall**: tries terminal-specific strategies by bundle ID, then falls back to app activation.

| # | Strategy | Precision | Condition |
|---|----------|-----------|-----------|
| 1 | Terminal-specific scripting | Tab/window | Scriptable terminal (Ghostty, iTerm2, etc.) |
| 2 | Accessibility window focus | Window by title | AX permission granted (stretch -- stubbed) |
| 3 | **App activation** (fallback) | App-level | Always available |

The first strategy that returns `Focused` wins. Strategies return `NotApplicable` (skip) or `Failed` (try next).

See `specs/terminal_integration.md` for the full architecture, supported terminals, and integration details.

#### tmux strategy

When tmux is detected:

1. `tmux list-panes -a` -- find the pane whose PID matches the session process (or its ancestor)
2. `tmux list-clients` -- find a client, preferring one already attached to the target session
3. `tmux switch-client` + `tmux select-pane` -- switch to the exact pane
4. Activate the terminal app

#### App activation fallback

Activates the terminal app via `System Events` AppleScript. Brings the app to front but can't target a specific window -- may focus the wrong one if multiple are open.

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
