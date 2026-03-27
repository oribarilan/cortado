// Prevent additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_settings;
mod command;
mod feed;
mod fns;
mod main_screen;
mod notification;
mod panel;
mod settings_config;
mod ui_snapshot;

use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager};

use crate::app_settings::{load_settings, AppSettingsState};
use crate::feed::{
    config::{self, ConfigChangeTracker},
    load_feed_registry,
    runtime::NotificationContext,
    BackgroundPoller, FeedSnapshotCache, RegistryBuildMode,
};

fn main() {
    let feed_configs = config::load_feeds_config().unwrap_or_default();
    let feed_notify_map: std::collections::HashMap<String, bool> = feed_configs
        .iter()
        .map(|c| (c.name.clone(), c.notify.unwrap_or(true)))
        .collect();

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
    let initial_settings = load_settings().unwrap_or_else(|err| {
        eprintln!("failed to load settings, using defaults: {err}");
        app_settings::AppSettings::default()
    });
    let show_menubar = initial_settings.show_menubar;
    let app_settings_state = AppSettingsState::new(initial_settings);

    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .manage(feed_cache.clone())
        .manage(feed_registry.clone())
        .manage(poller.clone())
        .manage(config_tracker)
        .manage(app_settings_state.clone())
        .invoke_handler(tauri::generate_handler![
            command::init_panel,
            command::init_main_screen_panel,
            command::hide_main_screen_panel,
            command::open_main_screen,
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
            settings_config::test_feed,
            app_settings::get_settings,
            app_settings::save_settings,
            command::open_notification_settings,
            command::send_test_notification
        ])
        .setup({
            let feed_registry = feed_registry.clone();
            let feed_cache = feed_cache.clone();
            let poller = poller.clone();
            let app_settings_state = app_settings_state.clone();
            let feed_notify_map = Arc::new(feed_notify_map);

            move |app| {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                let app_handle = app.app_handle().clone();

                if show_menubar {
                    panel::create(&app_handle)?;
                }

                // Register ⌘+Shift+Space global shortcut to toggle the main screen.
                {
                    use tauri_plugin_global_shortcut::ShortcutState;

                    app.handle().plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_shortcuts(["super+shift+space"])?
                            .with_handler(|app, _shortcut, event| {
                                if event.state == ShortcutState::Pressed {
                                    main_screen::toggle_main_screen_panel(app);
                                }
                            })
                            .build(),
                    )?;
                }

                let updates = poller.subscribe();

                start_refresh_loop(app_handle.clone(), feed_cache.clone(), updates);

                let notify_ctx = NotificationContext {
                    app_handle: app_handle.clone(),
                    settings_state: app_settings_state,
                    feed_notify_map,
                };
                let poller = poller.with_notifications(notify_ctx);

                // Seed feeds and start polling in the background.
                // The refresh loop will update the tray once the seed completes.
                // Notifications are suppressed during seed (no previous snapshot to diff against).
                tauri::async_runtime::spawn(async move {
                    poller
                        .seed_startup_best_effort(feed_registry.clone(), Duration::from_secs(15))
                        .await;
                    poller.start(feed_registry);
                });

                Ok(())
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                main_screen::toggle_main_screen_panel(_app_handle);
            }
        });
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
