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

type MainScreenSettings = {
  show_priority_section: boolean;
};

type AppSettings = {
  notifications: NotificationSettings;
  main_screen: MainScreenSettings;
  show_menubar: boolean;
  theme: string;
  text_size: string;
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

type FeedType = "github-pr" | "ado-pr" | "shell";

const FEED_TYPE_LABELS: Record<FeedType, string> = {
  "github-pr": "GitHub PR",
  "ado-pr": "Azure DevOps PR",
  "shell": "Shell",
};

const FEED_TYPE_FIELDS: Record<FeedType, { key: string; label: string; placeholder: string; hint?: string; mono?: boolean; required?: boolean; sensitive?: boolean }[]> = {
  "github-pr": [
    { key: "repo", label: "Repository", placeholder: "owner/repo", hint: "GitHub owner and repo name", mono: true, required: true },
    { key: "user", label: "Author filter", placeholder: "@me", hint: "GitHub username or @me (default)", mono: true },
  ],
  "ado-pr": [
    { key: "url", label: "Repository URL", placeholder: "https://dev.azure.com/org/project/_git/repo", hint: "Full URL to the Azure DevOps Git repository", mono: true, required: true },
    { key: "user", label: "Creator filter", placeholder: "me", hint: "User identity or 'me' (default)", mono: true },
  ],
  "shell": [
    { key: "command", label: "Command", placeholder: "df -h /", hint: "Executed via sh -c", mono: true, required: true },
    { key: "field_name", label: "Field name", placeholder: "output", hint: "Name for the output field (default: output)", mono: true },
    { key: "field_type", label: "Field type", placeholder: "text", hint: "text, status, number, or url (default: text)", mono: true },
  ],
};

function emptyFeed(feedType: FeedType): FeedConfigDto {
  return {
    name: "",
    type: feedType,
    interval: "5m",
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

  return errors;
}

function SettingsApp() {
  useAppearance();
  const [section, setSection] = useState<"general" | "notifications" | "feeds">("general");
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
  const [theme, setTheme] = useState("system");
  const [textSize, setTextSize] = useState("m");

  // Notification settings state
  const [notifSettings, setNotifSettings] = useState<NotificationSettings>({
    enabled: true,
    mode: "all",
    delivery: "grouped",
    notify_new_activities: true,
    notify_removed_activities: true,
  });
  const [notifLoading, setNotifLoading] = useState(true);
  const [notifSaveSuccess, setNotifSaveSuccess] = useState<string | null>(null);
  const [notifSaveError, setNotifSaveError] = useState<string | null>(null);
  const [notifPermission, setNotifPermission] = useState<boolean | null>(null);

  // Feeds state
  const [feeds, setFeeds] = useState<FeedConfigDto[]>([]);
  const [feedsLoading, setFeedsLoading] = useState(true);
  const [configPath, setConfigPath] = useState("");

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
        setShowMenubar(s.show_menubar);
        setShowPrioritySection(s.main_screen?.show_priority_section ?? true);
        setTheme(s.theme ?? "system");
        setTextSize(s.text_size ?? "m");
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

  const notifSaveTimer = useState<ReturnType<typeof setTimeout> | null>(null);
  const generalSaveTimer = useState<ReturnType<typeof setTimeout> | null>(null);
  const [generalSaveSuccess, setGeneralSaveSuccess] = useState<string | null>(null);

  const saveNotifSettings = useCallback(async (updated: NotificationSettings, key?: string) => {
    setNotifSettings(updated);
    setNotifSaveSuccess(null);
    setNotifSaveError(null);
    if (notifSaveTimer[0]) clearTimeout(notifSaveTimer[0]);
    try {
      await invoke("save_settings", {
        settings: {
          notifications: updated,
          main_screen: { show_priority_section: showPrioritySection },
          show_menubar: showMenubar,
          theme,
          text_size: textSize,
        },
      });
      setNotifSaveSuccess(key ?? "general");
      const t = setTimeout(() => setNotifSaveSuccess(null), 1500);
      notifSaveTimer[0] = t;
    } catch (err) {
      setNotifSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [notifSaveTimer, showPrioritySection, showMenubar, theme, textSize]);

  const saveGeneralSetting = useCallback(async (updates: { showMenubar?: boolean; showPrioritySection?: boolean; theme?: string; textSize?: string }, key?: string) => {
    const newMenubar = updates.showMenubar ?? showMenubar;
    const newPriority = updates.showPrioritySection ?? showPrioritySection;
    const newTheme = updates.theme ?? theme;
    const newTextSize = updates.textSize ?? textSize;

    if (updates.showMenubar !== undefined) setShowMenubar(newMenubar);
    if (updates.showPrioritySection !== undefined) setShowPrioritySection(newPriority);
    if (updates.theme !== undefined) setTheme(newTheme);
    if (updates.textSize !== undefined) setTextSize(newTextSize);

    setGeneralSaveSuccess(null);
    if (generalSaveTimer[0]) clearTimeout(generalSaveTimer[0]);
    try {
      await invoke("save_settings", {
        settings: {
          notifications: notifSettings,
          main_screen: { show_priority_section: newPriority },
          show_menubar: newMenubar,
          theme: newTheme,
          text_size: newTextSize,
        },
      });
      if (key) {
        setGeneralSaveSuccess(key);
        const t = setTimeout(() => setGeneralSaveSuccess(null), 1500);
        generalSaveTimer[0] = t;
      }
    } catch (err) {
      console.error("failed saving general setting:", err);
    }
  }, [notifSettings, showMenubar, showPrioritySection, theme, textSize, generalSaveTimer]);

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
    setEditingIndex(feeds.length);
    setEditingFeed(emptyFeed("github-pr"));
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
    invoke<{ installed: boolean }>("check_feed_dependency", { feedType: "github-pr" })
      .then((r) => setDepInstalled(r.installed))
      .catch(() => setDepInstalled(null));
  }, [feeds.length, scheduleAnim]);

  const cancelEdit = useCallback(() => {
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

  const switchSection = useCallback((next: "general" | "notifications" | "feeds") => {
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
      </nav>
      <main className={`settings-main ${sectionFading ? "fading" : ""}`}>
        {section === "general" ? (
          <>
            <h2 className="settings-title">General</h2>

            <div className="section-header">
              Appearance
              <span className={`inline-saved ${generalSaveSuccess === "appearance" ? "visible" : ""}`}>Saved</span>
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Theme</div>
              </div>
              <div className="segmented-control">
                {(["light", "dark", "system"] as const).map((opt) => (
                  <button
                    key={opt}
                    className={`segmented-option ${theme === opt ? "active" : ""}`}
                    onClick={() => { void saveGeneralSetting({ theme: opt }, "appearance"); }}
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
                {(["s", "m", "l", "xl"] as const).map((opt) => (
                  <button
                    key={opt}
                    className={`segmented-option ${textSize === opt ? "active" : ""}`}
                    onClick={() => { void saveGeneralSetting({ textSize: opt }, "appearance"); }}
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
                <div className="setting-label">Show menubar icon</div>
                <div className="setting-hint">Show tray icon and menubar panel. When off, use ⌘⇧Space or Spotlight to access Cortado.</div>
              </div>
              <div className="control-with-status">
                <button
                  className={`toggle ${showMenubar ? "on" : ""}`}
                  onClick={() => { void saveGeneralSetting({ showMenubar: !showMenubar }, "menubar"); }}
                  aria-pressed={showMenubar}
                  aria-label="Show menubar icon"
                />
                <span className={`inline-saved ${generalSaveSuccess === "menubar" ? "visible" : ""}`}>Saved</span>
              </div>
            </div>

            <div className="setting-row">
              <div className="setting-info">
                <div className="setting-label">Needs Attention section</div>
                <div className="setting-hint">Show a priority section in the main screen for activities that need your attention</div>
              </div>
              <div className="control-with-status">
                <button
                  className={`toggle ${showPrioritySection ? "on" : ""}`}
                  onClick={() => { void saveGeneralSetting({ showPrioritySection: !showPrioritySection }, "priority"); }}
                  aria-pressed={showPrioritySection}
                  aria-label="Show Needs Attention section"
                />
                <span className={`inline-saved ${generalSaveSuccess === "priority" ? "visible" : ""}`}>Saved</span>
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
                        void saveNotifSettings({ ...notifSettings, enabled: !notifSettings.enabled }, "enable");
                      }}
                      aria-pressed={notifSettings.enabled}
                      aria-label="Enable notifications"
                    />
                    <span className={`inline-saved ${notifSaveSuccess === "enable" ? "visible" : ""}`}>Saved</span>
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
                    <span className={`inline-saved ${notifSaveSuccess === "mode" ? "visible" : ""}`}>Saved</span>
                  </div>
                  <div className="section-hint">Which status changes trigger notifications</div>

                  <div
                    className={`option-row ${notifSettings.mode === "all" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "all", kinds: undefined }, "mode"); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">All</div>
                      <div className="option-hint">Any status change</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.mode === "escalation_only" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "escalation_only", kinds: undefined }, "mode"); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Escalation only</div>
                      <div className="option-hint">Only when status worsens</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.mode === "specific_kinds" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, mode: "specific_kinds", kinds: notifSettings.kinds ?? [] }, "mode"); }}
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
                            void saveNotifSettings({ ...notifSettings, kinds: updated }, "mode");
                          }}
                        >
                          {label}
                        </button>
                      ))}
                    </div>
                  )}

                  <div className="section-header">
                    Delivery
                    <span className={`inline-saved ${notifSaveSuccess === "delivery" ? "visible" : ""}`}>Saved</span>
                  </div>
                  <div className="section-hint">How notifications are batched</div>

                  <div
                    className={`option-row ${notifSettings.delivery === "grouped" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, delivery: "grouped" }, "delivery"); }}
                  >
                    <span className="option-indicator" />
                    <div className="option-body">
                      <div className="option-label">Grouped</div>
                      <div className="option-hint">One notification per feed per poll</div>
                    </div>
                  </div>
                  <div
                    className={`option-row ${notifSettings.delivery === "immediate" ? "selected" : ""}`}
                    onClick={() => { void saveNotifSettings({ ...notifSettings, delivery: "immediate" }, "delivery"); }}
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
                        onClick={() => { void saveNotifSettings({ ...notifSettings, notify_new_activities: !notifSettings.notify_new_activities }, "new"); }}
                        aria-pressed={notifSettings.notify_new_activities}
                        aria-label="Notify on new activities"
                      />
                      <span className={`inline-saved ${notifSaveSuccess === "new" ? "visible" : ""}`}>Saved</span>
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
                        onClick={() => { void saveNotifSettings({ ...notifSettings, notify_removed_activities: !notifSettings.notify_removed_activities }, "removed"); }}
                        aria-pressed={notifSettings.notify_removed_activities}
                        aria-label="Notify on removed activities"
                      />
                      <span className={`inline-saved ${notifSaveSuccess === "removed" ? "visible" : ""}`}>Saved</span>
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

            <div className="form-row">
              <div className="form-group">
                <label className="form-label">Type</label>
                <select
                  className="form-select"
                  value={editingFeed.type}
                  onChange={(e) => updateField("type", e.target.value)}
                >
                  {Object.entries(FEED_TYPE_LABELS).map(([value, label]) => (
                    <option key={value} value={value}>{label}</option>
                  ))}
                </select>
              </div>
              <div className="form-group" />
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
                ? "Reset all general settings (theme, text size, behavior) to their default values?"
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
                    }, "enable");
                  } else {
                    void saveGeneralSetting({ showMenubar: true, showPrioritySection: true, theme: "system", textSize: "m" });
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
    </div>
  );
}

export default SettingsApp;
