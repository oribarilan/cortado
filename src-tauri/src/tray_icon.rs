use std::sync::Mutex;

use tauri::AppHandle;

use crate::feed::StatusKind;

/// Base tray icon PNG (22x22, black on transparent, template style).
const BASE_ICON_PNG: &[u8] = include_bytes!("../icons/tray.png");

const ICON_SIZE: u32 = 22;
const DOT_RADIUS: f32 = 3.0;
const DOT_CX: f32 = 17.0;
const DOT_CY: f32 = 17.0;
const RING_WIDTH: f32 = 1.5;

/// Menubar background colors for the dot ring (separates dot from icon).
const MENUBAR_BG_DARK: [u8; 4] = [45, 45, 45, 255];
const MENUBAR_BG_LIGHT: [u8; 4] = [240, 240, 240, 255];

/// Tracks the last rendered state to skip redundant icon updates.
static LAST_STATE: Mutex<Option<(StatusKind, bool)>> = Mutex::new(None);

/// RGBA dot color for each status kind (approximations of the OKLCH tokens).
fn dot_color(kind: StatusKind) -> [u8; 4] {
    match kind {
        StatusKind::AttentionNegative => [232, 78, 78, 255],
        StatusKind::Waiting => [215, 170, 55, 255],
        StatusKind::Running => [85, 140, 245, 255],
        StatusKind::AttentionPositive => [80, 200, 130, 255],
        StatusKind::Idle => [155, 155, 160, 255],
    }
}

/// Updates the menubar tray icon to reflect the global rollup status.
///
/// - `Idle` → pristine template icon (auto light/dark, no dot).
/// - Non-idle → composited icon with colored status dot (template mode off).
pub fn update_tray_status(app_handle: &AppHandle, status: StatusKind) {
    let Some(tray) = app_handle.tray_by_id("tray") else {
        return;
    };

    if status == StatusKind::Idle {
        // Skip if already idle.
        {
            let last = LAST_STATE.lock().unwrap();
            if matches!(*last, Some((StatusKind::Idle, _))) {
                return;
            }
        }

        if let Ok(icon) = tauri::image::Image::from_bytes(BASE_ICON_PNG) {
            let _ = tray.set_icon(Some(icon));
            let _ = tray.set_icon_as_template(true);
        }
        *LAST_STATE.lock().unwrap() = Some((StatusKind::Idle, false));
        return;
    }

    let is_dark = is_macos_dark_mode();

    // Skip if nothing changed.
    {
        let last = LAST_STATE.lock().unwrap();
        if *last == Some((status, is_dark)) {
            return;
        }
    }

    let rgba = compose_status_icon(status, is_dark);
    let icon = tauri::image::Image::new_owned(rgba, ICON_SIZE, ICON_SIZE);
    let _ = tray.set_icon_as_template(false);
    let _ = tray.set_icon(Some(icon));

    *LAST_STATE.lock().unwrap() = Some((status, is_dark));
}

/// Composes the base icon with theme tinting and a colored status dot.
fn compose_status_icon(status: StatusKind, is_dark: bool) -> Vec<u8> {
    let base =
        tauri::image::Image::from_bytes(BASE_ICON_PNG).expect("failed to decode base tray icon");
    let mut pixels = base.rgba().to_vec();

    // Tint for menubar theme.
    if is_dark {
        tint_icon(&mut pixels, 255, 255, 255, 0.85);
    } else {
        tint_icon(&mut pixels, 0, 0, 0, 0.78);
    }

    // Ring: menubar-colored circle slightly larger than the dot.
    let ring_bg = if is_dark {
        MENUBAR_BG_DARK
    } else {
        MENUBAR_BG_LIGHT
    };
    draw_filled_circle(
        &mut pixels,
        ICON_SIZE,
        DOT_CX,
        DOT_CY,
        DOT_RADIUS + RING_WIDTH,
        ring_bg,
    );

    // Status dot.
    draw_filled_circle(
        &mut pixels,
        ICON_SIZE,
        DOT_CX,
        DOT_CY,
        DOT_RADIUS,
        dot_color(status),
    );

    pixels
}

/// Recolors all non-transparent pixels and scales their alpha.
fn tint_icon(pixels: &mut [u8], r: u8, g: u8, b: u8, opacity_scale: f32) {
    for chunk in pixels.chunks_exact_mut(4) {
        if chunk[3] > 0 {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = (chunk[3] as f32 * opacity_scale) as u8;
        }
    }
}

/// Draws an anti-aliased filled circle into an RGBA pixel buffer.
fn draw_filled_circle(
    pixels: &mut [u8],
    width: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: [u8; 4],
) {
    let min_x = (cx - radius - 1.0).max(0.0) as u32;
    let max_x = ((cx + radius + 1.0) as u32).min(width - 1);
    let min_y = (cy - radius - 1.0).max(0.0) as u32;
    let max_y = ((cy + radius + 1.0) as u32).min(width - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius + 0.5 {
                continue;
            }

            // Anti-alias the edge over a 1px band.
            let coverage = (radius + 0.5 - dist).clamp(0.0, 1.0);
            let idx = ((y * width + x) * 4) as usize;

            if coverage >= 1.0 {
                pixels[idx] = color[0];
                pixels[idx + 1] = color[1];
                pixels[idx + 2] = color[2];
                pixels[idx + 3] = color[3];
            } else {
                // Alpha-blend with existing pixel.
                let src_a = (color[3] as f32 / 255.0) * coverage;
                let dst_a = pixels[idx + 3] as f32 / 255.0;
                let out_a = src_a + dst_a * (1.0 - src_a);
                if out_a > 0.0 {
                    let inv = 1.0 / out_a;
                    pixels[idx] = ((color[0] as f32 * src_a
                        + pixels[idx] as f32 * dst_a * (1.0 - src_a))
                        * inv) as u8;
                    pixels[idx + 1] = ((color[1] as f32 * src_a
                        + pixels[idx + 1] as f32 * dst_a * (1.0 - src_a))
                        * inv) as u8;
                    pixels[idx + 2] = ((color[2] as f32 * src_a
                        + pixels[idx + 2] as f32 * dst_a * (1.0 - src_a))
                        * inv) as u8;
                    pixels[idx + 3] = (out_a * 255.0) as u8;
                }
            }
        }
    }
}

/// Detects macOS dark mode by checking the system appearance preference.
fn is_macos_dark_mode() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_colors_are_opaque() {
        for kind in [
            StatusKind::AttentionNegative,
            StatusKind::Waiting,
            StatusKind::Running,
            StatusKind::AttentionPositive,
            StatusKind::Idle,
        ] {
            assert_eq!(
                dot_color(kind)[3],
                255,
                "{kind:?} dot should be fully opaque"
            );
        }
    }

    #[test]
    fn tint_preserves_transparent_pixels() {
        let mut pixels = vec![0, 0, 0, 0, 10, 10, 10, 200];
        tint_icon(&mut pixels, 255, 255, 255, 0.5);
        // Transparent pixel stays untouched.
        assert_eq!(pixels[0..4], [0, 0, 0, 0]);
        // Opaque pixel is tinted.
        assert_eq!(pixels[4], 255);
        assert_eq!(pixels[5], 255);
        assert_eq!(pixels[6], 255);
        assert_eq!(pixels[7], 100);
    }

    #[test]
    fn draw_circle_center_pixel_is_filled() {
        let size = 10u32;
        let mut pixels = vec![0u8; (size * size * 4) as usize];
        let color = [255, 0, 0, 255];
        draw_filled_circle(&mut pixels, size, 5.0, 5.0, 2.0, color);

        // Center pixel (5, 5) should be filled.
        let idx = ((5 * size + 5) * 4) as usize;
        assert_eq!(pixels[idx..idx + 4], color);
    }

    #[test]
    fn draw_circle_far_pixel_is_untouched() {
        let size = 10u32;
        let mut pixels = vec![0u8; (size * size * 4) as usize];
        draw_filled_circle(&mut pixels, size, 5.0, 5.0, 2.0, [255, 0, 0, 255]);

        // Corner pixel (0, 0) should be untouched.
        assert_eq!(pixels[0..4], [0, 0, 0, 0]);
    }

    #[test]
    fn compose_produces_correct_buffer_size() {
        let rgba = compose_status_icon(StatusKind::AttentionNegative, true);
        assert_eq!(
            rgba.len(),
            (ICON_SIZE * ICON_SIZE * 4) as usize,
            "output should be {w}x{w} RGBA",
            w = ICON_SIZE
        );
    }

    #[test]
    fn compose_dark_vs_light_differ() {
        let dark = compose_status_icon(StatusKind::Waiting, true);
        let light = compose_status_icon(StatusKind::Waiting, false);
        assert_ne!(dark, light, "dark and light icons should differ");
    }

    #[test]
    fn compose_different_statuses_differ() {
        let neg = compose_status_icon(StatusKind::AttentionNegative, true);
        let pos = compose_status_icon(StatusKind::AttentionPositive, true);
        assert_ne!(
            neg, pos,
            "different statuses should produce different icons"
        );
    }
}
