import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import { useAppearance } from "../shared/useAppearance";

type StatusKindKey = "attention-negative" | "attention-positive" | "waiting" | "running" | "idle";

type NotificationSettings = {
  enabled: boolean;
  mode: string;
  kinds?: StatusKindKey[];
  delivery: string;
  notify_new_activities: boolean;
  notify_removed_activities: boolean;
};

type GeneralSettings = {
  theme: string;
  text_size: string;
  show_menubar: boolean;
  global_hotkey: string;
};

type PanelSettings = {
  show_priority_section: boolean;
  hide_empty_feeds: boolean;
};

type AppSettings = {
  general: GeneralSettings;
  panel: PanelSettings;
  notifications: NotificationSettings;
  focus: { tmux_enabled: boolean; accessibility_enabled: boolean };
};

type FieldOverrideDto = {
  visible?: boolean;
  label?: string;
};

type FeedConfigDto = {
  name: string;
  type: string;
  interval?: string;
  retain?: string;
  notify?: boolean;
  type_specific: Record<string, unknown>;
  fields: Record<string, FieldOverrideDto>;
};

type FeedType = "github-pr" | "github-actions" | "ado-pr" | "http-health" | "shell" | "copilot-session";

const FEED_TYPE_LABELS: Record<FeedType, string> = {
  "github-pr": "GitHub PR",
  "github-actions": "GitHub Actions",
  "ado-pr": "Azure DevOps PR",
  "http-health": "HTTP Health Check",
  "shell": "Shell",
  "copilot-session": "Copilot Session",
};

const FEED_TYPE_FIELDS: Record<FeedType, { key: string; label: string; placeholder: string; hint?: string; mono?: boolean; required?: boolean; sensitive?: boolean }[]> = {
  "github-pr": [
    { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
    { key: "user", label: "Author filter", placeholder: "@me", hint: "GitHub username or @me (default)", mono: true },
  ],
  "github-actions": [
    { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
    { key: "branch", label: "Branch filter", placeholder: "main", hint: "Only runs on this branch", mono: true },
    { key: "workflow", label: "Workflow filter", placeholder: "ci.yml", hint: "Only this workflow file", mono: true },
    { key: "user", label: "Actor filter", placeholder: "@me", hint: "Only runs triggered by this user", mono: true },
  ],
  "ado-pr": [
    { key: "url", label: "Repository URL", placeholder: "https://dev.azure.com/org/project/_git/repo", hint: "Full URL to the Azure DevOps Git repository", mono: true, required: true },
    { key: "user", label: "Creator filter", placeholder: "me", hint: "User identity or 'me' (default)", mono: true },
  ],
  "http-health": [
    { key: "url", label: "URL", placeholder: "https://api.example.com/health", hint: "Endpoint to monitor", mono: true, required: true },
    { key: "method", label: "Method", placeholder: "GET", hint: "GET or HEAD (default: GET)", mono: true },
    { key: "expected_status", label: "Expected status", placeholder: "200", hint: "Expected HTTP status code (default: 200)", mono: true },
    { key: "timeout", label: "Timeout", placeholder: "10s", hint: "Request timeout (default: 10s)", mono: true },
  ],
  "shell": [
    { key: "command", label: "Command", placeholder: "df -h /", hint: "Executed via sh -c", mono: true, required: true },
    { key: "field_name", label: "Field name", placeholder: "output", hint: "Name for the output field (default: output)", mono: true },
    { key: "field_type", label: "Field type", placeholder: "text", hint: "text, status, number, or url (default: text)", mono: true },
  ],
  "copilot-session": [],
};

function emptyFeed(feedType: FeedType, interval?: string): FeedConfigDto {
  return {
    name: "",
    type: feedType,
    interval: interval ?? "5m",
    type_specific: {},
    fields: {},
  };
}

type DepInfo = {
  binary: string;
  name: string;
  installUrl: string;
  authCommand: string;
  extraSteps?: string[];
};

const FEED_TYPE_DEPS: Partial<Record<FeedType, DepInfo>> = {
  "github-pr": {
    binary: "gh",
    name: "GitHub CLI",
    installUrl: "https://cli.github.com",
    authCommand: "gh auth login",
  },
  "github-actions": {
    binary: "gh",
    name: "GitHub CLI",
    installUrl: "https://cli.github.com",
    authCommand: "gh auth login",
  },
  "ado-pr": {
    binary: "az",
    name: "Azure CLI",
    installUrl: "https://learn.microsoft.com/en-us/cli/azure/install-azure-cli",
    authCommand: "az login",
    extraSteps: [
      "Add the extension: az extension add --name azure-devops",
      "Sign in: az login",
    ],
  },
};

type CatalogFeedType = {
  feedType: FeedType;
  name: string;
  description: string;
  icon: string;
  defaultInterval: string;
};

type CatalogProvider = {
  id: string;
  name: string;
  icon: string;
  types: CatalogFeedType[];
};

const FEED_CATALOG: CatalogProvider[] = [
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
      },
      {
        feedType: "github-actions",
        name: "Actions",
        description: "Monitor CI/CD workflow run status",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="10 8 16 12 10 16 10 8"/></svg>`,
        defaultInterval: "2m",
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
      },
    ],
  },
  {
    id: "shell",
    name: "Shell",
    icon: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
    types: [
      {
        feedType: "shell",
        name: "Custom Command",
        description: "Run any shell command and track its output",
        icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
        defaultInterval: "30s",
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

type TestFeedResult = {
  success: boolean;
  error: string | null;
  activities: { title: string; status: string | null }[];
};

type DurationUnit = "s" | "m" | "h";

const DURATION_UNIT_LABELS: Record<DurationUnit, string> = {
  s: "seconds",
  m: "minutes",
  h: "hours",
};

function parseDurationString(raw: string | undefined): { value: number; unit: DurationUnit } {
  if (!raw) return { value: 5, unit: "m" };
  const match = raw.match(/^(\d+)\s*(s|m|h)$/);
  if (!match) return { value: 5, unit: "m" };
  return { value: parseInt(match[1], 10), unit: match[2] as DurationUnit };
}

function toDurationString(value: number, unit: DurationUnit): string {
  return `${value}${unit}`;
}

function DurationInput({
  value,
  onChange,
  placeholder,
}: {
  value: string | undefined;
  onChange: (val: string | undefined) => void;
  placeholder?: string;
}) {
  const parsed = useMemo(() => parseDurationString(value), [value]);

  return (
    <div className="duration-input">
      <input
        className="duration-number"
        type="number"
        min={1}
        value={value ? parsed.value : ""}
        onChange={(e) => {
          const num = parseInt(e.target.value, 10);
          if (!e.target.value) {
            onChange(undefined);
          } else if (num > 0) {
            onChange(toDurationString(num, parsed.unit));
          }
        }}
        placeholder={placeholder ?? "—"}
      />
      <select
        className="duration-unit"
        value={parsed.unit}
        onChange={(e) => {
          const unit = e.target.value as DurationUnit;
          if (value) {
            onChange(toDurationString(parsed.value, unit));
          }
        }}
      >
        {Object.entries(DURATION_UNIT_LABELS).map(([u, label]) => (
          <option key={u} value={u}>{label}</option>
        ))}
      </select>
    </div>
  );
}

function validateFeed(feed: FeedConfigDto): Record<string, string> {
  const errors: Record<string, string> = {};

  if (!feed.name.trim()) {
    errors.name = "Feed name is required";
  }

  if (!feed.interval) {
    errors.interval = "Poll interval is required";
  }

  const typeFields = FEED_TYPE_FIELDS[feed.type as FeedType] ?? [];
  for (const field of typeFields) {
    if (!field.required) continue;
    const val = String(feed.type_specific[field.key] ?? "").trim();
    if (!val) {
      errors[field.key] = `${field.label} is required`;
    }
  }

  // Type-specific validations
  if (feed.type === "ado-pr") {
    const url = String(feed.type_specific.url ?? "").trim();
    if (url && !url.startsWith("https://")) {
      errors.url = "Must be an https:// URL";
    } else if (url && !url.includes("/_git/")) {
      errors.url = "URL must contain /_git/ (e.g., https://dev.azure.com/org/project/_git/repo)";
    }
  }

  if (feed.type === "shell") {
    const fieldType = String(feed.type_specific.field_type ?? "").trim();
    if (fieldType && !["text", "status", "number", "url"].includes(fieldType)) {
      errors.field_type = "Must be text, status, number, or url";
    }
  }

  if (feed.type === "http-health") {
    const url = String(feed.type_specific.url ?? "").trim();
    if (url && !url.startsWith("http://") && !url.startsWith("https://")) {
      errors.url = "Must be an http:// or https:// URL";
    }
  }

  return errors;
}

/** Modifier key names in JS KeyboardEvent. */
const MODIFIER_KEYS = new Set(["Shift", "Meta", "Alt", "Control"]);

/** Maps a JS KeyboardEvent.code to a short display label. */
function codeToDisplayKey(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code === "Space") return "Space";
  if (code === "Minus") return "-";
  if (code === "Equal") return "=";
  if (code === "BracketLeft") return "[";
  if (code === "BracketRight") return "]";
  if (code === "Backslash") return "\\";
  if (code === "Semicolon") return ";";
  if (code === "Quote") return "'";
  if (code === "Comma") return ",";
  if (code === "Period") return ".";
  if (code === "Slash") return "/";
  if (code === "Backquote") return "`";
  return code;
}

/** Converts a Tauri shortcut string (e.g. "super+shift+Space") to macOS display symbols. */
function formatShortcut(shortcut: string): string {
  const parts = shortcut.split("+");
  const symbols: string[] = [];
  let key = "";
  for (const part of parts) {
    const upper = part.toUpperCase();
    if (upper === "SUPER" || upper === "CMD" || upper === "COMMAND") symbols.push("\u2318");
    else if (upper === "CONTROL" || upper === "CTRL") symbols.push("\u2303");
    else if (upper === "ALT" || upper === "OPTION") symbols.push("\u2325");
    else if (upper === "SHIFT") symbols.push("\u21E7");
    else key = codeToDisplayKey(part);
  }
  return symbols.join("") + key;
}

/** Converts a JS KeyboardEvent to a Tauri shortcut string, or null if invalid. */
function keyEventToShortcut(e: KeyboardEvent): string | null {
  // Ignore modifier-only presses.
  if (MODIFIER_KEYS.has(e.key)) return null;
  // Require at least one modifier.
  if (!e.metaKey && !e.altKey && !e.ctrlKey && !e.shiftKey) return null;

  const parts: string[] = [];
  if (e.metaKey) parts.push("super");
  if (e.ctrlKey) parts.push("control");
  if (e.altKey) parts.push("alt");
  if (e.shiftKey) parts.push("shift");
  parts.push(e.code);
  return parts.join("+");
}

function SettingsApp() {
  useAppearance();
  const [section, setSection] = useState<"general" | "notifications" | "feeds" | "focus">("general");
  const [sectionFading, setSectionFading] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [autostartLoading, setAutostartLoading] = useState(true);

  // Animation timeout cleanup
  const animTimers = useRef<ReturnType<typeof setTimeout>[]>([]);
  const scheduleAnim = useCallback((fn: () => void, ms: number) => {
    const id = setTimeout(fn, ms);
    animTimers.current.push(id);
    return id;
  }, []);
  useEffect(() => () => { animTimers.current.forEach(clearTimeout); }, []);

  // General settings state
  const [showMenubar, setShowMenubar] = useState(true);
  const [showPrioritySection, setShowPrioritySection] = useState(true);
  const [hideEmptyFeeds, setHideEmptyFeeds] = useState(false);
  const [theme, setTheme] = useState("system");
  const [textSize, setTextSize] = useState("m");
  const [globalHotkey, setGlobalHotkey] = useState("super+shift+space");
  const [hotkeyRecording, setHotkeyRecording] = useState(false);
  const [hotkeyError, setHotkeyError] = useState<string | null>(null);

  // Notification settings state
  const [notifSettings, setNotifSettings] = useState<NotificationSettings>({
    enabled: true,
    mode: "all",
    delivery: "grouped",
    notify_new_activities: true,
    notify_removed_activities: true,
  });
  const [notifLoading, setNotifLoading] = useState(true);
  const [notifSaveError, setNotifSaveError] = useState<string | null>(null);
  const [notifPermission, setNotifPermission] = useState<boolean | null>(null);

  // Toast state
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [toastVisible, setToastVisible] = useState(false);

  // Focus capabilities state
  type FocusCaps = {
    has_active_session: boolean;
    tmux_installed: boolean;
    tmux_detected: boolean;
    terminal_app: string | null;
    terminal_scriptable: boolean;
    ghostty_scriptable: boolean;
    ghostty_version: string | null;
    accessibility_permitted: boolean;
  };
  const [focusCaps, setFocusCaps] = useState<FocusCaps | null>(null);
  const [focusLoading, setFocusLoading] = useState(false);
  const [tmuxEnabled, setTmuxEnabled] = useState(true);
  const [accessibilityEnabled, setAccessibilityEnabled] = useState(false);
  const [showTmuxHelp, setShowTmuxHelp] = useState(false);
  const [showGhosttyHelp, setShowGhosttyHelp] = useState(false);
  const [showAccessibilityHelp, setShowAccessibilityHelp] = useState(false);

  const showToast = useCallback(() => {
    if (toastTimer.current) clearTimeout(toastTimer.current);
    setToastVisible(true);
    toastTimer.current = setTimeout(() => setToastVisible(false), 1500);
  }, []);

  useEffect(() => () => { if (toastTimer.current) clearTimeout(toastTimer.current); }, []);

  // Load focus capabilities when focus section is viewed.
  useEffect(() => {
    if (section !== "focus") return;
    setFocusLoading(true);
    invoke<FocusCaps>("get_focus_capabilities")
      .then(setFocusCaps)
      .catch(console.error)
      .finally(() => setFocusLoading(false));
  }, [section]);

  // Feeds state
  const [feeds, setFeeds] = useState<FeedConfigDto[]>([]);
  const [feedsLoading, setFeedsLoading] = useState(true);
  const [configPath, setConfigPath] = useState("");
  const [settingsPath, setSettingsPath] = useState("");

  // Edit state
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editingFeed, setEditingFeed] = useState<FeedConfigDto | null>(null);
  const [isNewFeed, setIsNewFeed] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [revealedTokens, setRevealedTokens] = useState<Set<string>>(new Set());
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [feedNavTransition, setFeedNavTransition] = useState<"idle" | "drill-in" | "drill-out">("idle");

  // Catalog state (for the new feed flow)
  const [catalogStep, setCatalogStep] = useState<"hidden" | "providers" | "types">("hidden");
  const [catalogProvider, setCatalogProvider] = useState<CatalogProvider | null>(null);

  // Dependency + test state
  const [depInstalled, setDepInstalled] = useState<boolean | null>(null);
  const [testResult, setTestResult] = useState<TestFeedResult | null>(null);
  const [testLoading, setTestLoading] = useState(false);
  const [testPreviewOpen, setTestPreviewOpen] = useState(false);

  useEffect(() => {
    isEnabled()
      .then(setAutostart)
      .catch(() => setAutostart(false))
      .finally(() => setAutostartLoading(false));

    invoke<AppSettings>("get_settings")
      .then((s) => {
        setNotifSettings(s.notifications);
        setShowMenubar(s.general?.show_menubar ?? true);
        setShowPrioritySection(s.panel?.show_priority_section ?? true);
        setHideEmptyFeeds(s.panel?.hide_empty_feeds ?? false);
        setTheme(s.general?.theme ?? "system");
        setTextSize(s.general?.text_size ?? "m");
        setGlobalHotkey(s.general?.global_hotkey ?? "super+shift+space");
        setTmuxEnabled(s.focus?.tmux_enabled ?? true);
        setAccessibilityEnabled(s.focus?.accessibility_enabled ?? false);
      })
      .catch(() => {})
      .finally(() => setNotifLoading(false));

    isPermissionGranted()
      .then(setNotifPermission)
      .catch(() => setNotifPermission(null));
  }, []);

  const loadFeeds = useCallback(async () => {
    try {
      const result = await invoke<FeedConfigDto[]>("get_feeds_config");
      setFeeds(result);
    } catch (err) {
      console.error("failed loading feeds config:", err);
    } finally {
      setFeedsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadFeeds();
    invoke<string>("get_config_path")
      .then(setConfigPath)
      .catch(() => {});
    invoke<string>("get_settings_path")
      .then(setSettingsPath)
      .catch(() => {});
  }, [loadFeeds]);

  const toggleAutostart = useCallback(async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
    } catch (err) {
      console.error("autostart toggle failed:", err);
    }
  }, [autostart]);

  const saveNotifSettings = useCallback(async (updated: NotificationSettings) => {
    setNotifSettings(updated);
    setNotifSaveError(null);
    try {
      await invoke("save_settings", {
        settings: {
          general: { show_menubar: showMenubar, theme, text_size: textSize, global_hotkey: globalHotkey },
          panel: { show_priority_section: showPrioritySection, hide_empty_feeds: hideEmptyFeeds },
          notifications: updated,
          focus: { tmux_enabled: tmuxEnabled, accessibility_enabled: accessibilityEnabled },
        },
      });
      showToast();
    } catch (err) {
      setNotifSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [showPrioritySection, hideEmptyFeeds, showMenubar, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveGeneralSetting = useCallback(async (updates: { showMenubar?: boolean; showPrioritySection?: boolean; hideEmptyFeeds?: boolean; theme?: string; textSize?: string }) => {
    const newMenubar = updates.showMenubar ?? showMenubar;
    const newPriority = updates.showPrioritySection ?? showPrioritySection;
    const newHideEmpty = updates.hideEmptyFeeds ?? hideEmptyFeeds;
    const newTheme = updates.theme ?? theme;
    const newTextSize = updates.textSize ?? textSize;

    if (updates.showMenubar !== undefined) setShowMenubar(newMenubar);
    if (updates.showPrioritySection !== undefined) setShowPrioritySection(newPriority);
    if (updates.hideEmptyFeeds !== undefined) setHideEmptyFeeds(newHideEmpty);
    if (updates.theme !== undefined) setTheme(newTheme);
    if (updates.textSize !== undefined) setTextSize(newTextSize);

    try {
      await invoke("save_settings", {
        settings: {
          general: { show_menubar: newMenubar, theme: newTheme, text_size: newTextSize, global_hotkey: globalHotkey },
          panel: { show_priority_section: newPriority, hide_empty_feeds: newHideEmpty },
          notifications: notifSettings,
          focus: { tmux_enabled: tmuxEnabled, accessibility_enabled: accessibilityEnabled },
        },
      });
      showToast();
    } catch (err) {
      console.error("failed saving general setting:", err);
    }
  }, [notifSettings, showMenubar, showPrioritySection, hideEmptyFeeds, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveFocusSetting = useCallback(async (updates: { tmuxEnabled?: boolean; accessibilityEnabled?: boolean }) => {
    const newTmux = updates.tmuxEnabled ?? tmuxEnabled;
    const newAccessibility = updates.accessibilityEnabled ?? accessibilityEnabled;

    if (updates.tmuxEnabled !== undefined) setTmuxEnabled(newTmux);
    if (updates.accessibilityEnabled !== undefined) setAccessibilityEnabled(newAccessibility);

    try {
      await invoke("save_settings", {
        settings: {
          general: { show_menubar: showMenubar, theme, text_size: textSize, global_hotkey: globalHotkey },
          panel: { show_priority_section: showPrioritySection, hide_empty_feeds: hideEmptyFeeds },
          notifications: notifSettings,
          focus: { tmux_enabled: newTmux, accessibility_enabled: newAccessibility },
        },
      });
      showToast();
    } catch (err) {
      console.error("failed saving focus setting:", err);
    }
  }, [notifSettings, showMenubar, showPrioritySection, hideEmptyFeeds, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveHotkey = useCallback(async (hotkey: string) => {
    setHotkeyError(null);
    try {
      await invoke("set_global_hotkey", { hotkey });
      setGlobalHotkey(hotkey);
      showToast();
    } catch (err) {
      setHotkeyError(err instanceof Error ? err.message : String(err));
    }
  }, [showToast]);

  // Recording mode: capture next key combo
  useEffect(() => {
    if (!hotkeyRecording) return;

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        setHotkeyRecording(false);
        return;
      }

      const shortcut = keyEventToShortcut(e);
      if (shortcut) {
        setHotkeyRecording(false);
        void saveHotkey(shortcut);
      }
    };

    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [hotkeyRecording, saveHotkey]);

  const handleRequestPermission = useCallback(async () => {
    try {
      const result = await requestPermission();
      setNotifPermission(result === "granted");
    } catch (err) {
      console.error("permission request failed:", err);
    }
  }, []);

  const [testNotifError, setTestNotifError] = useState<string | null>(null);
  const [resetConfirm, setResetConfirm] = useState<"general" | "notifications" | null>(null);
  const [modalExiting, setModalExiting] = useState(false);

  const closeModal = useCallback(() => {
    setModalExiting(true);
    scheduleAnim(() => {
      setResetConfirm(null);
      setModalExiting(false);
    }, 135); // ~75% of --duration-normal (180ms)
  }, [scheduleAnim]);

  const handleTestNotification = useCallback(async () => {
    setTestNotifError(null);
    try {
      await invoke("send_test_notification");
    } catch (err) {
      setTestNotifError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const startEdit = useCallback((index: number) => {
    setEditingIndex(index);
    setEditingFeed(structuredClone(feeds[index]));
    setIsNewFeed(false);
    setSaveError(null);
    setSaveSuccess(false);
    setRevealedTokens(new Set());
    setDeleteConfirm(false);
    setFieldErrors({});
    setTestResult(null);
    setTestLoading(false);
    setTestPreviewOpen(false);
    setFeedNavTransition("drill-in");
    scheduleAnim(() => setFeedNavTransition("idle"), 180);
    invoke<{ installed: boolean }>("check_feed_dependency", { feedType: feeds[index].type })
      .then((r) => setDepInstalled(r.installed))
      .catch(() => setDepInstalled(null));
  }, [feeds, scheduleAnim]);

  const startAdd = useCallback(() => {
    setCatalogStep("providers");
    setCatalogProvider(null);
    setEditingIndex(null);
    setEditingFeed(null);
    setIsNewFeed(false);
    setSaveError(null);
    setSaveSuccess(false);
    setFeedNavTransition("drill-in");
    scheduleAnim(() => setFeedNavTransition("idle"), 180);
  }, [scheduleAnim]);

  const selectFeedType = useCallback((catalogType: CatalogFeedType) => {
    const feedType = catalogType.feedType;
    setCatalogStep("hidden");
    setCatalogProvider(null);
    setEditingIndex(feeds.length);
    setEditingFeed(emptyFeed(feedType, catalogType.defaultInterval));
    setIsNewFeed(true);
    setSaveError(null);
    setSaveSuccess(false);
    setRevealedTokens(new Set());
    setDeleteConfirm(false);
    setFieldErrors({});
    setTestResult(null);
    setTestLoading(false);
    setTestPreviewOpen(false);
    setFeedNavTransition("drill-in");
    scheduleAnim(() => setFeedNavTransition("idle"), 180);

    const depInfoForType = FEED_TYPE_DEPS[feedType];
    if (depInfoForType) {
      invoke<{ installed: boolean }>("check_feed_dependency", { feedType })
        .then((r) => setDepInstalled(r.installed))
        .catch(() => setDepInstalled(null));
    } else {
      setDepInstalled(null);
    }
  }, [feeds.length, scheduleAnim]);

  const selectProvider = useCallback((provider: CatalogProvider) => {
    if (provider.types.length === 1) {
      selectFeedType(provider.types[0]);
      return;
    }
    setCatalogProvider(provider);
    setCatalogStep("types");
    setFeedNavTransition("drill-in");
    scheduleAnim(() => setFeedNavTransition("idle"), 180);
  }, [selectFeedType, scheduleAnim]);

  const catalogBack = useCallback(() => {
    if (catalogStep === "types") {
      setCatalogStep("providers");
      setCatalogProvider(null);
      setFeedNavTransition("drill-out");
      scheduleAnim(() => setFeedNavTransition("idle"), 180);
    } else {
      setCatalogStep("hidden");
      setFeedNavTransition("drill-out");
      scheduleAnim(() => setFeedNavTransition("idle"), 180);
    }
  }, [catalogStep, scheduleAnim]);

  const cancelEdit = useCallback(() => {
    setCatalogStep("hidden");
    setCatalogProvider(null);
    if (editingFeed === null) {
      setEditingIndex(null);
      setEditingFeed(null);
      setSaveError(null);
      setSaveSuccess(false);
      return;
    }
    setEditingIndex(null);
    setEditingFeed(null);
    setSaveError(null);
    setSaveSuccess(false);
    setFeedNavTransition("drill-out");
    scheduleAnim(() => setFeedNavTransition("idle"), 180);
  }, [editingFeed, scheduleAnim]);

  const saveFeed = useCallback(async () => {
    if (!editingFeed || editingIndex === null) return;
    setSaveError(null);
    setSaveSuccess(false);

    const errors = validateFeed(editingFeed);
    setFieldErrors(errors);
    if (Object.keys(errors).length > 0) return;

    const updatedFeeds = [...feeds];
    if (isNewFeed) {
      updatedFeeds.push(editingFeed);
    } else {
      updatedFeeds[editingIndex] = editingFeed;
    }

    try {
      await invoke("save_feeds_config", { feeds: updatedFeeds });
      setFeeds(updatedFeeds);
      setSaveSuccess(true);
      setSaveError(null);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [editingFeed, editingIndex, feeds, isNewFeed]);

  const deleteFeed = useCallback(async () => {
    if (editingIndex === null || isNewFeed) return;
    const updatedFeeds = feeds.filter((_, i) => i !== editingIndex);
    try {
      await invoke("save_feeds_config", { feeds: updatedFeeds });
      setFeeds(updatedFeeds);
      setEditingIndex(null);
      setEditingFeed(null);
      setSaveSuccess(false);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [editingIndex, feeds, isNewFeed]);

  const updateField = useCallback((key: string, value: string) => {
    if (!editingFeed) return;
    setSaveSuccess(false);
    setFieldErrors((prev) => {
      if (!prev[key]) return prev;
      const next = { ...prev };
      delete next[key];
      return next;
    });

    if (key === "name") {
      setEditingFeed({ ...editingFeed, name: value });
    } else if (key === "type") {
      setEditingFeed({ ...editingFeed, type: value, type_specific: {} });
      setTestResult(null);
      invoke<{ installed: boolean }>("check_feed_dependency", { feedType: value })
        .then((r) => setDepInstalled(r.installed))
        .catch(() => setDepInstalled(null));
    } else if (key === "interval") {
      setEditingFeed({ ...editingFeed, interval: value || undefined });
    } else if (key === "retain") {
      setEditingFeed({ ...editingFeed, retain: value || undefined });
    } else {
      setEditingFeed({
        ...editingFeed,
        type_specific: { ...editingFeed.type_specific, [key]: value },
      });
    }
  }, [editingFeed]);

  const toggleTokenReveal = useCallback((key: string) => {
    setRevealedTokens(prev => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  }, []);

  const runTest = useCallback(async () => {
    if (!editingFeed) return;
    setTestLoading(true);
    setTestResult(null);
    setTestPreviewOpen(false);
    try {
      const result = await invoke<TestFeedResult>("test_feed", { feedDto: editingFeed });
      setTestResult(result);
    } catch (err) {
      setTestResult({
        success: false,
        error: err instanceof Error ? err.message : String(err),
        activities: [],
      });
    } finally {
      setTestLoading(false);
    }
  }, [editingFeed]);

  const switchSection = useCallback((next: "general" | "notifications" | "feeds" | "focus") => {
    if (next === section || sectionFading) return;
    cancelEdit();
    setSectionFading(true);
    scheduleAnim(() => {
      setSection(next);
      setSectionFading(false);
    }, 110); // ~60% of --duration-normal
  }, [section, sectionFading, cancelEdit, scheduleAnim]);

  const feedTypeFields = editingFeed ? FEED_TYPE_FIELDS[editingFeed.type as FeedType] ?? [] : [];
  const depInfo = editingFeed ? FEED_TYPE_DEPS[editingFeed.type as FeedType] : undefined;

  return (
    <div className="settings-root">
      <nav className="settings-sidebar">
        <div
          className={`settings-nav ${section === "general" ? "active" : ""}`}
          onClick={() => switchSection("general")}
        >
          <span className="settings-nav-icon">⚙</span> General
        </div>
        <div
          className={`settings-nav ${section === "feeds" ? "active" : ""}`}
          onClick={() => switchSection("feeds")}
        >
          <span className="settings-nav-icon">◉</span> Feeds
        </div>
        <div
          className={`settings-nav ${section === "notifications" ? "active" : ""}`}
          onClick={() => switchSection("notifications")}
        >
          <span className="settings-nav-icon">♪</span> Notifications
        </div>
        <div
          className={`settings-nav ${section === "focus" ? "active" : ""}`}
          onClick={() => switchSection("focus")}
        >
          <span className="settings-nav-icon">▸</span> Agents
        </div>
      </nav>
      <main className={`settings-main ${sectionFading ? "fading" : ""}`}>
        {section === "general" ? (
          <>
            <h2 className="settings-title">General</h2>

            <div className="section-header">Appearance</div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Theme</div>
              </div>
              <div className="segmented-control">
                {(["light", "dark", "system"] as const).map((opt) => (
                  <button
                    key={opt}
                    className={`segmented-option ${theme === opt ? "active" : ""}`}
                    onClick={() => { void saveGeneralSetting({ theme: opt }); }}
                  >
                    {opt.charAt(0).toUpperCase() + opt.slice(1)}
                  </button>
                ))}
              </div>
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Text size</div>
              </div>
              <div className="segmented-control">
                {(["xs", "s", "m", "l", "xl"] as const).map((opt) => (
                  <button
                    key={opt}
                    className={`segmented-option ${textSize === opt ? "active" : ""}`}
                    onClick={() => { void saveGeneralSetting({ textSize: opt }); }}
                  >
                    {opt.toUpperCase()}
                  </button>
                ))}
              </div>
            </div>

            <div className="section-header">Behavior</div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Start on system startup</div>
                <div className="setting-hint">Launch Cortado when you log in</div>
              </div>
              <button
                className={`toggle ${autostart ? "on" : ""}`}
                onClick={() => { void toggleAutostart(); }}
                disabled={autostartLoading}
                aria-pressed={autostart}
                aria-label="Start on system startup"
              />
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Show tray icon</div>
                <div className="setting-hint">Show tray icon and tray menu. When off, use the global shortcut or Spotlight to access Cortado.</div>
              </div>
              <button
                className={`toggle ${showMenubar ? "on" : ""}`}
                onClick={() => { void saveGeneralSetting({ showMenubar: !showMenubar }); }}
                aria-pressed={showMenubar}
                aria-label="Show tray icon"
              />
            </div>

            <div className="section-header">Panel</div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Needs Attention section</div>
                <div className="setting-hint">Show a priority section at the top of the panel for activities that need your attention</div>
              </div>
              <button
                className={`toggle ${showPrioritySection ? "on" : ""}`}
                onClick={() => { void saveGeneralSetting({ showPrioritySection: !showPrioritySection }); }}
                aria-pressed={showPrioritySection}
                aria-label="Show Needs Attention section"
              />
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Hide empty feeds</div>
                <div className="setting-hint">Hide feeds with no activities from the panel</div>
              </div>
              <button
                className={`toggle ${hideEmptyFeeds ? "on" : ""}`}
                onClick={() => { void saveGeneralSetting({ hideEmptyFeeds: !hideEmptyFeeds }); }}
                aria-pressed={hideEmptyFeeds}
                aria-label="Hide empty feeds"
              />
            </div>

            <div className="section-header">Keyboard</div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Global shortcut</div>
                <div className="setting-hint">Toggle the panel from anywhere, even when Cortado is in the background</div>
                {hotkeyError && <div className="hotkey-error">{hotkeyError}</div>}
              </div>
              <div className="hotkey-recorder">
                <div className={`hotkey-display ${hotkeyRecording ? "recording" : ""} ${!globalHotkey && !hotkeyRecording ? "empty" : ""}`}>
                  {hotkeyRecording
                    ? "Press a shortcut\u2026"
                    : globalHotkey
                      ? formatShortcut(globalHotkey)
                      : "Not set"}
                </div>
                <button
                  className={`hotkey-record-btn ${hotkeyRecording ? "recording" : ""}`}
                  onClick={() => {
                    setHotkeyError(null);
                    setHotkeyRecording(!hotkeyRecording);
                  }}
                >
                  {hotkeyRecording ? "Cancel" : "Record"}
                </button>
                {globalHotkey && !hotkeyRecording && (
                  <button
                    className="hotkey-clear-btn"
                    onClick={() => { void saveHotkey(""); }}
                    title="Clear shortcut"
                  >
                    ✕
                  </button>
                )}
              </div>
            </div>

            <div className="btn-row">
              <div style={{ flex: 1 }} />
              <button
                className="btn-danger-sm"
                onClick={() => setResetConfirm("general")}
              >
                Reset to defaults
              </button>
            </div>

            {settingsPath && (
              <div className="config-path-bar">
                <span className="config-path-text">{settingsPath}</span>
                <div className="config-path-actions">
                  <button className="config-path-btn" onClick={() => { void invoke("open_settings_file"); }}>
                    Open in editor
                  </button>
                  <button className="config-path-btn" onClick={() => { void invoke("reveal_settings_file"); }}>
                    Reveal
                  </button>
                </div>
              </div>
            )}
          </>
        ) : section === "notifications" ? (
          <>
            <h2 className="settings-title">Notifications</h2>

            {notifLoading ? (
              <p className="settings-placeholder">Loading...</p>
            ) : (
              <>
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">Enable notifications</div>
                    <div className="setting-hint">Send system notifications when activity statuses change</div>
                  </div>
                  <div className="control-with-status">
                    <button
                      className={`toggle ${notifSettings.enabled ? "on" : ""}`}
                      onClick={() => {
                        void saveNotifSettings({ ...notifSettings, enabled: !notifSettings.enabled });
                      }}
                      aria-pressed={notifSettings.enabled}
                      aria-label="Enable notifications"
                    />
                  </div>
                </div>

                {notifPermission === false && (
                  <div className="dep-banner-warning">
                    <span className="dep-banner-icon">⚠</span>
                    <div>
                      <strong>Notification permission not granted.</strong>{" "}
                      <button className="btn-secondary-sm" onClick={() => { void handleRequestPermission(); }}>
                        Request permission
                      </button>
                    </div>
                  </div>
                )}

                <div className={!notifSettings.enabled ? "section-disabled" : ""}>
                  <div className="section-header">
                    Mode
                  </div>
                  <div className="section-hint">Which status changes trigger notifications</div>

                  <div
                    className={`option-row ${notifSettings.mode === "all" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "all", kinds: undefined }); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">All</div>
                      <div className="option-hint">Any status change</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.mode === "escalation_only" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "escalation_only", kinds: undefined }); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Escalation only</div>
                      <div className="option-hint">Only when status worsens</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.mode === "specific_kinds" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "specific_kinds", kinds: notifSettings.kinds ?? [] }); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Specific kinds</div>
                      <div className="option-hint">Only selected status types</div>
                    </div>
                  </div>

                  {notifSettings.mode === "specific_kinds" && (
                    <div className="kind-chips">
                      {([
                        ["attention-negative", "Needs attention"],
                        ["attention-positive", "Ready to go"],
                        ["waiting", "Waiting"],
                        ["running", "In progress"],
                        ["idle", "Idle"],
                      ] as [StatusKindKey, string][]).map(([kind, label]) => (
                        <button
                          className={`kind-chip ${notifSettings.kinds?.includes(kind) ? "active" : ""}`}
                          key={kind}
                          onClick={() => {
                            const current = notifSettings.kinds ?? [];
                            const updated = current.includes(kind)
                              ? current.filter((k) => k !== kind)
                              : [...current, kind];
                            void saveNotifSettings({ ...notifSettings, kinds: updated });
                          }}
                        >
                          {label}
                        </button>
                      ))}
                    </div>
                  )}

                  <div className="section-header">
                    Delivery
                  </div>
                  <div className="section-hint">How notifications are batched</div>

                  <div
                    className={`option-row ${notifSettings.delivery === "grouped" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, delivery: "grouped" }); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Grouped</div>
                      <div className="option-hint">One notification per feed per poll</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.delivery === "immediate" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, delivery: "immediate" }); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Immediate</div>
                      <div className="option-hint">One notification per change</div>
                    </div>
                  </div>

                  <div className="section-header">Activity events</div>

                  <div className="setting-row">
                    <div className="setting-info">
                      <div className="setting-label">New activities</div>
                      <div className="setting-hint">Notify when new activities appear</div>
                    </div>
                    <div className="control-with-status">
                      <button
                        className={`toggle ${notifSettings.notify_new_activities ? "on" : ""}`}
                        onClick={() => { void saveNotifSettings({ ...notifSettings, notify_new_activities: !notifSettings.notify_new_activities }); }}
                        aria-pressed={notifSettings.notify_new_activities}
                        aria-label="Notify on new activities"
                      />
                    </div>
                  </div>
                  <div className="setting-row">
                    <div className="setting-info">
                      <div className="setting-label">Removed activities</div>
                      <div className="setting-hint">Notify when activities disappear</div>
                    </div>
                    <div className="control-with-status">
                      <button
                        className={`toggle ${notifSettings.notify_removed_activities ? "on" : ""}`}
                        onClick={() => { void saveNotifSettings({ ...notifSettings, notify_removed_activities: !notifSettings.notify_removed_activities }); }}
                        aria-pressed={notifSettings.notify_removed_activities}
                        aria-label="Notify on removed activities"
                      />
                    </div>
                  </div>
                </div>

                {notifSaveError && <div className="save-error">{notifSaveError}</div>}
                {testNotifError && <div className="save-error">{testNotifError}</div>}

                <div className="btn-row" style={{ marginTop: 16 }}>
                  <button
                    className="btn-secondary"
                    disabled={!notifSettings.enabled || notifPermission === false}
                    onClick={() => { void handleTestNotification(); }}
                  >
                    Send test notification
                  </button>
                  <button
                    className="btn-secondary"
                    onClick={() => { void invoke("open_notification_settings"); }}
                  >
                    Configure in System Settings
                  </button>
                  <div style={{ flex: 1 }} />
                  <button
                    className="btn-danger-sm"
                    onClick={() => setResetConfirm("notifications")}
                  >
                    Reset to defaults
                  </button>
                </div>
              </>
            )}
          </>
        ) : section === "focus" ? (
          <>
            <h2 className="settings-title">Coding Agents</h2>
            <p className="settings-hint" style={{ marginBottom: 16 }}>
              How cortado tracks and navigates to your coding agent sessions.
            </p>

            {focusLoading ? (
              <p className="settings-placeholder">Loading...</p>
            ) : (
              <>
                <div className="section-header">tmux integration</div>

                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">
                      Enable tmux pane switching
                      <button
                        className="help-toggle"
                        onClick={() => setShowTmuxHelp((v) => !v)}
                      >
                        ?
                      </button>
                    </div>
                    <div className="setting-hint">Navigate to the exact tmux pane running an agent session.</div>
                  </div>
                  <button
                    className={`toggle ${tmuxEnabled ? "on" : ""}`}
                    onClick={() => { void saveFocusSetting({ tmuxEnabled: !tmuxEnabled }); }}
                  />
                </div>

                {showTmuxHelp ? (
                  <div className="help-detail">
                    When you open an agent activity, cortado navigates to the exact pane
                    running that session. If the session already has a terminal tab,
                    cortado selects the right window and pane without disrupting your other tabs.
                    If the session is detached, cortado switches an existing client to show it.
                  </div>
                ) : null}

                {focusCaps ? (
                  <div className="setting-row">
                    <div className="setting-info">
                      <div className="setting-hint">
                        {focusCaps.tmux_installed ? "tmux detected" : "tmux is not available"}
                      </div>
                    </div>
                  </div>
                ) : null}

                <div className="section-header">Ghostty tab switching</div>

                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">
                      Tab-level focus
                      <button
                        className="help-toggle"
                        onClick={() => setShowGhosttyHelp((v) => !v)}
                      >
                        ?
                      </button>
                    </div>
                    <div className="setting-hint">
                      Switch to the Ghostty tab running the agent session. Requires Ghostty 1.3+.
                    </div>
                  </div>
                  <span className={`status-badge ${focusCaps?.ghostty_scriptable ? "active" : "unavailable"}`}>
                    {focusCaps?.ghostty_scriptable ? "Available" : "Not available"}
                  </span>
                </div>

                {showGhosttyHelp ? (
                  <div className="help-detail">
                    Uses Ghostty's AppleScript API to switch to the correct tab. With tmux, matches
                    precisely by tmux session name. Without tmux, matches by working directory in the tab title
                    (best-effort, depends on shell config). Ghostty does not yet expose PID or TTY on terminal
                    objects, so precise matching without tmux is not possible.
                  </div>
                ) : null}

                {focusCaps ? (
                  <div className="setting-row">
                    <div className="setting-info">
                      <div className="setting-hint">
                        {focusCaps.ghostty_scriptable
                          ? `Ghostty ${focusCaps.ghostty_version ?? ""} -- scripting supported`
                          : focusCaps.ghostty_version
                            ? `Ghostty ${focusCaps.ghostty_version} -- requires 1.3+`
                            : "Ghostty not detected"}
                      </div>
                    </div>
                  </div>
                ) : null}

                <div className="section-header">Accessibility</div>

                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">
                      Enable window focus
                      <button
                        className="help-toggle"
                        onClick={() => setShowAccessibilityHelp((v) => !v)}
                      >
                        ?
                      </button>
                    </div>
                    <div className="setting-hint">Raise the specific terminal window by matching its title.</div>
                  </div>
                  <button
                    className={`toggle ${accessibilityEnabled ? "on" : ""}`}
                    onClick={() => { void saveFocusSetting({ accessibilityEnabled: !accessibilityEnabled }); }}
                  />
                </div>

                {showAccessibilityHelp ? (
                  <div className="help-detail">
                    With Accessibility permission, cortado can find and raise the specific terminal
                    window containing your agent session. Works with any terminal but less precise
                    than tmux — matches by window title, which depends on your shell and terminal config.
                  </div>
                ) : null}

                {accessibilityEnabled && !focusCaps?.accessibility_permitted ? (
                  <div className="setting-row">
                    <div className="setting-info">
                      <div className="setting-hint">
                        Accessibility permission not granted.
                      </div>
                    </div>
                    <button
                      className="config-path-btn"
                      onClick={() => {
                        void invoke("open_activity", { url: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility" });
                      }}
                    >
                      Open System Settings...
                    </button>
                  </div>
                ) : null}
              </>
            )}
          </>
        ) : editingFeed !== null ? (
          /* ===== FEED EDIT FORM (F2 breadcrumb replace) ===== */
          <div className={`feed-edit-content ${feedNavTransition === "drill-in" ? "slide-in" : ""}`}>
            <div className="breadcrumb">
              <span className="breadcrumb-link" onClick={cancelEdit}>Feeds</span>
              <span className="breadcrumb-sep">›</span>
              <span className="breadcrumb-current">
                {isNewFeed ? "New Feed" : editingFeed.name || "Untitled"}
              </span>
              {!isNewFeed && (
                <div className="breadcrumb-actions">
                  {deleteConfirm ? (
                    <>
                      <span className="delete-confirm-text">Delete this feed?</span>
                      <button className="btn-danger-sm" onClick={() => { void deleteFeed(); }}>Yes, delete</button>
                      <button className="btn-secondary-sm" onClick={() => setDeleteConfirm(false)}>Cancel</button>
                    </>
                  ) : (
                    <button className="btn-danger-sm" onClick={() => setDeleteConfirm(true)}>Delete</button>
                  )}
                </div>
              )}
            </div>

            {depInfo && depInstalled === false && (
              <div className="dep-banner-warning">
                <span className="dep-banner-icon">⚠</span>
                <div>
                  <strong>{depInfo.name} not found.</strong> This feed requires <code>{depInfo.binary}</code> to be installed.{" "}
                  <a className="ext-link" onClick={() => { void invoke("open_activity", { url: depInfo.installUrl }); }}>Install →</a>
                  {depInfo.extraSteps && (
                    <ul className="dep-steps">
                      {depInfo.extraSteps.map((step, i) => (
                        <li key={i}><code>{step}</code></li>
                      ))}
                    </ul>
                  )}
                </div>
              </div>
            )}

            <div className="form-group">
              <label className="form-label">Feed name</label>
              <input
                className={`form-input ${fieldErrors.name ? "error" : ""}`}
                type="text"
                value={editingFeed.name}
                onChange={(e) => updateField("name", e.target.value)}
                placeholder="e.g. My PRs"
              />
              {fieldErrors.name && <div className="field-error">{fieldErrors.name}</div>}
            </div>

            <div className="form-group">
              <label className="form-label">Type</label>
              <div className="form-type-display">
                {FEED_TYPE_LABELS[editingFeed.type as FeedType] ?? editingFeed.type}
              </div>
            </div>

            {feedTypeFields.map((field) => (
              <div className="form-group" key={field.key}>
                <label className="form-label">
                  {field.label}
                  {field.required && <span className="required-mark">*</span>}
                </label>
                {field.hint && <div className="form-hint">{field.hint}</div>}
                <div className={field.sensitive ? "input-with-toggle" : ""}>
                  <input
                    className={`form-input ${field.mono ? "mono" : ""} ${fieldErrors[field.key] ? "error" : ""}`}
                    type={field.sensitive && !revealedTokens.has(field.key) ? "password" : "text"}
                    value={String(editingFeed.type_specific[field.key] ?? "")}
                    onChange={(e) => updateField(field.key, e.target.value)}
                    placeholder={field.placeholder}
                  />
                  {field.sensitive && (
                    <button
                      className="reveal-toggle"
                      onClick={() => toggleTokenReveal(field.key)}
                      type="button"
                    >
                      {revealedTokens.has(field.key) ? "Hide" : "Show"}
                    </button>
                  )}
                </div>
                {fieldErrors[field.key] && <div className="field-error">{fieldErrors[field.key]}</div>}
              </div>
            ))}

            <div className="form-row">
              <div className="form-group">
                <label className="form-label">Interval<span className="required-mark">*</span></label>
                <div className="form-hint">How often to poll</div>
                <DurationInput
                  value={editingFeed.interval}
                  onChange={(val) => {
                    setSaveSuccess(false);
                    setFieldErrors((prev) => {
                      if (!prev.interval) return prev;
                      const next = { ...prev };
                      delete next.interval;
                      return next;
                    });
                    setEditingFeed({ ...editingFeed, interval: val });
                  }}
                  placeholder="5"
                />
                {fieldErrors.interval && <div className="field-error">{fieldErrors.interval}</div>}
              </div>
              <div className="form-group">
                <label className="form-label">Retain</label>
                <div className="form-hint">Keep completed items for</div>
                <DurationInput
                  value={editingFeed.retain}
                  onChange={(val) => {
                    setSaveSuccess(false);
                    setEditingFeed({ ...editingFeed, retain: val });
                  }}
                  placeholder="—"
                />
              </div>
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Notifications</div>
                <div className="setting-hint">Send system notifications for status changes in this feed</div>
              </div>
              <button
                className={`toggle ${editingFeed.notify !== false ? "on" : ""}`}
                onClick={() => {
                  setSaveSuccess(false);
                  const current = editingFeed.notify !== false;
                  setEditingFeed({ ...editingFeed, notify: current ? false : undefined });
                }}
                aria-pressed={editingFeed.notify !== false}
                aria-label="Enable notifications for this feed"
              />
            </div>

            {saveError && <div className="save-error">{saveError}</div>}
            {saveSuccess && (
              <div className="save-success">
                Saved. Restart Cortado for changes to take effect.
              </div>
            )}

            <div className="btn-row">
              <button className="btn-primary" onClick={() => { void saveFeed(); }}>Save</button>
              <button className="btn-secondary" onClick={cancelEdit}>Discard</button>
              <div style={{ flex: 1 }} />
              <button
                className="btn-test"
                onClick={() => { void runTest(); }}
                disabled={testLoading}
              >
                {testLoading ? (
                  <><span className="spinner-sm" /> Testing...</>
                ) : (
                  <><span className="btn-test-icon">▶</span> Test</>
                )}
              </button>
            </div>

            {/* T3 — Collapsible test results */}
            <div className={`test-panel-wrap ${testLoading || testResult ? "expanded" : ""}`}>
              <div className="test-panel-wrap-inner">
                {testLoading && !testResult && (
                  <div className="test-panel loading">
                    <span className="spinner-sm" /> Polling feed...
                  </div>
                )}

                {testResult && (
                  <div className={`test-panel ${testResult.success ? "success" : "error"}`}>
                    <div className="test-header">
                      <span className="test-status">
                        {testResult.success
                          ? `✓ Connected — ${testResult.activities.length} ${testResult.activities.length === 1 ? "activity" : "activities"}`
                          : "✕ Poll failed"}
                      </span>
                      <span className="test-toggle" onClick={() => setTestPreviewOpen(!testPreviewOpen)}>
                        {testPreviewOpen ? "Hide details ▾" : "Show details ▸"}
                      </span>
                    </div>
                    {testPreviewOpen && (
                      <div className="test-details">
                        {testResult.success ? (
                          testResult.activities.length > 0 ? (
                            testResult.activities.slice(0, 5).map((a, i) => (
                              <div className="test-activity" key={i}>
                                <span className="test-dot" />
                                <span className="test-activity-name">{a.title}</span>
                                {a.status && <span className="test-activity-status">{a.status}</span>}
                              </div>
                            ))
                          ) : (
                            <div className="test-empty">No activities returned (the feed is working but has no items).</div>
                          )
                        ) : (
                          <div className="test-error-detail">{testResult.error}</div>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>

            {/* D4 — Footer dep note (when CLI is installed) */}
            {depInfo && depInstalled !== false && (
              <div className="dep-footer">
                Requires <code>{depInfo.binary}</code> CLI, authenticated via <code>{depInfo.authCommand}</code>.
                {depInfo.extraSteps && (
                  <ul className="dep-steps">
                    {depInfo.extraSteps.map((step, i) => (
                      <li key={i}><code>{step}</code></li>
                    ))}
                  </ul>
                )}
                <a className="ext-link" onClick={() => { void invoke("open_activity", { url: depInfo.installUrl }); }}>Install guide →</a>
              </div>
            )}

            {/* Copilot session feed info */}
            {editingFeed.type === "copilot-session" && (
              <div className="dep-footer">
                Discovers active sessions automatically from <code>~/.copilot/session-state/</code>. No CLI or authentication required.
                <ul className="dep-steps">
                  <li>Shows one activity per working directory (multiple resumed sessions are deduplicated)</li>
                  <li>Opening an activity focuses the terminal — exact tmux pane when available</li>
                </ul>
              </div>
            )}
          </div>
        ) : catalogStep !== "hidden" ? (
          /* ===== FEED CATALOG ===== */
          <div className={`feed-edit-content ${feedNavTransition === "drill-in" ? "slide-in" : ""}`}>
            {catalogStep === "providers" ? (
              <>
                <div className="breadcrumb">
                  <span className="breadcrumb-link" onClick={catalogBack}>Feeds</span>
                  <span className="breadcrumb-sep">{"\u203A"}</span>
                  <span className="breadcrumb-current">New Feed</span>
                </div>
                <div className="catalog-grid">
                  {FEED_CATALOG.map((provider) => (
                    <div
                      className="catalog-provider-card"
                      key={provider.id}
                      onClick={() => selectProvider(provider)}
                    >
                      <div className="catalog-provider-head">
                        <span
                          className="catalog-provider-icon"
                          dangerouslySetInnerHTML={{ __html: provider.icon }}
                        />
                        <span className="catalog-provider-count">
                          {provider.types.length} {provider.types.length === 1 ? "type" : "types"}
                        </span>
                      </div>
                      <div className="catalog-provider-name">{provider.name}</div>
                      <div className="catalog-provider-types">
                        {provider.types.map((t) => t.name).join(", ")}
                      </div>
                    </div>
                  ))}
                </div>
              </>
            ) : catalogStep === "types" && catalogProvider ? (
              <>
                <div className="breadcrumb">
                  <span className="breadcrumb-link" onClick={catalogBack}>New Feed</span>
                  <span className="breadcrumb-sep">{"\u203A"}</span>
                  <span className="breadcrumb-current">{catalogProvider.name}</span>
                </div>
                <div className="catalog-type-list">
                  {catalogProvider.types.map((ct) => (
                    <div
                      className="catalog-type-card"
                      key={ct.feedType}
                      onClick={() => selectFeedType(ct)}
                    >
                      <div
                        className="catalog-type-icon"
                        dangerouslySetInnerHTML={{ __html: ct.icon }}
                      />
                      <div className="catalog-type-info">
                        <div className="catalog-type-name">{ct.name}</div>
                        <div className="catalog-type-desc">{ct.description}</div>
                      </div>
                      <span className="catalog-type-arrow">{"\u203A"}</span>
                    </div>
                  ))}
                </div>
              </>
            ) : null}
          </div>
        ) : (
          /* ===== FEED LIST ===== */
          <div className={`feed-list-content ${feedNavTransition === "drill-out" ? "slide-in" : ""}`}>
            <div className="toolbar">
              <h2 className="settings-title" style={{ margin: 0 }}>Feeds</h2>
              <button className="add-btn" onClick={startAdd}>+ New feed</button>
            </div>

            {feedsLoading ? (
              <p className="settings-placeholder">Loading...</p>
            ) : feeds.length === 0 ? (
              <div className="empty-state-settings">
                <p>No feeds configured.</p>
                <button className="add-btn" onClick={startAdd}>+ Add your first feed</button>
              </div>
            ) : (
              <div className="feed-card-list">
                {feeds.map((feed, index) => (
                  <div className="feed-card" key={`${feed.name}-${index}`} onClick={() => startEdit(index)}>
                    <div className="feed-indicator" />
                    <div className="feed-card-body">
                      <div className="feed-card-top">
                        <span className="feed-card-name">{feed.name}</span>
                        <span className="feed-card-badge">{feed.type}</span>
                      </div>
                      <div className="feed-card-meta">
                        {feed.interval && (
                          <span className="feed-card-detail">
                            <span className="feed-card-detail-icon">↻</span> {feed.interval}
                          </span>
                        )}
                        {Object.entries(feed.type_specific).slice(0, 1).map(([key, val]) => (
                          <span className="feed-card-detail" key={key}>
                            <span className="feed-card-detail-icon">⊞</span> {String(val)}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {configPath && (
              <div className="config-path-bar">
                <span className="config-path-text">{configPath}</span>
                <div className="config-path-actions">
                  <button className="config-path-btn" onClick={() => { void invoke("open_config_file"); }}>
                    Open in editor
                  </button>
                  <button className="config-path-btn" onClick={() => { void invoke("reveal_config_file"); }}>
                    Reveal
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </main>
      {resetConfirm && (
        <div className={`modal-backdrop ${modalExiting ? "exiting" : ""}`} onClick={closeModal}>
          <div className="modal-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="modal-title">Reset to defaults</div>
            <div className="modal-body">
              {resetConfirm === "general"
                ? "Reset all general settings (theme, text size, shortcut, behavior) to their default values?"
                : "Reset all notification settings to their default values?"}
            </div>
            <div className="modal-actions">
              <button className="btn-secondary" onClick={closeModal}>Cancel</button>
              <button
                className="btn-danger-sm"
                onClick={() => {
                  const target = resetConfirm;
                  closeModal();
                  if (target === "notifications") {
                    void saveNotifSettings({
                      enabled: true,
                      mode: "all",
                      delivery: "grouped",
                      notify_new_activities: true,
                      notify_removed_activities: true,
                    });
                  } else {
                    void saveGeneralSetting({ showMenubar: true, showPrioritySection: true, hideEmptyFeeds: false, theme: "system", textSize: "m" });
                    void saveHotkey("super+shift+space");
                    if (autostart) void toggleAutostart();
                  }
                }}
              >
                Reset
              </button>
            </div>
          </div>
        </div>
      )}
      <div className={`save-toast ${toastVisible ? "visible" : ""}`}>✓ Saved</div>
    </div>
  );
}

export default SettingsApp;
