**Status:** v1 — implemented (plan #26).

# Spec: search field

> Implemented in `oxyn-core::primitives::search` (`clear_visible`, `filter_strings`), `oxyn-dioxus::SearchField`, `oxyn-gpui::SearchFieldEl`.
> Source: Arc UI Search field; Beautiful UI Search (MIT, © 2026 Shane Levine) — live filtering behaviour ported with attribution.

## 1. Purpose
Filter a list or start a search: settings pages, files, members. The field must not jump when the clear button appears, and Escape must clear.

## 2. State machine
```
Empty ─type─▶ Filled (clear button scales in inside its reserved slot)
Filled ─Esc / clear─▶ Empty (focus stays in the field)
With items: Filled ─▶ matches with highlighted letters | "No results found"
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| empty | icon, placeholder | — | native search input with a label |
| filled | clear button | Esc clears | clear button labelled |
| results (items) | list with gliding highlight | — | listbox, count announced |
| no results | "No results found / Adjust your search to try again" | — | status |
| disabled | dimmed | — | `disabled` |

## 4. Micro-interactions
The clear button scales in with a spring and presses to 0.96. A single highlight glides between result rows under the pointer.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Clear in | 200 ms | spring | none |
| Highlight glide | 214 ms | spring | 80 ms fade |

## 6. Variants
Free: sizes `sm` · `md` · `lg`, hidden label, disabled, live results (Beautiful UI).

**Oxyn plus:** scope chips, an Ask AI row for the query.

## 7. Edge cases
Long values scroll inside the input. Results cap at 220 px and scroll. The clear slot is always reserved so the width never shifts.

## 8. Accessibility
A real `<label>`; the clear button is labelled "Clear search" and returns focus to the input. Result counts are announced politely.

## 9. Props sketch
```rust
SearchField { label, value, placeholder, hide_label, size, disabled, items: Vec<String>,
              scopes: Vec<String>, ask_ai, width, on_change: EventHandler<String>, on_pick: EventHandler<String> }
```

## 10. Related components and references
`expanding-search.md`, `ai-search-bar.md`, `command-palette.md`. Arc UI `/docs/components/search-field` — behaviour only. Beautiful UI Search — MIT, © 2026 Shane Levine.
