use std::process::Command;
use std::sync::Once;

use tauri::{AppHandle, Manager};

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
pub fn set_panel_height(app_handle: AppHandle, height: f64) -> Result<(), String> {
    fns::set_panel_height(&app_handle, height)
}
