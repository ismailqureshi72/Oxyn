**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Effort pill (`effort-pill`)

Family: Thinking Selector. Reference: a user-supplied screenshot of a "Select effort" pill with a popover slider
(layout idea only; colours are monochrome here, wording and the model are our own). Components: `EffortPill` (Dioxus)
· `EffortPillEl` (GPUI), model in `oxyn_core::parity::thinkfid`.

## 1. Purpose
An effort control for a composer that starts unset: the pill reads "Select effort" until the user picks a level in a
popover with a big, touch-friendly step slider.

## 2. Parts
| Part | Look |
|---|---|
| Pill | 34 px, fully round, hairline; "Select effort" grey until set, then the level in ink; a down caret |
| Popover | 380 px, 20 px radius, soft shadow, centred text |
| Heading | level 20/600; the model it applies to as a link-styled line with › |
| Slider | 36 px round track; ink fill up to the 38 px white thumb (ink ring); a 5 px dot per level (light behind, grey ahead) |
| Reset | 30 px ghost button top-right |

## 3. Behaviour
Press, drag (snapping) or keys pick a level; the pill shows it. Reset returns to `reset` (Medium). The model line fires
`on_model` (open a model picker). Controlled (`value: Some(Option<usize>)`) or uncontrolled; `on_change`.
Escape closes the popover. A press outside closes it when the user opened it; a start-open preview draws no catcher. prefers-reduced-motion: no glide, no pop-in.

## 4. Variants
open (default) · unset (closed, grey pill) · max.

## 5. Keyboard and accessibility
Pill: `aria-expanded`, `aria-haspopup="dialog"`; popover `role="dialog"` named Thinking effort. Slider `role="slider"`,
value text = level name, ←/→ and Home/End. A polite status line says each change.

## 6. Responsive
380 px popover capped at the viewport minus 32 px.

## 7. Known gaps
No focus trap or click-outside close in the popover. GPUI: static snapshot.
