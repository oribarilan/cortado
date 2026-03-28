use std::process::Command;
use std::sync::Once;

use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};
use tauri_nspanel::ManagerExt;

use crate::{
    feed::{BackgroundPoller, FeedRegistry, FeedSnapshot},
    fns, main_screen, ui_snapshot,
};
static PANEL_INIT: Once = Once::new();
static MAIN_SCREEN_INIT: Once = Once::new();

fn hide_menubar_panel(app_handle: &AppHandle) {
    if let Ok(panel) = app_handle.get_webview_panel("main") {
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
pub fn open_settings(app_handle: AppHandle) -> Result<(), String> {
    hide_menubar_panel(&app_handle);
    if let Some(window) = app_handle.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

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
