import { describe, it, expect } from "vitest";
import { findFeedType, generateDefaultName } from "./feedTypes";
import { normalizeTypeSpecific, parseIntegerList, updateTypeSpecific, validateFieldRelationships } from "./feedFieldConfig";

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

describe("ado-pr hosted URL validation", () => {
  const catalog = findFeedType("ado-pr");
  const validate = (value: string) => {
    const rule = catalog?.validations?.find((candidate) => candidate.field === "url");
    if (!rule || rule.raw) throw new Error("ADO PR URL must use string validation");
    return rule.check(value);
  };

  it("rejects credential-routing hazards and accepts hosted repository URLs", () => {
    expect(validate("https://dev.azure.com/acme/Platform/_git/API")).toBeNull();
    expect(validate("https://acme.visualstudio.com/Platform/_git/API")).toBeNull();
    expect(validate("https://dev.azure.com.attacker.example/acme/Platform/_git/API"))
      .not.toBeNull();
    expect(validate("https://user:secret@dev.azure.com/acme/Platform/_git/API"))
      .not.toBeNull();
    expect(validate("https://dev.azure.com/acme/Platform/_git/API?x=1"))
      .not.toBeNull();
  });
});

describe("ado-pipelines catalog", () => {
  const catalog = findFeedType("ado-pipelines");

  it("uses shared ADO dependencies and catalog-driven selector controls", () => {
    expect(catalog?.dependency?.binary).toBe("az");
    const ids = catalog?.fields.find((field) => field.key === "pipeline_ids");
    const folder = catalog?.fields.find((field) => field.key === "folder");
    expect(ids).toMatchObject({
      kind: "integer-list",
      oneOfGroup: "pipeline-selector",
      exclusiveWith: "folder",
    });
    expect(folder).toMatchObject({
      oneOfGroup: "pipeline-selector",
      exclusiveWith: "pipeline_ids",
    });
  });

  it("generates a project-scoped default name", () => {
    expect(generateDefaultName("ado-pipelines", { project: "Platform" })).toBe(
      "Platform pipelines",
    );
  });

  it("parses and normalizes integer ID lists", () => {
    expect(parseIntegerList("42, 73 108")).toEqual({
      values: [42, 73, 108],
      error: null,
    });
    expect(normalizeTypeSpecific(catalog?.fields ?? [], {
      pipeline_ids: "42, 73",
    })).toEqual({ pipeline_ids: [42, 73] });
  });

  it("rejects invalid, coerced, duplicate, and oversized ID lists", () => {
    expect(parseIntegerList("42.5").error).toContain("positive numeric IDs");
    expect(parseIntegerList("0").error).toContain("positive 32-bit integers");
    expect(parseIntegerList([true]).error).toContain("only numbers");
    expect(parseIntegerList([[1]]).error).toContain("only numbers");
    expect(parseIntegerList(["1"]).error).toContain("only numbers");
    expect(parseIntegerList("42, 42").error).toContain("duplicate");
    expect(parseIntegerList(Array.from({ length: 21 }, (_, index) => index + 1)).error)
      .toBe("Choose at most 20 pipeline IDs");
  });

  it("keeps folder-only selection through update, normalization, and validation", () => {
    const fields = catalog?.fields ?? [];
    const updated = updateTypeSpecific(
      fields,
      { pipeline_ids: [] },
      "folder",
      "\\Team",
    );
    const normalized = normalizeTypeSpecific(fields, updated);
    expect(normalized).toEqual({ folder: "\\Team" });
    expect(validateFieldRelationships(fields, normalized)).toEqual({});
    expect("pipeline_ids" in normalized).toBe(false);

    const idUpdated = updateTypeSpecific(fields, normalized, "pipeline_ids", "42, 73");
    const idNormalized = normalizeTypeSpecific(fields, idUpdated);
    expect(idNormalized).toEqual({ pipeline_ids: [42, 73] });
    expect(validateFieldRelationships(fields, idNormalized)).toEqual({});
    const idRule = catalog?.validations?.find((rule) => rule.field === "pipeline_ids");
    if (!idRule?.raw) throw new Error("pipeline ID validation must receive raw values");
    expect(idRule.check(idNormalized.pipeline_ids)).toBeNull();
  });

  it("preserves malformed raw arrays so validation rejects them", () => {
    const fields = catalog?.fields ?? [];
    for (const malformed of [[true], [[1]], ["1"]]) {
      const normalized = normalizeTypeSpecific(fields, { pipeline_ids: malformed });
      expect(normalized.pipeline_ids).toEqual(malformed);
      const rule = catalog?.validations?.find((candidate) => candidate.field === "pipeline_ids");
      expect(rule?.raw).toBe(true);
      if (!rule?.raw) throw new Error("pipeline ID validation must receive raw values");
      expect(rule.check(normalized.pipeline_ids)).not.toBeNull();
    }
  });

  it("clears the other selector when one receives a value", () => {
    const withIds = updateTypeSpecific(
      catalog?.fields ?? [],
      { folder: "\\Team" },
      "pipeline_ids",
      "42, 73",
    );
    expect(withIds).toEqual({ pipeline_ids: "42, 73" });
    const withFolder = updateTypeSpecific(
      catalog?.fields ?? [],
      { pipeline_ids: [42, 73] },
      "folder",
      "\\Team",
    );
    expect(withFolder).toEqual({ folder: "\\Team" });
  });

  it("requires exactly one catalog-declared selector", () => {
    expect(validateFieldRelationships(catalog?.fields ?? [], {})).toHaveProperty(
      "pipeline_ids",
    );
    const both = validateFieldRelationships(catalog?.fields ?? [], {
      pipeline_ids: [42],
      folder: "\\Team",
    });
    expect(both.pipeline_ids).toContain("either");
    expect(both.folder).toContain("either");
    expect(validateFieldRelationships(catalog?.fields ?? [], { pipeline_ids: [42] }))
      .toEqual({});
    expect(validateFieldRelationships(catalog?.fields ?? [], { folder: "\\Team" }))
      .toEqual({});
  });

  it("validates hosted organization and exact folder fields", () => {
    const validation = (field: string, value: string) => {
      const rule = catalog?.validations?.find((candidate) => candidate.field === field);
      if (!rule || rule.raw) throw new Error(`${field} must use string validation`);
      return rule.check(value);
    };
    expect(validation("organization", "https://dev.azure.com.attacker.example/acme"))
      .not.toBeNull();
    expect(validation("organization", "https://dev.azure.com/acme")).toBeNull();
    expect(validation("folder", "Team/CI")).toContain("exact folder");
    expect(validation("folder", "\\Team\\CI")).toBeNull();
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
    const validation = (field: string, value: string) => {
      const rule = catalog?.validations?.find((candidate) => candidate.field === field);
      if (!rule || rule.raw) throw new Error(`${field} must use string validation`);
      return rule.check(value);
    };

    expect(validation("reference_amount_usd", "0")).toBe("Must be a number greater than zero");
    expect(validation("reference_amount_usd", "2000")).toBeNull();
    expect(validation("attention_at_percent", "101")).toBe("Must be between 1 and 100");
    expect(validation("attention_at_percent", "80")).toBeNull();
    expect(validation("details_url", "http://example.com")).toBe(
      "Must be an https:// URL without embedded credentials",
    );
    expect(validation("details_url", "https://user:secret@example.com")).toBe(
      "Must be an https:// URL without embedded credentials",
    );
    expect(validation("details_url", "https://example.com/usage")).toBeNull();
  });
});
