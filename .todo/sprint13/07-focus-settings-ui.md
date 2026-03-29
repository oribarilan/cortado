---
status: pending
---

# Focus settings UI

## Goal

Add a "Focus" section to the Settings sidebar that helps users understand and improve the terminal focus behavior. This section shows the current focus capability, explains what each strategy does, and guides users to enable higher-precision options.

## Design

### New settings section: "Focus"

Add a fourth top-level section to the settings sidebar (alongside General, Feeds, Notifications):

**Focus** — controls how cortado focuses terminal windows when opening a copilot session activity.

### Section content

```
Focus
-----

How cortado focuses your terminal when you open a Copilot session.

[Current capability]
  Detected: Ghostty + tmux
  Best strategy: tmux pane switching (exact)

[Strategies]  (ordered by precision, top = best)

  1. tmux pane switching                        [Active]
     Switches to the exact tmux pane.
     Detected automatically when tmux is in use.

  2. Terminal-specific scripting                 [Not available]
     Focuses the specific tab/window in your terminal.
     Supported: Terminal.app, iTerm2, Ghostty 1.3+
     Your terminal: Ghostty 1.2.3 (not yet supported)

  3. Accessibility window focus                  [Not enabled]
     Raises the specific window by matching its title.
     Works with any terminal but less precise.
     Requires Accessibility permission.
     [Open System Settings...]

  4. App activation (fallback)                   [Active]
     Brings the terminal app to front.
     Always available. May focus wrong window
     if multiple are open.
```

### Behavior

- The section is **read-only / informational** for strategies 1, 2, and 4 (auto-detected).
- Strategy 3 (Accessibility) has an actionable toggle:
  - Shows current permission status (granted / not granted).
  - "Open System Settings..." button links to `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`.
  - When permission is granted, the strategy becomes active automatically.
- The "Current capability" header dynamically shows what the resolver will do based on detected environment.

### Backend

Add to `AppSettings`:

```rust
struct FocusSettings {
    /// Whether to attempt the accessibility strategy (user opt-in).
    /// Even if true, requires OS-level permission to actually work.
    accessibility_enabled: bool,
}
```

Add a Tauri command to query current focus capabilities:

```rust
#[tauri::command]
fn get_focus_capabilities() -> FocusCapabilities {
    FocusCapabilities {
        tmux_detected: bool,
        terminal_app: Option<String>,
        terminal_scriptable: bool,
        accessibility_permitted: bool,
        accessibility_enabled: bool,
    }
}
```

This command runs the PID ancestry walk (or caches recent results) and checks `AXIsProcessTrusted()`.

### Settings TOML

```toml
[focus]
accessibility_enabled = false    # User opt-in for accessibility strategy
```

## Acceptance criteria

- [ ] New "Focus" section in settings sidebar
- [ ] Shows detected terminal app and best available strategy
- [ ] Lists all 4 strategies with their status (Active / Not available / Not enabled)
- [ ] Accessibility strategy: shows permission status, "Open System Settings..." button
- [ ] `FocusSettings` in `AppSettings` with `accessibility_enabled`
- [ ] `get_focus_capabilities` Tauri command
- [ ] Settings persist to `settings.toml` under `[focus]`
- [ ] `specs/main.md` updated with Focus settings section
- [ ] `just check` passes

## Notes

- The focus capabilities query involves a PID ancestry walk, which needs an active copilot session. If no copilot sessions are active, show "No active sessions — start a Copilot session to detect capabilities" with a brief explanation that the terminal and tmux are detected from session processes.
- No caching of last-known state — keep it simple. Capabilities are live-detected only.
- The settings section should follow existing settings UI patterns (same styling, spacing, component structure).
- Keep the copy concise and actionable — avoid jargon. Each strategy should have a one-line description and a clear status indicator.

## Relevant files

- `src/settings/SettingsApp.tsx` — add Focus section
- `src/settings/settings.css` — styling if needed
- `src-tauri/src/app_settings.rs` — `FocusSettings` struct
- `src-tauri/src/command.rs` — `get_focus_capabilities` command
- `src-tauri/src/terminal_focus/mod.rs` — expose capability query
- `specs/main.md` — document Focus settings
