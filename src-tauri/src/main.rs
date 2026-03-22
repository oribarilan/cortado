// Prevent additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod feed;
mod fns;
mod tray;

use std::sync::Arc;

use anyhow::Result;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::feed::{
    config::{load_feeds_config, FeedConfig},
    github_pr::GithubPrFeed,
    shell::ShellFeed,
    FeedRegistry,
};

fn main() {
    let feed_registry =
        build_feed_registry().unwrap_or_else(|err| panic!("failed to initialize feeds: {err}"));

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(feed_registry)))
        .invoke_handler(tauri::generate_handler![command::init, command::list_feeds])
        .plugin(tauri_nspanel::init())
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let app_handle = app.app_handle();

            tray::create(app_handle)?;

            Ok(())
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
