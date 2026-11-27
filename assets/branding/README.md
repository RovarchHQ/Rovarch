# Rovarch branding assets

| Asset | File | Format | Dimensions | Usage |
| --- | --- | --- | --- | --- |
| Rovarch Logo | `rovarch-logo.png` | PNG (RGBA) | 2133×2133 | Application icon, About dialog, tray icon (embedded via `include_bytes!`) |
| Rovarch Play Logo | `rovarch-play.svg` | SVG (`viewBox="0 0 1080 1080"`) | Vector | Rovarch Play section |
| Rovarch Play Logo fallback | `rovarch-play.png` | PNG (RGBA) | 512×512 | Used when SVG decoding is unavailable |
| Rovarch Studio Logo | `rovarch-studio.svg` | SVG (`viewBox="0 0 1080 1080"`) | Vector | Rovarch Studio section |
| Rovarch Studio Logo fallback | `rovarch-studio.png` | PNG (RGBA) | 512×512 | Used when SVG decoding is unavailable |

All assets keep transparent backgrounds and their original aspect ratios. The application icon is
distributed in the standard freedesktop `hicolor` sizes (16, 24, 32, 48, 64, 128, 256, 512) under
`data/icons/hicolor/<size>/<size>/apps/io.rovarch.app.png`, generated from
`rovarch-logo.png`.
