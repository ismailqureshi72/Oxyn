**Status:** v1 — implemented (plan #27).

# Spec: drawer

> Implemented in `oxyn-dioxus::Drawer` (built on `Sheet` and `oxyn-core::primitives::modal`), GPUI via `SheetEl`.
> Source: Arc UI Drawer (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
A temporary side surface for focused work next to the transcript: run details, a file preview, a settings panel.

## 2. State machine
```
Closed ─trigger─▶ Opening (spring in from side) ─▶ Open
Open ─Escape / close / scrim / drag past ⅓ or flick─▶ Closing (tween) ─▶ Closed (focus → trigger)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| open | panel + scrim | Escape, Tab cycles | `dialog` `aria-modal` |
| dragging | follows the pointer | — | — |
| closed | trigger only | — | — |

## 4. Micro-interactions
Springs in from its edge and leaves faster; the header is the drag handle and rubber-bands away from the edge. Inside previews it is `contained` so it never covers the page.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Enter | 280 ms | spring | fade |
| Exit | 200 ms | `ease.in` | fade |
| Scrim | 200 ms | linear | linear |

## 6. Variants
Free: right, left, top, bottom; description; footer.

**Oxyn plus:** `inset` floating panel, `contained` mode.

## 7. Edge cases
Side drawers cap at the window minus a gutter; top / bottom span the width and cap at 32rem. Start-open demos render inside an app frame.

## 8. Accessibility
Dialog semantics with the title and description wired; close button labelled; dragging is never the only way out.

## 9. Props sketch
```rust
Drawer { title, description: Option<String>, side: "right"|"left"|"top"|"bottom", trigger_label, size: f32,
        inset: bool, contained: bool, footer: Option<String>, children }
```

## 10. Related components and references
`sheet.md`, `bottom-sheet.md`, `side-panels.md`. Arc UI Drawer — behaviour only.
