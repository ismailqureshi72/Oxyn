**Status:** v1 — implemented (plan #26).

# Spec: pagination

> Implemented in `oxyn-core::primitives::scroll` (`Pagination`, `PageMode`, `PageSlot`, `range_label`, `page_for_size`), `oxyn-dioxus::Pagination`, `oxyn-gpui::PaginationEl`.
> Source: Arc UI Pagination (behaviour only, no source used).

## 1. Purpose
Moving through runs, logs and table pages without losing your place. The user wants to see where they are and step or jump.

## 2. State machine
```
Page(p) ─prev / next / number─▶ Page(p'), window slides when p' leaves the middle
window: five slots centred on p, clamped at both ends
ellipsis (Oxyn plus): 1 … p−1 p p+1 … n
cursor (Oxyn plus): no total; Page n with prev / next
load-more (Oxyn plus): Showing k of N + Load more
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| first page | previous disabled | Tab skips | `disabled` |
| current | mark under the number | — | `aria-current=page` |
| last page | next disabled | — | — |
| with total | range "21–40 of 312" + rows per page | — | native select |

## 4. Micro-interactions
The current mark travels between numbers on a soft spring. When the window shifts the numbers slide like a belt by the number of slots it moved.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Mark travel | 300 ms | spring (1.3 overshoot) | instant |
| Belt shift | 340 ms | `ease.out` | instant |

## 6. Variants
Free: five-page window, previous / next, count.

**Oxyn plus:** ellipsis layout, cursor mode, load-more, range label with a page-size picker that keeps the first visible row.

## 7. Edge cases
Count 0 shows a single disabled page. Changing page size recomputes the page so the first visible row stays on screen. Window mode never changes width.

## 8. Accessibility
A `nav` landmark with a label. Each number is a button labelled "Go to page n" (the current one "Page n, current") with `aria-current`. Previous / next have labels and are disabled at the ends.

## 9. Props sketch
```rust
Pagination { page: usize, count: usize, mode: "window"|"ellipsis"|"cursor"|"load-more", label,
             total: Option<usize>, sizes: Vec<usize>, on_page_change: EventHandler<usize> }
```

## 10. Related components and references
`scroll-area.md`, `table` specs. Arc UI `/docs/components/pagination` — behaviour only.
