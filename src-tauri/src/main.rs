// Prevent additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_env;
mod app_settings;
mod command;
mod feed;
mod fns;
mod main_screen;
mod notification;
mod panel;
mod settings_config;
mod terminal_focus;
mod tray_icon;
mod ui_snapshot;

use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager};

use crate::app_settings::{load_settings, AppSettingsState};
use crate::feed::{
    config::{self, ConfigChangeTracker},
    cortado_update::CortadoUpdateFeed,
    load_feed_registry,
    runtime::NotificationContext,
    BackgroundPoller, FeedSnapshotCache, RegistryBuildMode, StatusKind,
};

fn main() {
    // Packaged macOS apps launched from Finder inherit a minimal PATH.
    // Resolve the user's login shell PATH to find tools like az, gh, etc.
    let path_before = std::env::var("PATH").unwrap_or_default();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    if let Ok(output) = std::process::Command::new(&shell)
        .args(["-l", "-c", "printf '%s' \"$PATH\""])
        .output()
    {
        if output.status.success() {
            let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !resolved.is_empty() && resolved != path_before {
                std::env::set_var("PATH", &resolved);
            }
        }
    }

    let context = tauri::generate_context!();
    app_env::init(&context.config().identifier);

    let feed_configs = config::load_feeds_config().unwrap_or_default();
    let feed_notify_map: std::collections::HashMap<String, bool> = feed_configs
        .iter()
        .map(|c| (c.name.clone(), c.notify.unwrap_or(true)))
        .collect();

    let mut feed_registry = load_feed_registry(RegistryBuildMode::Tolerant)
        .unwrap_or_else(|err| panic!("failed to initialize feeds: {err}"));

    // Built-in feeds (always registered, not user-configured).
    feed_registry.register(Arc::new(CortadoUpdateFeed::new()));

    let feed_registry = Arc::new(feed_registry);
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
    let show_menubar = initial_settings.general.show_menubar;
    let initial_hotkey = initial_settings.general.global_hotkey.clone();
    let app_settings_state = AppSettingsState::new(initial_settings);

    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Another instance with the same bundle ID tried to launch.
            // The first instance keeps running; the duplicate exits.
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            app_settings::get_settings_path,
            app_settings::open_settings_file,
            app_settings::reveal_settings_file,
            command::open_notification_settings,
            command::send_test_notification,
            command::set_global_hotkey,
            command::focus_session,
            command::get_focus_capabilities,
            command::is_dev_mode,
            command::install_update
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

                // Register global shortcut plugin with handler; then register
                // the user-configured hotkey — but only in production mode
                // to avoid stealing the hotkey from a running release build.
                {
                    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

                    app.handle().plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_handler(|app, _shortcut, event| {
                                if event.state == ShortcutState::Pressed {
                                    main_screen::toggle_main_screen_panel(app);
                                }
                            })
                            .build(),
                    )?;

                    if !app_env::is_dev() && !initial_hotkey.is_empty() {
                        if let Err(err) = app.global_shortcut().register(initial_hotkey.as_str()) {
                            eprintln!("failed to register global hotkey '{initial_hotkey}': {err}");
                        }
                    }
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

                // Auto-open the panel so users see the app immediately on launch.
                main_screen::show_main_screen_panel(&app_handle);

                Ok(())
            }
        })
        .build(context)
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

            // Update tray icon to reflect global status rollup.
            let global_status = StatusKind::rollup_for_feeds(&snapshots);
            tray_icon::update_tray_status(&app_handle, global_status);

            if let Err(err) = app_handle.emit("feeds-updated", snapshots) {
                eprintln!("failed emitting feeds-updated event: {err}");
            }
        }
    });
}
