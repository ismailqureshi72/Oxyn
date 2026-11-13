**Status:** v1 — implemented (plan #23).

# Spec: segmented control

> Implemented in `oxyn-core::primitives::segmented` (`Segmented`, `Segment`, `SegmentedVariant`,
> `SegmentedKind`, `indicator`, drag) with `nav`, `oxyn-dioxus::SegmentedControl`,
> `oxyn-gpui::SegmentedEl`. Sources: affectUI Segmented Control, Arc Segmented control (behaviour
> only). The composer's Ask / Plan / Agent switch (`composer-mode-switch.md`) is built on it.

## 1. Purpose
Switch between two to eight related views or modes in place: Ask / Plan / Agent, All / Running /
Done, Diff / Files / Logs.

## 2. State machine
```
Selected(i) ─click / arrow (radio) / Enter·Space (tabs)─▶ Gliding(from i, to j, at) ─280 ms─▶ Selected(j)
Selected(i) ─drag start─▶ Dragging(x) ─move─▶ Dragging(x') ─release─▶ Gliding(i → option under x)
Tabs kind: Focus(k) moves with arrows independently of Selected(i).
Disabled control or option: ignored.
```
Default value: the first enabled option.

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| tray (default) | grey tray, raised pill under the selection | radio: ← / → select; tabs: ← / → focus, Enter / Space select; Home / End | `radiogroup` / `radio` + `aria-checked`, or `tablist` / `tab` + `aria-selected` |
| underline | 2 px line under the selection | same | same |
| selected label | semibold, without shifting neighbours (the semibold copy holds the width) | — | — |
| count / badge | "Running 4", "Done New"; a new count blurs in | — | part of the name |
| pressed | pill scales to 0.97 | — | — |
| dragging | pill follows the pointer, clamped to the track | — | — |
| disabled option / control | 40 % / 50 % | skipped | `disabled` |
| vertical (Oxyn plus) | column; ↑ / ↓ | ↑ / ↓ | `aria-orientation=vertical` |

## 4. Micro-interactions
- The indicator glides to the picked option; **the edge facing the target leads** (35 % head
  start), so it stretches a little and settles — calm, no bounce (affectUI). `Segmented::indicator`
  computes both edges.
- Labels crossfade regular → semibold while a hidden semibold copy holds the width, so neighbours
  never shift.
- When a label or count changes, the option and the control morph to the new width and the pill
  follows.
- Drag: the pill tracks the pointer 1:1 and drops onto the option under it.

## 5. Timing table
| Transition | Duration | Easing | Notes |
|---|---|---|---|
| Glide (trailing edge) | 280 ms | `ease.out` | |
| Leading edge | 280 ms × 0.65 | `ease.out` | the stretch |
| Weight crossfade | 180 ms | `ease.out` | |
| Count blur-in | 180 ms | `ease.out` | |
| Press | 80 ms, 0.97 | `ease.out` | |
| **Reduced motion** | pill moves instantly; no stretch, no press scale; label weights swap | — | Arc |

## 6. Variants
Source: `options`, `value`, `defaultValue`, `size` sm / md / lg, `variant` tray / underline,
`fullWidth`, `kind` radio / tabs, `disabled`, `aria-label`, `onChange`; Arc `onOptionIntent`.

**Oxyn plus:** 2–8 options (`MAX_OPTIONS`), vertical orientation, drag-to-switch toggle (`draggable`),
icon-only options (named by their icon id when unlabeled).

## 7. Edge cases
Eight options in a narrow panel: use `full_width` and `sm`; labels truncate before they wrap.
Disabled current value (data changed): the pill stays but arrows skip to enabled options. Options
added or removed: the indicator re-measures and jumps (no glide from a stale rect).

## 8. Accessibility
Roving `tabindex`; radio kind follows radio-group keys, tabs kind follows the WAI tabs pattern with
manual activation. Counts and badges are part of the option's name. The pill is `aria-hidden`.

## 9. Props sketch
```rust
SegmentedControl { options: Vec<Segment { value, label, icon, count, badge, disabled }>, value,
                   size: "sm|md|lg", variant: "tray|underline", kind: "radio|tabs", full_width,
                   vertical, disabled, draggable, aria_label, on_change }
```

## 10. Related components and references
`composer-mode-switch.md`, `toggles.md` (radio semantics), `button-groups.md` (same highlight).
affectUI `/components/segmented-control`; Arc `/components/segmented-control` — behaviour only.
