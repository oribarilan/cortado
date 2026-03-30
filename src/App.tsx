import { useCallback, useEffect, useRef, useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Activity, FeedSnapshot } from "./shared/types";
import { useAppearance } from "./shared/useAppearance";
import {
  deriveActivityKind,
  highestStatusField,
  supportsOpen,
  supportsFocus,
  formatFieldValue,
  activityKey,
} from "./shared/utils";

function App() {
  useAppearance();
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedActivityKey, setExpandedActivityKey] = useState<string | null>(null);
  const [suppressCollapseAnimation, setSuppressCollapseAnimation] = useState(false);
  const panelContentRef = useRef<HTMLDivElement | null>(null);
  const panelRootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const root = panelRootRef.current;
    if (!root) return;

    const onKeyDown = () => root.classList.add("keyboard-active");
    const onMouseDown = () => root.classList.remove("keyboard-active");

    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("mousedown", onMouseDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("mousedown", onMouseDown);
    };
  }, []);

  const sortedFeeds = feeds;

  const [refreshing, setRefreshing] = useState(false);
  const [refreshProgress, setRefreshProgress] = useState<[number, number] | null>(null);

  const refreshNow = useCallback(async () => {
    setRefreshing(true);
    setRefreshProgress(null);
    try {
      await invoke("refresh_feeds");
      setLoadError(null);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : String(error));
    } finally {
      setRefreshing(false);
      setRefreshProgress(null);
    }
  }, []);

  const openActivity = useCallback(async (activity: Activity, feed?: FeedSnapshot) => {
    // Try focus action first (copilot-session feeds).
    if (feed) {
      const focus = supportsFocus(feed, activity);
      if (focus) {
        try {
          await invoke("focus_session", { sessionId: focus.sessionId });
          setLoadError(null);
        } catch (error) {
          setLoadError(error instanceof Error ? error.message : String(error));
        }
        return;
      }
    }

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
          if (document.activeElement instanceof HTMLElement) {
            document.activeElement.blur();
          }
          panelRootRef.current?.classList.remove("keyboard-active");

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

      const unlistenProgress = await listen<[number, number]>("refresh-progress", (event) => {
        setRefreshProgress(event.payload);
      });
      unlistenFns.push(unlistenProgress);
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
      ref={panelRootRef}
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
                        const focus = supportsFocus(feed, activity);
                        const canOpen = openUrl || focus;

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
                                  {canOpen ? (
                                    <button
                                      className="open-activity"
                                      onClick={() => {
                                        void openActivity(activity, feed);
                                      }}
                                    >
                                      {focus ? `↗ ${focus.label}` : "↗ Open Activity"}
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
              void invoke("open_main_screen");
            }}
          >
            Open App
          </button>
          <button
            className="footer-row"
            onClick={() => {
              void invoke("open_settings");
            }}
          >
            Settings
          </button>
          <button
            className="footer-row"
            onClick={() => {
              void refreshNow();
            }}
            disabled={refreshing}
          >
            {refreshing ? (
              <><span className="refresh-spinner" /> Refreshing{refreshProgress ? ` (${refreshProgress[0]}/${refreshProgress[1]})` : ""}…</>
            ) : "Refresh feeds"}
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
