**Status:** built v0.1 (2026-11-16, variants 2: Attachments family).

# Spec: Polaroid stack (`polaroid-stack`)

Family: Attachments. Reference: none (an Oxyn original).
Components: `PolaroidStack` (Dioxus) · `PolaroidStackEl` (GPUI), models in `oxyn_core::kit::attachfid`.

## 1. Purpose
Image attachments that stay small in the composer but are easy to check and prune.

## 2. Parts
| Part | Look |
|---|---|
| Prints | 132 px white-framed cards (26 px bottom lip) with generated abstract pictures (no real photos), soft shadow; the front one ringed |
| Fan | closed: each leans a few degrees (`fan`); open: spread across `spread` degrees and lifted 10 px, gliding with a little overshoot |
| Foot | caption "photo-2.jpg · 2 of 4" and "+ Add image" (up to six) |

## 3. Behaviour
Pointing at the stack, or focus inside it, fans it out. Pressing a print brings it to the front. × (shown when fanned or focused) takes a print out. `on_remove(name)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
stacked · fanned (six, wide) · pair.

## 5. Keyboard and accessibility
A group labelled with the count; prints are buttons named after their files; × names its print. Changes are said in a status line.

## 6. Responsive
340 px stage; the fan stays inside it up to 60°.

## 7. Known gaps
No full-size preview yet. GPUI: no rotation (the fan is a horizontal spread).
