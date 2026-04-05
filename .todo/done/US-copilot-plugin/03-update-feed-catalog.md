---
status: done
---

# Update feed catalog for copilot-session

## Goal

Add `dependency` and `setup` fields to the `copilot-session` feed type in `feedTypes.ts`, so the settings UI prompts users to install the Cortado extension when configuring a Copilot Sessions feed. Also update the notes to reflect the new discovery mechanism.

## Acceptance criteria

- [ ] `copilot-session` in `feedTypes.ts` gains a `dependency` field:
  ```typescript
  dependency: {
    binary: "copilot",
    name: "GitHub Copilot CLI",
    installUrl: "https://docs.github.com/en/copilot/how-tos/copilot-cli",
  },
  ```
- [ ] `copilot-session` gains a `setup` field:
  ```typescript
  setup: {
    label: "Copilot CLI extension",
    description: "The Cortado extension must be installed in Copilot CLI to publish session state to Cortado.",
    checkCommand: "check_copilot_extension",
    installCommand: "install_copilot_extension",
    installLabel: "Install Extension",
  },
  ```
- [ ] Notes updated to reflect new mechanism:
  - "Sessions are detected via file changes in ~/.config/cortado/harness/ with near-instant updates." (matches OpenCode wording)
  - "Shows one activity per working directory with repo, branch, and status."
  - "Opening an activity focuses the terminal -- exact tmux pane when available"
- [ ] The old note about `~/.copilot/session-state/` is removed
- [ ] Settings UI shows the setup banner with "Install Extension" button when extension is not installed
- [ ] Settings UI shows update prompt when extension version is outdated
- [ ] Feed can be saved after extension is installed (setup is a prerequisite for data, not for saving -- same as OpenCode)

## Notes

### Binary name for dependency check

The Copilot CLI binary is `copilot`. The dependency check runs `which copilot` (or equivalent) to verify it's installed. Verify the exact binary name before implementing.

### No auth command

Unlike OpenCode (`authCommand: "opencode auth"`), Copilot CLI authenticates via `gh auth` (GitHub CLI) or device flow. No `authCommand` field needed -- just the binary check.
