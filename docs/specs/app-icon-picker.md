**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: App icon (`app-icon-picker`)

Family: Theme Selector. Reference: none (an Oxyn original; icons drawn by us).
Components: `AppIconPicker` (Dioxus) · `AppIconPickerEl` (GPUI), models in `oxyn_core::kit::themefid`.

## 1. Purpose
Let someone choose how the app's icon looks – one of six we draw, in colour, dark or a single tint – and see it among other apps.

## 2. Parts
| Part | Look |
|---|---|
| Head | "App icon" and Colour / Dark / Tinted |
| Icons | six 52 px rounded squares: a ring and a dot on Classic, Paper, Daybreak, Ocean, Mint and Nebula grounds (`app_icons`); the picked one lifts with an ink ring |
| Tint | hue slider over a spectrum track, live only in Tinted |
| Home row | three plain app icons and ours on a wallpaper gradient; the wallpaper darkens for Dark and Tinted |

## 3. Behaviour
An icon or style click restyles every icon at once (`icon_colors`: Colour uses its own two stops, Dark lays them on near-black, Tinted draws ground and mark in one hue). The home-row icon pops in when it changes. `on_change((icon, style))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (Classic, Colour) · ocean dark · tinted.

## 5. Keyboard and accessibility
Icons and styles are radio groups named in words; the tint slider is disabled outside Tinted; the home row is an image labelled with the chosen icon.

## 6. Responsive
Icons drop to three columns under 480 px.

## 7. Known gaps
The pick is only shown; setting the real app icon is up to the host platform. GPUI: static snapshot (host routes `icon-<i>`, `style-<i>`).
