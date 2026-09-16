import { describe, it, expect, vi, afterEach } from "vitest";
import { supportsFocus, supportsOpen, supportsRestart, formatRelativeTime } from "./utils";
import type { Activity } from "./types";

function makeActivity(fields: Activity["fields"] = []): Activity {
  return { id: "session-123", title: "Test Session", fields, retained: false };
}

describe("supportsOpen", () => {
  it.each([
    "https://dev.azure.com/acme/project/_build?definitionId=42",
    "https://dev.azure.com/acme/project/_build/results?buildId=1",
    "https://dev.azure.com/acme/project/_build/results?buildId=2",
  ])("opens the explicit action with a hidden Link field: %s", (url) => {
    const activity = makeActivity();
    activity.id = "ado-pipeline:stable";
    activity.action = { open_url: url };
    expect(supportsOpen(activity)).toBe(url);
    expect(supportsRestart(activity)).toBe(false);
  });

  it("prefers the explicit action over a URL ID", () => {
    const activity = makeActivity();
    activity.id = "https://example.com/overview";
    activity.action = { open_url: "https://example.com/latest-run" };
    expect(supportsOpen(activity)).toBe("https://example.com/latest-run");
  });

  it.each(["javascript:alert(1)", "file:///tmp/run", ""])("rejects a non-web action URL: %s", (url) => {
    const activity = makeActivity();
    activity.action = { open_url: url };
    expect(supportsOpen(activity)).toBeNull();
  });

  it("keeps restart actions separate from URL actions", () => {
    const activity = makeActivity();
    activity.action = "restart_app";
    expect(supportsOpen(activity)).toBeNull();
    expect(supportsRestart(activity)).toBe(true);
  });

  it("uses a URL field for stable non-URL activity IDs", () => {
    const activity = makeActivity([
      {
        name: "link",
        label: "Link",
        value: { type: "url", value: "https://dev.azure.com/acme/project/_build/results?buildId=42" },
      },
    ]);
    activity.id = "ado-pipeline:stable";
    expect(supportsOpen(activity)).toBe(
      "https://dev.azure.com/acme/project/_build/results?buildId=42",
    );
  });

  it("prefers a URL activity ID", () => {
    const activity = makeActivity([
      { name: "link", label: "Link", value: { type: "url", value: "https://example.com/field" } },
    ]);
    activity.id = "https://example.com/id";
    expect(supportsOpen(activity)).toBe("https://example.com/id");
  });
});

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

describe("formatRelativeTime", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns 'just now' for timestamps less than 60 seconds ago", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now - 30_000)).toBe("just now");
  });

  it("returns 'just now' for future timestamps", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now + 10_000)).toBe("just now");
  });

  it("returns minutes for 1-59 minutes ago", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now - 60_000)).toBe("1m ago");
    expect(formatRelativeTime(now - 5 * 60_000)).toBe("5m ago");
    expect(formatRelativeTime(now - 59 * 60_000)).toBe("59m ago");
  });

  it("returns hours for 1-23 hours ago", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now - 60 * 60_000)).toBe("1h ago");
    expect(formatRelativeTime(now - 3 * 60 * 60_000)).toBe("3h ago");
    expect(formatRelativeTime(now - 23 * 60 * 60_000)).toBe("23h ago");
  });

  it("returns days for 24+ hours ago", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now - 24 * 60 * 60_000)).toBe("1d ago");
    expect(formatRelativeTime(now - 7 * 24 * 60 * 60_000)).toBe("7d ago");
  });

  it("returns 'just now' for exactly 0ms difference", () => {
    const now = Date.now();
    vi.spyOn(Date, "now").mockReturnValue(now);
    expect(formatRelativeTime(now)).toBe("just now");
  });
});
