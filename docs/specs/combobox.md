**Status:** v1 — implemented (plan #25).

# Spec: combobox

> Implemented in `oxyn-core::primitives::combobox` (`Combobox`, `ComboOption`, `ComboRow`,
> `ComboKey`, `ComboEvent`, `MatchMode`), `oxyn-dioxus::Combobox`, `oxyn-gpui::ComboboxEl`.
> Source: affectUI Combobox (behaviour only, no source used).

## 1. Purpose
Pick one or many values from a long, searchable list: assign people and agents, choose tools,
tag a run. The user types a few letters and wants the right row highlighted and chosen without
leaving the keyboard.

## 2. State machine
```
Closed ─focus + type / ↓ / ↑ / click─▶ Open(active = first enabled match)
Open ─type─▶ Open(query', rows filtered, active = first match; groups keep headings)
Open ─↑ ↓ Home End / pointer─▶ Open(active')
Open ─Enter on option─▶ multiple: toggle, stay open, query clears | single: pick, close
Open ─Enter on Create─▶ Created(query) + picked
Open ─Esc─▶ Closed (query kept) ; Closed ─Esc─▶ query cleared
Open ─Tab / blur─▶ Closed
Empty query ─⌫─▶ chip marked ─⌫─▶ chip removed
Loading (async source) ─results─▶ Open(rows)                     Oxyn plus
At max_selected ─pick─▶ refused + reason announced             Oxyn plus
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed, empty | placeholder, chevron | ↓ opens | `combobox`, `aria-expanded=false` |
| closed, values | chips (avatar), `+N` past `max_visible_tags` | ⌫ marks last chip | — |
| open | grouped listbox under the field | ↑ ↓ Home End Enter Esc Tab | `aria-expanded`, `aria-controls`, `aria-activedescendant` |
| active row | soft highlight | Enter toggles | `option` |
| selected row | drawn check / filled checkbox | — | `aria-selected=true` |
| match | matched letters bold + underlined | — | — |
| create row | `+ Create “…”` | Enter creates | `option` |
| no results | "No results" | — | `status` |
| loading (Oxyn plus) | two skeleton rows | — | `aria-busy` |
| disabled option | `fg.disabled`, skipped | — | `aria-disabled` |

## 4. Micro-interactions
The list lifts in from the field edge and its height animates as rows filter. The checkbox fills
and its tick draws. Chips grow in; `+N` rolls. The chevron turns 180°. Matched letters highlight
without layout shift.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| List open | 180 ms, 4 px lift | `ease.out` | 80 ms fade |
| List height | 214 ms | `ease.out` | instant |
| Check fill + tick | 220 ms + 240 ms draw | `ease.out` | drawn |
| Chip in | 160 ms | `ease.out` | 1 ms |
| `+N` roll | 200 ms | `ease.out` | swap |
| Async debounce (Oxyn plus) | 200 ms | — | — |

## 6. Variants
Free: multiple with chips, single, grouped, avatars, icons + second line, creatable, overflow
`+N`, default open.

**Oxyn plus:** match mode (contains / prefix / fuzzy), keywords per option, `max_selected` with a
spoken reason, Select all within the current filter, async source with a debounce and skeletons.

## 7. Edge cases
Long labels truncate in rows and chips. Very long lists scroll inside 280 px; the active row
scrolls into view. Typing a value that exists exactly hides Create. Fuzzy match keeps order of
letters. Narrow screens: the list matches the field width; touch rows are 44 px. RTL: chips and
checkmarks mirror.

## 8. Accessibility
ARIA 1.2 combobox pattern: the input keeps focus, `aria-activedescendant` points at the active
row; the list is a `listbox` (`aria-multiselectable` in multiple mode) with group headings as
`presentation`. Picks and removals are announced politely with the count ("Added Maya Patel, 2
selected"). Chip × buttons are labelled and do not steal focus.

## 9. Props sketch
```rust
Combobox { options: Vec<ComboOption { value, label, hint, group, icon, avatar, keywords, disabled }>,
           value: Vec<String>, multiple, placeholder, creatable, create_label, max_visible_tags,
           label, default_open, empty_text, match_mode, max_selected, select_all, disabled,
           on_change: EventHandler<Vec<String>>, on_create: EventHandler<String> }
```

## 10. Related components and references
`multi-select.md` (button trigger, same model), `select.md` (single value, no search by default),
`tag-input.md` (free entry). affectUI `/components/combobox` — behaviour only.
