use std::process::Command;
use std::sync::Once;

use tauri::{AppHandle, Manager, WebviewWindowBuilder};

use crate::{
    feed::{BackgroundPoller, FeedRegistry, FeedSnapshot},
    fns, ui_snapshot,
};
static PANEL_INIT: Once = Once::new();

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

    poller.refresh_now(registry).await;

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
