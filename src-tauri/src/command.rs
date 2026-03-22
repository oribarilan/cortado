use std::sync::{Arc, Once};

use tauri::State;
use tokio::sync::Mutex;

use crate::feed::{FeedRegistry, FeedSnapshot};
use crate::fns::{
    setup_menubar_panel_listeners, swizzle_to_menubar_panel, update_menubar_appearance,
};

static INIT: Once = Once::new();

#[tauri::command]
pub fn init(app_handle: tauri::AppHandle) {
    INIT.call_once(|| {
        swizzle_to_menubar_panel(&app_handle);

        update_menubar_appearance(&app_handle);

        setup_menubar_panel_listeners(&app_handle);
    });
}

#[tauri::command]
pub async fn list_feeds(
    registry: State<'_, Arc<Mutex<FeedRegistry>>>,
) -> Result<Vec<FeedSnapshot>, String> {
    let registry = registry.lock().await;
    Ok(registry.poll_all().await)
}
