# App Icon

The Rsync Studio icon is a purple circle with gold counterclockwise arrows and a bold "R" in the center (Verdana Bold).

## Design

| Element              | Value                          |
|----------------------|--------------------------------|
| Background           | Purple circle `#7C3AED`        |
| Inner ring (depth)   | Darker purple `#6D28D9`        |
| Arrows & letter      | Gold `#F59E0B`                 |
| Letter               | "R" in Verdana Bold, 380pt     |
| Arrow style          | Two counterclockwise arcs with triangular arrowheads |
| Canvas size          | 1024 x 1024 px                 |

## Generated files

The generator script produces a single 1024px PNG. The Tauri CLI then creates all platform-specific variants:

| File               | Purpose                        |
|--------------------|--------------------------------|
| `icon.icns`        | macOS app bundle               |
| `icon.ico`         | Windows                        |
| `icon.png`         | Main icon (512 x 512)          |
| `32x32.png`        | Small PNG                      |
| `64x64.png`        | Medium PNG                     |
| `128x128.png`      | Large PNG                      |
| `128x128@2x.png`   | Retina PNG (256 x 256)         |
| `Square*.png`      | Windows Store logos             |
| `StoreLogo.png`    | Windows Store                  |

## Prerequisites

- Python 3 with [Pillow](https://pypi.org/project/Pillow/) (`pip install Pillow`)
- Verdana Bold font (ships with macOS; the script falls back to Helvetica/Arial if missing)
- Tauri CLI (`npm install @tauri-apps/cli`)

## Regenerating the icon

From the project root:

```bash
cd src-tauri/icons
python3 generate_icon.py          # produces icon_1024.png
npx tauri icon icon_1024.png -o . # generates all platform variants
rm -rf android ios icon_1024.png  # clean up mobile/intermediate files
```

## Tunable parameters

All constants are at the top of `src-tauri/icons/generate_icon.py`:

| Constant               | Description                              | Current |
|------------------------|------------------------------------------|---------|
| `PURPLE`               | Background fill color (RGBA)             | `(124, 58, 237, 255)` |
| `PURPLE_DARK`          | Inner ring color for depth effect        | `(109, 40, 217, 255)` |
| `GOLD`                 | Arrow and letter fill color (RGBA)       | `(245, 158, 11, 255)` |
| `ARROW_RADIUS`         | Distance from center to arrow centerline | `355`   |
| `ARROW_THICKNESS`      | Stroke width of arc lines                | `68`    |
| `ARROWHEAD_LENGTH`     | Length of triangular arrowhead           | `90`    |
| `ARROWHEAD_HALF_WIDTH` | Half-width of arrowhead base             | `52`    |
| `ARC1_START_ANGLE`     | Start angle of first arc (Pillow CW degrees from east) | `165` |
| `ARC1_END_ANGLE`       | End angle of first arc                   | `315`   |
| `ARC2_START_ANGLE`     | Start angle of second arc                | `345`   |
| `ARC2_END_ANGLE`       | End angle of second arc                  | `135`   |
| `font_size`            | Size of the "R" letter                   | `380`   |

## Tray icon

The system tray uses the same icon with `icon_as_template(false)` in `src-tauri/src/lib.rs`, so the full-color purple and gold icon is displayed in the macOS menu bar. Set it to `true` if you prefer macOS to render it as a monochrome template that adapts to light/dark mode.
