use std::process::Command;
use std::sync::Once;

use tauri::{AppHandle, Emitter, Listener, Manager, WebviewWindowBuilder};
use tauri_nspanel::ManagerExt;

use crate::{
    app_settings::{self, AppSettingsState},
    feed::{BackgroundPoller, FeedRegistry, FeedSnapshot},
    fns, main_screen, terminal_focus, ui_snapshot,
};
static PANEL_INIT: Once = Once::new();
static MAIN_SCREEN_INIT: Once = Once::new();

fn hide_menubar_panel(app_handle: &AppHandle) {
    if let Ok(panel) = app_handle.get_webview_panel("main") {
        panel.order_out(None);
    }
}

fn hide_all_panels(app_handle: &AppHandle) {
    hide_menubar_panel(app_handle);
    if let Ok(panel) = app_handle.get_webview_panel("main-screen") {
        panel.order_out(None);
    }
}

/// Tauri command: one-time NSPanel setup for the main screen window.
/// Called from the frontend on first mount.
#[tauri::command]
pub fn init_main_screen_panel(app_handle: AppHandle) {
    MAIN_SCREEN_INIT.call_once(|| {
        main_screen::swizzle_to_main_screen_panel(&app_handle);
        main_screen::update_main_screen_appearance(&app_handle);
        main_screen::setup_main_screen_panel_listeners(&app_handle);

        // Auto-open the panel on first launch so users see the app immediately.
        // This must happen here (not in setup()) because the NSPanel doesn't
        // exist until swizzle completes.
        main_screen::show_main_screen_panel(&app_handle);
    });
}

/// Tauri command: hides the main screen panel (used by Esc handler).
#[tauri::command]
pub fn hide_main_screen_panel(app_handle: AppHandle) {
    if let Ok(panel) = app_handle.get_webview_panel("main-screen") {
        panel.order_out(None);
    }
}

/// Tauri command: hides the menubar panel and opens the main screen.
#[tauri::command]
pub fn open_main_screen(app_handle: AppHandle) {
    hide_menubar_panel(&app_handle);
    main_screen::toggle_main_screen_panel(&app_handle);
}

#[tauri::command]
pub async fn list_feeds(app_handle: AppHandle) -> Result<Vec<FeedSnapshot>, String> {
    ui_snapshot::list_for_ui(&app_handle).await
}

#[tauri::command]
pub fn init_panel(app_handle: AppHandle) {
    PANEL_INIT.call_once(|| {
        fns::swizzle_to_menubar_panel(&app_handle);
        fns::update_menubar_appearance(&app_handle);
        fns::setup_menubar_panel_listeners(&app_handle);
    });
}

#[tauri::command]
pub async fn refresh_feeds(app_handle: AppHandle) -> Result<(), String> {
    let poller = app_handle
        .try_state::<BackgroundPoller>()
        .ok_or_else(|| "background poller state is missing".to_string())?
        .inner()
        .clone();
    let registry = app_handle
        .try_state::<std::sync::Arc<FeedRegistry>>()
        .ok_or_else(|| "feed registry state is missing".to_string())?
        .inner()
        .clone();

    let handle = app_handle.clone();
    poller
        .refresh_now(registry, move |completed, total| {
            let _ = handle.emit("refresh-progress", (completed, total));
        })
        .await;

    Ok(())
}

#[tauri::command]
pub fn open_activity(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only http/https URLs are supported".to_string());
    }

    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|err| format!("failed to spawn `open`: {err}"))?;

    Ok(())
}

#[tauri::command]
pub fn quit_app(app_handle: AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
pub fn open_settings(
    app_handle: AppHandle,
    section: Option<String>,
    feed_type: Option<String>,
) -> Result<(), String> {
    hide_all_panels(&app_handle);
    if let Some(window) = app_handle.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        let config = &app_handle.config().app.windows;
        let settings_config = config
            .iter()
            .find(|w| w.label == "settings")
            .ok_or_else(|| "settings window config not found".to_string())?;

        let window = WebviewWindowBuilder::from_config(&app_handle, settings_config)
            .map_err(|e| e.to_string())?
            .build()
            .map_err(|e| e.to_string())?;

        let _ = window.center();
    }

    if let Some(section) = section {
        #[derive(Clone, serde::Serialize)]
        struct SettingsNavigate {
            section: String,
            feed_type: Option<String>,
        }
        let payload = SettingsNavigate { section, feed_type };
        // Wait for the Settings window to finish loading before emitting.
        // When freshly created, React needs time to mount and register listeners.
        let handle = app_handle.clone();
        if let Some(window) = handle.get_webview_window("settings") {
            let payload_clone = payload.clone();
            let handle_clone = handle.clone();
            window.once("settings-ready", move |_| {
                let _ = handle_clone.emit("settings-navigate", payload_clone);
            });
            // Also emit after a delay as fallback (window may already be loaded).
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                let _ = handle.emit("settings-navigate", payload);
            });
        }
    }

    Ok(())
}

/// Opens macOS System Settings to Cortado's notification preferences.
#[tauri::command]
pub fn open_notification_settings(app_handle: AppHandle) -> Result<(), String> {
    let bundle_id = app_handle.config().identifier.as_str();

    let url = format!(
        "x-apple.systempreferences:com.apple.Notifications-Settings.extension?id={bundle_id}"
    );

    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|err| format!("failed to open system notification settings: {err}"))?;

    Ok(())
}

/// Sends a test notification through the native notification pipeline.
#[tauri::command]
pub fn send_test_notification(app_handle: AppHandle) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    app_handle
        .notification()
        .builder()
        .title("Cortado")
        .body("Test notification -- notifications are working!")
        .show()
        .map_err(|err| format!("failed to send test notification: {err}"))
}

/// Updates the global hotkey registration and persists the new value.
///
/// `hotkey` is a Tauri shortcut string (e.g. `"super+shift+space"`)
/// or an empty string to clear the hotkey entirely.
#[tauri::command]
pub async fn set_global_hotkey(
    hotkey: String,
    state: tauri::State<'_, AppSettingsState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Unregister all existing shortcuts.
    app_handle
        .global_shortcut()
        .unregister_all()
        .map_err(|e| format!("failed to unregister shortcuts: {e}"))?;

    // Register the new shortcut (uses the handler set by the Builder in main.rs).
    if !hotkey.is_empty() {
        app_handle
            .global_shortcut()
            .register(hotkey.as_str())
            .map_err(|e| format!("failed to register shortcut '{hotkey}': {e}"))?;
    }

    // Persist to settings.
    let mut settings = state.read().await.clone();
    settings.general.global_hotkey = hotkey;
    app_settings::save_settings_to_file(&settings).map_err(|e| e.to_string())?;
    state.update(settings).await;

    Ok(())
}

/// Focuses the terminal containing a copilot session, identified by session ID.
#[tauri::command]
pub async fn focus_session(
    session_id: String,
    registry: tauri::State<'_, std::sync::Arc<FeedRegistry>>,
    settings_state: tauri::State<'_, AppSettingsState>,
) -> Result<(), String> {
    eprintln!("focus_session called for session_id={session_id}");

    let session = registry.find_harness_session(&session_id).ok_or_else(|| {
        let msg = format!("session '{session_id}' not found");
        eprintln!("focus_session error: {msg}");
        msg
    })?;

    let settings = settings_state.read().await;
    let result = terminal_focus::focus_terminal(
        &session,
        settings.focus.tmux_enabled,
        settings.focus.accessibility_enabled,
    );

    if let Err(ref e) = result {
        eprintln!("focus_session error: {e}");
    }

    result
}

/// Returns current focus capabilities for the settings UI.
#[tauri::command]
pub fn get_focus_capabilities() -> terminal_focus::FocusCapabilities {
    terminal_focus::get_capabilities()
}

/// Returns whether this is a dev build.
#[tauri::command]
pub fn is_dev_mode() -> bool {
    crate::app_env::is_dev()
}

/// Restarts the app to apply config changes.
#[tauri::command]
pub fn restart_app(app_handle: AppHandle) {
    app_handle.restart();
}

/// Downloads and installs an available Cortado update, then restarts the app.
#[tauri::command]
pub async fn install_update(app_handle: AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let update = app_handle
        .updater()
        .map_err(|e| format!("updater not available: {e}"))?
        .check()
        .await
        .map_err(|e| format!("update check failed: {e}"))?;

    let update = match update {
        Some(u) => u,
        None => return Err("no update available".to_string()),
    };

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("update install failed: {e}"))?;

    app_handle.restart();
}

/// Triggers an immediate connectivity retry check.
#[tauri::command]
pub async fn retry_connection(app_handle: AppHandle) -> Result<(), String> {
    let cm = app_handle
        .try_state::<std::sync::Arc<crate::feed::connectivity::ConnectivityManager>>()
        .ok_or_else(|| "connectivity manager not available".to_string())?;
    cm.trigger_retry();
    Ok(())
}
