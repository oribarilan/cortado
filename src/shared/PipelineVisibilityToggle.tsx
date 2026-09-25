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
  if (feed.feed_type !== "ado-pipelines") return null;
  return (
    <span className="pipeline-visibility">
      {feed.hiddenPipelineCount > 0 ? (
        <span className="pipeline-hidden-count">{feed.hiddenPipelineCount} hidden</span>
      ) : null}
      <button
        type="button"
        className="pipeline-visibility-toggle"
        aria-label={`All pipelines for ${feed.name}`}
        aria-pressed={feed.showingAllPipelines}
        title={feed.showingAllPipelines
          ? "Hide older passing and never-run pipelines"
          : "Show every tracked pipeline"}
        onClick={() => onToggle(feed.name)}
      >
        All pipelines
      </button>
    </span>
  );
}
