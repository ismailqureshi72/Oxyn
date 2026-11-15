**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A).

# Spec: Thinking State (`thinking-state`)

Family: Chat. Closes the parity row: affectUI · Thinking State. Behaviour-only rebuild; nothing is copied.
Components: `ThinkingPanel` (Dioxus) · `ThinkingPanelEl` (GPUI), models in `oxyn_core::parity::answers`.
Oxyn extras on the same page: the plan #27 `ReasoningBlock` (variant `oxyn-reasoning-block`; also its own page).

## 1. Purpose
Show an agent working: a live timer, the sources it searched and its thinking steps as they happen.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Header | A button: blue 3×3 dot grid (dots pulse in turn) + "Thinking 3s" shimmering → "Worked for 7s"; a chevron → when folded |
| Timeline | Dashed 1 px rule down the left; 18 px nodes |
| Searched | Globe node, grey label, 28 px source chips (hairline, 8 px radius, coloured brand mark, 13 px) popping in one by one; "+3 more" grey chip → "Show less" |
| Steps | 14 px lines with a 5 px square node; the active one pulses blue and its text shimmers |
| Finish | A 15 px blue circle whose white tick draws in: "Finished thinking" |

## 3. Behaviour
- A step every 1.3 s; the timer ticks each second; done one step after the last.
- The header folds and unfolds the steps; collapseOnDone folds them by itself.
- Loop restarts 3.5 s after finishing.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Running, looping |
| finished | Done |
| more-sources | All sources |
| collapsed | Folded |
| classic | Chevron on the left instead of the dot grid, flat grey capsule chips, plain "+3 more" |
| oxyn-reasoning-block | Oxyn extra |

## 5. Keyboard and accessibility
The header is one button with aria-expanded; steps are an ordered list; chips are links with their address as a tooltip; "+N more" is aria-expanded. Reduced motion shows everything still.

## 6. Responsive
Up to 420 px; chips wrap.
