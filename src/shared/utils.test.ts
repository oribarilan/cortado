import { describe, it, expect } from "vitest";
import { supportsFocus } from "./utils";
import type { Activity } from "./types";

function makeActivity(fields: Activity["fields"] = []): Activity {
  return { id: "session-123", title: "Test Session", fields, retained: false };
}

describe("supportsFocus", () => {
  it("returns focus info when activity has a focus_app text field", () => {
    const activity = makeActivity([
      { name: "focus_app", label: "App", value: { type: "text", value: "Ghostty" } },
    ]);
    expect(supportsFocus(activity)).toEqual({
      sessionId: "session-123",
      label: "Open in Ghostty",
    });
  });

  it("returns null when activity has no focus_app field", () => {
    const activity = makeActivity([
      { name: "status", label: "Status", value: { type: "text", value: "running" } },
    ]);
    expect(supportsFocus(activity)).toBeNull();
  });

  it("defaults label to 'terminal' when focus_app is not a text field", () => {
    const activity = makeActivity([
      { name: "focus_app", label: "App", value: { type: "number", value: 42 } },
    ]);
    expect(supportsFocus(activity)).toEqual({
      sessionId: "session-123",
      label: "Open in terminal",
    });
  });

  it("works regardless of feed type, any activity with focus_app gets focus", () => {
    const activity = makeActivity([
      { name: "focus_app", label: "App", value: { type: "text", value: "iTerm2" } },
    ]);
    // This test exists to document that feed type is irrelevant;
    // supportsFocus is purely field-based.
    const result = supportsFocus(activity);
    expect(result).not.toBeNull();
    expect(result!.label).toBe("Open in iTerm2");
  });
});
