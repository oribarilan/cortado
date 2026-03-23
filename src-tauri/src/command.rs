use tauri::State;

use crate::feed::{FeedSnapshot, FeedSnapshotCache};

#[tauri::command]
pub async fn list_feeds(cache: State<'_, FeedSnapshotCache>) -> Result<Vec<FeedSnapshot>, String> {
    Ok(cache.list().await)
}
