import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { DisclosureChevron } from "../shared/DisclosureChevron";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import { useAppearance } from "../shared/useAppearance";
import type { FeedSnapshot } from "../shared/types";
import { FEED_CATALOG, findFeedType, generateDefaultName, type FeedType, type CatalogFeedType, type CatalogProvider } from "../shared/feedTypes";

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
  show_empty_feeds: boolean;
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

function emptyFeed(feedType: FeedType, interval?: string): FeedConfigDto {
  return {
    name: "",
    type: feedType,
    interval: interval ?? "5m",
    type_specific: {},
    fields: {},
  };
}

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

/** Inline SVG icons for terminal emulators (16x16, currentColor). */
const TERMINAL_ICONS: Record<string, string> = {
  ghostty: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M12 0C6.7 0 2.4 4.3 2.4 9.6v11.146c0 1.772 1.45 3.267 3.222 3.254a3.18 3.18 0 0 0 1.955-.686 1.96 1.96 0 0 1 2.444 0 3.18 3.18 0 0 0 1.976.686c.75 0 1.436-.257 1.98-.686.715-.563 1.71-.587 2.419-.018.59.476 1.355.743 2.182.699 1.705-.094 3.022-1.537 3.022-3.244V9.601C21.6 4.3 17.302 0 12 0M6.069 6.562a1 1 0 0 1 .46.131l3.578 2.065v.002a.974.974 0 0 1 0 1.687L6.53 12.512a.975.975 0 0 1-.976-1.687L7.67 9.602 5.553 8.38a.975.975 0 0 1 .515-1.818m7.438 2.063h4.7a.975.975 0 1 1 0 1.95h-4.7a.975.975 0 0 1 0-1.95"/></svg>`,
  iterm2: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M24 5.359v13.282A5.36 5.36 0 0 1 18.641 24H5.359A5.36 5.36 0 0 1 0 18.641V5.359A5.36 5.36 0 0 1 5.359 0h13.282A5.36 5.36 0 0 1 24 5.359m-.932-.233A4.196 4.196 0 0 0 18.874.932H5.126A4.196 4.196 0 0 0 .932 5.126v13.748a4.196 4.196 0 0 0 4.194 4.194h13.748a4.196 4.196 0 0 0 4.194-4.194zm-.816.233v13.282a3.613 3.613 0 0 1-3.611 3.611H5.359a3.613 3.613 0 0 1-3.611-3.611V5.359a3.613 3.613 0 0 1 3.611-3.611h13.282a3.613 3.613 0 0 1 3.611 3.611M8.854 4.194v6.495h.962V4.194zM5.483 9.493v1.085h.597V9.48q.283-.037.508-.133.373-.165.575-.448.208-.284.208-.649a.9.9 0 0 0-.171-.568 1.4 1.4 0 0 0-.426-.388 3 3 0 0 0-.544-.261 32 32 0 0 0-.545-.209 1.8 1.8 0 0 1-.426-.216q-.164-.12-.164-.284 0-.223.179-.351.18-.126.485-.127.344 0 .575.105.239.105.5.298l.433-.5a2.3 2.3 0 0 0-.605-.433 1.6 1.6 0 0 0-.582-.159v-.968h-.597v.978a2 2 0 0 0-.477.127 1.2 1.2 0 0 0-.545.411q-.194.268-.194.634 0 .335.164.56.164.224.418.38a4 4 0 0 0 .552.262q.291.104.545.209.261.104.425.238a.39.39 0 0 1 .165.321q0 .225-.187.359-.18.134-.537.134-.381 0-.717-.134a4.4 4.4 0 0 1-.649-.351l-.388.589q.209.173.477.306.276.135.575.217.191.046.373.064"/></svg>`,
  wezterm: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M3.27 8.524c0-.623.62-1.007 2.123-1.007l-.5 2.757c-.931-.623-1.624-1.199-1.624-1.75zm4.008 6.807c0 .647-.644 1.079-2.123 1.15l.524-2.924c.931.624 1.6 1.175 1.6 1.774zm-2.625 5.992.454-2.708c3.603-.336 5.01-1.798 5.01-3.404 0-1.653-2.004-2.948-3.841-4.074l.668-3.548c.764.072 1.67.216 2.744.432l.31-2.469c-.81-.12-1.575-.168-2.29-.216L8.257 2.7l-2.363-.024-.453 2.684C1.838 5.648.43 7.158.43 8.764c0 1.63 2.004 2.876 3.841 3.954l-.668 3.716c-.859-.048-1.908-.192-3.125-.408L0 18.495c1.026.12 1.98.192 2.84.216l-.525 2.588zm15.553-1.894h2.673c.334-2.804.81-8.46 1.121-14.86h-2.553c-.071 1.51-.334 10.498-.43 11.241h-.071c-.644-2.42-1.169-4.386-1.813-6.782h-1.456c-.62 2.396-1.05 4.194-1.694 6.782h-.096c-.071-.743-.477-9.73-.525-11.24h-2.648c.31 6.399.763 12.055 1.097 14.86h2.625l1.838-7.12z"/></svg>`,
  tmux: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M24 2.251V10.5H12.45V0h9.3A2.251 2.251 0 0 1 24 2.251zM12.45 11.4H24v10.5h-.008A2.25 2.25 0 0 1 21.75 24H2.25a2.247 2.247 0 0 1-2.242-2.1H0V2.251A2.251 2.251 0 0 1 2.25 0h9.3v21.6h.9V11.4zm11.242 10.5H.308a1.948 1.948 0 0 0 1.942 1.8h19.5a1.95 1.95 0 0 0 1.942-1.8z"/></svg>`,
};

/** Generic terminal icon for emulators without a brand SVG. */
const TERMINAL_ICON_GENERIC = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><polyline points="7 8 10 11 7 14"/><line x1="12" y1="14" x2="17" y2="14"/></svg>`;

/** Download/homepage URLs for terminal apps. */
const TERMINAL_URLS: Record<string, string> = {
  ghostty: "https://ghostty.org/",
  iterm2: "https://iterm2.com/",
  wezterm: "https://wezterm.org/index.html",
  kitty: "https://github.com/kovidgoyal/kitty",
  tmux: "https://github.com/tmux/tmux",
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
        placeholder={placeholder ?? "-"}
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

  if (!feed.interval && !findFeedType(feed.type)?.hideInterval) {
    errors.interval = "Poll interval is required";
  }

  const catalogType = findFeedType(feed.type);
  const typeFields = catalogType?.fields ?? [];
  for (const field of typeFields) {
    if (!field.required) continue;
    const val = String(feed.type_specific[field.key] ?? "").trim();
    if (!val) {
      errors[field.key] = `${field.label} is required`;
    }
  }

  for (const rule of catalogType?.validations ?? []) {
    if (errors[rule.field]) continue;
    const val = String(feed.type_specific[rule.field] ?? "").trim();
    const msg = rule.check(val);
    if (msg) errors[rule.field] = msg;
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
  const [showEmptyFeeds, setShowEmptyFeeds] = useState(false);
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
    notify_removed_activities: false,
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
    tmux_version: string | null;
    terminal_app: string | null;
    terminal_scriptable: boolean;
    ghostty_scriptable: boolean;
    ghostty_version: string | null;
    accessibility_permitted: boolean;
    terminals: { id: string; name: string; installed: boolean }[];
  };
  const [focusCaps, setFocusCaps] = useState<FocusCaps | null>(null);
  const [focusLoading, setFocusLoading] = useState(false);
  const [tmuxEnabled, setTmuxEnabled] = useState(true);
  const [accessibilityEnabled, setAccessibilityEnabled] = useState(false);
  const [showTmuxHelp, setShowTmuxHelp] = useState(false);
  const [showAccessibilityHelp, setShowAccessibilityHelp] = useState(false);
  const [expandedTerminal, setExpandedTerminal] = useState<string | null>(null);

  const [toastMessage, setToastMessage] = useState("✓ Saved");

  // Restart-required tracking: config watcher (feeds) + local (show_menubar).
  const [restartNeeded, setRestartNeeded] = useState(false);
  const initialMenubar = useRef<boolean | null>(null);

  const showToast = useCallback((message?: string) => {
    if (toastTimer.current) clearTimeout(toastTimer.current);
    setToastMessage(message ?? "✓ Saved");
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
  const [swapAnim, setSwapAnim] = useState<{ up: number; down: number } | null>(null);
  const nameManuallyEdited = useRef(false);

  // Catalog state (for the new feed flow)
  const [catalogStep, setCatalogStep] = useState<"hidden" | "providers" | "types">("hidden");
  const [catalogProvider, setCatalogProvider] = useState<CatalogProvider | null>(null);

  // Dependency + test state
  const [depInstalled, setDepInstalled] = useState<boolean | null>(null);
  const [testResult, setTestResult] = useState<TestFeedResult | null>(null);
  const [testLoading, setTestLoading] = useState(false);
  const [testPreviewOpen, setTestPreviewOpen] = useState(false);
  const [setupReady, setSetupReady] = useState<boolean | null>(null);
  const [setupOutdated, setSetupOutdated] = useState(false);
  const [setupInstalling, setSetupInstalling] = useState(false);
  const [setupUninstalling, setSetupUninstalling] = useState(false);
  const [setupError, setSetupError] = useState<string | null>(null);

  useEffect(() => {
    isEnabled()
      .then(setAutostart)
      .catch(() => setAutostart(false))
      .finally(() => setAutostartLoading(false));

    invoke<AppSettings>("get_settings")
      .then((s) => {
        setNotifSettings(s.notifications);
        setShowMenubar(s.general?.show_menubar ?? true);
        initialMenubar.current = s.general?.show_menubar ?? true;
        setShowPrioritySection(s.panel?.show_priority_section ?? true);
        setShowEmptyFeeds(s.panel?.show_empty_feeds ?? false);
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

    emit("settings-ready").catch(() => {});
  }, []);

  // Listen for config change detection from the backend watcher.
  useEffect(() => {
    const unlisten = listen<FeedSnapshot[]>("feeds-updated", (event) => {
      const configFeed = event.payload.find(
        (f) => f.feed_type === "app" && f.activities.some((a) => a.action === "restart_app")
      );
      if (configFeed) setRestartNeeded(true);
    });
    // Also check on mount.
    invoke<FeedSnapshot[]>("list_feeds")
      .then((snapshots) => {
        const configFeed = snapshots.find(
          (f) => f.feed_type === "app" && f.activities.some((a) => a.action === "restart_app")
        );
        if (configFeed) setRestartNeeded(true);
      })
      .catch(() => {});
    return () => { unlisten.then((fn) => fn()); };
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
          panel: { show_priority_section: showPrioritySection, show_empty_feeds: showEmptyFeeds },
          notifications: updated,
          focus: { tmux_enabled: tmuxEnabled, accessibility_enabled: accessibilityEnabled },
        },
      });
      showToast("✓ Saved (Changes Applied)");
    } catch (err) {
      setNotifSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [showPrioritySection, showEmptyFeeds, showMenubar, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveGeneralSetting = useCallback(async (updates: { showMenubar?: boolean; showPrioritySection?: boolean; showEmptyFeeds?: boolean; theme?: string; textSize?: string }) => {
    const newMenubar = updates.showMenubar ?? showMenubar;
    const newPriority = updates.showPrioritySection ?? showPrioritySection;
    const newShowEmpty = updates.showEmptyFeeds ?? showEmptyFeeds;
    const newTheme = updates.theme ?? theme;
    const newTextSize = updates.textSize ?? textSize;

    if (updates.showMenubar !== undefined) setShowMenubar(newMenubar);
    if (updates.showPrioritySection !== undefined) setShowPrioritySection(newPriority);
    if (updates.showEmptyFeeds !== undefined) setShowEmptyFeeds(newShowEmpty);
    if (updates.theme !== undefined) setTheme(newTheme);
    if (updates.textSize !== undefined) setTextSize(newTextSize);

    try {
      await invoke("save_settings", {
        settings: {
          general: { show_menubar: newMenubar, theme: newTheme, text_size: newTextSize, global_hotkey: globalHotkey },
          panel: { show_priority_section: newPriority, show_empty_feeds: newShowEmpty },
          notifications: notifSettings,
          focus: { tmux_enabled: tmuxEnabled, accessibility_enabled: accessibilityEnabled },
        },
      });
      const menubarChanged = updates.showMenubar !== undefined && initialMenubar.current !== null && updates.showMenubar !== initialMenubar.current;
      if (menubarChanged) {
        setRestartNeeded(true);
        showToast("✓ Saved (Restart Required)");
      } else {
        showToast("✓ Saved (Changes Applied)");
      }
    } catch (err) {
      console.error("failed saving general setting:", err);
    }
  }, [notifSettings, showMenubar, showPrioritySection, showEmptyFeeds, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveFocusSetting = useCallback(async (updates: { tmuxEnabled?: boolean; accessibilityEnabled?: boolean }) => {
    const newTmux = updates.tmuxEnabled ?? tmuxEnabled;
    const newAccessibility = updates.accessibilityEnabled ?? accessibilityEnabled;

    if (updates.tmuxEnabled !== undefined) setTmuxEnabled(newTmux);
    if (updates.accessibilityEnabled !== undefined) setAccessibilityEnabled(newAccessibility);

    try {
      await invoke("save_settings", {
        settings: {
          general: { show_menubar: showMenubar, theme, text_size: textSize, global_hotkey: globalHotkey },
          panel: { show_priority_section: showPrioritySection, show_empty_feeds: showEmptyFeeds },
          notifications: notifSettings,
          focus: { tmux_enabled: newTmux, accessibility_enabled: newAccessibility },
        },
      });
      showToast("✓ Saved (Changes Applied)");
    } catch (err) {
      console.error("failed saving focus setting:", err);
    }
  }, [notifSettings, showMenubar, showPrioritySection, showEmptyFeeds, theme, textSize, globalHotkey, tmuxEnabled, accessibilityEnabled, showToast]);

  const saveHotkey = useCallback(async (hotkey: string) => {
    setHotkeyError(null);
    try {
      await invoke("set_global_hotkey", { hotkey });
      setGlobalHotkey(hotkey);
      showToast("✓ Saved (Changes Applied)");
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

  const checkSetup = useCallback((feedType: string) => {
    const setup = findFeedType(feedType)?.setup;
    if (setup) {
      invoke<{ ready: boolean; outdated: boolean }>(setup.checkCommand)
        .then((r) => {
          setSetupReady(r.ready);
          setSetupOutdated(r.outdated);
        })
        .catch(() => {
          setSetupReady(null);
          setSetupOutdated(false);
        });
    } else {
      setSetupReady(null);
      setSetupOutdated(false);
    }
    setSetupError(null);
    setSetupInstalling(false);
  }, []);

  const startEdit = useCallback((index: number) => {
    setEditingIndex(index);
    setEditingFeed(structuredClone(feeds[index]));
    setIsNewFeed(false);
    nameManuallyEdited.current = true;
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
    const dep = findFeedType(feeds[index].type)?.dependency;
    if (dep) {
      invoke<{ installed: boolean }>("check_feed_dependency", { binary: dep.binary })
        .then((r) => setDepInstalled(r.installed))
        .catch(() => setDepInstalled(null));
    } else {
      setDepInstalled(null);
    }
    checkSetup(feeds[index].type);
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
    const newFeed = emptyFeed(feedType, catalogType.defaultInterval);
    // Auto-populate name for feed types with static patterns (no field placeholders)
    const defaultName = generateDefaultName(feedType, {});
    if (defaultName) newFeed.name = defaultName;
    setEditingFeed(newFeed);
    setIsNewFeed(true);
    nameManuallyEdited.current = false;
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

    const depForType = findFeedType(feedType)?.dependency;
    if (depForType) {
      invoke<{ installed: boolean }>("check_feed_dependency", { binary: depForType.binary })
        .then((r) => setDepInstalled(r.installed))
        .catch(() => setDepInstalled(null));
    } else {
      setDepInstalled(null);
    }
    checkSetup(feedType);
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
      showToast("✓ Saved (Restart Required)");
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
    }
  }, [editingIndex, feeds, isNewFeed, showToast]);

  const moveFeed = useCallback(async (index: number, direction: -1 | 1) => {
    const swapIndex = index + direction;
    if (swapIndex < 0 || swapIndex >= feeds.length) return;
    const updatedFeeds = [...feeds];
    [updatedFeeds[index], updatedFeeds[swapIndex]] = [updatedFeeds[swapIndex], updatedFeeds[index]];
    try {
      await invoke("save_feeds_config", { feeds: updatedFeeds });
      setFeeds(updatedFeeds);
      setSwapAnim({ up: Math.min(index, swapIndex), down: Math.max(index, swapIndex) });
      scheduleAnim(() => setSwapAnim(null), 180);
      showToast("✓ Saved (Restart Required)");
    } catch (err) {
      console.error("failed to reorder feeds:", err);
    }
  }, [feeds, showToast, scheduleAnim]);

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
      // Track whether the user manually edited the name
      if (value === "") {
        nameManuallyEdited.current = false;
        // Regenerate default name when field is cleared
        const defaultName = generateDefaultName(editingFeed.type, editingFeed.type_specific);
        setEditingFeed({ ...editingFeed, name: defaultName ?? "" });
      } else {
        nameManuallyEdited.current = true;
        setEditingFeed({ ...editingFeed, name: value });
      }
    } else if (key === "type") {
      setEditingFeed({ ...editingFeed, type: value, type_specific: {} });
      setTestResult(null);
      const dep = findFeedType(value)?.dependency;
      if (dep) {
        invoke<{ installed: boolean }>("check_feed_dependency", { binary: dep.binary })
          .then((r) => setDepInstalled(r.installed))
          .catch(() => setDepInstalled(null));
      } else {
        setDepInstalled(null);
      }
    } else if (key === "interval") {
      setEditingFeed({ ...editingFeed, interval: value || undefined });
    } else if (key === "retain") {
      setEditingFeed({ ...editingFeed, retain: value || undefined });
    } else {
      const newTypeSpecific = { ...editingFeed.type_specific, [key]: value };
      const updatedFeed = { ...editingFeed, type_specific: newTypeSpecific };
      // Auto-populate name if user hasn't manually edited it
      if (!nameManuallyEdited.current) {
        const defaultName = generateDefaultName(editingFeed.type, newTypeSpecific);
        if (defaultName) {
          updatedFeed.name = defaultName;
        }
      }
      setEditingFeed(updatedFeed);
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

  const editingCatalogType = editingFeed ? findFeedType(editingFeed.type) : undefined;
  const feedTypeFields = editingCatalogType?.fields ?? [];
  const depInfo = editingCatalogType?.dependency;
  const setupInfo = editingCatalogType?.setup;

  const runSetupInstall = useCallback(async () => {
    if (!setupInfo) return;
    setSetupInstalling(true);
    setSetupError(null);
    try {
      const result = await invoke<{ success: boolean; error?: string }>(setupInfo.installCommand);
      if (result.success) {
        setSetupReady(true);
        setSetupOutdated(false);
      } else {
        setSetupError(result.error ?? "Installation failed");
      }
    } catch (e) {
      setSetupError(String(e));
    } finally {
      setSetupInstalling(false);
    }
  }, [setupInfo]);

  const runSetupUninstall = useCallback(async () => {
    if (!setupInfo) return;
    setSetupUninstalling(true);
    setSetupError(null);
    try {
      const result = await invoke<{ success: boolean; error?: string }>(setupInfo.uninstallCommand);
      if (result.success) {
        setSetupReady(false);
        setSetupOutdated(false);
      } else {
        setSetupError(result.error ?? "Uninstall failed");
      }
    } catch (e) {
      setSetupError(String(e));
    } finally {
      setSetupUninstalling(false);
    }
  }, [setupInfo]);

  const switchSection = useCallback((next: "general" | "notifications" | "feeds" | "focus") => {
    if (next === section || sectionFading) return;
    cancelEdit();
    setSectionFading(true);
    scheduleAnim(() => {
      setSection(next);
      setSectionFading(false);
    }, 110); // ~60% of --duration-normal
  }, [section, sectionFading, cancelEdit, scheduleAnim]);

  // Deep-link: external surfaces can open Settings to a specific section and feed type.
  const lastNavTimestamp = useRef(0);
  useEffect(() => {
    const unlisten = listen<{ section: string; feed_type?: string }>("settings-navigate", (event) => {
      // Deduplicate: ignore events within 500ms of each other (ready + fallback timer).
      const now = Date.now();
      if (now - lastNavTimestamp.current < 500) return;
      lastNavTimestamp.current = now;

      const { section: target, feed_type } = event.payload;
      const validSections = ["general", "notifications", "feeds", "focus"] as const;
      type Section = typeof validSections[number];
      if (!validSections.includes(target as Section)) return;

      cancelEdit();
      setSection(target as Section);
      setSectionFading(false);

      if (target === "feeds" && feed_type) {
        const catalogType = FEED_CATALOG
          .flatMap((p) => p.types)
          .find((t) => t.feedType === feed_type);
        if (catalogType) {
          selectFeedType(catalogType);
        } else {
          startAdd();
        }
      } else if (target === "feeds") {
        startAdd();
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [cancelEdit, selectFeedType, startAdd]);

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
          <span className="settings-nav-icon">&gt;_</span> Terminals
        </div>
        {restartNeeded && (
          <>
            <div className="sidebar-spacer" />
            <button
              className="settings-nav restart-action"
              onClick={() => { invoke("restart_app").catch(console.error); }}
            >
              <span className="settings-nav-icon">↻</span> Restart
            </button>
          </>
        )}
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
                <div className="setting-label">Show empty feeds</div>
                <div className="setting-hint">Show feeds that have no activities</div>
              </div>
              <button
                className={`toggle ${showEmptyFeeds ? "on" : ""}`}
                onClick={() => { void saveGeneralSetting({ showEmptyFeeds: !showEmptyFeeds }); }}
                aria-pressed={showEmptyFeeds}
                aria-label="Show empty feeds"
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
            <p className="settings-hint" style={{ marginTop: 8 }}>
              Set <code style={{ fontSize: "inherit" }}>$XDG_CONFIG_HOME</code> to override the config directory (default: <code style={{ fontSize: "inherit" }}>~/.config</code>).
            </p>
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
            <h2 className="settings-title">Terminals</h2>
            <p className="settings-hint" style={{ marginBottom: 16 }}>
              Terminal emulators and their integration capabilities with cortado.
            </p>

            {focusLoading ? (
              <p className="settings-placeholder">Loading...</p>
            ) : (
              <div className="terminal-list">
                {/* Terminal rows from backend, sorted: detected first */}
                {[...(focusCaps?.terminals ?? [])].sort((a, b) => Number(b.installed) - Number(a.installed)).map((term) => {
                  const isExpanded = expandedTerminal === term.id;
                  const toggleExpand = () => setExpandedTerminal(isExpanded ? null : term.id);
                  const handleKeyDown = (e: React.KeyboardEvent) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      toggleExpand();
                    }
                  };

                  // Version badge for ghostty
                  const version = term.id === "ghostty" ? focusCaps?.ghostty_version ?? null : null;
                  const icon = TERMINAL_ICONS[term.id] ?? TERMINAL_ICON_GENERIC;

                  return (
                    <div
                      className={`terminal-row ${isExpanded ? "expanded" : ""} ${!term.installed ? "not-detected" : ""}`}
                      key={term.id}
                      data-terminal-id={term.id}
                    >
                      <div
                        className="terminal-header"
                        onClick={toggleExpand}
                        onKeyDown={handleKeyDown}
                        tabIndex={0}
                        role="button"
                        aria-expanded={isExpanded}
                      >
                        <DisclosureChevron />
                        <span className="terminal-icon" dangerouslySetInnerHTML={{ __html: icon }} />
                        <span className="terminal-name">{term.name}</span>
                        {version && <span className="terminal-version">{version}</span>}
                      </div>
                      <div className="terminal-detail-wrap">
                        <div className="terminal-detail-inner">
                           <div className="terminal-detail">
                            {!term.installed ? (
                              <div className="terminal-detail-desc">
                                Not detected on this system.
                                {TERMINAL_URLS[term.id] && (
                                  <>
                                    {" "}
                                    <a
                                      href="#"
                                      className="terminal-link"
                                      onClick={(e) => { e.preventDefault(); void invoke("open_activity", { url: TERMINAL_URLS[term.id] }); }}
                                    >
                                      Get {term.name}
                                    </a>
                                  </>
                                )}
                              </div>
                            ) : term.id === "ghostty" ? (
                              <>
                                <div className="terminal-detail-desc">
                                  Click "Focus session" and cortado switches to the exact Ghostty tab running your agent. With tmux, matching is precise (by session name). Without tmux, it matches by working directory in the tab title. Requires Ghostty 1.3+.
                                </div>
                                <div className="terminal-detail-line">
                                  <span className="terminal-detail-label">AppleScript</span>
                                  <span className={`status-badge ${focusCaps?.ghostty_scriptable ? "active" : "unavailable"}`}>
                                    {focusCaps?.ghostty_scriptable ? "Available" : "Not available"}
                                  </span>
                                </div>
                                <div className="terminal-detail-line">
                                  <span className="terminal-detail-label">Version</span>
                                  <span className="terminal-detail-value">
                                    {focusCaps?.ghostty_version ?? "Not detected"}
                                  </span>
                                </div>
                                <div className="terminal-detail-line">
                                  <div className="setting-info">
                                    <div className="setting-label">
                                      Accessibility permission
                                      <button
                                        className="help-toggle"
                                        onClick={() => setShowAccessibilityHelp((v) => !v)}
                                      >
                                        ?
                                      </button>
                                    </div>
                                    <div className="setting-hint">Also raises the Ghostty window to the front, not just the tab</div>
                                  </div>
                                  <button
                                    className={`toggle ${accessibilityEnabled ? "on" : ""}`}
                                    onClick={() => { void saveFocusSetting({ accessibilityEnabled: !accessibilityEnabled }); }}
                                    aria-pressed={accessibilityEnabled}
                                    aria-label="Enable accessibility permission"
                                  />
                                </div>
                                {accessibilityEnabled && !focusCaps?.accessibility_permitted ? (
                                  <div className="terminal-detail-line">
                                    <span className="status-badge unavailable">Not granted</span>
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
                                {showAccessibilityHelp ? (
                                  <div className="help-detail">
                                    Without this permission, cortado can switch to the right tab but cannot bring a background Ghostty window to the front. Grant Accessibility access in System Settings to enable full window-level focus.
                                  </div>
                                ) : null}
                              </>
                            ) : term.id === "iterm2" ? (
                              <div className="terminal-detail-desc">
                                Click "Focus session" and cortado jumps to the exact iTerm2 split pane running your agent. The right window, tab, and pane are all selected automatically. Without tmux, matching is precise via TTY. With tmux, cortado brings iTerm2 to the front while tmux handles internal pane navigation.
                              </div>
                            ) : term.id === "terminal_app" ? (
                              <div className="terminal-detail-desc">
                                Click "Focus session" and cortado switches to the exact Terminal.app tab running your agent. Without tmux, matching is precise via TTY. With tmux, cortado brings Terminal to the front while tmux handles internal pane navigation. No setup needed. Always available on macOS.
                              </div>
                            ) : term.id === "wezterm" ? (
                              <div className="terminal-detail-desc">
                                Click "Focus session" and cortado navigates to the exact WezTerm pane running your agent, matched by working directory. Works with or without tmux. Requires the <code style={{ fontSize: "inherit" }}>wezterm</code> CLI on your PATH.
                              </div>
                            ) : term.id === "kitty" ? (
                              <div className="terminal-detail-desc">
                                Click "Focus session" and cortado focuses the exact Kitty window (pane) running your agent, matched by process ID. Requires <code style={{ fontSize: "inherit" }}>allow_remote_control</code> enabled in your kitty.conf.
                              </div>
                            ) : (
                              <div className="terminal-detail-desc">
                                {term.name} terminal emulator.
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })}

                {/* tmux separator + row */}
                <div className="terminal-separator">
                  <span className="terminal-separator-label">Multiplexer</span>
                </div>
                {(() => {
                  const isExpanded = expandedTerminal === "tmux";
                  const toggleExpand = () => setExpandedTerminal(isExpanded ? null : "tmux");
                  const tmuxDetected = focusCaps?.tmux_installed ?? false;
                  const handleKeyDown = (e: React.KeyboardEvent) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      toggleExpand();
                    }
                  };
                  return (
                    <div
                      className={`terminal-row ${isExpanded ? "expanded" : ""} ${!tmuxDetected ? "not-detected" : ""}`}
                      data-terminal-id="tmux"
                    >
                      <div
                        className="terminal-header"
                        onClick={toggleExpand}
                        onKeyDown={handleKeyDown}
                        tabIndex={0}
                        role="button"
                        aria-expanded={isExpanded}
                      >
                        <DisclosureChevron />
                        <span className="terminal-icon" dangerouslySetInnerHTML={{ __html: TERMINAL_ICONS.tmux }} />
                        <span className="terminal-name">tmux</span>
                        {focusCaps?.tmux_version && (
                          <span className="terminal-version">{focusCaps.tmux_version}</span>
                        )}
                      </div>
                      <div className="terminal-detail-wrap">
                        <div className="terminal-detail-inner">
                          <div className="terminal-detail">
                            {!tmuxDetected ? (
                              <div className="terminal-detail-desc">
                                Not detected on this system.
                                {" "}
                                <a
                                  href="#"
                                  className="terminal-link"
                                  onClick={(e) => { e.preventDefault(); void invoke("open_activity", { url: TERMINAL_URLS.tmux }); }}
                                >
                                  Get tmux
                                </a>
                              </div>
                            ) : (
                              <>
                                <div className="terminal-detail-desc">
                                  If your agent sessions run inside tmux, cortado navigates to the exact pane (right session, window, and pane) before switching to the terminal tab. This adds pane-level precision on top of any terminal integration above.
                                </div>
                            <div className="terminal-detail-line">
                              <span className="terminal-detail-label">Version</span>
                              <span className="terminal-detail-value">
                                {focusCaps?.tmux_version ?? "Not detected"}
                              </span>
                            </div>
                            <div className="setting-row">
                              <div className="setting-info">
                                <div className="setting-label">
                                  Enable pane switching
                                  <button
                                    className="help-toggle"
                                    onClick={() => setShowTmuxHelp((v) => !v)}
                                  >
                                    ?
                                  </button>
                                </div>
                                <div className="setting-hint">Navigate to the exact tmux pane running an agent session</div>
                              </div>
                              <button
                                className={`toggle ${tmuxEnabled ? "on" : ""}`}
                                onClick={() => { void saveFocusSetting({ tmuxEnabled: !tmuxEnabled }); }}
                                aria-pressed={tmuxEnabled}
                                aria-label="Enable tmux pane switching"
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
                              </>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })()}
              </div>
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
              <span className="breadcrumb-sep">›</span>
              <span className="breadcrumb-type">{editingCatalogType?.label ?? editingFeed.type}</span>
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

            {setupInfo && setupReady === false && (
              <div className="setup-banner">
                <span className="setup-banner-icon">⚙</span>
                <div className="setup-banner-content">
                  <strong>{setupInfo.label} required.</strong> {setupInfo.description} <span className="setup-help-tip" title={setupInfo.helpText}>(?)</span>
                  <div className="setup-banner-action">
                    <button
                      className="btn-primary-sm"
                      onClick={() => { void runSetupInstall(); }}
                      disabled={setupInstalling}
                    >
                      {setupInstalling ? (
                        <><span className="spinner-sm" /> Installing...</>
                      ) : (
                        setupInfo.installLabel
                      )}
                    </button>
                  </div>
                  {setupError && (
                    <details className="setup-error-details">
                      <summary className="setup-error-summary">✕ Installation failed</summary>
                      <pre className="setup-error-pre">{setupError}</pre>
                    </details>
                  )}
                </div>
              </div>
            )}

            {setupInfo && setupReady === true && setupOutdated && (
              <div className="setup-banner">
                <span className="setup-banner-icon">↑</span>
                <div className="setup-banner-content">
                  <strong>Plugin update available.</strong> Update for question and approval detection.
                  <div className="setup-banner-action">
                    <button
                      className="btn-primary-sm"
                      onClick={() => { void runSetupInstall(); }}
                      disabled={setupInstalling}
                    >
                      {setupInstalling ? (
                        <><span className="spinner-sm" /> Updating...</>
                      ) : (
                        "Update Plugin"
                      )}
                    </button>
                  </div>
                  {setupError && (
                    <details className="setup-error-details">
                      <summary className="setup-error-summary">✕ Update failed</summary>
                      <pre className="setup-error-pre">{setupError}</pre>
                    </details>
                  )}
                </div>
              </div>
            )}

            {setupInfo && setupReady === true && !setupOutdated && (
              <div className="setup-banner-ok">
                <span className="setup-banner-icon">✓</span>
                <span>{setupInfo.label} installed</span>
                <span className="setup-help-group">
                  <span className="setup-help-tip" title={setupInfo.helpText}>(?)</span>
                  <button
                    className="btn-link-subtle"
                    onClick={() => { void runSetupUninstall(); }}
                    disabled={setupUninstalling}
                  >
                    {setupUninstalling ? "Removing..." : "Uninstall"}
                  </button>
                </span>
                {setupError && (
                  <details className="setup-error-details">
                    <summary className="setup-error-summary">✕ Uninstall failed</summary>
                    <pre className="setup-error-pre">{setupError}</pre>
                  </details>
                )}
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
              {!editingCatalogType?.hideInterval && (
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
              )}
              <div className="form-group">
                <label className="form-label">Retain</label>
                <div className="form-hint">Keep completed items for</div>
                <DurationInput
                  value={editingFeed.retain}
                  onChange={(val) => {
                    setSaveSuccess(false);
                    setEditingFeed({ ...editingFeed, retain: val });
                  }}
                  placeholder="-"
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
                Saved (Restart Required)
              </div>
            )}

            <div className="btn-row">
              <button className="btn-primary" onClick={() => { void saveFeed(); }} disabled={setupInfo !== undefined && setupReady !== true}>Save</button>
              <button className="btn-secondary" onClick={cancelEdit}>Discard</button>
              <div style={{ flex: 1 }} />
              <button
                className="btn-test"
                onClick={() => { void runTest(); }}
                disabled={testLoading || (setupInfo !== undefined && setupReady !== true)}
              >
                {testLoading ? (
                  <><span className="spinner-sm" /> Testing...</>
                ) : (
                  <><span className="btn-test-icon">▶</span> Test</>
                )}
              </button>
            </div>

            {/* T3 -- Collapsible test results */}
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
                          ? `✓ Connected: ${testResult.activities.length} ${testResult.activities.length === 1 ? "activity" : "activities"}`
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

            {/* D4 -- Footer dep note (when CLI is installed) */}
            {depInfo && depInstalled !== false && (
              <div className="dep-footer">
                Requires <code>{depInfo.binary}</code> CLI{depInfo.authCommand ? <>, authenticated via <code>{depInfo.authCommand}</code></> : ""}.
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

            {/* Feed type notes (generic, driven by catalog) */}
            {editingCatalogType?.notes && editingCatalogType.notes.length > 0 && !depInfo && (
              <div className="dep-footer">
                {editingCatalogType.notes[0]}
                {editingCatalogType.notes.length > 1 && (
                  <ul className="dep-steps">
                    {editingCatalogType.notes.slice(1).map((note, i) => (
                      <li key={i}>{note}</li>
                    ))}
                  </ul>
                )}
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
                  <div className={`feed-card${swapAnim?.up === index ? " swap-up" : ""}${swapAnim?.down === index ? " swap-down" : ""}`} key={`${feed.name}-${index}`} onClick={() => startEdit(index)}>
                    <div className="feed-indicator" />
                    <div className="feed-card-body">
                      <div className="feed-card-top">
                        <span className="feed-card-name">{feed.name}</span>
                        <span className="feed-card-badge">{findFeedType(feed.type)?.label ?? feed.type}</span>
                      </div>
                      <div className="feed-card-meta">
                        {feed.interval && !findFeedType(feed.type)?.hideInterval && (
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
                    {feeds.length > 1 && (
                      <div className="move-buttons">
                        <button
                          className="move-btn"
                          disabled={index === 0}
                          title="Move up"
                          onClick={(e) => { e.stopPropagation(); void moveFeed(index, -1); }}
                        >&#x25B2;</button>
                        <button
                          className="move-btn"
                          disabled={index === feeds.length - 1}
                          title="Move down"
                          onClick={(e) => { e.stopPropagation(); void moveFeed(index, 1); }}
                        >&#x25BC;</button>
                      </div>
                    )}
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
                      notify_removed_activities: false,
                    });
                  } else {
                    void saveGeneralSetting({ showMenubar: true, showPrioritySection: true, showEmptyFeeds: false, theme: "system", textSize: "m" });
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
      <div className={`save-toast ${toastVisible ? "visible" : ""}`}>{toastMessage}</div>
    </div>
  );
}

export default SettingsApp;
