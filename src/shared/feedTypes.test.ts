import { describe, it, expect } from "vitest";
import { generateDefaultName } from "./feedTypes";

describe("generateDefaultName", () => {
  it("returns null for an unknown feed type", () => {
    expect(generateDefaultName("nonexistent", {})).toBeNull();
  });

  it("returns null when feed type has no defaultNamePattern", () => {
    // cortado-update is a built-in type with no catalog entry
    expect(generateDefaultName("cortado-update", {})).toBeNull();
  });

  it("substitutes {repo} for github-pr", () => {
    expect(generateDefaultName("github-pr", { repo: "octocat/hello" })).toBe(
      "octocat/hello PRs",
    );
  });

  it("substitutes {repo} for github-actions", () => {
    expect(
      generateDefaultName("github-actions", { repo: "org/repo" }),
    ).toBe("org/repo Actions");
  });

  it("returns null when required placeholder is unfilled", () => {
    expect(generateDefaultName("github-pr", {})).toBeNull();
    expect(generateDefaultName("github-pr", { repo: "" })).toBeNull();
  });

  it("returns static pattern for copilot-session (no placeholders)", () => {
    expect(generateDefaultName("copilot-session", {})).toBe("Copilot");
  });

  it("returns static pattern for opencode-session (no placeholders)", () => {
    expect(generateDefaultName("opencode-session", {})).toBe("OpenCode");
  });

  it("extracts hostname from URL for http-health", () => {
    expect(
      generateDefaultName("http-health", {
        url: "https://api.example.com/health",
      }),
    ).toBe("api.example.com");
  });

  it("extracts project/repo from ADO URL for ado-pr", () => {
    expect(
      generateDefaultName("ado-pr", {
        url: "https://dev.azure.com/org/myproject/_git/myrepo",
      }),
    ).toBe("myproject/myrepo PRs");
  });

  it("ignores non-string values in typeSpecific", () => {
    expect(
      generateDefaultName("github-pr", { repo: 42 as unknown }),
    ).toBeNull();
  });

  it("falls back to raw value for invalid URLs", () => {
    expect(
      generateDefaultName("http-health", { url: "not-a-url" }),
    ).toBe("not-a-url");
  });
});
