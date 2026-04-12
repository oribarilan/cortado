import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";

import type { Activity, FeedSnapshot } from "../shared/types";
import { useAppearance } from "../shared/useAppearance";
import { POPULAR_FEED_TYPES, type FeedType } from "../shared/feedTypes";
import {
  deriveActivityKind,
  highestStatusField,
  supportsOpen,
  supportsFocus,
  supportsRestart,
  supportsUpdate,
  isPluginUpdate,
  formatFieldValue,
  activityKey,
  formatRelativeTime,
} from "../shared/utils";

type AppSettings = {
  panel: { show_priority_section: boolean; show_empty_feeds: boolean };
};

/** A flat list item for keyboard navigation. */
type ListItem = {
  feed: FeedSnapshot;
  activity: Activity;
  key: string;
  /** If present, this item is in the priority section and came from this feed. */
  priorityFeedHint?: string;
};

function buildFlatList(
  feeds: FeedSnapshot[],
  showPriority: boolean,
): { items: ListItem[]; priorityItems: ListItem[]; feedItems: ListItem[] } {
  const priorityItems: ListItem[] = [];
  const feedItems: ListItem[] = [];
  const priorityKeys = new Set<string>();

  if (showPriority) {
    for (const feed of feeds) {
      if (feed.is_disconnected) continue;
      for (const activity of feed.activities) {
        if (activity.retained) continue;
        const kind = deriveActivityKind(activity);
        if (kind === "attention-negative" || kind === "attention-positive") {
          const key = activityKey(feed, activity);
          priorityItems.push({
            feed,
            activity,
            key,
            priorityFeedHint: feed.name,
          });
          priorityKeys.add(key);
        }
      }
    }
  }

  for (const feed of feeds) {
    if (feed.is_disconnected) continue;
    for (const activity of feed.activities) {
      const key = activityKey(feed, activity);
      if (!priorityKeys.has(key)) {
        feedItems.push({ feed, activity, key });
      }
    }
  }

  return { items: [...priorityItems, ...feedItems], priorityItems, feedItems };
}

function EmptyState() {
  const openSettings = (feedType?: FeedType) => {
    invoke("open_settings", {
      section: "feeds",
      feedType: feedType ?? null,
    }).catch(console.error);
  };

  return (
    <div className="ms-split">
      <div className="ms-list ms-empty-list">
        <div className="ms-empty-welcome">
          <div className="ms-empty-icon">☕</div>
          <div className="ms-empty-heading">Welcome to Cortado</div>
          <div className="ms-empty-body">
            A feed tracks a data source and surfaces its activities,
            like PRs, CI runs, and endpoints, so you can check their
            status at a glance.
          </div>
          <button className="ms-empty-cta" onClick={() => openSettings()}>
            + Add your first feed
          </button>
          <div className="ms-empty-secondary">
            or edit ~/.config/cortado/feeds.toml
          </div>
          <div className="ms-empty-hotkey-hint">
            <kbd>⌘</kbd><kbd>⇧</kbd><kbd>Space</kbd> to toggle this panel
          </div>
        </div>
      </div>
      <div className="ms-detail ms-empty-detail">
        <div className="ms-empty-types">
          <div className="ms-empty-types-header">Feed types</div>
          {POPULAR_FEED_TYPES.map((ft) => (
            <button
              key={ft.feedType}
              className="ms-empty-type-card"
              onClick={() => openSettings(ft.feedType)}
            >
              <span className="ms-empty-type-icon" dangerouslySetInnerHTML={{ __html: ft.icon }} />
              <div>
                <div className="ms-empty-type-name">{ft.name}</div>
                <div className="ms-empty-type-desc">{ft.description}</div>
              </div>
            </button>
          ))}
          <button
            className="ms-empty-type-card ms-empty-type-more"
            onClick={() => openSettings()}
          >
            <span className="ms-empty-type-icon">...</span>
            <div>
              <div className="ms-empty-type-name">See all feed types</div>
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}

function DetailPane({
  item,
  installing,
  pluginInstalling,
  onInstallUpdate,
  onInstallPluginUpdate,
}: {
  item: ListItem | null;
  installing: boolean;
  pluginInstalling: boolean;
  onInstallUpdate: () => void;
  onInstallPluginUpdate: () => void;
}) {
  if (!item) {
    return (
      <div className="ms-detail">
        <div className="ms-detail-empty">No activity selected</div>
      </div>
    );
  }

  const { feed, activity } = item;
  const isUpdate = supportsUpdate(feed);
  const focus = supportsFocus(activity);
  const openUrl = supportsOpen(activity);
  const canOpen = focus || openUrl;
  const restart = supportsRestart(activity);

  const handleOpen = () => {
    if (focus) {
      invoke("focus_session", { sessionId: focus.sessionId }).catch(console.error);
    } else if (openUrl) {
      invoke("open_activity", { url: openUrl }).catch(console.error);
    }
  };

  const handleRestart = () => {
    invoke("restart_app").catch(console.error);
  };

  return (
    <div className="ms-detail">
      <div className="ms-detail-content" key={item.key}>
        <div className="ms-detail-title">{activity.title}</div>
        {isUpdate && !isPluginUpdate(activity) ? (
          <button
            className="ms-detail-open update-action"
            onClick={onInstallUpdate}
            disabled={installing}
          >
            {installing ? "Installing..." : "↗ Install update"}
          </button>
        ) : isUpdate && isPluginUpdate(activity) ? (
          <button
            className="ms-detail-open update-action"
            onClick={onInstallPluginUpdate}
            disabled={pluginInstalling}
          >
            {pluginInstalling ? "Updating..." : "↗ Update plugin"}
          </button>
        ) : restart ? (
          <button className="ms-detail-open" onClick={handleRestart}>
            ↗ Restart Cortado
          </button>
        ) : canOpen ? (
          <button className="ms-detail-open" onClick={handleOpen}>
            ↗ {focus ? focus.label : "Open Activity"}
          </button>
        ) : null}
        {activity.fields.length > 0 ? (
          <div className="ms-detail-fields">
            {activity.fields.filter((f) => !f.name.startsWith("focus_")).map((field) => {
              const statusClass =
                field.value.type === "status" ? `status kind-${field.value.kind}` : "";
              return (
                <div className="ms-detail-field" key={field.name}>
                  <span className="ms-detail-key">{field.label}</span>
                  <span className={`ms-detail-val ${statusClass}`}>
                    {formatFieldValue(field)}
                  </span>
                </div>
              );
            })}
          </div>
        ) : null}
        {feed.last_refreshed != null ? (
          <span className="last-refreshed">Updated {formatRelativeTime(feed.last_refreshed)}</span>
        ) : null}
      </div>
    </div>
  );
}

function MainScreenApp() {
  useAppearance();
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [seeded, setSeeded] = useState(false);
  const [focusIndex, setFocusIndex] = useState(0);
  const [showPrioritySection, setShowPrioritySection] = useState(true);
  const [showEmptyFeeds, setShowEmptyFeeds] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshProgress, setRefreshProgress] = useState<[number, number] | null>(null);
  const [isDev, setIsDev] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  const listRef = useRef<HTMLDivElement | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);

  // Update install state
  const [installing, setInstalling] = useState(false);
  const [pluginInstalling, setPluginInstalling] = useState(false);

  const installUpdate = useCallback(async () => {
    setInstalling(true);
    try {
      await invoke("install_update");
    } catch {
      setInstalling(false);
    }
  }, []);

  const installPluginUpdate = useCallback(async () => {
    setPluginInstalling(true);
    try {
      const result = await invoke<{ success: boolean; error?: string }>("install_opencode_plugin");
      if (result.success) {
        setFeeds((prev) =>
          prev.map((f) =>
            f.feed_type === "cortado-update"
              ? { ...f, activities: f.activities.filter((a) => !isPluginUpdate(a)) }
              : f
          )
        );
      }
    } catch {
      // ignore
    } finally {
      setPluginInstalling(false);
    }
  }, []);

  // Tick counter to keep relative timestamps fresh.
  const [, setTick] = useState(0);
  useEffect(() => {
    const timer = setInterval(() => setTick((t) => t + 1), 30_000);
    return () => clearInterval(timer);
  }, []);

  // Filter out empty feeds when the setting is off
  const visibleFeeds = useMemo(() => {
    return feeds.filter((feed) => {
      if (feed.activities.length > 0 || feed.error) return true;
      if (feed.hide_when_empty) return false;
      if (!seeded) return true;
      return showEmptyFeeds;
    });
  }, [feeds, showEmptyFeeds, seeded]);

  const { items: flatList, priorityItems, feedItems } = useMemo(
    () => buildFlatList(visibleFeeds, showPrioritySection),
    [visibleFeeds, showPrioritySection],
  );

  const focusedItem = flatList[focusIndex] ?? null;

  // Track keyboard vs mouse input for focus ring visibility
  useEffect(() => {
    const root = rootRef.current;
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

  // Init panel
  useEffect(() => {
    invoke("init_main_screen_panel").catch((err) => {
      console.error("failed to init main screen panel:", err);
    });
    invoke<boolean>("is_dev_mode").then(setIsDev).catch(() => {});
    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  // Load data + subscribe to updates
  useEffect(() => {
    let isMounted = true;
    const unlistenFns: UnlistenFn[] = [];

    const bootstrap = async () => {
      try {
        const [initialFeeds, settings] = await Promise.all([
          invoke<FeedSnapshot[]>("list_feeds"),
          invoke<AppSettings>("get_settings"),
        ]);
        if (isMounted) {
          setFeeds(initialFeeds);
          setShowPrioritySection(settings.panel?.show_priority_section ?? true);
          setShowEmptyFeeds(settings.panel?.show_empty_feeds ?? false);
        }
      } catch (err) {
        console.error("failed to load feeds:", err);
      } finally {
        if (isMounted) setLoading(false);
      }

      const unlisten = await listen<FeedSnapshot[]>("feeds-updated", (event) => {
        setFeeds(event.payload);
        setSeeded(true);
      });
      unlistenFns.push(unlisten);

      const unlistenShow = await listen("main_screen_panel_will_show", () => {
        setFocusIndex(0);
        if (listRef.current) listRef.current.scrollTop = 0;
        invoke<AppSettings>("get_settings")
          .then((s) => {
            if (isMounted) {
              setShowPrioritySection(s.panel?.show_priority_section ?? true);
              setShowEmptyFeeds(s.panel?.show_empty_feeds ?? false);
            }
          })
          .catch(() => {});
      });
      unlistenFns.push(unlistenShow);

      const unlistenProgress = await listen<[number, number]>("refresh-progress", (event) => {
        setRefreshProgress(event.payload);
      });
      unlistenFns.push(unlistenProgress);
    };

    void bootstrap();

    return () => {
      isMounted = false;
      for (const fn of unlistenFns) void fn();
    };
  }, []);

  // Keyboard navigation
  const openFocusedActivity = useCallback(() => {
    if (!focusedItem) return;
    if (supportsUpdate(focusedItem.feed)) {
      if (isPluginUpdate(focusedItem.activity)) {
        void installPluginUpdate();
      } else {
        void installUpdate();
      }
      return;
    }
    if (supportsRestart(focusedItem.activity)) {
      invoke("restart_app").catch(console.error);
      return;
    }
    const focus = supportsFocus(focusedItem.activity);
    if (focus) {
      invoke("focus_session", { sessionId: focus.sessionId }).catch(console.error);
      return;
    }
    const url = supportsOpen(focusedItem.activity);
    if (url) invoke("open_activity", { url }).catch(console.error);
  }, [focusedItem, installUpdate, installPluginUpdate]);

  const refreshFeeds = useCallback(async () => {
    if (refreshing) return;
    setRefreshing(true);
    setRefreshProgress(null);
    try {
      await invoke("refresh_feeds");
    } catch (err) {
      console.error("refresh failed:", err);
    } finally {
      setRefreshing(false);
      setRefreshProgress(null);
    }
  }, [refreshing]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        invoke("hide_main_screen_panel").catch(console.error);
        return;
      }

      if (e.key === "q" && e.metaKey) {
        invoke("quit_app").catch(console.error);
        return;
      }

      if (e.key === "," && e.metaKey) {
        e.preventDefault();
        invoke("open_settings").catch(console.error);
        return;
      }

      if (e.key === "ArrowDown" || (e.key === "j" && !e.metaKey && !e.ctrlKey && !e.altKey)) {
        e.preventDefault();
        if (flatList.length === 0) return;
        setFocusIndex((i) => (i + 1) % flatList.length);
        return;
      }

      if (e.key === "ArrowUp" || (e.key === "k" && !e.metaKey && !e.ctrlKey && !e.altKey)) {
        e.preventDefault();
        if (flatList.length === 0) return;
        setFocusIndex((i) => (i - 1 + flatList.length) % flatList.length);
        return;
      }

      if (e.key === "Enter") {
        e.preventDefault();
        if (flatList.length === 0) return;
        openFocusedActivity();
        return;
      }

      if (e.key === "r" && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        void refreshFeeds();
        return;
      }
    };

    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [flatList.length, openFocusedActivity, refreshFeeds]);

  // Scroll focused row into view
  useEffect(() => {
    const row = listRef.current?.querySelector(`[data-index="${focusIndex}"]`);
    if (row) row.scrollIntoView({ block: "nearest" });
  }, [focusIndex]);

  // Group feed activities for rendering (deduped, priority items excluded)
  const feedSections = useMemo(() => {
    const priorityCount = priorityItems.length;
    const feedKeySet = new Set(feedItems.map((i) => i.key));
    let globalIndex = priorityCount;

    return visibleFeeds.map((feed) => {
      const items = feed.activities
        .filter((activity) => feedKeySet.has(activityKey(feed, activity)))
        .map((activity) => ({
          activity,
          kind: deriveActivityKind(activity),
          key: activityKey(feed, activity),
          index: globalIndex++,
        }));
      return { feed, items };
    });
  }, [visibleFeeds, feedItems, priorityItems.length]);

  const hasUserFeeds = feeds.some(
    (f) => !f.hide_when_empty || f.activities.length > 0 || f.error,
  );

  return (
    <div className="main-screen-root" ref={rootRef}>
      {isDev ? <div className="dev-badge">DEV</div> : null}
      {!loading && !hasUserFeeds ? (
        <EmptyState />
      ) : (
      <div className="ms-split">
        {/* List pane */}
        <div className="ms-list" ref={listRef}>
          {loading ? (
            <div className="ms-loading-state">
              <div className="ms-skel-row stagger-0"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "65%" }} /></div>
              <div className="ms-skel-row stagger-1"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "80%" }} /></div>
              <div className="ms-skel-row stagger-2"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "50%" }} /></div>
              <div className="ms-skel-row stagger-3"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "72%" }} /></div>
              <div className="ms-skel-row stagger-4"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "58%" }} /></div>
            </div>
          ) : !seeded && flatList.length === 0 ? (
            <div className="ms-loading-state">
              <div className="ms-skel-row stagger-0"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "65%" }} /></div>
              <div className="ms-skel-row stagger-1"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "80%" }} /></div>
              <div className="ms-skel-row stagger-2"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "50%" }} /></div>
              <div className="ms-skel-row stagger-3"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "72%" }} /></div>
              <div className="ms-skel-row stagger-4"><div className="ms-skel-dot" /><div className="ms-skel-title" style={{ width: "58%" }} /></div>
            </div>
          ) : (
            <>
              {/* Priority section */}
              <section className={`ms-feed-section ms-priority-section ${priorityItems.length === 0 ? "collapsing" : ""}`}>
                <div className="ms-priority-inner">
                  <header className="ms-feed-header ms-priority-header">Attention</header>
                  {priorityItems.map((item, index) => {
                    const kind = deriveActivityKind(item.activity);
                    const topStatus = highestStatusField(item.activity);
                    const isFocused = index === focusIndex;

                    return (
                      <div
                        key={item.key}
                        data-index={index}
                        className={`ms-activity-row kind-${kind} ${isFocused ? "focused" : ""}`}
                        onClick={() => setFocusIndex(index)}
                        onDoubleClick={() => {
                          const focus = supportsFocus(item.activity);
                          if (focus) { invoke("focus_session", { sessionId: focus.sessionId }).catch(console.error); return; }
                          const url = supportsOpen(item.activity);
                          if (url) invoke("open_activity", { url }).catch(console.error);
                        }}
                      >
                        <span className="ms-dot" aria-hidden="true" />
                        <span className="ms-activity-title">{item.activity.title}</span>
                        {topStatus && topStatus.value.type === "status" ? (
                          <span className={`ms-chip kind-${topStatus.value.kind}`}>{topStatus.value.value}</span>
                        ) : null}
                        <span className="ms-feed-hint">{item.priorityFeedHint}</span>
                      </div>
                    );
                  })}
                  <div className="ms-priority-separator" />
                </div>
              </section>

              {/* Feed sections */}
              {feedSections.map(({ feed, items }) => (
                <section className={`ms-feed-section ${feed.is_disconnected ? "disconnected" : ""}`} key={`${feed.name}::${feed.feed_type}`}>
                  <header className="ms-feed-header">
                    {feed.name}
                    {feed.is_disconnected ? (
                      <span className="disconnected-label">disconnected</span>
                    ) : null}
                  </header>

                  {feed.is_disconnected ? null : (
                    <>
                      {feed.error ? (
                        <div className="ms-feed-error">{feed.error}</div>
                      ) : items.length === 0 ? (
                        <div className="ms-feed-empty">No activities</div>
                      ) : (
                        items.map(({ activity, kind, key, index }) => {
                          const isFocused = index === focusIndex;
                          const topStatus = highestStatusField(activity);

                          return (
                            <div
                              key={key}
                              data-index={index}
                              className={`ms-activity-row kind-${kind} ${isFocused ? "focused" : ""}`}
                              onClick={() => setFocusIndex(index)}
                              onDoubleClick={() => {
                                const focus = supportsFocus(activity);
                                if (focus) { invoke("focus_session", { sessionId: focus.sessionId }).catch(console.error); return; }
                                const url = supportsOpen(activity);
                                if (url) invoke("open_activity", { url }).catch(console.error);
                              }}
                            >
                              <span
                                className={`ms-dot ${activity.retained ? "retained" : ""}`}
                                aria-hidden="true"
                              />
                              <span className="ms-activity-title">{activity.title}</span>
                              {topStatus && topStatus.value.type === "status" ? (
                                <span className={`ms-chip kind-${topStatus.value.kind}`}>{topStatus.value.value}</span>
                              ) : null}
                            </div>
                          );
                        })
                      )}
                    </>
                  )}
                </section>
              ))}
            </>
          )}
        </div>

        {/* Detail pane */}
        <DetailPane
              item={focusedItem}
              installing={installing}
              pluginInstalling={pluginInstalling}
              onInstallUpdate={() => void installUpdate()}
              onInstallPluginUpdate={() => void installPluginUpdate()}
            />
      </div>
      )}

      {/* Footer */}
      <footer className="ms-footer">
        <span className="ms-footer-hints">
          {refreshing ? (
            <><span className="ms-footer-spinner" />Refreshing{refreshProgress ? ` (${refreshProgress[0]}/${refreshProgress[1]})` : ""}…</>
          ) : feeds.some(f => f.is_disconnected) ? (
            <>No connection · <button className="ms-retry-link" onClick={() => invoke("retry_connection").catch(console.error)}>Retry</button></>
          ) : (
            <><kbd>↑/↓</kbd><kbd>j/k</kbd> navigate · <kbd>↵</kbd> open · <kbd>r</kbd> refresh · <kbd>esc</kbd> close</>
          )}
        </span>
        <span className="ms-footer-right">
          {appVersion ? <span className="ms-footer-version">v{appVersion}{isDev ? "-dev" : ""}</span> : null}
          <button
            className="ms-footer-settings"
            onClick={() => invoke("open_settings").catch(console.error)}
            title="Settings"
          >
            ⚙
          </button>
        </span>
      </footer>
    </div>
  );
}

export default MainScreenApp;
