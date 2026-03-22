use std::sync::Arc;
use std::{fmt::Write, process::Command};

use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuEvent, MenuItem, NativeIcon, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Wry,
};
use tokio::sync::Mutex;

use crate::feed::{Activity, FeedRegistry, FeedSnapshot, Field, FieldValue, StatusKind};

const MENU_ID_RELOAD: &str = "reload";
const MENU_ID_QUIT: &str = "quit";
const MENU_ID_EMPTY: &str = "empty";
const MENU_ID_ERROR_PREFIX: &str = "feed-error:";
const MENU_ID_FIELD_PREFIX: &str = "field:";
const MENU_ID_HEADING_PREFIX: &str = "heading:";
const MENU_ID_OPEN_PREFIX: &str = "open:";

#[derive(Clone, Copy)]
enum UiStatus {
    Error,
    Warning,
    Pending,
    Success,
    Neutral,
}

pub fn create(app_handle: &AppHandle) -> tauri::Result<TrayIcon> {
    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    let initial_menu = build_tray_menu(app_handle, Vec::new())?;

    TrayIconBuilder::with_id("tray")
        .icon(icon)
        .icon_as_template(true)
        .show_menu_on_left_click(true)
        .menu(&initial_menu)
        .on_menu_event(handle_menu_event)
        .build(app_handle)
}

pub fn refresh_menu(app_handle: &AppHandle, snapshots: &[FeedSnapshot]) -> tauri::Result<()> {
    if let Some(tray) = app_handle.tray_by_id("tray") {
        let menu = build_tray_menu(app_handle, snapshots.to_vec())?;
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

fn handle_menu_event(app_handle: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_ID_QUIT => {
            app_handle.exit(0);
        }
        MENU_ID_RELOAD => {
            let app = app_handle.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(err) = refresh_from_registry(app).await {
                    eprintln!("failed refreshing tray menu: {err}");
                }
            });
        }
        id if id.starts_with(MENU_ID_OPEN_PREFIX) => {
            let encoded = &id[MENU_ID_OPEN_PREFIX.len()..];

            match decode_menu_id_payload(encoded) {
                Some(url) => {
                    if let Err(err) = open_in_browser(&url) {
                        eprintln!("failed opening URL from menu: {err}");
                    }
                }
                None => {
                    eprintln!("failed decoding open URL payload");
                }
            }
        }
        id if id.starts_with(MENU_ID_FIELD_PREFIX)
            || id.starts_with(MENU_ID_ERROR_PREFIX)
            || id.starts_with(MENU_ID_HEADING_PREFIX)
            || id == MENU_ID_EMPTY => {}
        _ => {}
    }
}

async fn refresh_from_registry(app_handle: AppHandle) -> Result<(), String> {
    let registry_state = app_handle
        .try_state::<Arc<Mutex<FeedRegistry>>>()
        .ok_or_else(|| "feed registry state is missing".to_string())?;

    let registry = registry_state.lock().await;
    let snapshots = registry.poll_all().await;

    refresh_menu(&app_handle, &snapshots).map_err(|err| err.to_string())
}

fn build_tray_menu(
    app_handle: &AppHandle,
    snapshots: Vec<FeedSnapshot>,
) -> tauri::Result<Menu<Wry>> {
    let mut top_level: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    if snapshots.is_empty() {
        top_level.push(Box::new(MenuItem::with_id(
            app_handle,
            MENU_ID_EMPTY,
            "No feeds configured",
            false,
            None::<&str>,
        )?));
    } else {
        for (index, feed) in snapshots.into_iter().enumerate() {
            if index > 0 {
                top_level.push(Box::new(PredefinedMenuItem::separator(app_handle)?));
            }

            top_level.push(Box::new(MenuItem::with_id(
                app_handle,
                format!(
                    "{MENU_ID_HEADING_PREFIX}{}",
                    sanitize_id_component(&format!("{}-{}", feed.name, feed.feed_type))
                ),
                feed.name.clone(),
                false,
                None::<&str>,
            )?));

            let mut feed_items = build_feed_section_items(app_handle, feed)?;
            top_level.append(&mut feed_items);
        }
    }

    top_level.push(Box::new(PredefinedMenuItem::separator(app_handle)?));
    top_level.push(Box::new(MenuItem::with_id(
        app_handle,
        MENU_ID_RELOAD,
        "Reload",
        true,
        None::<&str>,
    )?));
    top_level.push(Box::new(MenuItem::with_id(
        app_handle,
        MENU_ID_QUIT,
        "Quit Cortado",
        true,
        None::<&str>,
    )?));

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        top_level.iter().map(|item| item.as_ref()).collect();

    Menu::with_items(app_handle, &refs)
}

fn build_feed_section_items(
    app_handle: &AppHandle,
    feed: FeedSnapshot,
) -> tauri::Result<Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>>> {
    let feed_name = feed.name.clone();
    let feed_type = feed.feed_type.clone();
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    if let Some(error) = &feed.error {
        items.push(Box::new(MenuItem::with_id(
            app_handle,
            format!(
                "{MENU_ID_ERROR_PREFIX}{}",
                sanitize_id_component(&feed.name)
            ),
            format!("Error: {error}"),
            false,
            None::<&str>,
        )?));

        if feed.activities.is_empty() {
            return Ok(items);
        }

        items.push(Box::new(PredefinedMenuItem::separator(app_handle)?));
    }

    if feed.activities.is_empty() {
        items.push(Box::new(MenuItem::with_id(
            app_handle,
            format!(
                "{MENU_ID_EMPTY}:{}",
                sanitize_id_component(&format!("{}-{}", feed.name, feed.feed_type))
            ),
            "No activities",
            false,
            None::<&str>,
        )?));

        return Ok(items);
    }

    for activity in feed.activities {
        let activity_submenu =
            build_activity_submenu(app_handle, &feed_name, &feed_type, activity)?;
        items.push(Box::new(activity_submenu));
    }

    Ok(items)
}

fn build_activity_submenu(
    app_handle: &AppHandle,
    feed_name: &str,
    feed_type: &str,
    activity: Activity,
) -> tauri::Result<Submenu<Wry>> {
    let fields_for_menu = fields_for_activity_menu(feed_type, &activity.fields);
    let symbol = status_symbol_for_activity(&fields_for_menu);
    let title = format!("{symbol} {}", activity.title);
    let open_target = open_target_for_activity(feed_type, &activity);

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    if fields_for_menu.is_empty() {
        items.push(Box::new(MenuItem::with_id(
            app_handle,
            field_item_id(feed_name, &activity.id, "empty"),
            "No fields",
            false,
            None::<&str>,
        )?));
    } else {
        for field in &fields_for_menu {
            let status = field_status_info(field);

            if let Some((summary, ui_status)) = status {
                let icon = status_icon_image(ui_status);

                items.push(Box::new(IconMenuItem::with_id(
                    app_handle,
                    field_item_id(feed_name, &activity.id, &field.name),
                    format!("{}: {} {summary}", field.label, status_symbol(ui_status)),
                    false,
                    Some(icon),
                    None::<&str>,
                )?));
            } else {
                items.push(Box::new(MenuItem::with_id(
                    app_handle,
                    field_item_id(feed_name, &activity.id, &field.name),
                    format!("{}: {}", field.label, format_field_value(field)),
                    false,
                    None::<&str>,
                )?));
            }
        }
    }

    let open_item: Box<dyn tauri::menu::IsMenuItem<tauri::Wry>> = match open_target {
        Some(url) => Box::new(IconMenuItem::with_id_and_native_icon(
            app_handle,
            format!("{MENU_ID_OPEN_PREFIX}{}", encode_menu_id_payload(&url)),
            "Open",
            true,
            Some(NativeIcon::FollowLinkFreestanding),
            None::<&str>,
        )?),
        None => Box::new(IconMenuItem::with_id_and_native_icon(
            app_handle,
            format!(
                "{MENU_ID_HEADING_PREFIX}{}:{}:open-disabled",
                sanitize_id_component(feed_name),
                sanitize_id_component(&activity.id)
            ),
            "Open",
            false,
            Some(NativeIcon::FollowLinkFreestanding),
            None::<&str>,
        )?),
    };

    let mut activity_items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();
    activity_items.push(open_item);

    if !items.is_empty() {
        activity_items.push(Box::new(PredefinedMenuItem::separator(app_handle)?));
    }

    activity_items.extend(items);

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        activity_items.iter().map(|item| item.as_ref()).collect();

    Submenu::with_items(app_handle, title, true, &refs)
}

fn fields_for_activity_menu(feed_type: &str, fields: &[Field]) -> Vec<Field> {
    if feed_type != "github-pr" {
        return fields.to_vec();
    }

    let mut selected = Vec::new();

    for key in ["review", "checks", "mergeable"] {
        if let Some(field) = fields.iter().find(|field| field.name == key) {
            selected.push(field.clone());
        }
    }

    selected
}

fn open_target_for_activity(feed_type: &str, activity: &Activity) -> Option<String> {
    if feed_type == "github-pr" {
        return github_pr_url_for_id(&activity.id);
    }

    activity.fields.iter().find_map(|field| {
        let FieldValue::Url { value } = &field.value else {
            return None;
        };

        if is_http_url(value) {
            Some(value.clone())
        } else {
            None
        }
    })
}

fn github_pr_url_for_id(activity_id: &str) -> Option<String> {
    let trimmed = activity_id.trim();

    if trimmed.is_empty() {
        return None;
    }

    if is_http_url(trimmed) {
        return Some(trimmed.to_string());
    }

    Some(format!(
        "https://github.com/{}",
        trimmed.trim_start_matches('/')
    ))
}

fn encode_menu_id_payload(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);

    for byte in value.bytes() {
        let _ = write!(encoded, "{byte:02x}");
    }

    encoded
}

fn decode_menu_id_payload(value: &str) -> Option<String> {
    if !value.len().is_multiple_of(2) {
        return None;
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);

    for index in (0..value.len()).step_by(2) {
        let byte = u8::from_str_radix(&value[index..index + 2], 16).ok()?;
        bytes.push(byte);
    }

    String::from_utf8(bytes).ok()
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://")
}

fn open_in_browser(url: &str) -> Result<(), String> {
    if !is_http_url(url) {
        return Err("only http/https URLs are supported".to_string());
    }

    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|err| format!("failed to spawn `open`: {err}"))?;

    Ok(())
}

fn field_status_info(field: &Field) -> Option<(String, UiStatus)> {
    let FieldValue::Status { value, severity } = &field.value else {
        return None;
    };

    let status = match severity {
        StatusKind::Error => UiStatus::Error,
        StatusKind::Warning => UiStatus::Warning,
        StatusKind::Pending => UiStatus::Pending,
        StatusKind::Success => UiStatus::Success,
        StatusKind::Neutral => UiStatus::Neutral,
    };

    Some((value.clone(), status))
}

fn status_symbol_for_activity(fields: &[Field]) -> &'static str {
    status_symbol(infer_status(fields))
}

fn infer_status(fields: &[Field]) -> UiStatus {
    let mut has_success = false;
    let mut has_pending = false;
    let mut has_warning = false;

    for field in fields {
        let FieldValue::Status { severity, .. } = &field.value else {
            continue;
        };

        if matches!(severity, StatusKind::Error) {
            return UiStatus::Error;
        }

        if matches!(severity, StatusKind::Warning) {
            has_warning = true;
            continue;
        }

        if matches!(severity, StatusKind::Pending) {
            has_pending = true;
            continue;
        }

        if matches!(severity, StatusKind::Success) {
            has_success = true;
        }
    }

    if has_warning {
        return UiStatus::Warning;
    }

    if has_pending {
        return UiStatus::Pending;
    }

    if has_success {
        return UiStatus::Success;
    }

    UiStatus::Neutral
}

fn status_symbol(status: UiStatus) -> &'static str {
    match status {
        UiStatus::Error => "✕",
        UiStatus::Warning => "⚠︎",
        UiStatus::Pending => "◷",
        UiStatus::Success => "✓",
        UiStatus::Neutral => "•",
    }
}

fn status_icon_image(status: UiStatus) -> Image<'static> {
    let (red, green, blue) = match status {
        UiStatus::Error => (255, 59, 48),
        UiStatus::Warning => (255, 149, 0),
        UiStatus::Pending => (0, 122, 255),
        UiStatus::Success => (52, 199, 89),
        UiStatus::Neutral => (142, 142, 147),
    };

    const SIZE: u32 = 18;
    const RADIUS_SQUARED: i32 = 16;

    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    let center_x = (SIZE as i32) / 2;
    let center_y = (SIZE as i32) / 2;

    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let delta_x = x - center_x;
            let delta_y = y - center_y;

            if delta_x * delta_x + delta_y * delta_y > RADIUS_SQUARED {
                continue;
            }

            let index = ((y as u32 * SIZE + x as u32) * 4) as usize;
            rgba[index] = red;
            rgba[index + 1] = green;
            rgba[index + 2] = blue;
            rgba[index + 3] = 255;
        }
    }

    Image::new_owned(rgba, SIZE, SIZE)
}

fn format_field_value(field: &Field) -> String {
    match &field.value {
        FieldValue::Text { value } => value.clone(),
        FieldValue::Status { value, .. } => value.clone(),
        FieldValue::Number { value } => {
            if value.fract() == 0.0 {
                (*value as i64).to_string()
            } else {
                format!("{value}")
            }
        }
        FieldValue::Url { value } => value.clone(),
    }
}

fn field_item_id(feed_name: &str, activity_id: &str, field_name: &str) -> String {
    format!(
        "{MENU_ID_FIELD_PREFIX}{}:{}:{}",
        sanitize_id_component(feed_name),
        sanitize_id_component(activity_id),
        sanitize_id_component(field_name)
    )
}

fn sanitize_id_component(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            ':' | '/' | '\\' | ' ' | '\t' | '\n' | '\r' => '-',
            _ => c,
        })
        .collect()
}
