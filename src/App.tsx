import { useEffect, useMemo, useState } from "react";

import { invoke } from "@tauri-apps/api/core";

type StatusKind = "success" | "warning" | "error" | "pending" | "neutral";

type FieldValue =
  | {
      type: "text";
      value: string;
    }
  | {
      type: "status";
      value: string;
      severity: StatusKind;
    }
  | {
      type: "number";
      value: number;
    }
  | {
      type: "url";
      value: string;
    };

type Field = {
  name: string;
  label: string;
  value: FieldValue;
};

type Activity = {
  id: string;
  title: string;
  fields: Field[];
};

type FieldDefinition = {
  name: string;
  label: string;
  field_type: "text" | "status" | "number" | "url";
  description: string;
};

type FeedSnapshot = {
  name: string;
  feed_type: string;
  activities: Activity[];
  provided_fields: FieldDefinition[];
  error: string | null;
};

type DotStatus = "green" | "yellow" | "red" | "blue" | "gray";

const FEEDS_CONFIG_PATH = "~/.config/cortado/feeds.toml";

function App() {
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedActivityId, setExpandedActivityId] = useState<string | null>(null);

  useEffect(() => {
    void invoke("init");

    const loadFeeds = async () => {
      try {
        const snapshots = await invoke<FeedSnapshot[]>("list_feeds");

        setFeeds(snapshots);
      } catch (error) {
        setLoadError(error instanceof Error ? error.message : String(error));
      } finally {
        setLoading(false);
      }
    };

    void loadFeeds();
  }, []);

  const availableActivityIds = useMemo(
    () =>
      feeds.flatMap((feed) => {
        const hasConfigError =
          Boolean(feed.error) &&
          feed.provided_fields.length === 0 &&
          feed.activities.length === 0;

        if (hasConfigError) {
          return [];
        }

        return feed.activities.map((activity) => activity.id);
      }),
    [feeds],
  );

  useEffect(() => {
    if (expandedActivityId !== null && !availableActivityIds.includes(expandedActivityId)) {
      setExpandedActivityId(null);
    }
  }, [availableActivityIds, expandedActivityId]);

  return (
    <div className="panel">
      {loadError ? <p className="load-error">{loadError}</p> : null}

      {loading ? <p className="state-msg">Loading…</p> : null}

      {!loading && !loadError && feeds.length === 0 ? (
        <p className="state-msg">
          No feeds configured. Edit <code>{FEEDS_CONFIG_PATH}</code> and restart.
        </p>
      ) : null}

      {!loading && !loadError && feeds.length > 0 ? (
        <div className="feed-list">
          {feeds.map((feed) => {
            const isConfigError =
              Boolean(feed.error) &&
              feed.provided_fields.length === 0 &&
              feed.activities.length === 0;

            return (
              <section className="feed" key={`${feed.feed_type}-${feed.name}`}>
                <header className="feed-header">
                  <span className="feed-name">{feed.name}</span>
                  <span className="feed-meta">
                    <span className="feed-type">{feed.feed_type}</span>
                    {!feed.error && feed.activities.length > 0 ? (
                      <span className="feed-count">{feed.activities.length}</span>
                    ) : null}
                  </span>
                </header>

                {feed.error ? (
                  <p className={isConfigError ? "feed-error config-error" : "feed-error poll-error"}>
                    {isConfigError ? "Config: " : "Poll: "}
                    {feed.error}
                  </p>
                ) : null}

                {!feed.error && feed.activities.length === 0 ? (
                  <p className="state-msg">No activities.</p>
                ) : null}

                {(!isConfigError && feed.activities.length > 0) || (!feed.error && feed.activities.length > 0) ? (
                  <div className="menu-list">
                    {feed.activities.map((activity) => {
                      const dotStatus = deriveDotStatus(activity.fields);
                      const expanded = expandedActivityId === activity.id;

                      return (
                        <div className="menu-item-group" key={activity.id}>
                          <button
                            aria-expanded={expanded}
                            className={`menu-item-btn ${expanded ? "active" : ""}`}
                            onClick={() => {
                              setExpandedActivityId((current) =>
                                current === activity.id ? null : activity.id,
                              );
                            }}
                            type="button"
                          >
                            <span
                              aria-label={`activity status ${dotStatus}`}
                              className={`activity-dot dot-${dotStatus}`}
                            />
                            <span className="menu-item-title">{activity.title}</span>
                            <span className="submenu-arrow" aria-hidden="true">
                              {expanded ? "▾" : "▸"}
                            </span>
                          </button>

                          {expanded ? (
                            <div className="submenu" role="group">
                              {activity.fields.length === 0 ? (
                                <p className="submenu-empty">No fields.</p>
                              ) : (
                                activity.fields.map((field) => (
                                  <div className="submenu-row" key={field.name}>
                                    <span className="submenu-key">{field.label}</span>
                                    <span className="submenu-value">
                                      {renderFieldValue(field)}
                                    </span>
                                  </div>
                                ))
                              )}
                            </div>
                          ) : null}
                        </div>
                      );
                    })}
                  </div>
                ) : null}
              </section>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}

function deriveDotStatus(fields: Field[]): DotStatus {
  let hasSuccess = false;
  let hasPending = false;
  let hasWarning = false;

  for (const field of fields) {
    if (field.value.type !== "status") {
      continue;
    }

    if (field.value.severity === "error") {
      return "red";
    }

    if (field.value.severity === "warning") {
      hasWarning = true;
      continue;
    }

    if (field.value.severity === "pending") {
      hasPending = true;
      continue;
    }

    if (field.value.severity === "success") {
      hasSuccess = true;
    }
  }

  if (hasWarning) {
    return "yellow";
  }

  if (hasPending) {
    return "blue";
  }

  if (hasSuccess) {
    return "green";
  }

  return "gray";
}

function renderFieldValue(field: Field) {
  switch (field.value.type) {
    case "status":
      return (
        <span className={`field-status status-${field.value.severity}`}>{field.value.value}</span>
      );
    case "url":
      return (
        <a className="field-link" href={field.value.value} rel="noreferrer" target="_blank">
          {field.value.value}
        </a>
      );
    case "number":
      return <span className="field-number">{field.value.value}</span>;
    case "text":
      return field.value.value;
    default:
      return null;
  }
}

export default App;
