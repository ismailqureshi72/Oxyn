**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Display comfort (`display-comfort`)

Family: Theme Selector. Reference: none (an Oxyn original).
Components: `DisplayComfort` (Dioxus) · `DisplayComfortEl` (GPUI), models in `oxyn_core::parity::themefid`.

## 1. Purpose
Gather the comfort settings that change how the whole app feels, and show what each one does before leaving the page.

## 2. Parts
| Part | Look |
|---|---|
| List | "Comfort" with a count ("2 on", `comfort_said`) and four switch rows: Increase contrast, Reduce motion, Reduce transparency, Bold text, each with a short line |
| Sample | 240 px: two colour blurs under a frosted glass panel with a title, a line, a looping progress bar and a spinner |

## 3. Behaviour
Contrast firms the glass edge and darkens quiet text; Reduce motion stops the bar and spinner; Reduce transparency makes the glass solid; Bold text thickens type. Each applies at once. `on_change((key, on))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (all off) · calm (motion and transparency) · all on.

## 5. Keyboard and accessibility
Switches are `role="switch"` with `aria-checked`, named by their labels; the sample is an image labelled with the count.

## 6. Responsive
The sample moves under the switches under 560 px.

## 7. Known gaps
Settings apply to the sample only, not the page. GPUI: static snapshot (host routes `comfort-<i>`).
