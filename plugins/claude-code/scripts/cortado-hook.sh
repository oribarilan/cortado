#!/usr/bin/env bash
# cortado-plugin-version: 1
# cortado-claude-code -- Claude Code plugin for session tracking.
# This file is auto-embedded in the Cortado binary and installed via
# a local marketplace when the user clicks "Install Plugin" in Settings.
#
# Hooks fire as shell commands on lifecycle events. Each hook receives
# a JSON payload on stdin with session_id, cwd, tool_name, etc.
# (Claude Code uses snake_case fields, unlike Copilot's camelCase.)
#
# Key differences from the Copilot plugin:
#
#   1. Stop is per-turn (fires every time Claude finishes responding).
#      We use SessionEnd for session termination instead.
#
#   2. PermissionRequest is a dedicated event (no Copilot equivalent).
#      It fires when a permission dialog appears -> maps to "approval".
#
#   3. AskUserQuestion is the tool that asks the user questions
#      (Copilot's equivalent is ask_user).
#
#   4. Concurrent tool calls: same issue as Copilot. PostToolUse for
#      other tools can fire while AskUserQuestion or PermissionRequest
#      is still waiting. PostToolUse checks the current file status and
#      refuses to overwrite "question" or "approval".
#
#   5. Session end: writes "idle" instead of deleting the file, matching
#      Copilot/OpenCode behavior. GenericProvider's PID liveness check
#      cleans up the file once the Claude Code process exits.

set -euo pipefail

HOOK_TYPE="${1:-}"
HARNESS_DIR="${HOME}/.config/cortado/harness"
CLAUDE_PID="$PPID"
SESSION_FILE="${HARNESS_DIR}/${CLAUDE_PID}.json"

# -- Read stdin ---------------------------------------------------------------

INPUT="$(cat)"

json_field() {
  # Extract a flat string field from JSON. Returns empty string if missing.
  # Claude Code uses snake_case fields (session_id, tool_name, cwd).
  printf '%s' "$INPUT" | grep -o "\"$1\":\"[^\"]*\"" | head -1 | sed "s/\"$1\":\"//;s/\"$//"
}

SESSION_CWD="$(json_field cwd)"
SESSION_ID="$(json_field session_id)"

# -- Git metadata -------------------------------------------------------------

git_remote_url() {
  git -C "$1" remote get-url origin 2>/dev/null || true
}

git_branch() {
  git -C "$1" rev-parse --abbrev-ref HEAD 2>/dev/null || true
}

parse_repo_from_url() {
  local url="$1"
  [ -z "$url" ] && return

  # SSH: git@host:owner/repo.git
  if [[ "$url" =~ git@[^:]+:(.+) ]]; then
    printf '%s' "${BASH_REMATCH[1]}" | sed 's/\.git$//'
    return
  fi

  # HTTPS: https://host/owner/repo.git
  if [[ "$url" =~ https?://[^/]+/(.+) ]]; then
    printf '%s' "${BASH_REMATCH[1]}" | sed 's/\.git$//'
    return
  fi
}

# -- Interchange file ---------------------------------------------------------

write_session() {
  local status="$1"

  mkdir -p "$HARNESS_DIR"

  local remote_url repo branch
  remote_url="$(git_remote_url "$SESSION_CWD")"
  repo="$(parse_repo_from_url "$remote_url")"
  branch="$(git_branch "$SESSION_CWD")"

  local timestamp
  timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  local tmp_file="${HARNESS_DIR}/.${CLAUDE_PID}.json.$RANDOM"

  # Build JSON without jq -- all fields are simple strings/numbers.
  {
    printf '{\n'
    printf '  "version": 1,\n'
    printf '  "harness": "claude-code",\n'
    printf '  "id": "%s",\n' "$SESSION_ID"
    printf '  "pid": %s,\n' "$CLAUDE_PID"
    printf '  "cwd": "%s",\n' "$SESSION_CWD"
    printf '  "status": "%s",\n' "$status"
    printf '  "last_active_at": "%s"' "$timestamp"
    [ -n "$repo" ] && printf ',\n  "repository": "%s"' "$repo"
    [ -n "$branch" ] && printf ',\n  "branch": "%s"' "$branch"
    printf '\n}\n'
  } > "$tmp_file"

  mv -f "$tmp_file" "$SESSION_FILE"
}

current_status() {
  # Read the current status from the session file, if it exists.
  [ -f "$SESSION_FILE" ] || return
  grep -o '"status":"[^"]*"' "$SESSION_FILE" | head -1 | sed 's/"status":"//;s/"$//'
}

# -- Dispatch -----------------------------------------------------------------

case "$HOOK_TYPE" in
  SessionStart)
    # Guard: don't overwrite an existing file. Mirrors the Copilot
    # prompt-mode guard where UserPromptSubmit may fire first.
    if [ ! -f "$SESSION_FILE" ]; then
      write_session "working"
    fi
    ;;
  UserPromptSubmit)
    write_session "working"
    ;;
  PreToolUse)
    tool_name="$(json_field tool_name)"
    if [ "$tool_name" = "AskUserQuestion" ]; then
      write_session "question"
    else
      write_session "working"
    fi
    ;;
  PermissionRequest)
    write_session "approval"
    ;;
  PostToolUse)
    # Don't overwrite "question" or "approval" status -- PostToolUse for
    # other tools can fire while the user is still responding.
    cur="$(current_status)"
    if [ "$cur" != "question" ] && [ "$cur" != "approval" ]; then
      write_session "working"
    fi
    ;;
  SessionEnd)
    # Write "idle" instead of deleting -- matches Copilot/OpenCode behavior.
    # GenericProvider cleans up the file when the Claude Code PID dies.
    write_session "idle"
    ;;
  *)
    # Unknown hook type -- ignore silently.
    ;;
esac
