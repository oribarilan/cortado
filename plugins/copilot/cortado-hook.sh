#!/usr/bin/env bash
# cortado-plugin-version: 1
# cortado-copilot -- Copilot CLI plugin for session tracking.
# This file is auto-embedded in the Cortado binary and installed via
# "copilot plugin install" when the user clicks "Install Plugin" in Settings.
#
# Hooks fire as shell commands on session events. Each hook receives
# a JSON payload on stdin with sessionId, cwd, timestamp, and
# hook-specific fields (toolName, reason, etc.).
#
# Hook ordering quirks:
#
#   1. Prompt mode (-p): userPromptSubmitted fires BEFORE sessionStart
#      because copilot delivers the prompt before initializing the session.
#      sessionStart skips writing if a file already exists to avoid
#      overwriting the "working" status set by userPromptSubmitted.
#
#   2. Concurrent tool calls: copilot batches tool requests. When ask_user
#      is requested alongside report_intent, the hook system fires
#      preToolUse for each tool sequentially, then postToolUse for each.
#      Without protection, postToolUse(report_intent) would overwrite the
#      "question" status set by preToolUse(ask_user). To prevent this,
#      postToolUse checks the current file status and refuses to overwrite
#      "question" -- the status clears naturally when the user responds and
#      the next userPromptSubmitted fires.
#
#   3. Session end: writes "idle" instead of deleting the file, matching
#      OpenCode behavior. GenericProvider's PID liveness check cleans up
#      the file once the copilot process exits.

set -euo pipefail

HOOK_TYPE="${1:-}"
HARNESS_DIR="${HOME}/.config/cortado/harness"
COPILOT_PID="$PPID"
SESSION_FILE="${HARNESS_DIR}/${COPILOT_PID}.json"

# -- Read stdin ---------------------------------------------------------------

INPUT="$(cat)"

json_field() {
  # Extract a flat string field from JSON. Returns empty string if missing.
  printf '%s' "$INPUT" | grep -o "\"$1\":\"[^\"]*\"" | head -1 | sed "s/\"$1\":\"//;s/\"$//"
}

SESSION_CWD="$(json_field cwd)"
SESSION_ID="$(json_field sessionId)"

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

  local tmp_file="${HARNESS_DIR}/.${COPILOT_PID}.json.$RANDOM"

  # Build JSON without jq -- all fields are simple strings/numbers.
  {
    printf '{\n'
    printf '  "version": 1,\n'
    printf '  "harness": "copilot",\n'
    printf '  "id": "%s",\n' "$SESSION_ID"
    printf '  "pid": %s,\n' "$COPILOT_PID"
    printf '  "cwd": "%s",\n' "$SESSION_CWD"
    printf '  "status": "%s",\n' "$status"
    printf '  "last_active_at": "%s"' "$timestamp"
    [ -n "$repo" ] && printf ',\n  "repository": "%s"' "$repo"
    [ -n "$branch" ] && printf ',\n  "branch": "%s"' "$branch"
    printf '\n}\n'
  } > "$tmp_file"

  mv -f "$tmp_file" "$SESSION_FILE"
}

delete_session() {
  rm -f "$SESSION_FILE"
}

current_status() {
  # Read the current status from the session file, if it exists.
  [ -f "$SESSION_FILE" ] || return
  grep -o '"status":"[^"]*"' "$SESSION_FILE" | head -1 | sed 's/"status":"//;s/"$//'
}

# -- Dispatch -----------------------------------------------------------------

case "$HOOK_TYPE" in
  sessionStart)
    # In prompt mode (-p), userPromptSubmitted fires before sessionStart.
    # Don't overwrite an existing file -- it already has the right status.
    if [ ! -f "$SESSION_FILE" ]; then
      write_session "working"
    fi
    ;;
  userPromptSubmitted)
    write_session "working"
    ;;
  preToolUse)
    tool_name="$(json_field toolName)"
    if [ "$tool_name" = "ask_user" ]; then
      write_session "question"
    else
      write_session "working"
    fi
    ;;
  postToolUse)
    # Don't overwrite "question" status -- postToolUse for other tools
    # (e.g. report_intent) can fire while ask_user is still waiting.
    tool_name="$(json_field toolName)"
    if [ "$tool_name" != "ask_user" ] && [ "$(current_status)" != "question" ]; then
      write_session "working"
    fi
    ;;
  sessionEnd)
    # Write "idle" instead of deleting -- matches OpenCode behavior.
    # GenericProvider cleans up the file when the copilot PID dies.
    write_session "idle"
    ;;
  *)
    # Unknown hook type -- ignore silently.
    ;;
esac
