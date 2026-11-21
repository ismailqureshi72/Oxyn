**Status:** built v0.2 (2026-11-16, variants 1: Thinking Selector family). v0.2 follows the user's clean reference: even gradient lines and a solid nozzle (v0.1 was the hand-drawn red line with a see-through ring, kept as `tone="red"` colours).

# Spec: Nozzle line (`nozzle-slider`)

Family: Thinking Selector. Reference: the user's own design (an Oxyn original): a row of even, thin vertical lines in a colour gradient with a selector sliding along them. The reference's stamp is replaced by a nozzle; names, wording and the nozzle are our own.
Components: `NozzleSlider` (Dioxus) · `NozzleSliderEl` (GPUI), models in `oxyn_core::kit::{nozzlefid, thinkfid}`.

## 1. Purpose
A playful, minimal effort control: a calm line of evenly spaced coloured lines and a solid nozzle that slides along and lands on a level.

## 2. Parts
| Part | Look |
|---|---|
| Surface | 440 px (max 100%), 14 px radius, 1 px `--oxth-line` border, own background (#0D0D0F dark, #FFFFFF light) |
| Line | 52 px tall hit area, 20 px side padding; `strokes` (37) lines, all 2 px × 16 px with rounded ends, evenly spaced by percentage |
| Gradient | `nozzle_rgb(f, tone, dark)`: steel blue #8FACC0 → grey #ACB5B4 → pale yellow #BDBFA3 / #DCC496 → peach #E5B27F → coral #E46854 / #F6644C (centre) → rose #B5596D → plum #8D4D79 → violet #563C77 → indigo #504984 / #67629C → periwinkle #717CB1. Light mode uses darker stops with the same hues (`NOZZLE_LIGHT`, every one ≥ 4:1 on white) |
| Glow | lines near the nozzle are brightest: opacity `lo + (1 − lo) × line_glow` (falls off over 0.4 of the line); lo = 0.85 in both themes |
| Nozzle | solid 28 × 32 knob, arched top (14 px) and 8 px bottom corners, filled with the gradient colour at its position (coral at the centre), 1.5 px light rim (#FFF6EE dark, white light), a 4 px halo in the surface colour that parts the lines around it, soft shadow; a small orifice-and-spray glyph in `nozzle_ink` (cream, or near-black on pale fills) |
| Caption | the level's name under the line (optional) |

## 3. Behaviour
Press anywhere on the line: the nozzle jumps to the nearest level. Drag: it follows the pointer freely (its fill changes
colour as it travels, 1.08× while held) and the level snaps to the nearest stop; on release it springs onto the stop
(200 ms, slight overshoot). Arrows / Page keys step one level, Home / End go to the ends. `on_change`, polite live region.
prefers-reduced-motion: no transitions.

## 4. Props and variants
`tone`: `gradient` (default) · `red` (every line and the nozzle in `--oxth-red`). `size`: `sm` (12 px lines, 22 × 26 knob) ·
`md` (default) · `lg` (20 px lines, 34 × 38 knob). `levels` (any scale), `strokes`, `caption`.
Demo variants: high · low · max · red · small (sm, four levels, no caption). Demo controls: tone, levels (five / four / api), level, size, lines, caption.

## 5. Keyboard and accessibility
`role="slider"` with `aria-valuemin/max/now` and the level name as `aria-valuetext`; lines and nozzle are `aria-hidden`.
The level is named in the caption and value text, so it never rests on colour.

## 6. Responsive
440 px up to its container; lines are placed by percentage, so they close up on narrow screens (390 px fits).

## 7. Known gaps
GPUI is a static snapshot (no glide or drag, md size only) with a ring glyph instead of the spray mark, and a 1 px rim.
Dark-mode violet/indigo keep the reference colours, so the far lines are dim (about 2:1) by design; they are decoration.
