use tauri::{
    menu::{MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use crate::{command, fns};

const MENU_ID_REFRESH_FEEDS: &str = "refresh-feeds";
const MENU_ID_QUIT: &str = "quit";
const PANEL_PADDING_TOP: f64 = 6.0;

pub fn create(app_handle: &AppHandle) -> tauri::Result<TrayIcon> {
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    let refresh_item = MenuItem::with_id(
        app_handle,
        MENU_ID_REFRESH_FEEDS,
        "Refresh feeds",
        true,
        None::<&str>,
    )?;
    let quit_item =
        MenuItem::with_id(app_handle, MENU_ID_QUIT, "Quit Cortado", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app_handle)?;
    let tray_menu =
        tauri::menu::Menu::with_items(app_handle, &[&refresh_item, &separator, &quit_item])?;

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
        MENU_ID_QUIT => {
            app_handle.exit(0);
        }
        MENU_ID_REFRESH_FEEDS => {
            let app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = command::refresh_feeds(app).await {
                    eprintln!("failed refreshing feeds from tray action: {err}");
                }
            });
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
