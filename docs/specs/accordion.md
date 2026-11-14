**Status:** v1 — implemented (plan #27).

# Spec: accordion

> Implemented in `oxyn-core::primitives::layout (`Accordion`)`, `oxyn-dioxus::Accordion, AccordionItem`, `oxyn-gpui::AccordionEl`.
> Source: Arc UI Accordion (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
FAQ and settings sections, and collapsible detail in agent screens (policy, sandbox, billing) read in order.

## 2. State machine
```
Closed(i) ─toggle─▶ Open(i)   (single: closes the other open row)
Open(i) ─toggle─▶ Closed(i)   (single + collapsible)
multiple: Expand all / Collapse all
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| closed | trigger + chevron | Enter / Space, ↑ ↓ Home End | `aria-expanded=false` |
| open | answer below | same | `aria-expanded=true`, region |
| disabled | dimmed | skipped | `disabled` |

## 4. Micro-interactions
The panel grows on a grid-rows transition (retargets mid-flight) while the answer slides down 6 px out of a 2 px blur; the chevron rotates on a snappy spring.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Height | 320 ms | `ease.out` | instant |
| Answer slide | 320 ms | `ease.out` | instant |
| Chevron | 260 ms | spring-like overshoot | instant |

## 6. Variants
Free: single-open, `md` / `lg`, start closed (`default_open: None`).

**Oxyn plus:** `multiple` with Expand / Collapse all, `separated` and `card` styles, a meta column.

## 7. Edge cases
Closed panels stay mounted but `visibility:hidden`, so they leave the accessibility tree. `lg` caps answers at 62ch and tightens below 520 px.

## 8. Accessibility
Triggers in `h3`, wired with `aria-controls` / `aria-labelledby` to `region` panels; chevrons `aria-hidden`.

## 9. Props sketch
```rust
Accordion { items: Vec<AccordionItem>, default_open: Option<usize>, multiple: bool, size: "md"|"lg",
            variant: "plain"|"separated"|"card", controls: bool }
```

## 10. Related components and references
`reasoning-block.md` (collapsible thinking), `expandable-text.md`. Arc UI Accordion — behaviour only.
