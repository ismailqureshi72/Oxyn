**Status:** v1 — implemented (plan #27).

# Spec: breadcrumb

> Implemented in `oxyn-core::primitives::layout (`Crumb`, `crumb_slots`)`, `oxyn-dioxus::Breadcrumb`, `oxyn-gpui::BreadcrumbEl`.
> Source: Arc UI Breadcrumb (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Shows where a run, file or setting sits (Acme › Atlas › Runs › Run 251) and lets the user jump back up.

## 2. State machine
```
Path(n) ─n grows─▶ new crumb slides in from a blur (old crumbs stay still)
Path(n > max) ─▶ Collapsed(root, …, tail) ─press …─▶ Expanded(all)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| link | muted text, hover fill | Tab, Enter | `a` / `button` |
| current | medium weight | — | `aria-current=page` |
| collapsed | “…” button | Enter expands | “Show N more” |

## 4. Micro-interactions
Only the newest crumb animates in (8 px from the left out of a 3 px blur); the rest never move. Hover fills the crumb.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Crumb in | 260 ms | `ease.out` | instant |
| Hover fill | 120 ms | linear | instant |

## 6. Variants
Free: chevron separators, links or buttons, wrap.

**Oxyn plus:** `max` collapse behind “…”, `slash` separators, crumb icons.

## 7. Edge cases
Long paths wrap between crumbs, never inside one. A single crumb renders just the current page. Crumbs without `href` fire `on_select(i)`.

## 8. Accessibility
`nav` landmark (“Breadcrumb”) with an `ol`; separators `aria-hidden`; the current page is a `span` with `aria-current=page`, never a link.

## 9. Props sketch
```rust
Breadcrumb { items: Vec<Crumb>, label: String, max: usize, separator: "chevron"|"slash", on_select: EventHandler<usize> }
```

## 10. Related components and references
`page-header.md`, `workspace-chrome.md`. Arc UI Breadcrumb — behaviour only.
