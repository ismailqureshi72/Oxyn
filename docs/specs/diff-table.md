**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Diff table (`diff-table`)

Family: Diff & review. Oxyn plus for plan #30 (Beautiful UI Diff Table idea, behaviour only; diff-viewer.md `table-cells`). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
An agent changed settings, a record or environment variables. `table_diff(before, after)` aligns the keys by LCS and reports each row as added, removed, changed or same, keeping the new order and placing removed rows where they were.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| changed only | Added, changed and removed rows | toggle |
| all rows | Unchanged rows muted | toggle |

## 3. Timing and motion
- No motion; rows are static.
- Reduced motion: n/a.

## 4. Variants
| Variant | What differs |
|---|---|
| columns | Before and After columns |
| inline | One value column with the old value struck in front |
| all-rows | Unchanged rows shown |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Focus the scroll region |
| Space | Toggle Changed only when focused |

## 6. Accessibility
- Real table with a row header per key; a hidden cell speaks the change kind; removed values are struck and tinted.

## 7. Responsive
- Scrolls sideways inside its card below its natural width.

## 8. Edge cases
- A key moved and changed shows as changed in its new place.
- Long values wrap anywhere.
- No changes: "No changes".

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| rows | Vec<RowChange> | — | from `table_diff` |
| key_label | String | "Key" |  |
| changed_only | bool | true |  |
| columns | bool | true |  |

## 10. Gaps against the source
- No per-cell diff for multi-column records; one value per key.

## 11. GPUI
`DiffTableEl::new(rows).changed_only(bool)`.
