**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Light playground (`light-playground`)

Family: Theme Selector. Reference: none (an Oxyn original; the "drag a light source over the UI" idea from the family plan).
Components: `LightPlayground` (Dioxus) · `LightPlaygroundEl` (GPUI), models in `oxyn_core::parity::themefid`.

## 1. Purpose
Show how depth reads in a theme by letting someone move the light: shadows and highlights follow the lamp.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Light playground" and Shadow / Glow |
| Stage | 240 px inset with a soft pool of light under the lamp; a sample card, a pill button and a chip |
| Lamp | 30 px round knob in ink with a halo; focusable |
| Shadows | offset away from the lamp and softer with distance (`lamp_shadow`); a highlight gradient on the side facing it (`lamp_angle`) |
| Strength | 0–100 slider: how far shadows reach and how dark they are |

## 3. Behaviour
Press and drag anywhere on the stage (a hit layer over the cards reads the pointer; the stage size is measured on mount and on each press) and the lamp follows, clamped to 4–96 % (`lamp_clamp`). Arrow keys on the lamp move it 4 %. Glow turns each shadow into a lit rim toward the lamp. `on_move((x, y))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (top left) · low sun (full strength) · glow.

## 5. Keyboard and accessibility
The lamp is a button that moves with the arrow keys; its position is said in a polite note. The sample cards are decoration and not focusable.

## 6. Responsive
The stage keeps its height (220 px under 480 px); the sample card widens.

## 7. Known gaps
One light only; no colour temperature. GPUI: static snapshot of the lamp and one card's shadow.
