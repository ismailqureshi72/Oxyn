**Status:** v1 — implemented (plan #26).

# Spec: expanding search

> Implemented in `oxyn-core::primitives::search` (`ExpandingSearch`, `Anchor`, `SearchOutcome`), `oxyn-dioxus::ExpandingSearch`, `oxyn-gpui::ExpandingSearchEl`.
> Source: Arc UI Expanding search (behaviour only, no source used).

## 1. Purpose
A search that takes no room until needed: an icon in a toolbar that becomes a field with results, then folds back after a pick.

## 2. State machine
```
Folded(icon) ─Enter / Space / click / ⌘K─▶ Expanded(width grows to min(expanded_width, lane))
Expanded, empty ─▶ suggestions (Recent)
Expanded ─type─▶ grouped results (max_results), first active
Expanded ─Enter─▶ Select(id), Folded ; ─Esc / ×─▶ Folded
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| folded | round icon button | Enter opens | `button` named by `label` |
| expanded | field + results panel tracking its width | ↑ ↓ Enter Esc | combobox + listbox |
| no results | "No results for …" + hint | — | announced |

## 4. Micro-interactions
The icon morphs into the field: width grows with a soft spring away from the anchor and corners square off; the panel drops in and tracks the field width.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Expand / fold | 380 ms | spring (1.15) | instant |
| Panel in | 240 ms | `ease.out` | fade |

## 6. Variants
Free: anchors start · center · end, expanded width, max results, suggestions, empty hint.

**Oxyn plus:** remembered recent queries, ⌘K to expand.

## 7. Edge cases
The field never grows past its container. Results cap at min(22 rem, 60 vh). Folding clears the query.

## 8. Accessibility
The icon is a labelled button; the field is a combobox with `aria-activedescendant`; groups are labelled; counts are announced.

## 9. Props sketch
```rust
ExpandingSearch { label, items: Vec<Command>, suggestions, suggestions_label, placeholder,
                  expanded_width, max_results, anchor, empty_hint, default_expanded, query, hotkey,
                  lane, on_select: EventHandler<String>, on_expanded_change: EventHandler<bool> }
```

## 10. Related components and references
`search-field.md`, `command-palette.md`. Arc UI `/docs/components/expanding-search` — behaviour only.
