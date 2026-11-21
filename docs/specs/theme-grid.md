**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Theme grid (`theme-grid`)

Family: Theme Selector. Reference: a user screenshot of a display setting (a grid of mini-window theme swatches and a font size dropdown). Behaviour only; the themes, their colours and names are our own.
Components: `ThemeGrid` (Dioxus) · `ThemeGridEl` (GPUI), models in `oxyn_core::kit::themefid`.

## 1. Purpose
Pick the look of the whole app from small pictures of it, and the text size, and see both on a sample at once.

## 2. Parts
| Part | Look |
|---|---|
| Swatch | 16:9 mini window: sidebar of four bars and a pane holding a button block, in the theme's own colours (`swatches`) |
| System | light and dark windows split corner to corner, a round monitor badge in the middle |
| Themes | System, Light, Dark, Midnight, Plum, Moss, Harbor, Contrast (black panes outlined in white) |
| Picked | 2 px ink ring and a white check badge; the label under it brightens |
| Font size | row under a hairline: a 220 px button with chevron; the list opens above it, each option drawn at its own size |
| Sample | sidebar and pane in the chosen theme at the chosen size; System follows `device_dark` |

## 3. Behaviour
A swatch click picks the theme; the sample recolours. The font button opens a listbox (Up/Down, Home/End, Enter, Escape, click outside closes) and the sample text resizes (`font_px`). `on_change((theme, font))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (System on a dark device) · contrast with large text · light device with small text · two columns without the sample.

## 5. Keyboard and accessibility
Swatches are a radio group named by theme. The font size is a button + listbox with `aria-activedescendant`. The sample is `role="img"` labelled with theme and size; changes are said in a status line.

## 6. Responsive
Swatches drop to two columns and the sample's sidebar folds into a row under 480 px.

## 7. Known gaps
The sample is a picture of the app, not the page itself. GPUI: static snapshot (host routes `theme-<i>` and `font`).
