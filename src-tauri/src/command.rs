use std::sync::Arc;

use tauri::State;
use tokio::sync::Mutex;

use crate::feed::{FeedRegistry, FeedSnapshot};

#[tauri::command]
pub async fn list_feeds(
    registry: State<'_, Arc<Mutex<FeedRegistry>>>,
) -> Result<Vec<FeedSnapshot>, String> {
    let registry = registry.lock().await;
    Ok(registry.poll_all().await)
}
