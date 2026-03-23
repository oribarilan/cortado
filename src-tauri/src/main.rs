// Prevent additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod feed;
mod tray;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tauri::Manager;

use crate::feed::{
    config::{load_feeds_config, FeedConfig},
    github_pr::GithubPrFeed,
    shell::ShellFeed,
    BackgroundPoller, FeedRegistry, FeedSnapshotCache,
};

fn main() {
    let feed_registry = Arc::new(
        build_feed_registry().unwrap_or_else(|err| panic!("failed to initialize feeds: {err}")),
    );
    let feed_cache = FeedSnapshotCache::from_registry(feed_registry.as_ref());
    let poller = BackgroundPoller::new(feed_cache.clone());

    tauri::Builder::default()
        .manage(feed_cache.clone())
        .manage(feed_registry.clone())
        .manage(poller.clone())
        .invoke_handler(tauri::generate_handler![command::list_feeds])
        .setup({
            let feed_registry = feed_registry.clone();
            let feed_cache = feed_cache.clone();
            let poller = poller.clone();

            move |app| {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                let app_handle = app.app_handle().clone();

                tray::create(&app_handle)?;

                let updates = poller.subscribe();

                tray::start_refresh_loop(app_handle.clone(), feed_cache.clone(), updates);

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

fn build_feed_registry() -> Result<FeedRegistry> {
    let configs = load_feeds_config()?;
    let mut registry = FeedRegistry::new();

    for config in configs {
        register_configured_feed(&mut registry, config);
    }

    Ok(registry)
}

fn register_configured_feed(registry: &mut FeedRegistry, config: FeedConfig) {
    let feed_name = config.name.clone();
    let feed_type = config.feed_type.clone();

    let maybe_feed = match config.feed_type.as_str() {
        "github-pr" => GithubPrFeed::from_config(&config)
            .map(|feed| Arc::new(feed) as Arc<dyn crate::feed::Feed>),
        "shell" => {
            ShellFeed::from_config(&config).map(|feed| Arc::new(feed) as Arc<dyn crate::feed::Feed>)
        }
        unknown => Err(anyhow::anyhow!("unknown feed type `{unknown}`")),
    };

    match maybe_feed {
        Ok(feed) => registry.register(feed),
        Err(err) => registry.register_error(feed_name, feed_type, err.to_string()),
    }
}
