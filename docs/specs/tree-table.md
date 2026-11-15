**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Tree table (`tree-table`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Tree table.

## 1. Purpose
A table with nested rows: expand and collapse, tri-state selection where checking a parent checks every leaf under it and parents follow their children, sort that cycles ascending → descending → off and keeps each row under its parent, lazy children that load on first expand with a retry row on failure, a pinned header and an optional max height.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| collapsed / expanded | Chevron rotates |  |
| selected / mixed | Check / dash |  |
| loading | Spinner row |  |
| load failed | Couldn't load + Retry |  |
| empty | Nothing to show |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| select | Selectable |
| plain | Read-only |
| retry | Lazy load failed |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Sort buttons, toggles, checkboxes |

## 6. Accessibility
- role=treegrid; aria-level, aria-selected; checkboxes report mixed; aria-sort on headers.

## 7. Responsive
- Scrolls inside its frame with a pinned header.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | TreeTable | — | rows, expanded, selected, sort |
| columns | Vec<String> | — |  |
| selectable | bool | false |  |
| max_height | u32 | 0 |  |
| on_toggle/on_select/on_retry | EventHandler<String> | — |  |
| on_sort | EventHandler<usize> | — |  |

## 9. Edge cases
- Numeric keys sort numerically; others as text.

## 10. Implementation
- Core: `TreeTable`, `TRow`, `Tri` (`oxyn_core::parity`), with tests.
- Dioxus: `TreeTable` (parity/run.rs, run.css).
- GPUI: `TreeTableEl`.

## 11. References
- Arc UI Tree table (https://uiarc.dev/components/tree-table) — behaviour-only rebuild, no source copied.
