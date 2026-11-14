**Status:** v1.1 — implemented (plan #23); Arc parity gaps closed in plan #25.

# Spec: switch, checkbox, radio group

> Implemented in `oxyn-core::primitives::toggle` (`Switch`, `Check`, `check_draw`, `select_all`,
> `RadioGroup`, `RadioOption`, `RadioLayout`, `ToggleSize`) and `nav` (arrows), `oxyn-dioxus::{Switch,
> Checkbox, RadioGroup}`, `oxyn-gpui::{SwitchEl, CheckboxEl, RadioGroupEl}`. Sources: affectUI Switch
> and Checkbox, Arc Switch and Radio group (behaviour only).

---

## A. Switch

### 1. Purpose
On / off for settings that apply right away: auto-approve reads, stream reasoning, sounds.

### 2. State machine
```
Off ─press─▶ Off·Pressed (thumb stretches toward On) ─release─▶ On   (leave while pressed: cancel)
On  ─press─▶ On·Pressed  (stretches toward Off)       ─release─▶ Off
Async (Oxyn plus): Off ─toggle_async─▶ On·Pending (spinner thumb, input ignored) ─ok─▶ On
                                                                               └fail─▶ Off (rolls back)
Disabled: ignores everything.
```

### 3. States table
| State | Track | Thumb | Keyboard | ARIA |
|---|---|---|---|---|
| off | `border.strong` | left | Space / Enter toggles | `role=switch`, `aria-checked=false` |
| on | `fg.default` (monochrome) | right | same | `aria-checked=true` |
| pressed | — | 28 % wider toward the target | Space held | — |
| focus | 2 px focus ring | — | — | — |
| pending | — | spinner inside | ignored | `aria-busy=true` |
| disabled | 45 % | — | skipped | `disabled` |
| restart note | "Requires restart" under the label after a change | — | — | `role=status` |

### 4–5. Micro-interactions and timing
| Transition | Value | Reduced motion |
|---|---|---|
| Thumb travel | 214 ms `spring.snap` | instant |
| Press stretch | 214 ms, +28 % width, anchored edge still | none |
| Track colour | 180 ms fade | 80 ms |
| Note | 180 ms fade in | 80 ms |

### 6. Variants
Source: `label`, `description`, `placement` start / end, `size` sm / md; Arc `checked`,
`defaultChecked`, `onCheckedChange`. **Oxyn plus:** `xs` and `lg` sizes, pending (async apply) with
roll-back, "requires restart" note, hidden label, form `name`. Space held down stretches the thumb
(Arc).

### 7. Edge cases
Label click toggles (label wraps the control). `end` placement spans the row (settings lists). A
failed async apply rolls back and the host shows why (toast). Double clicks during pending are
ignored.

### 8. Accessibility
`button role=switch` with `aria-checked`; the second line is `aria-describedby`. Enter is accepted
as well as Space. State is never colour-only: the thumb position carries it.

### 9. Props
`Switch { label, description, checked, placement: "start|end", size: "sm|md|lg", disabled, pending, restart_note, on_change }`

---

## B. Checkbox

### 1. Purpose
Choices that are confirmed later (forms, file pickers, "I understand"), and Select all over a list.

### 2. State machine
`Unchecked ⇄ Checked`; `Indeterminate → Checked` on activation (native). Tri-state (Oxyn plus):
`Unchecked → Checked → Indeterminate → Unchecked`. `Check::of_all(children)` derives Select all;
`select_all` checks all unless all are checked.

### 3. States table
| State | Box | ARIA |
|---|---|---|
| unchecked | 1.5 px `border.strong` | native checkbox, `aria-checked=false` |
| checked | filled `fg.default`, check draws itself | `true` |
| indeterminate | filled, dash grows from the middle | `mixed` |
| error | danger border + message in the second line | `aria-invalid=true`, described by the message |
| card | bordered row as the hit target; border strengthens when checked | same |
| pressed | box nudges to 92 % | — |
| disabled | 45 % | `disabled` |

### 4–5. Timing
Check draw 240 ms (`dur.check.draw`) × `speed` (Oxyn plus); dash grow 240 ms; fill 120 ms; press
80 ms. Reduced motion: drawn at once.

### 6. Variants
Source: `label`, `description`, `indeterminate`, `error`, `variant` plain / card; Arc `name`,
`required`, `aria-label` fallback. **Oxyn plus:** sizes sm / md / lg, draw speed, tri-state cycle,
`todo` (strike-through on check), `bulk` (approve-all row). Tick ⇄ dash is one morphing path
(200 ms `d` transition).

### 7. Edge cases
Long labels wrap beside the box (box stays top-aligned). Error replaces the description in the same
slot so nothing below moves.

### 8. Accessibility
A real `<input type=checkbox>` under the drawn box (keyboard, forms, screen readers for free);
Space toggles; `aria-checked=mixed` for some-of-them.

### 9. Props
`Checkbox { label, description, value: "unchecked|checked|indeterminate", error, variant: "plain|card", size, speed, tri_state, disabled, on_change }`

---

## C. Radio group

### 1. Purpose
Pick exactly one from a short visible set: approval mode, effort, plan tier.

### 2. State machine
One tab stop (the selected option, else the first enabled). Arrow keys (both axes) move *and
select*, skipping disabled options and wrapping; Home / End jump. Click selects. The highlight glides
from the previous selection to the new one while the new dot springs in and the old one shrinks.

### 3. States table
| State | Shows | ARIA |
|---|---|---|
| list (default) | dot + label + second line; selected row has the soft highlight | `radiogroup` labelled by the legend; `radio` + `aria-checked` |
| cards (Oxyn plus) | bordered cards, selected border strengthens | same |
| inline (Oxyn plus) | dots in a row | same |
| disabled option | 45 %, reason as tooltip (Oxyn plus) | `aria-disabled=true` |

### 4–5. Timing
Highlight glide `spring.snap` 214 ms; dot spring-in 214 ms; old dot shrink 120 ms. Reduced motion:
highlight and dot placed instantly; resizes never animate (Arc).

### 6. Variants
Arc: a fieldset of radios with one gliding highlight, a `name` for form submission (hidden input).
**Oxyn plus:** list / cards / inline / segmented renderers, descriptions per option,
disabled-with-reason tooltips.

### 7. Edge cases
No initial value: first enabled option is the tab stop and nothing is selected until the user acts.
All options disabled: no tab stop. Long descriptions wrap.

### 8. Accessibility
`role=radiogroup` + `aria-labelledby` legend; roving `tabindex`; `aria-checked` per option;
disabled reasons exposed through `title`.

### 9. Props
`RadioGroup { label, options: Vec<RadioOption { value, label, description, disabled: Option<reason> }>, value, layout: "list|cards|inline|segmented", name, on_change }`

## 10. Related components and references
`segmented-control.md` (same keyboard semantics as radio kind), `genui-forms.md` (forms built from
these), `composer-permission-selector.md`. affectUI `/components/switch`, `/components/checkbox`;
Arc `/components/switch`, `/components/radio-group` — behaviour only.
