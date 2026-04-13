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
  /// When set to "user-filter", renders a segmented control with "All" / "Me" / "User"
  /// options instead of a plain text input. The "Me" option stores `meValue` in config;
  /// "All" stores an empty string; "User" shows a text input for a specific identity.
  kind?: "user-filter";
  /// The config value stored when the user selects "Me" (e.g., "@me" for GitHub, "me" for ADO).
  /// Ignored when `resolveMeCommand` is set.
  meValue?: string;
  /// Tauri command to resolve "Me" to a specific value (e.g., GitHub username).
  /// When set, clicking "Me" invokes this command and stores the resolved value.
  resolveMeCommand?: string;
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
  /// Pattern for generating a default feed name from type-specific field values.
  /// Use `{fieldKey}` placeholders that reference the `key` of a field in `fields`.
  defaultNamePattern?: string;
  /// Placeholder text for the feed name input (e.g., "my-org/repo Actions").
  namePlaceholder?: string;
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
        defaultNamePattern: "{repo} PRs",
        namePlaceholder: "my-org/repo PRs",
        fields: [
          { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
          { key: "user", label: "Author filter", placeholder: "octocat", hint: "GitHub username", mono: true, kind: "user-filter", meValue: "@me" },
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
        defaultNamePattern: "{repo} Actions",
        namePlaceholder: "my-org/repo Actions",
        fields: [
          { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
          { key: "branch", label: "Branch filter", placeholder: "main", hint: "Only runs on this branch (empty = all branches)", mono: true },
          { key: "workflow", label: "Workflow filter", placeholder: "ci.yml", hint: "Workflow filename (empty = all workflows)", mono: true },
          { key: "user", label: "Actor filter", placeholder: "octocat", hint: "GitHub username", mono: true, kind: "user-filter", resolveMeCommand: "resolve_github_username" },
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
        defaultNamePattern: "{url} PRs",
        namePlaceholder: "my-project/repo PRs",
        fields: [
          { key: "url", label: "Repository URL", placeholder: "https://dev.azure.com/org/project/_git/repo", hint: "Full URL to the Azure DevOps Git repository", mono: true, required: true },
          { key: "user", label: "Creator filter", placeholder: "user@org.com", hint: "Email address (display names may be ambiguous)", mono: true, kind: "user-filter", meValue: "me" },
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
        defaultNamePattern: "{url}",
        namePlaceholder: "api.example.com",
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
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M23.922 16.997C23.061 18.492 18.063 22.02 12 22.02 5.937 22.02.939 18.492.078 16.997A.641.641 0 0 1 0 16.741v-2.869a.883.883 0 0 1 .053-.22c.372-.935 1.347-2.292 2.605-2.656.167-.429.414-1.055.644-1.517a10.098 10.098 0 0 1-.052-1.086c0-1.331.282-2.499 1.132-3.368.397-.406.89-.717 1.474-.952C7.255 2.937 9.248 1.98 11.978 1.98c2.731 0 4.767.957 6.166 2.093.584.235 1.077.546 1.474.952.85.869 1.132 2.037 1.132 3.368 0 .368-.014.733-.052 1.086.23.462.477 1.088.644 1.517 1.258.364 2.233 1.721 2.605 2.656a.841.841 0 0 1 .053.22v2.869a.641.641 0 0 1-.078.256Zm-11.75-5.992h-.344a4.359 4.359 0 0 1-.355.508c-.77.947-1.918 1.492-3.508 1.492-1.725 0-2.989-.359-3.782-1.259a2.137 2.137 0 0 1-.085-.104L4 11.746v6.585c1.435.779 4.514 2.179 8 2.179 3.486 0 6.565-1.4 8-2.179v-6.585l-.098-.104s-.033.045-.085.104c-.793.9-2.057 1.259-3.782 1.259-1.59 0-2.738-.545-3.508-1.492a4.359 4.359 0 0 1-.355-.508Zm2.328 3.25c.549 0 1 .451 1 1v2c0 .549-.451 1-1 1-.549 0-1-.451-1-1v-2c0-.549.451-1 1-1Zm-5 0c.549 0 1 .451 1 1v2c0 .549-.451 1-1 1-.549 0-1-.451-1-1v-2c0-.549.451-1 1-1Zm3.313-6.185c.136 1.057.403 1.913.878 2.497.442.544 1.134.938 2.344.938 1.573 0 2.292-.337 2.657-.751.384-.435.558-1.15.558-2.361 0-1.14-.243-1.847-.705-2.319-.477-.488-1.319-.862-2.824-1.025-1.487-.161-2.192.138-2.533.529-.269.307-.437.808-.438 1.578v.021c0 .265.021.562.063.893Zm-1.626 0c.042-.331.063-.628.063-.894v-.02c-.001-.77-.169-1.271-.438-1.578-.341-.391-1.046-.69-2.533-.529-1.505.163-2.347.537-2.824 1.025-.462.472-.705 1.179-.705 2.319 0 1.211.175 1.926.558 2.361.365.414 1.084.751 2.657.751 1.21 0 1.902-.394 2.344-.938.475-.584.742-1.44.878-2.497Z"/></svg>`,
        defaultInterval: "30s",
        hideInterval: true,
        defaultNamePattern: "Copilot",
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
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" fill-rule="evenodd" xmlns="http://www.w3.org/2000/svg"><path d="M16 6H8v12h8V6zm4 16H4V2h16v20z"/></svg>`,
        defaultInterval: "30s",
        hideInterval: true,
        defaultNamePattern: "OpenCode",
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

/// Generate a default feed name by interpolating field values into the pattern.
/// For URL-based fields, extracts a readable name (hostname or path segments).
/// Returns null if the pattern has unfilled placeholders or no pattern is defined.
export function generateDefaultName(
  feedType: string,
  typeSpecific: Record<string, unknown>,
): string | null {
  const catalog = findFeedType(feedType);
  if (!catalog?.defaultNamePattern) return null;

  let result = catalog.defaultNamePattern;
  result = result.replace(/\{(\w+)\}/g, (match, key: string) => {
    const val = typeSpecific[key];
    if (!val || typeof val !== "string") return match;
    // For URL fields, extract a readable name
    if (key === "url") {
      try {
        const url = new URL(val);
        // ADO URLs like https://dev.azure.com/org/project/_git/repo
        const gitIdx = url.pathname.indexOf("/_git/");
        if (gitIdx !== -1) {
          const parts = url.pathname.substring(0, gitIdx).split("/").filter(Boolean);
          const repo = url.pathname.substring(gitIdx + 6).split("/")[0];
          return parts.length > 0 ? `${parts[parts.length - 1]}/${repo}` : repo;
        }
        return url.hostname;
      } catch {
        return val;
      }
    }
    return val;
  });

  // If any placeholders remain unfilled, return null
  if (/\{\w+\}/.test(result)) return null;
  return result;
}
