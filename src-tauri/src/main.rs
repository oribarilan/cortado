// Prevent additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod feed;
mod fns;
mod panel;
mod settings_config;
mod ui_snapshot;

use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager};

use crate::feed::{
    config::ConfigChangeTracker, load_feed_registry, BackgroundPoller, FeedSnapshotCache,
    RegistryBuildMode,
};

fn main() {
    let feed_registry = Arc::new(
        load_feed_registry(RegistryBuildMode::Tolerant)
            .unwrap_or_else(|err| panic!("failed to initialize feeds: {err}")),
    );
    let feed_cache = FeedSnapshotCache::from_registry(feed_registry.as_ref());
    let poller = BackgroundPoller::new(feed_cache.clone());
    let config_tracker = Arc::new(
        ConfigChangeTracker::initialize()
            .unwrap_or_else(|err| panic!("failed to initialize config change tracker: {err}")),
    );

    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(feed_cache.clone())
        .manage(feed_registry.clone())
        .manage(poller.clone())
        .manage(config_tracker)
        .invoke_handler(tauri::generate_handler![
            command::init_panel,
            command::list_feeds,
            command::refresh_feeds,
            command::open_activity,
            command::quit_app,
            command::open_settings,
            settings_config::get_feeds_config,
            settings_config::save_feeds_config,
            settings_config::get_config_path,
            settings_config::open_config_file,
            settings_config::reveal_config_file,
            settings_config::check_feed_dependency,
            settings_config::test_feed
        ])
        .setup({
            let feed_registry = feed_registry.clone();
            let feed_cache = feed_cache.clone();
            let poller = poller.clone();

            move |app| {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                let app_handle = app.app_handle().clone();

                panel::create(&app_handle)?;

                let updates = poller.subscribe();

                start_refresh_loop(app_handle.clone(), feed_cache.clone(), updates);

                // Seed feeds and start polling in the background.
                // The refresh loop will update the tray once the seed completes.
                tauri::async_runtime::spawn(async move {
                    poller
                        .seed_startup_best_effort(feed_registry.clone(), Duration::from_secs(15))
                        .await;
                    poller.start(feed_registry);
                });

                Ok(())
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_refresh_loop(
    app_handle: tauri::AppHandle,
    snapshots_cache: FeedSnapshotCache,
    mut updates: tokio::sync::watch::Receiver<u64>,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            if updates.changed().await.is_err() {
                break;
            }

            if let Err(err) = ui_snapshot::refresh_config_change_state(&app_handle).await {
                eprintln!("failed checking config change state: {err}");
            }

            let snapshots = match ui_snapshot::list_for_ui(&app_handle).await {
                Ok(snapshots) => snapshots,
                Err(err) => {
                    eprintln!("failed collecting snapshots for UI: {err}");
                    snapshots_cache.list().await
                }
            };

            if let Err(err) = app_handle.emit("feeds-updated", snapshots) {
                eprintln!("failed emitting feeds-updated event: {err}");
            }
        }
    });
}
