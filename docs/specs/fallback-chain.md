**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8).

# Spec: Fallback Chain (`fallback-chain`)

Family: Agents. Closes the parity row: affectUI · Fallback Chain (Pro). Behaviour-only rebuild from a one-paragraph
description (no visual capture); the models, timings, reasons and copy are our own. Related Oxyn component: trace-view.
Components: `FallbackChain` (Dioxus) · `FallbackChainEl` (GPUI), model in `oxyn_core::parity::fallbackfid`.

## 1. Purpose
Which model actually served a request: the fallback order drawn as a small flow, each failed try marked with why, the
one that answered carried through to the reply.

## 2. Parts
| Part | Look (Clean, light) |
|---|---|
| Root | 760 px (max 100 %), container-sized, white, #E6E6E9 hairline, 12 px radius |
| Bar | 48 px: blue fork glyph, "Fallback chain" 13/600, "n models in order" muted; a status pill (blue pulsing "Trying …", green "Answered by … · 2nd try", red "Every model failed") and a Replay icon button |
| Flow | #FAFAFA band, grid: Request card 150 px · 64 px SVG links · model column · 64 px links · Response card 170 px |
| Model | 52 px button, 10 px radius: order chip, 26 px brand tile (coloured mark), model 13/600 and a muted line (reason in red, "Answered in …" in green), a red ✕ / green ✓ / blue spinner |
| Links | 1.5 px #D8DCE3 curves from the request to every model; failed ones red and broken; skipped ones dotted; a 2.5 px blue light runs along the branch being tried; the answering model's curve runs on to the response |
| Response | caption, the reply's first words, "model · total time"; ringed blue once answered |

## 3. Behaviour
- `fb_states(attempts, elapsed, gap)` walks the chain: each try takes its `ms`, a failure hands over after `gap`, the
  first `Ok` answers and everything after it is skipped. `fb_final` is the settled chain.
- `live` plays the chain from the moment it mounts; Replay restarts it. A click on a model opens a small card with
  its time and what happened next.
- prefers-reduced-motion: no light, spinner or pulse.

## 4. Variants
live (default) · settled · first-try · third-try · all-failed · details · narrow (380 px) · classic.

## 5. Keyboard and accessibility
Tab reaches Replay then every model; Enter / Space opens details (aria-expanded). Model buttons are named with their
place, model and outcome; the headline is a status region; the SVG links are aria-hidden.

## 6. Responsive
Under 600 px the links give way to a vertical rail and the request, models and response stack; the subtitle hides.

## 7. Known gaps
- Needs capture: built from the catalogue description only – sizes, colours and the exact light are our reading.
- The travelling light restarts every 4 s rather than once per try.
- GPUI: a static snapshot – straight layout, no curves or light.
