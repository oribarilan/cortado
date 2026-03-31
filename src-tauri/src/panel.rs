use tauri::{
    menu::{MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use crate::{command, fns, main_screen};

const MENU_ID_OPEN_APP: &str = "open-app";
const MENU_ID_SETTINGS: &str = "settings";
const MENU_ID_QUIT: &str = "quit";
const PANEL_PADDING_TOP: f64 = 6.0;

pub fn create(app_handle: &AppHandle) -> tauri::Result<TrayIcon> {
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    let version = app_handle.config().version.as_deref().unwrap_or("unknown");
    let version_label = if crate::app_env::is_dev() {
        format!("Cortado Dev v{version}")
    } else {
        format!("Cortado v{version}")
    };
    let version_item =
        MenuItem::with_id(app_handle, "version", &version_label, false, None::<&str>)?;
    let open_app_item =
        MenuItem::with_id(app_handle, MENU_ID_OPEN_APP, "Open App", true, None::<&str>)?;
    let settings_item =
        MenuItem::with_id(app_handle, MENU_ID_SETTINGS, "Settings", true, None::<&str>)?;
    let quit_item =
        MenuItem::with_id(app_handle, MENU_ID_QUIT, "Quit Cortado", true, None::<&str>)?;
    let separator0 = PredefinedMenuItem::separator(app_handle)?;
    let separator = PredefinedMenuItem::separator(app_handle)?;
    let separator2 = PredefinedMenuItem::separator(app_handle)?;
    let tray_menu = tauri::menu::Menu::with_items(
        app_handle,
        &[
            &version_item,
            &separator0,
            &open_app_item,
            &separator,
            &settings_item,
            &separator2,
            &quit_item,
        ],
    )?;

    TrayIconBuilder::with_id("tray")
        .icon(icon)
        .icon_as_template(true)
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_icon_event)
        .build(app_handle)
}

fn handle_menu_event(app_handle: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_ID_OPEN_APP => {
            main_screen::toggle_main_screen_panel(app_handle);
        }
        MENU_ID_QUIT => {
            app_handle.exit(0);
        }
        MENU_ID_SETTINGS => {
            if let Err(err) = command::open_settings(app_handle.clone()) {
                eprintln!("failed opening settings from tray action: {err}");
            }
        }
        _ => {}
    }
}

fn handle_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button_state: MouseButtonState::Up,
        rect,
        ..
    } = event
    {
        let app = tray.app_handle();
        let Some(monitor_with_cursor) = monitor::get_monitor_with_cursor() else {
            eprintln!("cannot handle tray click: monitor with cursor not found");
            return;
        };
        let scale_factor = monitor_with_cursor.scale_factor();

        let icon_position = rect.position.to_logical::<f64>(scale_factor);
        let icon_size = rect.size.to_logical::<f64>(scale_factor);

        fns::toggle_menubar_panel(app, icon_position, icon_size, PANEL_PADDING_TOP);
    }
}
