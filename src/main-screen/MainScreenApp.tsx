import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Activity, FeedSnapshot } from "../shared/types";
import { useAppearance } from "../shared/useAppearance";
import {
  deriveActivityKind,
  highestStatusField,
  supportsOpen,
  formatFieldValue,
  activityKey,
} from "../shared/utils";

type AppSettings = {
  panel: { show_priority_section: boolean };
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
      for (const activity of feed.activities) {
        if (activity.retained) continue;
        const kind = deriveActivityKind(activity);
        if (kind === "attention-negative") {
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
    const sorted = [...feed.activities].sort(
      (a, b) => Number(a.retained) - Number(b.retained)
    );
    for (const activity of sorted) {
      const key = activityKey(feed, activity);
      if (!priorityKeys.has(key)) {
        feedItems.push({ feed, activity, key });
      }
    }
  }

  return { items: [...priorityItems, ...feedItems], priorityItems, feedItems };
}

function DetailPane({ item }: { item: ListItem | null }) {
  if (!item) {
    return (
      <div className="ms-detail">
        <div className="ms-detail-empty">No activity selected</div>
      </div>
    );
  }

  const { activity } = item;

  return (
    <div className="ms-detail">
      <div className="ms-detail-content" key={item.key}>
        <div className="ms-detail-title">{activity.title}</div>
        {activity.fields.length > 0 ? (
          <div className="ms-detail-fields">
            {activity.fields.map((field) => {
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
      </div>
    </div>
  );
}

function MainScreenApp() {
  useAppearance();
  const [feeds, setFeeds] = useState<FeedSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [focusIndex, setFocusIndex] = useState(0);
  const [showPrioritySection, setShowPrioritySection] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshProgress, setRefreshProgress] = useState<[number, number] | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);

  const { items: flatList, priorityItems, feedItems } = useMemo(
    () => buildFlatList(feeds, showPrioritySection),
    [feeds, showPrioritySection],
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
        }
      } catch (err) {
        console.error("failed to load feeds:", err);
      } finally {
        if (isMounted) setLoading(false);
      }

      const unlisten = await listen<FeedSnapshot[]>("feeds-updated", (event) => {
        setFeeds(event.payload);
      });
      unlistenFns.push(unlisten);

      const unlistenShow = await listen("main_screen_panel_will_show", () => {
        setFocusIndex(0);
        if (listRef.current) listRef.current.scrollTop = 0;
        invoke<AppSettings>("get_settings")
          .then((s) => {
            if (isMounted) setShowPrioritySection(s.panel?.show_priority_section ?? true);
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
    const url = supportsOpen(focusedItem.activity);
    if (url) invoke("open_activity", { url }).catch(console.error);
  }, [focusedItem]);

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

      if (e.key === "ArrowDown") {
        e.preventDefault();
        if (flatList.length === 0) return;
        setFocusIndex((i) => Math.min(i + 1, flatList.length - 1));
        return;
      }

      if (e.key === "ArrowUp") {
        e.preventDefault();
        if (flatList.length === 0) return;
        setFocusIndex((i) => Math.max(i - 1, 0));
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

  // Group feed activities for rendering (deduped — priority items excluded)
  const feedSections = useMemo(() => {
    const priorityCount = priorityItems.length;
    const feedKeySet = new Set(feedItems.map((i) => i.key));
    let globalIndex = priorityCount;

    return feeds.map((feed) => {
      const sorted = [...feed.activities].sort(
        (a, b) => Number(a.retained) - Number(b.retained)
      );
      const items = sorted
        .filter((activity) => feedKeySet.has(activityKey(feed, activity)))
        .map((activity) => ({
          activity,
          kind: deriveActivityKind(activity),
          key: activityKey(feed, activity),
          index: globalIndex++,
        }));
      return { feed, items };
    });
  }, [feeds, feedItems, priorityItems.length]);

  return (
    <div className="main-screen-root" ref={rootRef}>
      <div className="ms-split">
        {/* List pane */}
        <div className="ms-list" ref={listRef}>
          {loading ? (
            <div className="ms-loading-state">
              <div className="ms-skeleton w-55" />
              <div className="ms-skeleton w-85" />
              <div className="ms-skeleton w-70" />
              <div className="ms-skeleton w-40" />
            </div>
          ) : feeds.length === 0 ? (
            <div className="ms-empty-state">
              No feeds configured. Add feeds in Settings.
            </div>
          ) : (
            <>
              {/* Priority section */}
              <section className={`ms-feed-section ms-priority-section ${priorityItems.length === 0 ? "collapsing" : ""}`}>
                <div className="ms-priority-inner">
                  <header className="ms-feed-header ms-priority-header">⚑ Needs Attention</header>
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
                <section className="ms-feed-section" key={`${feed.name}::${feed.feed_type}`}>
                  <header className="ms-feed-header">{feed.name}</header>

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
                </section>
              ))}
            </>
          )}
        </div>

        {/* Detail pane */}
        <DetailPane item={focusedItem} />
      </div>

      {/* Footer */}
      <footer className="ms-footer">
        <span className="ms-footer-hints">
          {refreshing ? (
            <>Refreshing{refreshProgress ? ` (${refreshProgress[0]}/${refreshProgress[1]})` : ""}…</>
          ) : (
            <><kbd>↑</kbd><kbd>↓</kbd> navigate · <kbd>↵</kbd> open · <kbd>r</kbd> refresh · <kbd>esc</kbd> close</>
          )}
        </span>
        <button
          className="ms-footer-settings"
          onClick={() => invoke("open_settings").catch(console.error)}
          title="Settings"
        >
          ⚙
        </button>
      </footer>
    </div>
  );
}

export default MainScreenApp;
