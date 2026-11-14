**Status:** v1 — implemented (plan #27).

# Spec: page header

> Implemented in `oxyn-core::primitives::layout (`header_folded`)`, `oxyn-dioxus::PageHeader`, `oxyn-gpui::PageHeaderEl`.
> Source: Arc UI Page header (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
The top of a project or run page: where you are, what it is, actions, and its sections.

## 2. State machine
```
Expanded ─scroll > fold_at─▶ Folded (compact bar) ─scroll < fold_at − 8─▶ Expanded
Follow / Share / Copy link / Archive ─▶ status message
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| expanded | crumbs, title, meta, actions, tabs | Tab | tabs, `aria-current` crumb |
| folded | title + tabs, icon actions | same | same |
| following | pressed Follow | Enter | `aria-pressed` |

## 4. Micro-interactions
Folding shrinks the title and collapses crumbs and meta; tab counts roll.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Fold | 240 ms | `ease.out` | instant |

## 6. Variants
Free: crumbs, actions menu, tabs with counts.

**Oxyn plus:** agent status pill, start folded, 8 px hysteresis.

## 7. Edge cases
Long titles ellipsize; action labels hide when folded or narrow.

## 8. Accessibility
Tabs with tab roles, labelled More actions trigger, disabled items `aria-disabled`, polite status for results.

## 9. Props sketch
```rust
PageHeader { title, crumbs: Vec<Crumb>, meta: String, tabs: Vec<Tab>, fold_at: f32, height: f32, status: Option<String>, folded: bool }
```

## 10. Related components and references
`tabs.md`, `breadcrumb.md`, `app-shell.md`. Arc UI Page header — behaviour only.
