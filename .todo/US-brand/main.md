# US-brand: App Identity & Icons

## Theme

Give Cortado a visual identity. The app needs two distinct icon assets — a **menubar icon** and an **app icon** — both with a coffee/cortado motif. This story covers designing, generating, and wiring both.

## How It Works

### Menubar icon (tray icon)

The menubar icon is what users see 100% of the time in the macOS menu bar. It's loaded at runtime from `src-tauri/icons/tray.png`.

**Key constraints:**
- Must be a **template image** — macOS auto-tints it to match the menu bar appearance (dark on light bar, light on dark bar). This is set via `icon_as_template(true)` in `panel.rs`.
- A template image is a **monochrome silhouette with alpha transparency**. Only the alpha channel matters — macOS ignores the RGB values and applies its own tint. Use **black (#000000) for the shape, fully transparent for the background**.
- Recommended size: **22x22 points**, which means a **22x22 @1x** and **44x44 @2x** PNG (currently we ship a single 24x24 — close enough but non-standard).
- Must be instantly recognizable at a tiny size. Fine detail is lost. Simple shapes work best.
- Apple's HIG recommends the icon be ~18px tall within the 22pt canvas (small top/bottom padding).

**What lives where:**
- Source SVG: `src-tauri/icons/tray.svg` (new, for editability)
- Runtime asset: `src-tauri/icons/tray.png` (loaded by `panel.rs`)

### App icon (bundle icon)

The app icon appears in: Finder, Spotlight, the About dialog, the DMG, Activity Monitor, and (if ever enabled) the Dock. It's baked into the app bundle at build time.

**Key constraints:**
- Must be a **1024x1024 PNG** source with transparency, from which `tauri icon` generates all platform variants (`.icns`, `.ico`, iOS/Android assets, various sizes).
- Can be **full-color and detailed** — unlike the tray icon, this has room for gradients, shadows, and depth.
- macOS app icons sit on a rounded-rect "squircle" mask automatically, but the icon source should still be a full square with the design centered.
- All generated files live in `src-tauri/icons/` and are referenced by `tauri.conf.json`'s `bundle.icon` array.

**What lives where:**
- Source SVG: `src-tauri/icons/app-icon.svg` (new, for editability)
- Source PNG: `src-tauri/icons/app-icon.png` (1024x1024, input to `tauri icon`)
- Generated assets: `src-tauri/icons/` (32x32, 128x128, icon.icns, icon.ico, etc.)

### Generation pipeline

```
app-icon.svg  --(rsvg-convert)--> app-icon.png (1024x1024)
                                       |
                                  tauri icon
                                       |
                              icon.icns, icon.ico,
                              32x32.png, 128x128.png,
                              128x128@2x.png, icon.png,
                              ios/*, android/*, Square*.png

tray.svg  --(rsvg-convert)--> tray.png (22x22)
```

### Required tools

- **`rsvg-convert`** — from `librsvg` (Homebrew: `brew install librsvg`). Converts SVG to PNG at arbitrary resolution. Preferred over ImageMagick for clean SVG rasterization.
- **`pnpm exec tauri icon`** — built-in Tauri CLI command. Takes a source PNG and generates all platform icon variants.

## Design Direction

- **Coffee/cortado theme**: an abstract cortado glass — the layered espresso-over-milk look in a small rocks glass. Clean, geometric, minimal.
- **Tray icon**: a simplified silhouette of the glass shape. Recognizable at 22px. No text.
- **App icon**: a richer version with color — warm coffee tones, maybe a subtle gradient showing the espresso/milk layers. Sits well on macOS's squircle mask.

## Task Sequencing

Tasks are sequential — each builds on the previous:

1. **Install tools** — ensure `rsvg-convert` is available
2. **Design tray icon** — create SVG, convert to PNG, verify in menu bar
3. **Design app icon** — create SVG, convert to 1024x1024 PNG
4. **Generate all variants** — run `tauri icon`, verify bundle
5. **Verify** — build app, check tray + app icon appearance
