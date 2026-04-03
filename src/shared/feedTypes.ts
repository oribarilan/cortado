/// Canonical feed type identifier.
export type FeedType =
  | "github-pr"
  | "github-actions"
  | "ado-pr"
  | "http-health"
  | "copilot-session";

/// A single feed type within a catalog provider.
export type CatalogFeedType = {
  feedType: FeedType;
  name: string;
  description: string;
  icon: string;
  defaultInterval: string;
  popular?: boolean;
};

/// A provider that groups one or more feed types (e.g., "GitHub" has PR + Actions).
export type CatalogProvider = {
  id: string;
  name: string;
  icon: string;
  types: CatalogFeedType[];
};

/// All available feed types, grouped by provider.
export const FEED_CATALOG: CatalogProvider[] = [
  {
    id: "github",
    name: "GitHub",
    icon: `<svg width="26" height="26" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>`,
    types: [
      {
        feedType: "github-pr",
        name: "Pull Requests",
        description: "Track PRs with review status, checks, and mergeability",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>`,
        defaultInterval: "2m",
        popular: true,
      },
      {
        feedType: "github-actions",
        name: "Actions",
        description: "Monitor CI/CD workflow run status",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="10 8 16 12 10 16 10 8"/></svg>`,
        defaultInterval: "2m",
        popular: true,
      },
    ],
  },
  {
    id: "ado",
    name: "Azure DevOps",
    icon: `<svg width="26" height="26" viewBox="0 0 18 18" fill="currentColor"><path d="M17 4v10.97l-4 3.03V4.03L7 8.56v8.97L3.63 14.99A1 1 0 0 1 3 14.13V5.73c0-.31.14-.6.38-.79L10 0l7 4z"/></svg>`,
    types: [
      {
        feedType: "ado-pr",
        name: "Pull Requests",
        description: "Track PRs with review status and merge conflicts",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>`,
        defaultInterval: "2m",
      },
    ],
  },
  {
    id: "http",
    name: "HTTP",
    icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
    types: [
      {
        feedType: "http-health",
        name: "Health Check",
        description: "Monitor endpoint availability and response time",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
        defaultInterval: "1m",
        popular: true,
      },
    ],
  },
  {
    id: "coding-agents",
    name: "Coding Agents",
    icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/><polyline points="7 8 10 11 7 14"/><line x1="12" y1="14" x2="17" y2="14"/></svg>`,
    types: [
      {
        feedType: "copilot-session" as FeedType,
        name: "Copilot Sessions",
        description: "Track active GitHub Copilot CLI sessions",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2C6.48 2 2 6 2 10.5c0 2.49 1.13 4.71 3 6.24V20l3.5-2C9.62 18.32 10.78 18.5 12 18.5c5.52 0 10-3.98 10-8.5S17.52 2 12 2z"/><circle cx="8.5" cy="10.5" r="1.5"/><circle cx="15.5" cy="10.5" r="1.5"/></svg>`,
        defaultInterval: "30s",
      },
    ],
  },
];

/// Flat list of all feed types across all providers.
export const ALL_FEED_TYPES: CatalogFeedType[] = FEED_CATALOG.flatMap((p) => p.types);

/// Feed types marked as popular, for use in the empty state.
export const POPULAR_FEED_TYPES: CatalogFeedType[] = ALL_FEED_TYPES.filter((t) => t.popular);
