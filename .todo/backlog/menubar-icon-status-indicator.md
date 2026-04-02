---
status: pending
---

# Menubar Icon Status Indicator — Exploration & Alternatives

## Problem

The tray icon is static — it doesn't reflect the global rollup status. Users must click the tray or open the panel to know if anything needs attention. The menubar icon should serve as a passive, glanceable signal of the highest-priority status across all feeds.

## Current State

- Tray icon: `src-tauri/icons/tray.png` (22x22 RGBA), loaded in `panel.rs`
- Uses `icon_as_template(true)` — macOS auto-tints for light/dark menubar (monochrome only)
- Per-activity rollup exists (`rollup_for_activity` in `feed/mod.rs`)
- No feed-level or global rollup is implemented yet
- Status kinds: `AttentionNegative` (red) > `Waiting` (yellow) > `Running` (blue) > `AttentionPositive` (green) > `Idle` (gray)

## Rollup Algorithm

Same highest-kind-wins at every level:

```
Fields -> Activity dot -> Feed header -> Tray icon (global)
```

- Retained activities contribute `Idle`
- Errored feeds contribute `Idle`
- When everything is `Idle`, tray shows neutral/no indicator

---

## Alternatives

### Alternative A: Colored Dot Overlay (Runtime Compositing)

**How it looks:** A small colored circle (4-6px) composited onto the bottom-right corner of the existing icon. Similar to Microsoft Teams' status dot on the user avatar, or Slack's online status dot.

**How it works:**
1. Disable `icon_as_template(true)` so we control pixel data and can use color
2. At runtime, composite a colored dot onto the base icon in Rust using raw RGBA manipulation
3. Call `tray.set_icon()` whenever the global rollup changes
4. Handle light/dark menubar theme detection to tint the base icon appropriately

**Implementation:**
```rust
// Pseudo-code
fn generate_tray_icon(base: &[u8], status: StatusKind, is_dark: bool) -> Vec<u8> {
    let mut pixels = tint_for_theme(base, is_dark);
    let dot_color = status_to_rgba(status);
    draw_circle(&mut pixels, 22, 22, /* center */ (17, 17), /* radius */ 3, dot_color);
    pixels
}
```

**Dependencies:** None new — raw RGBA pixel manipulation, no image crate needed for a simple circle.

**Complexity:** Medium
- RGBA compositing: simple (draw filled circle into pixel buffer)
- Theme detection: medium (need `NSAppearance` observation via `objc` or polling)
- Retina support: need @2x variant (44x44) for HiDPI displays

**Pros:**
- Dynamic — single base asset, colors computed at runtime
- Matches well-known pattern (Teams, Slack, Discord)
- Compact — no extra menubar space
- Most expressive — can show any status color

**Cons:**
- Loses template image auto-tinting — must handle light/dark manually
- Dot may be small at 22px (but 44px @2x helps)
- Animation (e.g., pulse for `Running`) not feasible without rapid icon swaps

---

### Alternative B: Pre-Rendered Icon Set (Static Assets)

**How it looks:** Identical to Alternative A visually — dot overlay in the corner. But the icons are pre-made PNG files rather than generated at runtime.

**How it works:**
1. Create icon variants: `tray.png`, `tray-attention-neg.png`, `tray-waiting.png`, `tray-running.png`, `tray-attention-pos.png` (x2 for light/dark = 10 files, x2 for @2x = 20 files)
2. Load all at startup with `include_bytes!`
3. Swap via `tray.set_icon()` on status change

**Complexity:** Low (code) / Medium (design)
- Code is trivial — just `match status { ... }` and `set_icon`
- Design work: creating 10-20 pixel-perfect icon variants
- Theme detection still needed to pick light vs dark variant

**Pros:**
- Simplest code path
- Pixel-perfect control over every variant
- No runtime image manipulation

**Cons:**
- Many assets to maintain (10-20 PNGs)
- Adding a new status kind requires new icon files
- Still need theme detection for light/dark variants
- Inflexible — can't adjust dot size/position without regenerating all assets

---

### Alternative C: Separate Dot Tray Item

**How it looks:** A small colored dot appears as a separate, independent menubar item immediately to the left of the main icon. The main icon stays as a template image.

**How it works:**
1. Create a second `TrayIcon` (e.g., `TrayIconBuilder::with_id("status-dot")`)
2. Use a tiny (8x8 or 10x10) colored circle as its icon
3. Update or hide it based on global rollup

**Complexity:** Low
- Very simple code
- No theme detection needed for the base icon (stays template)
- Dot icon is non-template, pure color

**Pros:**
- Base icon keeps template mode — zero theme handling
- Very simple implementation
- Dot can be hidden entirely when `Idle`

**Cons:**
- Takes extra menubar space (~10px)
- Looks disconnected — not a cohesive single icon
- Unusual pattern — no major macOS apps do this
- User might not associate the dot with Cortado
- Two clickable tray items could confuse right-click menu

---

### Alternative D: Icon Swap (Full Icon Per Status)

**How it looks:** The entire icon changes shape or fill to indicate status. For example: hollow glass = idle, filled glass = attention, glass with exclamation = negative.

**How it works:**
1. Design distinct icon variants for each status kind
2. All stay as template images (monochrome, auto-tinted)
3. Swap via `tray.set_icon()` on status change

**Complexity:** Low (code) / High (design)
- Code is trivial
- Design challenge: creating 5 distinct but recognizable variants of the cortado glass at 22x22
- Must be legible in both light and dark menubar

**Pros:**
- Keeps template image mode — perfect light/dark handling
- No color in menubar — respects macOS HIG strictly
- No theme detection code needed

**Cons:**
- Monochrome only — less glanceable than color
- Hard to design 5 meaningfully different silhouettes at 22px
- Users must learn the icon vocabulary
- Loses the instant "red = bad, green = good" recognition

---

### Alternative E: NSStatusItem Custom Subview (Native)

**How it looks:** Most polished — a native colored dot rendered as a subview on top of the template icon, with potential for smooth animations (pulse for `Running`).

**How it works:**
1. Via `objc` crate, access the `NSStatusItem`'s button
2. Add a small `NSView` subview with a colored `CALayer` circle
3. Update the layer's background color on status change
4. Optionally add `CABasicAnimation` for pulse effect

**Complexity:** High
- Requires unsafe Objective-C interop via `objc`/`objc2` crate
- Fragile — depends on internal `NSStatusBarButton` structure
- The app already uses `objc` for NSPanel swizzling, so the pattern exists
- Needs careful lifecycle management (subview must survive icon updates)

**Pros:**
- Best visual quality — native rendering, smooth animations
- Base icon stays as template (auto light/dark)
- Most like how native macOS apps would do it
- Can animate (pulse, fade) natively

**Cons:**
- Most complex implementation
- Fragile — Apple could change internal view hierarchy
- Hard to test
- May conflict with Tauri's tray icon management

---

## Recommendation Matrix

| Criterion | A: Runtime Dot | B: Pre-Rendered | C: Separate Item | D: Icon Swap | E: Native Subview |
|-----------|:-:|:-:|:-:|:-:|:-:|
| Visual quality | Good | Best (pixel-perfect) | Poor | Good | Best |
| Glanceability (color) | Yes | Yes | Yes | No (mono) | Yes |
| Template mode (auto theme) | No | No | Base: yes | Yes | Yes |
| Theme handling needed | Yes | Yes | Dot only | No | No |
| Code complexity | Medium | Low | Low | Low | High |
| Asset maintenance | Low (1 base) | High (10-20 PNGs) | Low | Medium (5 icons) | Low |
| Animation possible | Limited | No | No | No | Yes (native) |
| macOS HIG compliance | Good | Good | Poor | Best | Good |
| Menubar space | None | None | Extra ~10px | None | None |
| Precedent (other apps) | Teams, Slack | Many apps | Rare | Some (Docker) | Native apps |

## Suggested Approach

**Alternative A (Runtime Compositing)** is the recommended starting point:

- Best balance of visual quality, flexibility, and implementation complexity
- Matches the well-known Teams/Slack pattern users already understand
- Single base asset, dynamic colors — easy to maintain
- Theme detection is the main complexity, but manageable

**Fallback:** If theme detection proves too brittle, **Alternative B** (pre-rendered set) achieves the same visual result with more assets but simpler code.

**Future upgrade:** **Alternative E** could be explored later if we want pulse animation for `Running` status, but it's not worth the complexity for v1.

## Open Questions

- Dot placement: bottom-right or top-right corner?
- Dot size: 4px, 5px, or 6px diameter (at 1x)?
- Should `Idle` show a gray dot or no dot at all?
- Should `Running` attempt a slow icon-swap animation (e.g., alternating two frames)?
- @2x Retina: ship both 22px and 44px base icons?

## Related Files

- `src-tauri/src/panel.rs` — tray icon creation, `TrayIconBuilder`
- `src-tauri/src/main.rs` — app setup, snapshot update loop
- `src-tauri/src/feed/mod.rs` — `StatusKind`, `rollup_for_activity`, `FeedSnapshot`
- `src-tauri/icons/tray.png` — base icon (22x22)
- `src-tauri/icons/tray.svg` — source SVG
- `src/shared/tokens.css` — status color definitions

## Visual Showcase

Interactive comparison of all alternatives: `showcases/menubar-status-indicator-showcase.html`

Open in a browser to toggle between status kinds and see each approach in both dark and light menubar themes, with zoomed views for detail.

## Related Backlog Items

- `.todo/backlog/tray-icon-rollup.md` — implementation details for the rollup computation
- `.todo/backlog/status-kind-rollup.md` — three-level rollup architecture
- `.todo/backlog/tray-icon-status-indicator.md` — earlier notes on this feature

## Acceptance Criteria

- [ ] Tray icon visually indicates the global rolled-up status kind
- [ ] Indicator updates when feed snapshots change
- [ ] Icon adapts to light/dark menubar theme
- [ ] `Idle` state is visually distinct from active statuses
- [ ] Reduced-motion users see a static indicator (no pulse)
- [ ] `just check` passes
