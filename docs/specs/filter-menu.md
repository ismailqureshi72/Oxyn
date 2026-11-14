**Status:** v1 — implemented (plan #25).

# Spec: filter menu

> Implemented in `oxyn-core::primitives::filter_menu` (`FilterMenu`, `FilterGroup`,
> `FilterOption`, `FilterRow`, `FilterKey`, `Logic`), `oxyn-dioxus::FilterMenu`,
> `oxyn-gpui::FilterMenuEl`. Source: affectUI Filter Menu (behaviour only, no source used).

## 1. Purpose
Narrow a run list, an inbox of approvals or a trace by status, agent and repository. The user
wants to see at a glance which filters are on and remove one without reopening anything.

## 2. State machine
```
rows = headers + options of open groups; focus = one row (roving)
↑ / ↓ ─▶ focus ∓ 1 ; Home / End ─▶ first / last
Enter / Space on header ─▶ toggle open (focus stays on the header)
Enter / Space on option ─▶ toggle the filter; chip added / removed; count rolls
→ on a closed header ─▶ open ; ← on a header ─▶ close ; ← on an option ─▶ close its group, focus header
chip × ─▶ filter removed ; Clear all ─▶ none
Any / All per group (Oxyn plus) ; Save view / apply view (Oxyn plus)
```

## 3. States table
| Row | Shows | ARIA |
|---|---|---|
| group header | chevron, label, picked count, Any / All (Oxyn plus) | `treeitem`, `aria-expanded` |
| option | checkbox, label, result count (Oxyn plus) | `checkbox`, `aria-checked` |
| focused row | soft surface + focus ring when keyboard | via `aria-activedescendant` |
| chips | "Status: Waiting on you" with × | remove button named per filter |
| header count | number pill | — |
| summary (Oxyn plus) | "Status is Open or Blocked · Agent is Coder" | — |

## 4. Micro-interactions
Groups open and close with their options sliding in; the chevron turns 90°. Chips pop in and the
chip row's height animates. The active-count pill rolls. Checkboxes fill and draw their tick.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Option rows in | 160 ms | `ease.out` | 1 ms |
| Chevron | 214 ms | `ease.out` | instant |
| Chip in | 180 ms | `ease.out` | 1 ms |
| Count roll | 180 ms | `ease.out` | swap |
| Check | 220 ms + 240 ms draw | `ease.out` | drawn |

## 6. Variants
Free: groups with options, starting filters, which groups start open, title, keyboard hints.

**Oxyn plus:** AND / OR per group, saved views (presets), counts per option, plain-language
summary.

## 7. Edge cases
Many options: the list scrolls inside its container. Long labels wrap in rows and truncate in
chips. Removing the last filter hides Clear all. Up to 420 px wide; chips wrap at narrow widths.
RTL: chevrons and indentation mirror.

## 8. Accessibility
One tab stop for the list (`role=tree` with `aria-activedescendant`), so Tab leaves in one step.
Headers expose `aria-expanded`; options are `checkbox`es. Every chip has a remove button named
"Remove Status: Open". Changes are announced: "Added Coder filter, 2 active".

## 9. Props sketch
```rust
FilterMenu { groups: Vec<FilterGroup { id, label, options: Vec<FilterOption { id, label, count }>,
             chip_prefix }>, value: Vec<(String, String)>, default_open: Option<Vec<String>>, title,
             hints, logic, presets, summary, on_change: EventHandler<Vec<(String, String)>> }
```

## 10. Related components and references
`dropdown-menu.md`, `badges-chips.md`, `background-jobs.md` (where it filters). affectUI
`/components/filter-menu` — behaviour only.
