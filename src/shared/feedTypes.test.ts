import { describe, it, expect } from "vitest";
import { findFeedType, generateDefaultName } from "./feedTypes";

describe("generateDefaultName", () => {
  it("returns null for an unknown feed type", () => {
    expect(generateDefaultName("nonexistent", {})).toBeNull();
  });

  it("returns null when feed type has no defaultNamePattern", () => {
    // cortado-update is a built-in type with no catalog entry
    expect(generateDefaultName("cortado-update", {})).toBeNull();
  });

  it("substitutes a single repo for github-pr", () => {
    expect(generateDefaultName("github-pr", { repos: ["octocat/hello"] })).toBe(
      "octocat/hello PRs",
    );
  });

  it("substitutes a single repo for github-actions", () => {
    expect(
      generateDefaultName("github-actions", { repos: ["org/repo"] }),
    ).toBe("org/repo Actions");
  });

  it("returns null when required placeholder is unfilled", () => {
    expect(generateDefaultName("github-pr", {})).toBeNull();
    expect(generateDefaultName("github-pr", { repo: "" })).toBeNull();
  });

  it("uses the selected account for copilot-usage", () => {
    expect(generateDefaultName("copilot-usage", { account: "octocat" })).toBe(
      "octocat Copilot usage",
    );
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

describe("copilot-usage catalog", () => {
  const catalog = findFeedType("copilot-usage");

  it("is experimental and uses the GitHub dependency", () => {
    expect(catalog?.badge).toBe("experimental");
    expect(catalog?.dependency?.binary).toBe("gh");
  });

  it("provides the threshold default and browser-account note", () => {
    const threshold = catalog?.fields.find((field) => field.key === "attention_at_percent");
    expect(threshold?.defaultValue).toBe(80);
    expect(catalog?.notes?.some((note) => note.includes("browser's active GitHub session"))).toBe(true);
  });

  it("validates reference amount, threshold, and HTTPS override", () => {
    const validation = (field: string) =>
      catalog?.validations?.find((rule) => rule.field === field)?.check;

    expect(validation("reference_amount_usd")?.("0")).toBe("Must be a number greater than zero");
    expect(validation("reference_amount_usd")?.("2000")).toBeNull();
    expect(validation("attention_at_percent")?.("101")).toBe("Must be between 1 and 100");
    expect(validation("attention_at_percent")?.("80")).toBeNull();
    expect(validation("details_url")?.("http://example.com")).toBe(
      "Must be an https:// URL without embedded credentials",
    );
    expect(validation("details_url")?.("https://user:secret@example.com")).toBe(
      "Must be an https:// URL without embedded credentials",
    );
    expect(validation("details_url")?.("https://example.com/usage")).toBeNull();
  });
});
