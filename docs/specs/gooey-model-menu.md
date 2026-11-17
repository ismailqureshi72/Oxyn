**Status:** built v0.1 (2026-11-16, variants 2: Menu Motion family).

# Spec: Gooey model menu (`gooey-model-menu`)

Family: Menu Motion. Reference: the model menu of a public chat-composer component. Behaviour only; names are the makers' product families with their vetted marks, hints are our own.
Components: `GooeyModelMenu` (Dioxus) · `GooeyModelMenuEl` (GPUI), models in `oxyn_core::parity::gooeyfid`.

## 1. Purpose
A model picker with the same liquid motion, anchored to the chip's right edge.

## 2. Parts
| Part | Look |
|---|---|
| Chip | 132 px, maker's mark, name (blurs in when it changes), caret that flips |
| Sheet | 268 px, right-aligned, origin near the chip |
| Rows | mark, name, hint, check on the chosen one |

## 3. Behaviour
The shared gooey menu motion: the button, a stem and the menu sheet are drawn in one layer under an SVG goo filter (blur `goo` px, then a hard alpha curve), so shapes near each other join with a liquid neck; rows sit above it, unfiltered. Open: the stem stretches out of the button (0.24 s) and pinches off while the sheet swells from the corner nearest the button with a slight overshoot (0.46 s); rows arrive top to bottom (`stagger_ms`: 70 ms, then 24 ms apart, a 3 px blur clearing); a grey fill glides under the pointer (0.17 s). Close: the sheet runs back to the corner fast (0.18 s) and the menu leaves the page when that animation ends. Timings were tuned by eye (the reference publishes none). `on_change(key)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
open · closed · no goo (goo 0).

## 5. Keyboard and accessibility
Listbox pattern; `aria-selected` on the chosen option; marks are decoration.

## 6. Responsive
268 px list.

## 7. Known gaps
GPUI: static, initials instead of marks.
