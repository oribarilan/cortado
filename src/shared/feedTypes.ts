/// Canonical feed type identifier.
export type FeedType =
  | "github-pr"
  | "github-actions"
  | "ado-pr"
  | "http-health"
  | "copilot-session"
  | "opencode-session";

/// A form field for the Settings feed edit form.
export type FeedTypeField = {
  key: string;
  label: string;
  placeholder: string;
  hint?: string;
  mono?: boolean;
  required?: boolean;
  sensitive?: boolean;
};

/// External CLI dependency required by a feed type.
export type FeedTypeDep = {
  binary: string;
  name: string;
  installUrl: string;
  authCommand?: string;
  extraSteps?: string[];
};

/// A custom validation rule for a type-specific field.
export type FeedTypeValidation = {
  field: string;
  check: (value: string) => string | null;
};

/// A setup prerequisite that must be satisfied before a feed can be saved.
export type FeedTypeSetup = {
  /** Human-readable label (e.g., "OpenCode plugin") */
  label: string;
  /** Description shown in the setup banner */
  description: string;
  /** Tauri command to check if setup is done. Returns { ready: boolean } */
  checkCommand: string;
  /** Tauri command to perform setup. Returns { success: boolean, error?: string } */
  installCommand: string;
  /** Button text (e.g., "Install Plugin") */
  installLabel: string;
  /** Tauri command to uninstall. Returns { success: boolean, error?: string } */
  uninstallCommand: string;
  /** Brief explanation shown in a help tooltip */
  helpText: string;
};

/// A single feed type within a catalog provider.
export type CatalogFeedType = {
  feedType: FeedType;
  /// Short name shown in catalog cards (e.g., "Pull Requests").
  name: string;
  /// Full display label for badges and headers (e.g., "GitHub PR").
  label: string;
  description: string;
  icon: string;
  defaultInterval: string;
  popular?: boolean;
  /// Form fields for the Settings edit form.
  fields: FeedTypeField[];
  /// External CLI dependency, if any.
  dependency?: FeedTypeDep;
  /// Type-specific validation rules beyond the generic required-field check.
  validations?: FeedTypeValidation[];
  /// Informational notes shown in the edit form footer.
  notes?: string[];
  /// A setup prerequisite (e.g., plugin installation) required before this feed can be used.
  setup?: FeedTypeSetup;
  /// If true, the interval field is hidden in the edit form (e.g., file-watching feeds).
  hideInterval?: boolean;
};

/// A provider that groups one or more feed types (e.g., "GitHub" has PR + Actions).
export type CatalogProvider = {
  id: string;
  name: string;
  icon: string;
  types: CatalogFeedType[];
};

const GH_DEP: FeedTypeDep = {
  binary: "gh",
  name: "GitHub CLI",
  installUrl: "https://cli.github.com",
  authCommand: "gh auth login",
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
        label: "GitHub PR",
        description: "Track PRs with review status, checks, and mergeability",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>`,
        defaultInterval: "2m",
        popular: true,
        fields: [
          { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
          { key: "user", label: "Author filter", placeholder: "@me", hint: "GitHub username or @me (default)", mono: true },
        ],
        dependency: GH_DEP,
      },
      {
        feedType: "github-actions",
        name: "Actions",
        label: "GitHub Actions",
        description: "Monitor CI/CD workflow run status",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="10 8 16 12 10 16 10 8"/></svg>`,
        defaultInterval: "2m",
        popular: true,
        fields: [
          { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
          { key: "branch", label: "Branch filter", placeholder: "main", hint: "Only runs on this branch", mono: true },
          { key: "workflow", label: "Workflow filter", placeholder: "ci.yml", hint: "Only this workflow file", mono: true },
          { key: "user", label: "Actor filter", placeholder: "@me", hint: "Only runs triggered by this user", mono: true },
        ],
        dependency: GH_DEP,
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
        label: "Azure DevOps PR",
        description: "Track PRs with review status and merge conflicts",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>`,
        defaultInterval: "2m",
        fields: [
          { key: "url", label: "Repository URL", placeholder: "https://dev.azure.com/org/project/_git/repo", hint: "Full URL to the Azure DevOps Git repository", mono: true, required: true },
          { key: "user", label: "Creator filter", placeholder: "me", hint: "User identity or 'me' (default)", mono: true },
        ],
        dependency: {
          binary: "az",
          name: "Azure CLI",
          installUrl: "https://learn.microsoft.com/en-us/cli/azure/install-azure-cli",
          authCommand: "az login",
          extraSteps: [
            "Add the extension: az extension add --name azure-devops",
            "Sign in: az login",
          ],
        },
        validations: [
          { field: "url", check: (v) => {
            if (v && !v.startsWith("https://")) return "Must be an https:// URL";
            if (v && !v.includes("/_git/")) return "URL must contain /_git/ (e.g., https://dev.azure.com/org/project/_git/repo)";
            return null;
          }},
        ],
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
        label: "HTTP Health Check",
        description: "Monitor endpoint availability and response time",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
        defaultInterval: "1m",
        popular: true,
        fields: [
          { key: "url", label: "URL", placeholder: "https://api.example.com/health", hint: "Endpoint to monitor", mono: true, required: true },
          { key: "method", label: "Method", placeholder: "GET", hint: "GET or HEAD (default: GET)", mono: true },
          { key: "expected_status", label: "Expected status", placeholder: "200", hint: "Expected HTTP status code (default: 200)", mono: true },
          { key: "timeout", label: "Timeout", placeholder: "10s", hint: "Request timeout (default: 10s)", mono: true },
        ],
        validations: [
          { field: "url", check: (v) => {
            if (v && !v.startsWith("http://") && !v.startsWith("https://")) return "Must be an http:// or https:// URL";
            return null;
          }},
        ],
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
        label: "Copilot Session",
        description: "Track active GitHub Copilot CLI sessions",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2C6.48 2 2 6 2 10.5c0 2.49 1.13 4.71 3 6.24V20l3.5-2C9.62 18.32 10.78 18.5 12 18.5c5.52 0 10-3.98 10-8.5S17.52 2 12 2z"/><circle cx="8.5" cy="10.5" r="1.5"/><circle cx="15.5" cy="10.5" r="1.5"/></svg>`,
        defaultInterval: "30s",
        hideInterval: true,
        fields: [],
        dependency: {
          binary: "copilot",
          name: "GitHub Copilot CLI",
          installUrl: "https://docs.github.com/en/copilot/using-github-copilot/using-github-copilot-in-the-command-line",
        },
        setup: {
          label: "Copilot CLI plugin",
          description: "The Cortado plugin must be installed in Copilot CLI to publish session state to Cortado.",
          checkCommand: "check_copilot_extension",
          installCommand: "install_copilot_extension",
          installLabel: "Install Plugin",
          uninstallCommand: "uninstall_copilot_extension",
          helpText: "Installs a small hook-based plugin into Copilot CLI that reports session status to Cortado. Safe to uninstall at any time -- Copilot CLI continues to work normally without it.",
        },
        notes: [
          "Sessions are detected via file changes in ~/.config/cortado/harness/ with near-instant updates.",
          "Shows one activity per working directory with repo, branch, and status.",
          "Opening an activity focuses the terminal -- exact tmux pane when available",
        ],
      },
      {
        feedType: "opencode-session",
        name: "OpenCode Sessions",
        label: "OpenCode Session",
        description: "Track active OpenCode coding sessions",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`,
        defaultInterval: "30s",
        hideInterval: true,
        fields: [],
        dependency: {
          binary: "opencode",
          name: "OpenCode",
          installUrl: "https://opencode.ai",
          authCommand: "opencode auth",
        },
        setup: {
          label: "OpenCode plugin",
          description: "The cortado-opencode plugin must be installed in OpenCode to publish session state to Cortado.",
          checkCommand: "check_opencode_plugin",
          installCommand: "install_opencode_plugin",
          installLabel: "Install Plugin",
          uninstallCommand: "uninstall_opencode_plugin",
          helpText: "Installs a small plugin into OpenCode that reports session status to Cortado. Safe to uninstall at any time -- OpenCode continues to work normally without it.",
        },
        notes: [
          "Sessions are detected via file changes in ~/.config/cortado/harness/ with near-instant updates.",
          "Shows one activity per working directory with repo, branch, and status.",
          "Opening an activity focuses the terminal -- exact tmux pane when available",
        ],
      },
    ],
  },
];

/// Flat list of all feed types across all providers.
export const ALL_FEED_TYPES: CatalogFeedType[] = FEED_CATALOG.flatMap((p) => p.types);

/// Feed types marked as popular, for use in the empty state.
export const POPULAR_FEED_TYPES: CatalogFeedType[] = ALL_FEED_TYPES.filter((t) => t.popular);

/// Look up a feed type's catalog entry by its feedType string.
export function findFeedType(feedType: string): CatalogFeedType | undefined {
  return ALL_FEED_TYPES.find((t) => t.feedType === feedType);
}
