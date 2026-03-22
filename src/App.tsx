import { useEffect, useState } from "react";

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

const FEEDS_CONFIG_PATH = "~/.config/cortado/feeds.toml";

function App() {
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

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

  return (
    <div className="container">
      <h1>Cortado</h1>
      <p className="subtitle">Developer feed panel</p>

      <section className="section">
        <h2>Feeds</h2>

        {loadError ? (
          <p className="app-error">Failed to load feeds: {loadError}</p>
        ) : null}

        {loading ? <p>Loading feeds…</p> : null}

        {!loading && !loadError && feeds.length === 0 ? (
          <p className="empty-state">
            No feeds configured yet. Add one in <code>{FEEDS_CONFIG_PATH}</code> and restart
            Cortado.
          </p>
        ) : null}

        {!loading && !loadError && feeds.length > 0 ? (
          <ul className="feed-list">
            {feeds.map((feed) => {
              const isConfigError =
                Boolean(feed.error) &&
                feed.provided_fields.length === 0 &&
                feed.activities.length === 0;

              return (
                <li className="feed-card" key={`${feed.feed_type}-${feed.name}`}>
                  <div className="feed-header">
                    <p className="feed-name">{feed.name}</p>
                    <span className="feed-type">{feed.feed_type}</span>
                  </div>

                  {feed.error ? (
                    <p className={isConfigError ? "feed-error config-error" : "feed-error poll-error"}>
                      <strong>{isConfigError ? "Config error:" : "Poll error:"}</strong>{" "}
                      {feed.error}
                    </p>
                  ) : null}

                  {!feed.error && feed.activities.length === 0 ? (
                    <p className="feed-empty">No activities in this feed.</p>
                  ) : null}

                  {!feed.error ? (
                    <div className="activity-list">
                      {feed.activities.map((activity) => (
                        <article className="activity-row" key={activity.id}>
                          <p className="activity-title">{activity.title}</p>

                          <div className="field-list">
                            {activity.fields.map((field) => (
                              <div className="field-row" key={field.name}>
                                <span className="field-label">{field.label}</span>
                                <span className="field-value">{renderFieldValue(field)}</span>
                              </div>
                            ))}
                          </div>
                        </article>
                      ))}
                    </div>
                  ) : null}
                </li>
              );
            })}
          </ul>
        ) : null}
      </section>
    </div>
  );
}

function renderFieldValue(field: Field) {
  switch (field.value.type) {
    case "text":
      return field.value.value;
    case "number":
      return field.value.value.toString();
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
    default:
      return "";
  }
}

export default App;
