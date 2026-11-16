**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Ruler card (`effort-ruler`)

Family: Thinking Selector. Reference: a user-supplied screenshot of a "Max · thinks as long as it needs" card with a
dense tick ruler (layout idea only; names, wording and colours are our own).
Components: `EffortRuler` (Dioxus) · `EffortRulerEl` (GPUI), model in `oxyn_core::parity::thinkfid`.

## 1. Purpose
Pick how hard the model thinks with one glance at a ruler. The level's name and what it does sit above a ruler of
dense ticks; the ticks behind the knob tint toward it, so the amount reads like a gauge.

## 2. Parts
| Part | Look |
|---|---|
| Root | 440 px card (max 100 %), 18 px radius, hairline, soft shadow; neutral greys (`--oxth-*`) |
| Heading | level name 19/600, red at the top level; its line 15 muted |
| Reset | 28 px ghost button top-right, a back-arrow glyph; disabled at the reset level |
| Well | inset panel, 14 px radius, hairline |
| Ruler | `ticks` (44) 2 × 18 px ticks spread by percentage; each tinted by `tick_tint(t, ticks, knob, 0.55)`: red next to the knob, fading to a floor of 12 % over 55 % of the track, plain grey past it |
| Knob | 8 × 30 px red capsule with a soft red halo |

## 3. Behaviour
- The ruler is a slider over the scale's levels (Low, Medium, High, Extra high, Max by default). Press anywhere to
  jump, drag to scrub; the knob snaps to the nearest level (`snap_level`). Uncontrolled (`default_value`, top level)
  or controlled (`value`); `on_change(index)`.
- Reset returns to `reset` (the middle level by default).
- The knob glides 160 ms between levels; while dragging it follows the pointer level by level without the glide.
- prefers-reduced-motion: no glide, no tint transition.

## 4. Variants
max (default) · high · low · dense (72 ticks) · narrow (320 px).

## 5. Keyboard and accessibility
`role="slider"` with `aria-valuemin/max/now` and `aria-valuetext` = the level's name. ←/↓/PageDown one level down,
→/↑/PageUp one up, Home/End to the ends. A polite status line says "Effort: High. Works through the steps." after
each change. Reset is a named button ("Reset to High").

## 6. Responsive
Ticks are placed by percentage, so the ruler fits any width; under 340 px (container query) the heading tightens and
the well's side padding drops to 14 px.

## 7. Known gaps
- No haptic-like detent feedback while dragging between levels.
- GPUI: a static snapshot; each level is a press target over its stretch of the ruler (no drag).
