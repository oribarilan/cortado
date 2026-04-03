#![allow(deprecated)]

use system_notification::WorkspaceListener;

use tauri::{Emitter, Listener, Manager};
use tauri_nspanel::{
    cocoa::{
        appkit::{
            NSMainMenuWindowLevel, NSView, NSViewHeightSizable, NSViewWidthSizable,
            NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
            NSVisualEffectView, NSWindowCollectionBehavior, NSWindowOrderingMode,
        },
        base::{id, nil},
        foundation::{NSPoint, NSRect, NSSize},
    },
    objc::{class, msg_send, runtime::NO, sel, sel_impl},
    panel_delegate, ManagerExt, WebviewWindowExt,
};

#[allow(non_upper_case_globals)]
const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;

/// Converts the `main-screen` webview window to a floating NSPanel with its
/// own delegate, level, collection behavior, and style mask.
pub fn swizzle_to_main_screen_panel(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main-screen") else {
        eprintln!("failed to initialize main screen panel: missing `main-screen` webview window");
        return;
    };

    let panel_delegate = panel_delegate!(MainScreenPanelDelegate {
        window_did_resign_key
    });

    let handle = window.app_handle().clone();

    panel_delegate.set_listener(Box::new(move |delegate_name: String| {
        if delegate_name.as_str() == "window_did_resign_key" {
            let _ = handle.emit("main_screen_panel_did_resign_key", ());
        }
    }));

    let panel = match window.to_panel() {
        Ok(panel) => panel,
        Err(err) => {
            eprintln!("failed to convert main screen window to panel: {err}");
            return;
        }
    };

    panel.set_level(NSMainMenuWindowLevel + 1);

    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );

    panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

    panel.set_delegate(panel_delegate);
}

/// Registers listeners that auto-hide the main screen panel on resign-key,
/// app activation change, and desktop/space change.
pub fn setup_main_screen_panel_listeners(app_handle: &tauri::AppHandle) {
    fn hide_main_screen_panel(app_handle: tauri::AppHandle) {
        if crate::fns::check_menubar_frontmost() {
            return;
        }

        let panel = match app_handle.get_webview_panel("main-screen") {
            Ok(panel) => panel,
            Err(err) => {
                eprintln!("cannot hide main screen panel: {err:?}");
                return;
            }
        };

        panel.order_out(None);
    }

    let handle = app_handle.clone();

    app_handle.listen("main_screen_panel_did_resign_key", move |_| {
        hide_main_screen_panel(handle.clone());
    });

    app_handle.listen_workspace(
        "NSWorkspaceDidActivateApplicationNotification",
        hide_main_screen_panel,
    );

    app_handle.listen_workspace(
        "NSWorkspaceActiveSpaceDidChangeNotification",
        |app_handle| {
            let panel = match app_handle.get_webview_panel("main-screen") {
                Ok(panel) => panel,
                Err(_) => return,
            };
            panel.order_out(None);
        },
    );
}

/// Sets up the panel's visual appearance: native vibrancy material and rounded
/// corners. Unlike the menubar panel, no popover chrome (arrow) is added.
pub fn update_main_screen_appearance(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main-screen") else {
        eprintln!("failed to update main screen appearance: missing `main-screen` webview window");
        return;
    };

    let add_material = std::env::var_os("CORTADO_DISABLE_NATIVE_PANEL_MATERIAL").is_none();
    let win = window.clone();

    let result = window.app_handle().run_on_main_thread(move || {
        let handle: id = match win.ns_window() {
            Ok(h) => h as _,
            Err(err) => {
                eprintln!("failed to update main screen appearance: {err}");
                return;
            }
        };

        let content_view: id = unsafe { msg_send![handle, contentView] };
        if content_view == nil {
            eprintln!("failed to update main screen appearance: missing contentView");
            return;
        }

        if add_material {
            let content_bounds: NSRect = unsafe { msg_send![content_view, bounds] };

            let blur_view: id = unsafe {
                NSVisualEffectView::initWithFrame_(NSVisualEffectView::alloc(nil), content_bounds)
            };

            if blur_view != nil {
                unsafe {
                    blur_view.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
                    blur_view.setBlendingMode_(NSVisualEffectBlendingMode::BehindWindow);
                    blur_view.setState_(NSVisualEffectState::Active);
                    blur_view.setMaterial_(NSVisualEffectMaterial::Popover);

                    let _: () = msg_send![
                        content_view,
                        addSubview: blur_view
                        positioned: NSWindowOrderingMode::NSWindowBelow
                        relativeTo: 0 as id
                    ];

                    let clear_color: id = msg_send![class!(NSColor), clearColor];
                    let _: () = msg_send![handle, setBackgroundColor: clear_color];
                    let _: () = msg_send![handle, setOpaque: NO];
                }
            } else {
                eprintln!("failed to add native main screen material: blur view allocation failed");
            }
        }

        let _: () = unsafe { msg_send![handle, setAnimationBehavior: 4_isize] };
    });

    if let Err(err) = result {
        eprintln!("failed to update main screen appearance: {err}");
    }
}

fn center_on_active_monitor(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main-screen") else {
        return;
    };

    let Some(monitor) = monitor::get_monitor_with_cursor() else {
        return;
    };

    let scale = monitor.scale_factor();
    let monitor_pos = monitor.position().to_logical::<f64>(scale);
    let monitor_size = monitor.size().to_logical::<f64>(scale);

    let handle: id = match window.ns_window() {
        Ok(h) => h as _,
        Err(_) => return,
    };

    let win_frame: NSRect = unsafe { msg_send![handle, frame] };

    let x = monitor_pos.x + (monitor_size.width - win_frame.size.width) / 2.0;
    let y = monitor_pos.y + (monitor_size.height - win_frame.size.height) / 2.0;

    let new_frame = NSRect::new(
        NSPoint::new(x, y),
        NSSize::new(win_frame.size.width, win_frame.size.height),
    );

    let _: () = unsafe { msg_send![handle, setFrame: new_frame display: NO] };
}

/// Toggles the main screen panel: if hidden, centers on the active monitor
/// and shows; if visible, hides.
pub fn toggle_main_screen_panel(app_handle: &tauri::AppHandle) {
    let panel = match app_handle.get_webview_panel("main-screen") {
        Ok(panel) => panel,
        Err(err) => {
            eprintln!("cannot toggle main screen panel: {err:?}");
            return;
        }
    };

    if panel.is_visible() {
        panel.order_out(None);
        return;
    }

    center_on_active_monitor(app_handle);

    let _ = app_handle.emit("main_screen_panel_will_show", ());
    panel.show();
}

/// Shows the main screen panel if it is not already visible.
pub fn show_main_screen_panel(app_handle: &tauri::AppHandle) {
    let panel = match app_handle.get_webview_panel("main-screen") {
        Ok(panel) => panel,
        Err(err) => {
            eprintln!("cannot show main screen panel: {err:?}");
            return;
        }
    };

    if panel.is_visible() {
        return;
    }

    center_on_active_monitor(app_handle);

    let _ = app_handle.emit("main_screen_panel_will_show", ());
    panel.show();
}
