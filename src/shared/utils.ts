import type { Activity, Field, FeedSnapshot, StatusKind } from "./types";

export function kindPriority(kind: StatusKind): number {
  switch (kind) {
    case "attention-negative":
      return 5;
    case "waiting":
      return 4;
    case "running":
      return 3;
    case "attention-positive":
      return 2;
    case "idle":
      return 1;
  }
}

export function deriveActivityKind(activity: Activity): StatusKind {
  if (activity.retained) {
    return "idle";
  }

  let best: StatusKind = "idle";

  for (const field of activity.fields) {
    if (field.value.type !== "status") {
      continue;
    }

    if (kindPriority(field.value.kind) > kindPriority(best)) {
      best = field.value.kind;
    }
  }

  return best;
}

export function highestStatusField(activity: Activity): Field | null {
  let best: Field | null = null;
  let bestPriority = 0;

  for (const field of activity.fields) {
    if (field.value.type !== "status") {
      continue;
    }

    const priority = kindPriority(field.value.kind);
    if (priority > bestPriority) {
      best = field;
      bestPriority = priority;
    }
  }

  return best;
}

export function supportsOpen(activity: Activity): string | null {
  if (activity.id.startsWith("https://") || activity.id.startsWith("http://")) {
    return activity.id;
  }

  const fieldUrl = activity.fields.find(
    (field) =>
      field.value.type === "url" &&
      (field.value.value.startsWith("https://") || field.value.value.startsWith("http://"))
  );
  if (fieldUrl && fieldUrl.value.type === "url") {
    return fieldUrl.value.value;
  }

  return null;
}

export function formatFieldValue(field: Field): string {
  if (field.value.type === "number") {
    return Number.isInteger(field.value.value)
      ? String(field.value.value)
      : field.value.value.toFixed(2);
  }

  return field.value.value;
}

export function activityKey(feed: FeedSnapshot, activity: Activity): string {
  return `${feed.name}::${feed.feed_type}::${activity.id}`;
}

/// Returns true if this activity is an app update (cortado-update feed).
export function supportsUpdate(feed: FeedSnapshot): boolean {
  return feed.feed_type === "cortado-update";
}

/// Returns true if this activity is a plugin update (not an app update).
export function isPluginUpdate(activity: Activity): boolean {
  return activity.id.startsWith("plugin-update-");
}
/// Returns focus info if the activity has a `focus_app` field (any harness-based feed).
export function supportsFocus(
  activity: Activity
): { sessionId: string; label: string } | null {
  const appField = activity.fields.find((f) => f.name === "focus_app");
  if (!appField) {
    return null;
  }

  const appName = appField.value.type === "text" ? appField.value.value : "terminal";
  return { sessionId: activity.id, label: `Open in ${appName}` };
}
