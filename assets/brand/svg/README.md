# Oxyn logo v2 — vector files

The approved Oxyn brand (Nov 2026): the **E1 cube mark** (an eased hexagon cube with a V slot and two
cube eyes) and the **W2 OXYN wordmark** (custom paths, even stroke, squircle O). One colour only:
the mark is monochrome everywhere. It replaces the v3 particle-orb mark, whose animated renders
(`../oxyn-mark.mp4`, `../oxyn-banner.gif`) are kept only as history.

## Files

| File | Use |
|---|---|
| `oxyn-mark.svg` | The mark, `#0A0A0A` on transparent. Light backgrounds. |
| `oxyn-mark-light.svg` | The mark, white on transparent. Dark backgrounds. |
| `oxyn-mark-mono.svg` | The mark in `currentColor`, for one-colour UI (inline SVG). |
| `oxyn-mark-32.svg` | Drawn on the 32 px grid (eyes snapped to whole pixels). Favicons and 24–47 px. |
| `oxyn-mark-16.svg` | Hand-placed 16 px version. Use at 16–23 px. |
| `oxyn-wordmark.svg` / `oxyn-wordmark-light.svg` | OXYN (W2), dark on light / light on dark. |
| `oxyn-lockup.svg` / `oxyn-lockup-light.svg` | Mark + wordmark, dark on light / light on dark. |
| `oxyn-app-icon-dark.svg` / `oxyn-app-icon-light.svg` | 1024 continuous-corner app tile, white mark on near-black / black mark on near-white. |

The site uses the same shapes: `site/public/favicon.svg` (32 px grid, switches to white in dark
browser chrome), `favicon-16/32.png`, `favicon.ico`, `apple-touch-icon.png` (from the dark app icon)
and `og.png`; the navbar, sidebar and footer draw the lockup inline in `currentColor`
(`site/src/components/v2/logo.ts`), so it follows the light and dark themes.

## Colours

| Role | Hex |
|---|---|
| Ink (light backgrounds) | `#0A0A0A` |
| Ink (dark backgrounds) | `#FFFFFF` |
| App tile (dark) | `#17171A` → `#0A0A0A` |

## Minimum sizes

- Lockup: 72 px wide (the site navbar draws it 19 px tall).
- Mark: use `oxyn-mark-16.svg` below 24 px and `oxyn-mark-32.svg` up to 47 px; the full mark from 48 px.
