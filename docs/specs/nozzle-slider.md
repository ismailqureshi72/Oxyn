**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Nozzle line (`nozzle-slider`)

Family: Thinking Selector. Reference: the user's own drawing (an Oxyn original): short, very closely spaced vertical lines with a ring sliding along them (layout idea only; names, wording and colours are our own).
Components: `NozzleSlider` (Dioxus) · `NozzleSliderEl` (GPUI), models in `oxyn_core::parity::{nozzlefid, thinkfid}`.

## 1. Purpose
A playful, minimal effort control: a hand-drawn line of red strokes and a see-through ring – the nozzle – that slides along and lands on a level.

## 2. Parts
| Part | Look |
|---|---|
| Line | 48 px tall hit area; `strokes` (41) red strokes 2.5 px wide, 18 px × a wobble height (0.6–1.0) and a ±6° tilt (`nozzle_ticks`, deterministic) |
| Stops | the strokes under each level (`level_tick`) are 3 px and 21 px, fully opaque (`is_stop`) |
| Nozzle | 26 px ring, 2.5 px ink border, transparent fill (strokes show through), a faint halo; 30 px while dragging |
| Caption | the level's name under the line, muted (optional) |

## 3. Behaviour
Press or drag: the nozzle follows the pointer freely while held and the level snaps to the nearest stop; on release
it springs onto the stop (200 ms, slight overshoot). Arrows / Home / End step levels. `on_change`.
prefers-reduced-motion: no transitions or pop-in.

## 4. Variants
high (default) · low · max · dense (61 strokes, no caption).

## 5. Keyboard and accessibility
`role="slider"`, value text = level name; strokes and ring are `aria-hidden`. Stops differ in size, not only colour.

## 6. Responsive
420 px up to its container; strokes are placed by percentage.

## 7. Known gaps
GPUI strokes are upright (no rotation primitive); the wobble shows in heights only.
