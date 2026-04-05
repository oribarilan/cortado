# Task: Tray icon global rollup indicator

## Goal

The tray icon should visually express the global rolled-up status kind across all feeds via a colored dot overlay on the existing icon.

## Context

Feed-level rollup (dot in feed header) is already implemented. This task adds the final level: all feeds → tray icon.

The existing tray icon uses `icon_as_template(true)`, which means macOS controls the tinting (monochrome, adapts to light/dark menubar). Template icons cannot have color — the OS strips it.

## Approach

1. **Disable template mode** — use `icon_as_template(false)` so we control the pixel data.
2. **Generate icon with dot overlay** — composite a small colored dot (matching the status kind color) in the corner of the base icon. Use raw RGBA pixel data via `tauri::image::Image::new_owned()`.
3. **Handle light/dark theme** — the base icon needs different tints for light vs dark menubar. Listen for theme changes and regenerate.
4. **Swap icon on snapshot update** — compute global rollup from `FeedSnapshotCache`, generate the composited icon, call `tray.set_icon()`.

## API Notes (Tauri 2.5.1)

- `app_handle.tray_by_id("tray")` — retrieve the tray icon at runtime
- `tray.set_icon(Some(image))` — swap the icon
- `Image::new_owned(rgba_bytes, width, height)` — create from raw RGBA
- Base icon: `src-tauri/icons/tray.png` (24×24, RGBA)

## Design Decisions

- Global rollup uses same highest-kind-wins algorithm as feed and activity rollup
- Errored feeds contribute Idle
- Retained activities contribute Idle
- When everything is Idle, tray shows the base icon with an idle (gray) dot or no dot

## Open Questions

- Dot placement: bottom-right corner? How many pixels?
- Should Idle show no dot at all (cleaner) or a gray dot (consistent)?
- Theme detection mechanism on macOS — `NSAppearance` observation or Tauri API?

## Related Files

- `src-tauri/src/panel.rs` — tray icon creation, `TrayIconBuilder`
- `src-tauri/src/main.rs` — app setup, snapshot update loop
- `src-tauri/src/feed/mod.rs` — `StatusKind`, `FeedSnapshot`
- `src-tauri/icons/tray.png` — base icon (24×24)

## Acceptance Criteria

- [ ] Tray icon shows a colored dot matching the global rolled-up status kind
- [ ] Dot color updates when feed snapshots change
- [ ] Icon adapts to light/dark menubar theme
- [ ] Reduced-motion users see a static dot (no pulse in tray)
- [ ] `just check` passes

## Dependencies

- Feed-level rollup (done)
- Semantic status kinds (done)
