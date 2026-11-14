**Status:** v1 — implemented (plan #26).

# Spec: sheet stack

> Implemented in `oxyn-core::primitives::modal` (`SheetStack`, `StackSheet`, `StackMode`, `Level`), `oxyn-dioxus::SheetStack` (+ `StackPage`, `StackRow`), `oxyn-gpui::SheetStackEl`.
> Source: Arc UI Sheet stack (behaviour only, no source used).

## 1. Purpose
Settings that go deeper than one screen: run settings → tools → shell access. Each level keeps the one below visible so the user knows where they came from and how to get back.

## 2. State machine
```
Empty ─push(id)─▶ [a]
[a … x] ─push(y)─▶ [a … x, y]   (y rises; x scales back, dims, peeks above)
[a … x, y] ─Back / Esc / drag down / fling─▶ [a … x]
[…] ─Close all / scrim─▶ Empty
auto mode: width < breakpoint → sheets from the bottom, ≥ breakpoint → stacked dialogs
side mode (Oxyn plus): inspectors stacking from the right
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| one sheet | sheet with title, rows, close | Esc closes | `dialog`, `aria-modal` |
| stacked | top sheet + lower sheets scaled 0.95ⁿ, lifted, dimmed | Esc closes the top one | lower sheets `aria-hidden`, `inert` |
| back | "‹ <sheet below>" button | Enter | `aria-label=Back to <title>` |
| close | × labelled Close or Close all | Enter | — |
| dragging | top sheet follows; the one below rises toward its place | — | — |

## 4. Micro-interactions
A new sheet rises from the bottom while the one below scales back 5 %, dims 18 % and peeks 12 px above it. While the top sheet is dragged the lower one moves back toward full size in proportion. Titles rise in; the breadcrumb tracks the trail.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Push | 380 ms | `ease.out` | 120 ms fade |
| Pop / depth change | 340 ms | `ease.out` | instant |
| Dialog mode in | 220 ms | `ease.out` | 120 ms fade |

## 6. Variants
Free: bottom sheets, stacked dialogs, auto by breakpoint, dismissible or not per sheet.

**Oxyn plus:** `side` mode, a breadcrumb of open sheets, rows that push (`StackRow::push`), switch rows, footer actions.

## 7. Edge cases
Deep stacks keep at most three visible levels; deeper ones sit hidden behind. Narrow screens lose the peek room per level. A non-dismissible sheet ignores drag and scrim but still has Back.

## 8. Accessibility
Only the top sheet is modal and focusable; the rest are inert. The back button names its destination; the close button says Close all when it will close more than one. Escape closes one level at a time.

## 9. Props sketch
```rust
SheetStack { pages: Vec<StackPage { sheet: StackSheet { id, title, back_label, dismissible },
                                     description, rows: Vec<StackRow>, footer }>,
             default_stack: Vec<String>, mode: "auto"|"sheet"|"dialog"|"side", breakpoint: f32,
             contained, trigger_label, breadcrumb, on_stack_change: EventHandler<Vec<String>> }
```

## 10. Related components and references
`sheet.md`, `dialog.md`, `bottom-sheet.md`. Arc UI `/docs/components/sheet-stack` — behaviour only.
