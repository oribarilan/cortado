---
status: done
---

# Focus terminal: resolver + shared infrastructure

## Goal

Create the `TerminalFocusResolver` module and shared PID ancestry infrastructure. This is the skeleton that all focus strategies plug into.

macOS-only for now. Windows support is tracked in `.todo/backlog/optional-focus-terminal-windows.md`.

## Architecture

All focus logic lives in a single module:

```
src-tauri/src/terminal_focus/
  mod.rs              # TerminalFocusResolver, FocusContext, FocusResult, focus_terminal()
  pid_ancestry.rs     # Shared: PID walk via sysctl, tmux detection, terminal app discovery
```

### Core types

```rust
/// Context gathered during PID ancestry walk, shared by all strategies.
struct FocusContext {
    /// The copilot process PID (from lock file).
    copilot_pid: u32,
    /// Working directory of the session (from workspace.yaml). For CWD-based matching (Ghostty).
    cwd: String,
    /// Ancestor PIDs collected during the walk (copilot -> ... -> root).
    ancestors: Vec<u32>,
    /// If tmux was detected, the tmux server PID.
    tmux_server_pid: Option<u32>,
    /// The resolved terminal app PID (from direct walk or tmux client walk).
    terminal_app_pid: Option<u32>,
    /// The terminal app name (e.g., "Ghostty", "iTerm2", "Terminal").
    terminal_app_name: Option<String>,
    /// The terminal app bundle ID (e.g., "com.mitchellh.ghostty").
    terminal_app_bundle: Option<String>,
}

/// Result of a focus attempt.
enum FocusResult {
    /// Strategy succeeded -- stop the waterfall.
    Focused,
    /// Strategy doesn't apply to this context -- try the next one.
    NotApplicable,
    /// Strategy applies but failed -- try the next one.
    Failed(String),
}
```

### Resolver

```rust
pub fn focus_terminal(session: &SessionInfo) -> Result<(), String> {
    let ctx = build_focus_context(session)?;

    let strategies: &[(&str, fn(&FocusContext) -> FocusResult)] = &[
        ("tmux",             tmux::try_focus),
        ("terminal_script",  terminal_script::try_focus),
        ("accessibility",    accessibility::try_focus),
        ("app_activation",   app_activation::try_focus),
    ];

    for (name, strategy) in strategies {
        match strategy(&ctx) {
            FocusResult::Focused => return Ok(()),
            FocusResult::NotApplicable => continue,
            FocusResult::Failed(_) => continue,
        }
    }

    Err("Could not focus terminal: no strategy succeeded".into())
}
```

### PID ancestry walk (`pid_ancestry.rs`)

- `get_parent_pid(pid) -> Option<u32>` via `libc::sysctl(KERN_PROC_PID)`
- `get_process_name(pid) -> Option<String>` via `kinfo_proc.kp_proc.p_comm`
- `is_gui_app(pid) -> Option<(String, String)>` via `NSRunningApplication` -- returns (name, bundle_id) if the PID is a regular GUI app
- `build_focus_context(session: &SessionInfo) -> Result<FocusContext>` -- takes full `SessionInfo`, uses `pid` for ancestry walk and `cwd` for context. Walks ancestry, collects PIDs, detects tmux, resolves terminal app (direct or via tmux client).

### Frontend integration

New Tauri command in `command.rs`:

```rust
#[tauri::command]
async fn focus_session(session_id: String) -> Result<(), String>
```

The command accepts a session ID (not a raw PID). The backend looks up the `SessionInfo` from cached poll results in `HarnessFeed` and passes it directly to `focus_terminal(&session)`. `SessionInfo` carries pid, cwd, repo, branch -- everything strategies need today and in the future.

Frontend: add a parallel action path so `copilot-session` activities call `focus_session` instead of `open_activity`. Store the session ID in the activity's `id` field (already the session UUID).

## Acceptance criteria

- [ ] `src-tauri/src/terminal_focus/mod.rs` with `FocusContext`, `FocusResult`, `focus_terminal()`
- [ ] `src-tauri/src/terminal_focus/pid_ancestry.rs` with PID walk, tmux detection, GUI app lookup
- [ ] Resolver wires all 4 strategy slots (tmux + app_activation implemented, terminal_script + accessibility stubbed as `NotApplicable`)
- [ ] New Tauri command `focus_session(session_id)` in `command.rs`
- [ ] Frontend wires "open" for `copilot-session` activities to `focus_session`
- [ ] macOS-only: no-op or error on other platforms
- [ ] Unit tests: PID ancestry walk (mocked), resolver waterfall logic (mock strategies)
- [ ] `just check` passes

## Notes

- **`libc`** -- already added in task 01 (used here for `sysctl(KERN_PROC_PID)` ancestry walk).
- `cocoa`/`objc` via `tauri_nspanel` (already used in `fns.rs`).
- The resolver is the integration point -- individual strategies are separate tasks.
- The `app_activation` fallback strategy (just `NSRunningApplication.activate()`) is trivial and implemented directly in this task -- no separate task file needed. It activates the terminal app without targeting a specific window/pane.

## Relevant files

- `src-tauri/src/terminal_focus/` -- new module
- `src-tauri/src/command.rs` -- new `focus_session` command
- `src-tauri/src/feed/harness/` -- cache last poll results for session lookup
- `src/App.tsx`, `src/main-screen/MainScreenApp.tsx` -- wire focus action for `copilot-session` activities
- `src/shared/utils.ts` -- add `supportsFocus()` or parallel action path
- `src-tauri/Cargo.toml` -- add `libc`
