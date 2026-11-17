**Status:** built v0.1 (2026-11-16, variants 2: Menu Motion family).

# Spec: Gooey attach menu (`gooey-attach-menu`)

Family: Menu Motion. Reference: the plus menu of a public chat-composer component (corner morph, row cascade, hover fill). Behaviour only; rows, icons and wording are our own.
Components: `GooeyAttachMenu` (Dioxus) · `GooeyAttachMenuEl` (GPUI), models in `oxyn_core::parity::gooeyfid`.

## 1. Purpose
The attach drop-up on its own, for any toolbar: file types and sources pouring out of a + button.

## 2. Parts
| Part | Look |
|---|---|
| Button | 38 px, 13 px radius, grey; the + turns into × (45°) |
| Sheet | 280 px, 16 px radius, card fill with a hairline drop-shadow, `panel_height` rows (all eight fit; scrolls past 360 px) |
| Rows | 36 px: icon (file-type icon with badge, or a brand mark), label 13.5/500, hint 12.5 muted; group headings 11.5 muted |
| Stem | 26 px wide neck between button and sheet while it opens |

## 3. Behaviour
The shared gooey menu motion: the button, a stem and the menu sheet are drawn in one layer under an SVG goo filter (blur `goo` px, then a hard alpha curve), so shapes near each other join with a liquid neck; rows sit above it, unfiltered. Open: the stem stretches out of the button (0.24 s) and pinches off while the sheet swells from the corner nearest the button with a slight overshoot (0.46 s); rows arrive top to bottom (`stagger_ms`: 70 ms, then 24 ms apart, a 3 px blur clearing); a grey fill glides under the pointer (0.17 s). Close: the sheet runs back to the corner fast (0.18 s) and the menu leaves the page when that animation ends. Timings were tuned by eye (the reference publishes none). `side` = up (drop-up) or down. `on_pick(key)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
open · closed · drop-down.

## 5. Keyboard and accessibility
Menu button pattern as in the composer; rows are `menuitem`s.

## 6. Responsive
280 px menu.

## 7. Known gaps
No automatic side choice from the viewport (`open_side` exists in the core for a host that measures). GPUI: static.
