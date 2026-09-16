type ConfigField = {
  key: string;
  label: string;
  kind?: string;
  oneOfGroup?: string;
  exclusiveWith?: string;
};

export type IntegerListParseResult = {
  values: number[] | null;
  error: string | null;
};

/** Parses a comma- or whitespace-separated list of positive 32-bit integers. */
export function parseIntegerList(value: unknown, maxItems = 20): IntegerListParseResult {
  let rawValues: unknown[];
  if (Array.isArray(value)) {
    if (value.some((entry) => typeof entry !== "number")) {
      return { values: null, error: "Pipeline ID arrays must contain only numbers" };
    }
    rawValues = value;
  } else if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) return { values: [], error: null };
    if (!/^[0-9,\s]+$/.test(trimmed)) {
      return { values: null, error: "Use positive numeric IDs separated by commas" };
    }
    rawValues = trimmed.split(/[\s,]+/).filter(Boolean);
  } else if (value == null) {
    return { values: [], error: null };
  } else {
    return { values: null, error: "Use positive numeric IDs separated by commas" };
  }

  if (rawValues.length > maxItems) {
    return { values: null, error: `Choose at most ${maxItems} pipeline IDs` };
  }

  const values: number[] = [];
  const seen = new Set<number>();
  for (const raw of rawValues) {
    const parsed = typeof raw === "number" ? raw : Number(raw);
    if (!Number.isInteger(parsed) || parsed <= 0 || parsed > 2_147_483_647) {
      return { values: null, error: "Pipeline IDs must be positive 32-bit integers" };
    }
    if (seen.has(parsed)) {
      return { values: null, error: `Remove duplicate pipeline ID ${parsed}` };
    }
    seen.add(parsed);
    values.push(parsed);
  }
  return { values, error: null };
}

/** Returns whether a config value should satisfy a required or grouped field. */
export function hasConfigValue(value: unknown): boolean {
  if (Array.isArray(value)) return value.length > 0;
  if (typeof value === "string") return value.trim().length > 0;
  return value !== null && value !== undefined;
}

/** Validates catalog-declared one-of and mutually exclusive field relationships. */
export function validateFieldRelationships(
  fields: readonly ConfigField[],
  typeSpecific: Record<string, unknown>,
): Record<string, string> {
  const errors: Record<string, string> = {};
  const groups = new Map<string, ConfigField[]>();

  for (const field of fields) {
    if (field.oneOfGroup) {
      groups.set(field.oneOfGroup, [...(groups.get(field.oneOfGroup) ?? []), field]);
    }
    if (
      field.exclusiveWith &&
      hasConfigValue(typeSpecific[field.key]) &&
      hasConfigValue(typeSpecific[field.exclusiveWith])
    ) {
      const other = fields.find((candidate) => candidate.key === field.exclusiveWith);
      errors[field.key] = `Choose either ${field.label} or ${other?.label ?? field.exclusiveWith}`;
    }
  }

  for (const groupedFields of groups.values()) {
    if (!groupedFields.some((field) => hasConfigValue(typeSpecific[field.key]))) {
      errors[groupedFields[0].key] = `Choose one of: ${groupedFields
        .map((field) => field.label)
        .join(" or ")}`;
    }
  }
  return errors;
}

/** Updates one field and clears a catalog-declared mutually exclusive field. */
export function updateTypeSpecific(
  fields: readonly ConfigField[],
  typeSpecific: Record<string, unknown>,
  key: string,
  value: unknown,
): Record<string, unknown> {
  const updated = { ...typeSpecific, [key]: value };
  const field = fields.find((candidate) => candidate.key === key);
  if (field?.exclusiveWith && hasConfigValue(value)) {
    delete updated[field.exclusiveWith];
  }
  return updated;
}

/** Converts list fields to their persisted JSON representation. */
export function normalizeTypeSpecific(
  fields: readonly ConfigField[],
  typeSpecific: Record<string, unknown>,
): Record<string, unknown> {
  const normalized = { ...typeSpecific };
  for (const field of fields) {
    if ((field.oneOfGroup || field.exclusiveWith) && !hasConfigValue(normalized[field.key])) {
      delete normalized[field.key];
    }
  }
  for (const field of fields) {
    if (field.kind !== "integer-list" || !(field.key in normalized)) continue;
    const parsed = parseIntegerList(normalized[field.key]);
    if (!parsed.error && parsed.values) normalized[field.key] = parsed.values;
  }
  return normalized;
}
