**Status:** v1 — implemented (plan #26).

# Spec: sheet

> Implemented in `oxyn-core::primitives::modal` (`Dialog`, `SheetSide`, `EdgeDrag`), `oxyn-dioxus::Sheet`, `oxyn-gpui::SheetEl`.
> Source: Oxyn plus (pr-plan #26 lists a side sheet); drag rules shared with Arc UI Bottom sheet.

## 1. Purpose
Details that sit beside the work instead of over it: a run inspector, a tool call's full output, a settings panel, a file preview. The page stays visible under a lighter scrim.

## 2. State machine
```
Closed ─trigger─▶ Open (slides from its side, 320 ms)
Open ─Esc / scrim / ×─▶ Closing(230 ms) ─▶ Closed, focus → trigger
Open ─drag handle─▶ Dragging(offset follows 1:1 toward the edge, scrim lightens)
Dragging ─release past 35 % or fling > 450 px/s─▶ Closing
Dragging ─release otherwise─▶ Open (springs back)
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| open | panel docked to `side`, header, scrolling body, optional footer | Esc closes | `dialog`, `aria-modal`, `aria-labelledby` |
| dragging | panel offset, scrim opacity = 1 − offset / size | — | — |
| closing | slides out to its side | — | focus returns |
| inset | floating with a 10 px gutter and 14 px corners | same | same |

## 4. Micro-interactions
Slides in from its edge; the scrim fades. Dragging the grip tracks the pointer 1:1 and lightens the scrim; a release either springs back or completes the slide out.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Slide in | 320 ms | `ease.out` | 120 ms fade |
| Slide out | 230 ms | `ease.out` | instant |
| Spring back | 260 ms | `ease.out` | instant |

## 6. Variants
Free: right inspector, left, top, bottom tray.

**Oxyn plus:** `inset` floating style, drag-to-dismiss handle, pinned footer with an action, `contained`.

## 7. Edge cases
`size` is the width for left / right and the height for top / bottom, capped at the container. Long bodies scroll. On touch the grip is the drag target, so the body still scrolls.

## 8. Accessibility
Same dialog rules as `dialog.md`: modal, named by its title, focus inside, returned to the trigger. The grip is decorative; × and Escape are always available.

## 9. Props sketch
```rust
Sheet { title, description, open, trigger_label, side: "right"|"left"|"top"|"bottom",
        size: f32, inset, contained, close_on_backdrop, draggable, footer: Option<String>,
        children, on_open_change: EventHandler<bool> }
```

## 10. Related components and references
`dialog.md`, `sheet-stack.md`, `bottom-sheet.md`, `desktop-overlays.md`.
