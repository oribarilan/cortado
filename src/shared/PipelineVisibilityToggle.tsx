import type { PipelineFeedView } from "./pipelineVisibility";
import "./pipelineVisibility.css";

/** Reveals the full pipeline snapshot without changing the saved feed config. */
export function PipelineVisibilityToggle({
  feed,
  onToggle,
}: {
  feed: PipelineFeedView;
  onToggle: (name: string) => void;
}) {
  if (feed.feed_type !== "ado-pipelines" || (!feed.showingAllPipelines && feed.hiddenPipelineCount === 0)) return null;

  const label = feed.showingAllPipelines ? "Show less" : `${feed.hiddenPipelineCount} hidden`;
  return (
    <button
      type="button"
      className="pipeline-visibility-toggle"
      aria-label={feed.showingAllPipelines
        ? `Show less for ${feed.name}`
        : `${label} ${feed.hiddenPipelineCount === 1 ? "pipeline" : "pipelines"} for ${feed.name}; show all`}
      aria-expanded={feed.showingAllPipelines}
      title={feed.showingAllPipelines
        ? "Hide older passing and never-run pipelines"
        : "Show every tracked pipeline"}
      onClick={() => onToggle(feed.name)}
    >
      {label} <span aria-hidden="true">{feed.showingAllPipelines ? "▴" : "▾"}</span>
    </button>
  );
}
