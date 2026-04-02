#![allow(deprecated)]

use popover::macos::popover::{PopoverConfig, PopoverView};
use system_notification::WorkspaceListener;

use tauri::{Emitter, Listener, LogicalPosition, LogicalSize, Manager};
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

pub fn swizzle_to_menubar_panel(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main") else {
        eprintln!("failed to initialize panel: missing `main` webview window");
        return;
    };

    let panel_delegate = panel_delegate!(SpotlightPanelDelegate {
        window_did_resign_key
    });

    let handle = window.app_handle().clone();

    panel_delegate.set_listener(Box::new(move |delegate_name: String| {
        if delegate_name.as_str() == "window_did_resign_key" {
            let _ = handle.emit("menubar_panel_did_resign_key", ());
        }
    }));

    let panel = match window.to_panel() {
        Ok(panel) => panel,
        Err(err) => {
            eprintln!("failed to convert window to panel: {err}");
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

pub fn setup_menubar_panel_listeners(app_handle: &tauri::AppHandle) {
    fn hide_menubar_panel(app_handle: tauri::AppHandle) {
        if check_menubar_frontmost() {
            return;
        }

        let panel = match app_handle.get_webview_panel("main") {
            Ok(panel) => panel,
            Err(err) => {
                eprintln!("cannot hide panel: {err:?}");
                return;
            }
        };

        panel.order_out(None);
    }

    let handle = app_handle.clone();

    app_handle.listen("menubar_panel_did_resign_key", move |_| {
        hide_menubar_panel(handle.clone());
    });

    app_handle.listen_workspace(
        "NSWorkspaceDidActivateApplicationNotification",
        hide_menubar_panel,
    );

    app_handle.listen_workspace(
        "NSWorkspaceActiveSpaceDidChangeNotification",
        hide_menubar_panel,
    );
}

/// Sets up the panel's visual appearance: native vibrancy material and popover
/// chrome (rounded corners, arrow, border).
///
/// All AppKit work is dispatched to the main thread in a single closure so that
/// autoreleased NSColor objects stay alive for the PopoverView that references
/// them in `drawRect:`.
pub fn update_menubar_appearance(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main") else {
        eprintln!("failed to update panel appearance: missing `main` webview window");
        return;
    };

    let win = window.clone();
    let add_material = std::env::var_os("CORTADO_DISABLE_NATIVE_PANEL_MATERIAL").is_none();

    let result = window.app_handle().run_on_main_thread(move || {
        let handle: id = match win.ns_window() {
            Ok(h) => h as _,
            Err(err) => {
                eprintln!("failed to update panel appearance: {err}");
                return;
            }
        };

        let content_view: id = unsafe { msg_send![handle, contentView] };
        if content_view == nil {
            eprintln!("failed to update panel appearance: missing contentView");
            return;
        }

        // Native vibrancy behind the webview content.
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
                eprintln!("failed to add native panel material: blur view allocation failed");
            }
        }

        // Popover chrome (rounded corners, arrow, border).
        let content_frame: NSRect = unsafe { msg_send![handle, frame] };

        let mut config = PopoverConfig::default();

        let bg: id = unsafe { msg_send![class!(NSColor), windowBackgroundColor] };
        let bg: id = unsafe { msg_send![bg, colorWithAlphaComponent: 0.42] };
        // PopoverView stores raw id pointers without retaining.
        // The autoreleased color would be freed before drawRect: uses it.
        let _: id = unsafe { msg_send![bg, retain] };

        config.background_color = bg;
        config.border_width = 1.0;
        config.arrow_position = content_frame.size.width / 2.0;

        let view = PopoverView::new(config);

        let frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(content_frame.size.width, content_frame.size.height),
        );

        view.set_frame(frame);
        view.set_parent(content_view);
        view.set_autoresizing();

        let _: () = unsafe { msg_send![handle, setAnimationBehavior: 4_isize] };
    });

    if let Err(err) = result {
        eprintln!("failed to update panel appearance: {err}");
    }
}

pub fn position_panel_at_menubar_icon(
    app_handle: &tauri::AppHandle,
    icon_position: LogicalPosition<f64>,
    icon_size: LogicalSize<f64>,
    padding_top: f64,
) {
    let Some(window) = app_handle.get_webview_window("main") else {
        eprintln!("cannot position panel: missing `main` webview window");
        return;
    };

    let Some(monitor) = monitor::get_monitor_with_cursor() else {
        eprintln!("cannot position panel: monitor with cursor not found");
        return;
    };

    let scale_factor = monitor.scale_factor();

    let monitor_pos = monitor.position().to_logical::<f64>(scale_factor);

    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);

    let menubar_height = menubar::get_menubar().height();

    let handle: id = match window.ns_window() {
        Ok(handle) => handle as _,
        Err(err) => {
            eprintln!("cannot position panel: failed to access ns_window: {err}");
            return;
        }
    };

    let mut win_frame: NSRect = unsafe { msg_send![handle, frame] };

    // Size panel to 40% of screen height.
    win_frame.size.height = (monitor_size.height * 0.4).round();

    win_frame.origin.y =
        (monitor_pos.y + monitor_size.height) - menubar_height - win_frame.size.height;

    win_frame.origin.y -= padding_top * scale_factor;

    win_frame.origin.x = icon_position.x + icon_size.width / 2.0 - win_frame.size.width / 2.0;

    let _: () = unsafe { msg_send![handle, setFrame: win_frame display: NO] };
}

pub fn toggle_menubar_panel(
    app_handle: &tauri::AppHandle,
    icon_position: LogicalPosition<f64>,
    icon_size: LogicalSize<f64>,
    padding_top: f64,
) {
    let panel = match app_handle.get_webview_panel("main") {
        Ok(panel) => panel,
        Err(err) => {
            eprintln!("cannot toggle panel: {err:?}");
            return;
        }
    };

    if panel.is_visible() {
        panel.order_out(None);
        return;
    }

    position_panel_at_menubar_icon(app_handle, icon_position, icon_size, padding_top);

    let Some(window) = app_handle.get_webview_window("main") else {
        eprintln!("cannot toggle panel: missing `main` webview window");
        return;
    };
    let Some(monitor_with_cursor) = monitor::get_monitor_with_cursor() else {
        eprintln!("cannot toggle panel: monitor with cursor not found");
        return;
    };

    if let Ok(Some(window_monitor)) = window.current_monitor() {
        let is_window_in_monitor_with_cursor =
            window_monitor.position().x as f64 == monitor_with_cursor.position().x;

        if is_window_in_monitor_with_cursor {
            let _ = app_handle.emit("menubar_panel_will_show", ());
            panel.show();
        }

        return;
    }

    let _ = app_handle.emit("menubar_panel_will_show", ());
    panel.show();
}

fn app_pid() -> i32 {
    let process_info: id = unsafe { msg_send![class!(NSProcessInfo), processInfo] };

    let pid: i32 = unsafe { msg_send![process_info, processIdentifier] };

    pid
}

fn get_frontmost_app_pid() -> i32 {
    let workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };

    let frontmost_application: id = unsafe { msg_send![workspace, frontmostApplication] };

    let pid: i32 = unsafe { msg_send![frontmost_application, processIdentifier] };

    pid
}

pub fn check_menubar_frontmost() -> bool {
    get_frontmost_app_pid() == app_pid()
}
