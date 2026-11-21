**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Background picker (`background-picker`)

Family: Theme Selector. Reference: a user screenshot of a home-screen background setting (picture tiles, a Default tile, a tile for your own picture, a texture switch). Behaviour only; the pictures are drawn by us as SVG, and layout, names and wording are our own.
Components: `BackgroundPicker` (Dioxus) · `BackgroundPickerEl` (GPUI), models in `oxyn_core::kit::themefid`.

## 1. Purpose
Let someone put a picture behind their home screen, or their own, and lay a texture over it, seeing the result before leaving the page.

## 2. Parts
| Part | Look |
|---|---|
| Card | 560 px, 20 px radius, hairline, soft shadow; neutral greys (`--oxth-*`) |
| Preview | 150 px home screen over the chosen picture: greeting and a prompt box; fades in on change |
| Tiles | 16:9, 12 px radius, 3 columns (2–4); Default is plain with its name, pictures (Canopy, Orbit, Neon, Tide) are our own SVG drawings; a white check badge marks the picked one |
| Your own | tile with + and "Your own": a real file input; the picture becomes a new "Yours" tile and is picked |
| Texture | row under a hairline: Film (animated grain), Static (rolling scan lines), Dither (ordered dots) as a segmented group |

## 3. Behaviour
A tile click picks the background. Your own reads the file (`own_check`: images up to 8 MB) into a data URL (`data_url`) and picks it. Pressing a texture lays it over the preview and the picture tiles; pressing the lit one takes it off (`texture_press`). `on_change((background, texture))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · orbit with film · neon dithered · tiles only (no preview, two columns).

## 5. Keyboard and accessibility
Tiles are a radio group, each picture named and described. Your own is a native file input with a label; errors are said in the live note. Textures are toggle buttons (`aria-pressed`). The preview is `role="img"` labelled with picture and texture.

## 6. Responsive
Tiles drop to two columns and the texture row stacks under 480 px.

## 7. Known gaps
Your own keeps the picture in memory only (no upload). GPUI shows each picture as its two main colours; static snapshot (host routes `bg-<i>`, `own`, `tex-<i>`).
