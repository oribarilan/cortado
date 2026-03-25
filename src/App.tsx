import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type StatusKind = "attention-negative" | "attention-positive" | "waiting" | "running" | "idle";

type FieldValue =
  | {
      type: "text";
      value: string;
    }
  | {
      type: "status";
      value: string;
      kind: StatusKind;
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
  retained: boolean;
};

type FeedSnapshot = {
  name: string;
  feed_type: string;
  activities: Activity[];
  error: string | null;
};

function kindPriority(kind: StatusKind): number {
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

function deriveActivityKind(activity: Activity): StatusKind {
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

function highestStatusField(activity: Activity): Field | null {
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

function supportsOpen(activity: Activity): string | null {
  if (activity.id.startsWith("https://") || activity.id.startsWith("http://")) {
    return activity.id;
  }

  const fieldUrl = activity.fields.find(
    (field) => field.value.type === "url" && (field.value.value.startsWith("https://") || field.value.value.startsWith("http://"))
  );
  if (fieldUrl && fieldUrl.value.type === "url") {
    return fieldUrl.value.value;
  }

  return null;
}

function formatFieldValue(field: Field): string {
  if (field.value.type === "number") {
    return Number.isInteger(field.value.value)
      ? String(field.value.value)
      : field.value.value.toFixed(2);
  }

  return field.value.value;
}

function activityKey(feed: FeedSnapshot, activity: Activity): string {
  return `${feed.name}::${feed.feed_type}::${activity.id}`;
}

function App() {
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedActivityKey, setExpandedActivityKey] = useState<string | null>(null);
  const [suppressCollapseAnimation, setSuppressCollapseAnimation] = useState(false);
  const panelContentRef = useRef<HTMLDivElement | null>(null);

  const sortedFeeds = useMemo(() => {
    return feeds.map((feed) => ({
      ...feed,
      activities: [...feed.activities].sort((a, b) => Number(a.retained) - Number(b.retained)),
    }));
  }, [feeds]);

  const refreshNow = useCallback(async () => {
    try {
      await invoke("refresh_feeds");
      setLoadError(null);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : String(error));
    }
  }, []);

  const openActivity = useCallback(async (activity: Activity) => {
    const url = supportsOpen(activity);
    if (!url) {
      return;
    }

    try {
      await invoke("open_activity", { url });
      setLoadError(null);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : String(error));
    }
  }, []);

  const quitApp = useCallback(async () => {
    await invoke("quit_app");
  }, []);

  useEffect(() => {
    let isMounted = true;
    const unlistenFns: UnlistenFn[] = [];

    const bootstrap = async () => {
      try {
        await invoke("init_panel");
      } catch (error) {
        if (isMounted) {
          setLoadError(error instanceof Error ? error.message : String(error));
        }
      }

      try {
        const initialFeeds = await invoke<FeedSnapshot[]>("list_feeds");
        if (isMounted) {
          setFeeds(initialFeeds);
          setLoadError(null);
        }
      } catch (error) {
        if (isMounted) {
          setLoadError(error instanceof Error ? error.message : String(error));
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }

      const unlisten = await listen<FeedSnapshot[]>("feeds-updated", (event) => {
        setFeeds(event.payload);
        setLoadError(null);

        setExpandedActivityKey((current) => {
          if (!current) {
            return current;
          }

          const stillExists = event.payload.some((feed) =>
            feed.activities.some((activity) => activityKey(feed, activity) === current)
          );

          return stillExists ? current : null;
        });
      });

      unlistenFns.push(unlisten);

      const unlistenPanelWillShow = await listen("menubar_panel_will_show", () => {
        setSuppressCollapseAnimation(true);
        setExpandedActivityKey(null);

        requestAnimationFrame(() => {
          const panelContent = panelContentRef.current;
          if (panelContent) {
            panelContent.scrollTop = 0;
          }

          requestAnimationFrame(() => {
            setSuppressCollapseAnimation(false);
          });
        });
      });

      unlistenFns.push(unlistenPanelWillShow);
    };

    void bootstrap();

    return () => {
      isMounted = false;
      for (const unlisten of unlistenFns) {
        void unlisten();
      }
    };
  }, []);

  return (
    <div
      className={`panel-root ${suppressCollapseAnimation ? "suppress-collapse-animation" : ""}`}
      role="region"
      aria-label="Cortado menubar panel"
    >
      <div className="panel-content" ref={panelContentRef}>
        {loading ? (
          <div className="loading-state" aria-live="polite">
            <div className="skeleton w-55" />
            <div className="skeleton w-85" />
            <div className="skeleton w-70" />
            <div className="skeleton w-40" />
          </div>
        ) : null}

        {!loading && sortedFeeds.length === 0 ? (
          <p className="empty-state">No feeds configured. Add a Feed in <code>~/.config/cortado/feeds.toml</code>.</p>
        ) : null}

        {!loading && sortedFeeds.length > 0
          ? sortedFeeds.map((feed) => {
              const hasError = Boolean(feed.error);
              const isConfigWarning = feed.feed_type === "app";

              return (
                <section className="feed-block" key={`${feed.name}::${feed.feed_type}`}>
                  <header className="feed-header">
                    <span className="feed-name">{feed.name}</span>
                    {!hasError ? <span className="feed-count">{feed.activities.length}</span> : null}
                  </header>

                  {hasError ? (
                    <p className={`feed-error ${isConfigWarning ? "config" : "poll"}`}>{feed.error}</p>
                  ) : null}

                  {!hasError && feed.activities.length === 0 ? (
                    <p className="feed-empty">No activities</p>
                  ) : null}

                  {feed.activities.length > 0 ? (
                    <div className="activity-list">
                      {feed.activities.map((activity) => {
                        const activityKind = deriveActivityKind(activity);
                        const key = activityKey(feed, activity);
                        const expanded = expandedActivityKey === key;
                        const firstStatus = highestStatusField(activity);
                        const openUrl = supportsOpen(activity);

                        return (
                          <div
                            className={`activity-wrap kind-${activityKind} ${expanded ? "expanded" : ""}`}
                            key={key}
                          >
                            <button
                              className="activity-row"
                              onClick={() => {
                                setExpandedActivityKey((current) => (current === key ? null : key));
                              }}
                              aria-expanded={expanded}
                            >
                              <span className={`status-dot ${activity.retained ? "retained" : ""}`} aria-hidden="true" />
                              <span className="activity-title">{activity.title}</span>
                              {firstStatus && firstStatus.value.type === "status" ? (
                                <span className={`status-chip kind-${firstStatus.value.kind}`}>
                                  {firstStatus.value.value}
                                </span>
                              ) : null}
                              <span className="chevron" aria-hidden="true">▸</span>
                            </button>

                            <div className="detail-region" role="region" aria-label={`${activity.title} details`}>
                              <div className="detail-inner">
                                <div className="detail-body">
                                  {openUrl ? (
                                    <button
                                      className="open-activity"
                                      onClick={() => {
                                        void openActivity(activity);
                                      }}
                                    >
                                      ↗ Open Activity
                                    </button>
                                  ) : null}

                                  {activity.fields.map((field) => {
                                    const statusClass =
                                      field.value.type === "status"
                                        ? `status kind-${field.value.kind}`
                                        : "";

                                    return (
                                      <div className="field-row" key={`${activity.id}::${field.name}`}>
                                        <span className="field-key">{field.label}</span>
                                        <span className={`field-value ${statusClass}`}>{formatFieldValue(field)}</span>
                                      </div>
                                    );
                                  })}
                                </div>
                              </div>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  ) : null}
                </section>
              );
            })
          : null}

        {loadError ? <p className="panel-error">{loadError}</p> : null}

        <footer className="panel-footer">
          <button
            className="footer-row"
            onClick={() => {
              void refreshNow();
            }}
          >
            Refresh feeds
          </button>
          <button
            className="footer-row"
            onClick={() => {
              void quitApp();
            }}
          >
            Quit Cortado
          </button>
        </footer>
      </div>
    </div>
  );
}

export default App;
