**Status:** v1 — implemented (plan #23).

# Spec: slider

> Implemented in `oxyn-core::primitives::slider` (`Slider`, `Mark`, `SliderKey`, quantise, detents,
> range gap, pointer mapping, strain), `oxyn-dioxus::Slider`, `oxyn-gpui::SliderEl`. Source: Arc
> Slider (behaviour only). The composer effort control (`composer-effort-control.md`) can sit on it.

## 1. Purpose
Pick a value or a range on a track that follows the finger: temperature, effort, budget, volume.

## 2. State machine
```
Idle ─pointer down on track─▶ Active(nearest thumb jumps to the pointer) ─move─▶ Active(drag)
Active ─release─▶ Idle + commit (onValueCommit)
Idle ─key on thumb─▶ Idle(value ± step / large step / min / max) + change + commit
Range: thumbs keep at least `min_steps_between × step` apart and never cross.
Detents (Oxyn plus): values within `snap` of a detent mark land on it; arrows jump detent to detent.
Disabled: ignores input.
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| idle | rail, fill (start → value, or between thumbs), marks, thumbs | — | `role=slider` per thumb |
| hover | thumb grows to 1.12 | — | — |
| focus | focus ring + value bubble | → ↑ +step, ← ↓ −step, Shift+Arrow / PgUp / PgDn ± large step, Home / End | `aria-valuemin/max/now`, `aria-valuetext` |
| active / dragging | bubble springs up over the thumb, digits roll in the readout | — | — |
| at a limit | the thumb strains up to 6 px past the end, then springs back | — | — |
| detent | the readout shows the detent's name ("Medium") | — | `aria-valuetext` = name |
| disabled | 45 % | none | `aria-disabled=true` |
| vertical (Oxyn plus) | bottom = min | ↑ = more | `aria-orientation=vertical` |

## 4. Micro-interactions
Released drags carry a little momentum (≤ 180 ms) before snapping to the step; the bubble springs up
over the active thumb; digits roll in the readout. At a limit the thumb strains (rubber band
`6 × (1 − e^(−over/40))` px) and springs back (Arc). Keyboard moves glide on `spring.snap`.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Thumb glide (click / key) | 214 ms | `spring.snap` | instant |
| Bubble | 120 ms in / out | `ease.out` | fade 80 ms |
| Momentum | ≤ 180 ms | `ease.out` | none |
| Strain return | 214 ms | `spring.snap` | none |
| Readout digit roll | 280 ms | `ease.out` | swap |
| Hover grow | 80 ms | `ease.out` | none |

## 6. Variants
Source: `value`, `defaultValue`, `min`, `max`, `step`, `largeStep`, `marks`, `minStepsBetweenThumbs`,
`format`, `showValue`, `thumbLabels`, `start` / `end` (range), `disabled`, `onValueChange`,
`onValueCommit`.

**Oxyn plus:** named detents that snap (effort Low / Medium / High / Max), a live readout tag (e.g.
"≈ $0.42 / run"), vertical orientation.

## 7. Edge cases
Floating steps (0.1) are rounded so values never read 0.30000000000000004. `min == max` renders a
full fill at 0. A press exactly between range thumbs goes to the first. Very small tracks keep a
16 px thumb and 20 px hit height. Marks without labels are ticks; labelled marks reserve 14 px below.

## 8. Accessibility
Each thumb is focusable with `role=slider`, min / max / now and a formatted `aria-valuetext`; range
thumbs are named by `thumb_labels` ("Minimum", "Maximum"). Changes fire on keys as well as drags.

## 9. Props sketch
```rust
Slider { slider: Slider { values, min, max, step, large_step, marks: Vec<Mark { value, label, detent }>,
                          min_steps_between, snap, disabled, vertical },
         label, format: "#%", decimals, show_value, thumb_labels, tag, active, on_change, on_commit }
```

## 10. Related components and references
`composer-effort-control.md` (named levels), `number-ticker.md` (readout roll). Arc
`/components/slider` — behaviour only.
