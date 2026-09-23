import type { FeedSnapshot } from "./types";

/** A presentation-only snapshot; the original snapshot stays intact. */
export type PipelineFeedView = FeedSnapshot & {
  hiddenPipelineCount: number;
  showingAllPipelines: boolean;
};

/** Applies ADO pipeline deadlines without changing lifecycle or other feed types. */
export function pipelineFeedView(
  feed: FeedSnapshot,
  showAll: boolean,
  now: number,
): PipelineFeedView {
  const showingAllPipelines = feed.feed_type === "ado-pipelines" && showAll;
  const activities = feed.feed_type !== "ado-pipelines" || showingAllPipelines
    ? feed.activities
    : feed.activities.filter((activity) =>
        activity.visible_until == null || now < activity.visible_until,
      );
  return {
    ...feed,
    activities,
    hiddenPipelineCount: feed.activities.length - activities.length,
    showingAllPipelines,
  };
}

/** Keeps filtered pipeline feeds reachable even when empty feeds are hidden. */
export function shouldShowFeed(
  feed: PipelineFeedView,
  seeded: boolean,
  showEmptyFeeds: boolean,
): boolean {
  if (feed.activities.length > 0 || feed.hiddenPipelineCount > 0 || feed.error) return true;
  if (feed.hide_when_empty) return false;
  return !seeded || showEmptyFeeds;
}
