import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { PipelineVisibilityToggle } from "./PipelineVisibilityToggle";
import { pipelineFeedView, shouldShowFeed } from "./pipelineVisibility";
import type { Activity, FeedSnapshot, StatusKind } from "./types";

const NOW = Date.parse("2026-01-02T05:04:05Z");

function activity(id: string, kind: StatusKind = "idle", visible_until?: number): Activity {
  return {
    id, title: id, retained: false, visible_until,
    fields: [{ name: "status", label: "Status", value: { type: "status", value: id, kind } }],
  };
}

function feed(activities: Activity[], overrides: Partial<FeedSnapshot> = {}): FeedSnapshot {
  return {
    name: "Team CI", feed_type: "ado-pipelines", activities, error: null,
    hide_when_empty: false, last_refreshed: NOW, ...overrides,
  };
}

describe("pipeline visibility", () => {
  it("expires passing runs at the exact deadline without mutating the snapshot", () => {
    const original = feed([activity("passing", "idle", NOW)]);
    expect(pipelineFeedView(original, false, NOW - 1).activities).toHaveLength(1);
    const expired = pipelineFeedView(original, false, NOW);
    expect(expired.activities).toEqual([]);
    expect(expired.hiddenPipelineCount).toBe(1);
    expect(pipelineFeedView(original, false, NOW + 1).activities).toEqual([]);
    expect(original.activities).toHaveLength(1);
    expect(original.activities[0].retained).toBe(false);
  });

  it("keeps problems, active work, unknowns, and successes without a usable timestamp", () => {
    const original = feed([
      activity("failing", "attention-negative"),
      activity("partially succeeded", "attention-negative"),
      activity("cancelled", "attention-negative"),
      activity("queued", "waiting"),
      activity("running", "running"),
      activity("cancelling", "running"),
      activity("unknown"),
      activity("passing with missing finish time"),
      activity("recent success", "idle", NOW + 1),
    ]);
    expect(pipelineFeedView(original, false, NOW).activities).toEqual(original.activities);
  });

  it("hides immediate deadlines and never-run pipelines, even with fields hidden", () => {
    const original = feed([
      { ...activity("passing", "idle", 0), fields: [] },
      activity("not run", "idle", 0),
    ]);
    expect(pipelineFeedView(original, false, NOW).activities).toEqual([]);
    expect(pipelineFeedView(original, false, NOW).hiddenPipelineCount).toBe(2);
  });

  it("reveals the original order and restores filtering when collapsed", () => {
    const original = feed([
      activity("failing", "attention-negative"),
      activity("old success", "idle", NOW - 1),
      activity("not run", "idle", 0),
    ]);
    const all = pipelineFeedView(original, true, NOW);
    expect(all.activities).toBe(original.activities);
    expect(all.hiddenPipelineCount).toBe(0);
    expect(all.showingAllPipelines).toBe(true);
    expect(pipelineFeedView(original, false, NOW).activities.map((a) => a.id)).toEqual(["failing"]);
  });

  it("re-evaluates the same snapshot as time passes without a new poll", () => {
    const original = feed([activity("passing", "idle", NOW + 30_000)]);
    expect(pipelineFeedView(original, false, NOW).activities).toHaveLength(1);
    expect(pipelineFeedView(original, false, NOW + 30_000).activities).toHaveLength(0);
    expect(pipelineFeedView(original, true, NOW + 60_000).activities).toHaveLength(1);
  });

  it("a new active run on the same pipeline is visible again", () => {
    const original = feed([activity("pipeline-42", "idle", 0)]);
    expect(pipelineFeedView(original, false, NOW).activities).toHaveLength(0);
    const running = feed([activity("pipeline-42", "running")]);
    expect(pipelineFeedView(running, false, NOW).activities).toHaveLength(1);
  });

  it("does not filter any other feed type", () => {
    for (const feed_type of ["github-actions", "ado-pr", "http-health", "copilot-session", "app"]) {
      const original = feed([activity("item", "idle", 0)], { feed_type });
      for (const all of [false, true]) {
        const view = pipelineFeedView(original, all, NOW);
        expect(view.activities).toBe(original.activities);
        expect(view.hiddenPipelineCount).toBe(0);
        expect(view.showingAllPipelines).toBe(false);
      }
    }
  });

  it("keeps feed errors and disconnection metadata when all activities expire", () => {
    const original = feed([activity("passing", "idle", 0)], {
      error: "Authentication required", is_disconnected: true,
    });
    const view = pipelineFeedView(original, false, NOW);
    expect(view.error).toBe("Authentication required");
    expect(view.is_disconnected).toBe(true);
    expect(view.last_refreshed).toBe(NOW);
    expect(shouldShowFeed(view, true, false)).toBe(true);
  });

  it("keeps filtered feed headers reachable when empty feeds are disabled", () => {
    const view = pipelineFeedView(feed([activity("passing", "idle", 0)]), false, NOW);
    expect(shouldShowFeed(view, true, false)).toBe(true);
    const empty = pipelineFeedView(feed([]), false, NOW);
    expect(shouldShowFeed(empty, true, false)).toBe(false);
    expect(shouldShowFeed(empty, true, true)).toBe(true);
    expect(shouldShowFeed(empty, false, false)).toBe(true);
    expect(shouldShowFeed({ ...empty, error: "Bad config" }, true, false)).toBe(true);
    expect(shouldShowFeed({ ...empty, hide_when_empty: true }, false, true)).toBe(false);
  });

  it("keeps an expanded empty feed reachable until the user restores filtering", () => {
    const expanded = pipelineFeedView(feed([]), true, NOW);
    expect(shouldShowFeed(expanded, true, false)).toBe(true);
    const collapsed = pipelineFeedView(feed([]), false, NOW);
    expect(shouldShowFeed(collapsed, true, false)).toBe(false);
  });

  it("applies the same rules to retained activities without changing retention", () => {
    const retained = { ...activity("old success", "idle", 0), retained: true };
    const original = feed([retained]);
    expect(pipelineFeedView(original, false, NOW).activities).toEqual([]);
    expect(pipelineFeedView(original, true, NOW).activities[0]).toBe(retained);
    expect(retained.retained).toBe(true);
  });
});

describe("pipeline visibility disclosure", () => {
  it("makes the hidden count the only button even when every pipeline is hidden", () => {
    const view = pipelineFeedView(feed([
      activity("passing", "idle", 0), activity("not run", "idle", 0),
    ]), false, NOW);
    const html = renderToStaticMarkup(<PipelineVisibilityToggle feed={view} onToggle={() => {}} />);
    expect(html.match(/<button\b/g)).toHaveLength(1);
    expect(html).toContain('aria-label="2 hidden pipelines for Team CI; show all"');
    expect(html).toContain('aria-expanded="false"');
    expect(html).toContain('>2 hidden <span aria-hidden="true">▾</span></button>');
    expect(html).not.toContain("All pipelines");
    expect(html).not.toContain("pipeline-hidden-count");
  });

  it("uses a singular accessible name for one hidden pipeline", () => {
    const view = pipelineFeedView(feed([activity("passing", "idle", 0)]), false, NOW);
    const html = renderToStaticMarkup(<PipelineVisibilityToggle feed={view} onToggle={() => {}} />);
    expect(html).toContain('aria-label="1 hidden pipeline for Team CI; show all"');
    expect(html).toContain('>1 hidden <span aria-hidden="true">▾</span></button>');
  });

  it.each([
    [activity("not run", "idle", 0)],
    [activity("running", "running")],
    [],
  ])("keeps Show less available when expanded, including after live updates (%#)", (...activities) => {
    const view = pipelineFeedView(feed(activities), true, NOW);
    const html = renderToStaticMarkup(<PipelineVisibilityToggle feed={view} onToggle={() => {}} />);
    expect(html).toContain('aria-label="Show less for Team CI"');
    expect(html).toContain('aria-expanded="true"');
    expect(html).toContain('>Show less <span aria-hidden="true">▴</span></button>');
    expect(html).not.toContain("aria-pressed");
  });

  it.each([[], [activity("running", "running")], [activity("passing", "idle", NOW + 1)]])(
    "does not render a control when the normal view hides nothing (%#)", (...activities) => {
      const view = pipelineFeedView(feed(activities), false, NOW);
      expect(renderToStaticMarkup(<PipelineVisibilityToggle feed={view} onToggle={() => {}} />)).toBe("");
    },
  );

  it("does not render a control for other feed types", () => {
    for (const showAll of [false, true]) {
      const view = pipelineFeedView(feed([activity("item", "idle", 0)], { feed_type: "github-actions" }), showAll, NOW);
      expect(renderToStaticMarkup(<PipelineVisibilityToggle feed={view} onToggle={() => {}} />)).toBe("");
    }
  });
});
