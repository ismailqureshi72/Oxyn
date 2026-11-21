**Status:** built v0.1 (2026-11-16, variants 2: Menu Motion family).

# Spec: Liquid drops (`liquid-drops`)

Family: Menu Motion. Reference: none (an Oxyn original).
Components: `LiquidDrops` (Dioxus) · `LiquidDropsEl` (GPUI), models in `oxyn_core::kit::gooeyfid`.

## 1. Purpose
A compact action launcher whose actions visibly come out of the button.

## 2. Parts
| Part | Look |
|---|---|
| Button | 56 px ink circle, + turns to × (135°) |
| Drops | 40 px ink circles that rise `gap` px apart (`drop_rise`), each 55 ms after the one before, with a spring; the goo keeps them joined until they pull free |
| Labels | card pills beside each drop, fading in after it lands |

## 3. Behaviour
Press to open or close (closing runs the drops back in, top one first). Picking an action closes. Escape closes. `on_pick(key)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
open · closed · tight (heavy goo).

## 5. Keyboard and accessibility
The button reports `aria-expanded`; closed actions leave the tab order; each action is a button named by its label.

## 6. Responsive
220 px column anchored right.

## 7. Known gaps
Icons are fixed by key (file, shot, voice, link). GPUI: static.
