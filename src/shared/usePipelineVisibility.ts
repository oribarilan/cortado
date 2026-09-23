import { useCallback, useEffect, useMemo, useState } from "react";
import { pipelineFeedView } from "./pipelineVisibility";
import type { FeedSnapshot } from "./types";

/** Shares the visibility filter and the existing relative-time refresh cadence. */
export function usePipelineVisibility(feeds: FeedSnapshot[]) {
  const [allPipelineFeeds, setAllPipelineFeeds] = useState<Set<string>>(() => new Set());
  const [clock, setClock] = useState(Date.now);
  const refreshVisibility = useCallback(() => setClock(Date.now()), []);

  useEffect(() => {
    const timer = setInterval(refreshVisibility, 30_000);
    return () => clearInterval(timer);
  }, [refreshVisibility]);

  const displayFeeds = useMemo(() => {
    const now = Date.now();
    return feeds.map((feed) => pipelineFeedView(feed, allPipelineFeeds.has(feed.name), now));
  }, [feeds, allPipelineFeeds, clock]);

  const toggleAllPipelines = useCallback((name: string) => {
    setAllPipelineFeeds((current) => {
      const next = new Set(current);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  }, []);

  return { displayFeeds, toggleAllPipelines, refreshVisibility };
}
