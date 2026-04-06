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

// ── Plugin ──────────────────────────────────────────────────────────

const CortadoPlugin: Plugin = async ({ directory, worktree, $ }) => {
  ensureHarnessDir();

  const cwd = worktree || directory;
  const gitMeta = await resolveGitMeta($, cwd);

  let currentSessionId: string = String(process.pid);
  let currentStatus: InterchangeStatus = "idle";

  writeState();

  const cleanup = () => deleteSessionFile();
  process.on("exit", cleanup);
  process.on("SIGTERM", () => { cleanup(); process.exit(0); });
  process.on("SIGINT", () => { cleanup(); process.exit(0); });

  function writeState(summary?: string) {
    writeSessionFile(
      buildSession({
        id: currentSessionId,
        cwd,
        status: currentStatus,
        repository: gitMeta.repository,
        branch: gitMeta.branch,
        summary,
      })
    );
  }

  return {
    event: async ({ event }) => {
      // Cast to string so we can match event types that may not be in
      // the SDK type union yet (question.* and permission.* events).
      const eventType = event.type as string;

      switch (eventType) {
        case "session.status": {
          const { sessionID, status } = (event as any).properties;

          if (sessionID) {
            currentSessionId = sessionID;
          }

          let summary: string | undefined;

          switch (status.type) {
            case "busy":
              currentStatus = "working";
              break;
            case "idle":
              currentStatus = "idle";
              break;
            case "retry":
              currentStatus = "working";
              summary = `Retry #${status.attempt}: ${status.message}`;
              break;
          }

          writeState(summary);
          break;
        }

        // Agent is waiting for the user to answer a question.
        case "question.asked":
          currentStatus = "question";
          writeState();
          break;

        // User answered or dismissed -- agent resumes work.
        case "question.replied":
        case "question.rejected":
          currentStatus = "working";
          writeState();
          break;

        // Agent needs the user to approve a tool/action.
        case "permission.asked":
          currentStatus = "approval";
          writeState();
          break;

        // User approved/rejected -- agent resumes work.
        case "permission.replied":
          currentStatus = "working";
          writeState();
          break;
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
