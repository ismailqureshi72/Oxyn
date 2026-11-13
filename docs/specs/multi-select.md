**Status:** v1 — implemented (plan #25).

# Spec: multi-select

> Implemented in `oxyn-core::primitives::combobox` (`Combobox::multi_select`), `oxyn-dioxus::MultiSelect`,
> `oxyn-gpui::ComboboxEl`. Source: Arc UI Multi-select (behaviour only, no source used).

## 1. Purpose
Several values in a compact field that looks like a select: which tools an agent may use, which
repositories a run touches. The user opens a list, ticks a few, and wants the trigger to stay one
line tall.

## 2. State machine
```
Closed ─click / Enter / Space / ↓ / ↑─▶ Open(active = first selected, else first enabled)
Open ─↑ ↓ (wrap) Home End─▶ Open(active')
Open ─Enter / click─▶ toggle(active) (stays open)
Open ─Esc / Tab / outside─▶ Closed
Closed, values ─Clear─▶ Closed(empty)
At max_selected ─toggle on─▶ refused + reason                Oxyn plus
```

## 3. States table
| State | Trigger | List | ARIA |
|---|---|---|---|
| placeholder | "Select options" | — | `aria-haspopup=listbox`, `aria-expanded=false` |
| values | first `max_visible` chips + `+N` | — | labelled by label + hidden summary |
| open | chevron turned | listbox with checkboxes | `aria-expanded=true`, `aria-activedescendant` |
| selected option | — | box fills, tick draws | `aria-selected=true` |
| disabled option | — | dimmed, skipped | `aria-disabled=true` |
| Clear | × button | — | "Clear selections" |
| disabled | 50 % | — | `disabled` |

## 4. Micro-interactions
Chips grow in from the left; `+N` rolls as the count changes; the box fills and its tick draws;
the menu lifts in from the trigger. Clear collapses the chips together.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Chip grow | 200 ms | `ease.out` | 1 ms |
| `+N` roll | 200 ms | `ease.out` | swap |
| Check | 220 ms fill + 240 ms draw | `ease.out` | drawn |
| Menu | 180 ms | `ease.out` | 80 ms fade |

## 6. Variants
Free: placeholder, values with `+N`, description, disabled options, default open, Clear.

**Oxyn plus:** Select all row, `max_selected` with a reason, searchable mode (the same model as
`combobox`), review-sheet `open`.

## 7. Edge cases
Chips cap at 9 rem with ellipsis; `max_visible` keeps the trigger one line. Everything disabled:
the list shows but nothing toggles. Long lists scroll; the active row stays in view. Narrow
screens: the menu spans the field.

## 8. Accessibility
The trigger is a button with `aria-haspopup=listbox`; its name is the label plus a visually hidden
comma-joined list of picked labels (chips are `aria-hidden`). The menu is a `listbox` with
`aria-multiselectable`; options carry `aria-selected` / `aria-disabled`. Changes are announced
politely ("Added Shell, 3 selected").

## 9. Props sketch
```rust
MultiSelect { label, options: Vec<ComboOption>, value: Vec<String>, placeholder, description,
              max_visible: usize, select_all, max_selected, open, disabled,
              on_change: EventHandler<Vec<String>> }
```

## 10. Related components and references
`combobox.md` (searchable input trigger), `select.md`, `badges-chips.md`. Arc
`/components/multi-select` — behaviour only.
