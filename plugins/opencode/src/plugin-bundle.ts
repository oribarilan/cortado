// cortado-plugin-version: 2
// cortado-opencode -- single-file plugin for ~/.config/opencode/plugins/
// This file is auto-embedded in the Cortado binary and written to disk
// when the user clicks "Install Plugin" in Settings.

import type { Plugin } from "@opencode-ai/plugin";
import { mkdirSync, writeFileSync, renameSync, unlinkSync } from "node:fs";
import { join, dirname } from "node:path";
import { homedir } from "node:os";
import { randomBytes } from "node:crypto";

// ── Interchange ─────────────────────────────────────────────────────

const HARNESS_DIR = join(homedir(), ".config", "cortado", "harness");
const INTERCHANGE_VERSION = 1;

export type InterchangeStatus = "working" | "idle" | "question" | "approval";

export interface InterchangeSession {
  version: number;
  harness: string;
  id: string;
  pid: number;
  cwd: string;
  status: InterchangeStatus;
  last_active_at: string;
  repository?: string;
  branch?: string;
  summary?: string;
}

function sessionFilePath(): string {
  return join(HARNESS_DIR, `${process.pid}.json`);
}

function ensureHarnessDir(): void {
  mkdirSync(HARNESS_DIR, { recursive: true });
}

function writeSessionFile(session: InterchangeSession): void {
  const filePath = sessionFilePath();
  const tmpPath = join(dirname(filePath), `.${process.pid}.json.${randomBytes(4).toString("hex")}`);
  const content = JSON.stringify(session, null, 2) + "\n";
  writeFileSync(tmpPath, content, "utf-8");
  renameSync(tmpPath, filePath);
}

function deleteSessionFile(): void {
  try {
    unlinkSync(sessionFilePath());
  } catch {
    // Ignore -- file may not exist or already deleted.
  }
}

export function buildSession(opts: {
  id: string;
  cwd: string;
  status: InterchangeStatus;
  repository?: string;
  branch?: string;
  summary?: string;
}): InterchangeSession {
  return {
    version: INTERCHANGE_VERSION,
    harness: "opencode",
    pid: process.pid,
    last_active_at: new Date().toISOString(),
    ...opts,
  };
}

// ── Session tracking ────────────────────────────────────────────────

/** Tracks session state for a single OpenCode process, filtering out
 * sub-agent session events so they don't corrupt the parent session's
 * status or ID in the interchange file. */
export class SessionTracker {
  private _sessionId: string;
  private _status: InterchangeStatus = "idle";
  private _parentSessionId: string | null = null;

  constructor(initialSessionId: string) {
    this._sessionId = initialSessionId;
  }

  get sessionId(): string {
    return this._sessionId;
  }

  get status(): InterchangeStatus {
    return this._status;
  }

  /** Returns whether state changed and an optional summary string. */
  handleEvent(event: { type: string; properties?: any }): { changed: boolean; summary?: string } {
    const eventType = event.type as string;
    const props = event.properties;

    switch (eventType) {
      case "session.status": {
        const { sessionID, status } = props;
        if (!this._isParentSession(sessionID)) return { changed: false };

        let summary: string | undefined;
        const prevStatus = this._status;

        switch (status.type) {
          case "busy":
            this._status = "working";
            break;
          case "idle":
            this._status = "idle";
            break;
          case "retry":
            this._status = "working";
            summary = `Retry #${status.attempt}: ${status.message}`;
            break;
        }

        return { changed: this._status !== prevStatus || summary !== undefined, summary };
      }

      case "question.asked": {
        if (!this._isParentSession(props?.sessionID)) return { changed: false };
        this._status = "question";
        return { changed: true };
      }

      case "question.replied":
      case "question.rejected": {
        if (!this._isParentSession(props?.sessionID)) return { changed: false };
        this._status = "working";
        return { changed: true };
      }

      case "permission.asked": {
        if (!this._isParentSession(props?.sessionID)) return { changed: false };
        this._status = "approval";
        return { changed: true };
      }

      case "permission.replied": {
        if (!this._isParentSession(props?.sessionID)) return { changed: false };
        this._status = "working";
        return { changed: true };
      }

      default:
        return { changed: false };
    }
  }

  /** First session.status event pins the parent session ID.
   * Subsequent events from different session IDs are sub-agents. */
  private _isParentSession(sessionID: string | undefined): boolean {
    if (!sessionID) return true; // no sessionID means unscoped event, allow it

    if (this._parentSessionId === null) {
      // First event with a sessionID -- this is the parent session.
      this._parentSessionId = sessionID;
      this._sessionId = sessionID;
      return true;
    }

    return sessionID === this._parentSessionId;
  }
}

// ── Plugin ──────────────────────────────────────────────────────────

const CortadoPlugin: Plugin = async ({ directory, worktree, $ }) => {
  ensureHarnessDir();

  const cwd = worktree || directory;
  const gitMeta = await resolveGitMeta($, cwd);
  const tracker = new SessionTracker(String(process.pid));

  writeState();

  const cleanup = () => deleteSessionFile();
  process.on("exit", cleanup);
  process.on("SIGTERM", () => { cleanup(); process.exit(0); });
  process.on("SIGINT", () => { cleanup(); process.exit(0); });

  function writeState(summary?: string) {
    writeSessionFile(
      buildSession({
        id: tracker.sessionId,
        cwd,
        status: tracker.status,
        repository: gitMeta.repository,
        branch: gitMeta.branch,
        summary,
      })
    );
  }

  return {
    event: async ({ event }) => {
      const result = tracker.handleEvent(event as any);
      if (result.changed) {
        writeState(result.summary);
      }
    },
  };
};

async function resolveGitMeta(
  $: any,
  cwd: string
): Promise<{ repository?: string; branch?: string }> {
  let repository: string | undefined;
  let branch: string | undefined;

  try {
    const remoteUrl = (await $`git -C ${cwd} remote get-url origin`.quiet().text()).trim();
    repository = parseRepoFromUrl(remoteUrl);
  } catch {
    // No git remote -- that's fine.
  }

  try {
    branch = (await $`git -C ${cwd} rev-parse --abbrev-ref HEAD`.quiet().text()).trim();
  } catch {
    // Not a git repo or detached HEAD.
  }

  return { repository, branch };
}

function parseRepoFromUrl(url: string): string | undefined {
  const sshMatch = url.match(/git@[^:]+:(.+?)(?:\.git)?$/);
  if (sshMatch) return sshMatch[1];

  try {
    const parsed = new URL(url);
    const path = parsed.pathname.replace(/^\//, "").replace(/\.git$/, "");
    if (path) return path;
  } catch {
    // Not a valid URL.
  }

  return undefined;
}

export default CortadoPlugin;
