import { describe, test, expect } from "bun:test";

import { buildSession, type InterchangeSession } from "../src/plugin-bundle.js";

describe("buildSession", () => {
  test("creates a valid session with required fields", () => {
    const session = buildSession({
      id: "test-123",
      cwd: "/tmp/project",
      status: "working",
    });

    expect(session.version).toBe(1);
    expect(session.harness).toBe("opencode");
    expect(session.pid).toBe(process.pid);
    expect(session.id).toBe("test-123");
    expect(session.cwd).toBe("/tmp/project");
    expect(session.status).toBe("working");
    expect(session.last_active_at).toBeTruthy();
    // Verify ISO 8601 format
    expect(() => new Date(session.last_active_at)).not.toThrow();
  });

  test("includes optional fields when provided", () => {
    const session = buildSession({
      id: "test-456",
      cwd: "/tmp/project",
      status: "idle",
      repository: "user/repo",
      branch: "main",
      summary: "Working on feature",
    });

    expect(session.repository).toBe("user/repo");
    expect(session.branch).toBe("main");
    expect(session.summary).toBe("Working on feature");
  });

  test("omits optional fields when not provided", () => {
    const session = buildSession({
      id: "test-789",
      cwd: "/tmp/project",
      status: "idle",
    });

    expect(session.repository).toBeUndefined();
    expect(session.branch).toBeUndefined();
    expect(session.summary).toBeUndefined();
  });

  test("all valid statuses accepted", () => {
    for (const status of ["working", "idle", "question", "approval"] as const) {
      const session = buildSession({ id: "s", cwd: "/tmp", status });
      expect(session.status).toBe(status);
    }
  });
});

describe("interchange JSON format", () => {
  test("session serializes to valid JSON matching spec", () => {
    const session = buildSession({
      id: "sess_abc",
      cwd: "/Users/dev/project",
      status: "working",
      repository: "dev/project",
      branch: "feature",
      summary: "Implementing auth",
    });

    const json = JSON.stringify(session);
    const parsed = JSON.parse(json);

    expect(parsed.version).toBe(1);
    expect(parsed.harness).toBe("opencode");
    expect(typeof parsed.pid).toBe("number");
    expect(typeof parsed.last_active_at).toBe("string");
  });
});
