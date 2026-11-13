**Status:** v1.1 — implemented (plan #23); Arc parity gaps closed in plan #25.

# Spec: select

> Implemented in `oxyn-core::primitives::select` (`Select`, `SelectOption`, `SelectKey`,
> `ListRow`, `ValueMove`) with `nav::Typeahead` and `button::Glide`, `oxyn-dioxus::Select`,
> `oxyn-gpui::SelectEl`. Placement comes from `primitives::placement::place`. Sources: affectUI
> Select, Arc Select (behaviour only).

## 1. Purpose
Pick one option in the text field's look: model, region, language, fallback provider. Unlike the
dropdown menu it holds a *value* and lives in forms.

## 2. State machine
```
Closed(value) ─click / ↓ / ↑ / Enter / Space─▶ Open(highlight = chosen, else first enabled)
Closed ─A–Z─▶ Closed(value = next match)            (native type-ahead, no list)
Open ─↑ ↓ Home End / pointer─▶ Open(highlight')     (pointer glides, keys jump)
Open ─A–Z─▶ Open(highlight = match)                  (500 ms buffer; repeating a letter cycles)
Open ─Enter / Space / click option─▶ Closed(value') + ValueMove(later|earlier)
Open ─Esc / Tab / outside─▶ Closed(value)
Searchable (Oxyn plus): Open ─type─▶ Open(query, highlight = first enabled match) ; Backspace edits
Loading (Oxyn plus): Open shows three skeleton rows.
```

## 3. States table
| State | Trigger | List | ARIA |
|---|---|---|---|
| placeholder | "Choose…" in `fg.subtle` | — | `combobox`, `aria-haspopup=listbox`, `aria-expanded=false` |
| value | label (+ icon); the new label rises from below (later option) or drops from above (earlier) | — | — |
| hover | edge darkens (no ring) | — | — |
| focus / open | edge darkens more; chevron turned 180° | lifts in from the field's edge | `aria-expanded=true`, `aria-controls`, `aria-activedescendant` |
| highlighted row | — | one soft highlight glides between rows | `option`, `aria-selected` on the chosen one |
| chosen row | — | check draws itself | — |
| disabled option | — | `fg.disabled`, skipped | `aria-disabled=true` |
| group heading | — | small caps | `presentation` |
| empty filter | — | "No matches" | `role=status` |
| error | danger edge + one nudge; message replaces the hint | — | `aria-invalid=true` |
| disabled | 50 % | — | `disabled` |
| opens upward | list above when there is no room below | lifts downward | — |

## 4. Micro-interactions
The list fades in out of a 4 px lift from the field's edge (downward, or upward when it opens
above). One highlight glides between rows on `spring.snap` for the pointer and jumps for keys. The
chevron turns half a circle. The chosen check draws itself. The value label rises / drops with a
short blur (Arc).

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| List open | 180 ms fade + 4 px lift | `ease.out` | 80 ms fade |
| Highlight glide | 214 ms `spring.snap` | spring | jump |
| Chevron | 180 ms | `ease.out` | instant |
| Value rise / drop | 180 ms, 5 px, blur 3 → 0 | `ease.out` | crossfade 80 ms |
| Check draw | 240 ms | `ease.out` | drawn |
| Type-ahead window | 500 ms | — | — |
| Error nudge | 300 ms, 4 → 1 px | linear | none |

## 6. Variants
Source: `options` (icon, second line), `value`, `defaultValue`, `placeholder`, `label`, `hint`,
`error`, `optional`, `size` sm / md / lg, `disabled`, `name`, `required`, `aria-label`, `onChange`.

**Oxyn plus:** searchable filter, grouped options with headings, async options with skeleton rows,
`required` validation message ("Choose an option"), `native` renderer.

## 7. Edge cases
Long option lists scroll inside a 280 px list; the highlighted row scrolls into view. A value not in
the options shows the placeholder. Disabled chosen option still shows as the value. Narrow screens:
the list matches the trigger width. Type-ahead skips disabled options and wraps.

## 8. Accessibility
Trigger is a `combobox` button labelled by the field label + its value; the list is a `listbox`
with `option`s; `aria-activedescendant` follows the highlight so focus can stay on the trigger. A
hidden input carries `name` / value for forms. Esc returns without changing the value.

## 9. Props sketch
```rust
Select { options: Vec<SelectOption { value, label, icon, description, group, disabled }>, value,
         placeholder, label, hint, error, optional, required, size, disabled, searchable, loading,
         up, open, name, native, now, on_change }
```

## 10. Related components and references
`text-field.md` (shared field look), `dropdown-menu.md` (same highlight and type-ahead),
`composer-model-picker.md` (a richer model chooser). affectUI `/components/select`; Arc
`/components/select` — behaviour only.
