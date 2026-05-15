import { describe, test, expect } from "bun:test";

import { SessionTracker } from "../src/plugin-bundle.js";

// Helpers to build realistic event objects.

function sessionStatusEvent(sessionID: string, type: "busy" | "idle" | "retry") {
  const status: Record<string, unknown> = { type };
  if (type === "retry") {
    status.attempt = 2;
    status.message = "rate limited";
  }
  return { type: "session.status" as const, properties: { sessionID, status } };
}

function questionAskedEvent(sessionID: string) {
  return {
    type: "question.asked" as const,
    properties: { id: "q-1", sessionID, questions: [] },
  };
}

function questionRepliedEvent(sessionID: string) {
  return {
    type: "question.replied" as const,
    properties: { sessionID, requestID: "q-1", answers: [] },
  };
}

function questionRejectedEvent(sessionID: string) {
  return {
    type: "question.rejected" as const,
    properties: { sessionID, requestID: "q-1" },
  };
}

function permissionAskedEvent(sessionID: string) {
  return {
    type: "permission.asked" as const,
    properties: { id: "p-1", sessionID, permission: "write", patterns: [], metadata: {}, always: [] },
  };
}

function permissionRepliedEvent(sessionID: string) {
  return {
    type: "permission.replied" as const,
    properties: { sessionID, requestID: "p-1", reply: "once" },
  };
}

describe("SessionTracker", () => {
  describe("parent session pinning", () => {
    test("initial state is idle with the provided session ID", () => {
      const tracker = new SessionTracker("12345");

      expect(tracker.sessionId).toBe("12345");
      expect(tracker.status).toBe("idle");
    });

    test("first session.status event establishes parent and updates state", () => {
      const tracker = new SessionTracker("12345");

      const result = tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      expect(result.changed).toBe(true);
      expect(tracker.sessionId).toBe("sess-parent");
      expect(tracker.status).toBe("working");
    });

    test("parent session idle updates status correctly", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(sessionStatusEvent("sess-parent", "idle"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("idle");
      expect(tracker.sessionId).toBe("sess-parent");
    });

    test("retry status maps to working with summary", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(sessionStatusEvent("sess-parent", "retry"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("working");
      expect(result.summary).toContain("Retry");
      expect(result.summary).toContain("rate limited");
    });
  });

  describe("sub-agent filtering", () => {
    test("session.status from a different session ID is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(sessionStatusEvent("sess-child-1", "busy"));

      expect(result.changed).toBe(false);
      expect(tracker.sessionId).toBe("sess-parent");
      expect(tracker.status).toBe("working");
    });

    test("sub-agent idle does not override parent working status", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(sessionStatusEvent("sess-child-1", "idle"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("working");
    });

    test("question.asked from sub-agent is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(questionAskedEvent("sess-child-1"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("working");
    });

    test("question.replied from sub-agent is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(questionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(questionRepliedEvent("sess-child-1"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("question");
    });

    test("question.rejected from sub-agent is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(questionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(questionRejectedEvent("sess-child-1"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("question");
    });

    test("permission.asked from sub-agent is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(permissionAskedEvent("sess-child-1"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("working");
    });

    test("permission.replied from sub-agent is ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(permissionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(permissionRepliedEvent("sess-child-1"));

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("approval");
    });
  });

  describe("parent session question and permission events", () => {
    test("question.asked from parent updates to question", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(questionAskedEvent("sess-parent"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("question");
    });

    test("question.replied from parent resumes working", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(questionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(questionRepliedEvent("sess-parent"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("working");
    });

    test("question.rejected from parent resumes working", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(questionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(questionRejectedEvent("sess-parent"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("working");
    });

    test("permission.asked from parent updates to approval", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent(permissionAskedEvent("sess-parent"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("approval");
    });

    test("permission.replied from parent resumes working", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));
      tracker.handleEvent(permissionAskedEvent("sess-parent"));

      const result = tracker.handleEvent(permissionRepliedEvent("sess-parent"));

      expect(result.changed).toBe(true);
      expect(tracker.status).toBe("working");
    });
  });

  describe("unrelated events", () => {
    test("unknown event types are ignored", () => {
      const tracker = new SessionTracker("12345");
      tracker.handleEvent(sessionStatusEvent("sess-parent", "busy"));

      const result = tracker.handleEvent({ type: "file.edited", properties: { file: "/tmp/a.ts" } });

      expect(result.changed).toBe(false);
      expect(tracker.status).toBe("working");
    });
  });
});
