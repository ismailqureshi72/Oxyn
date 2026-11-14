**Status:** v1 — implemented (plan #27).

# Spec: reorderable list

> Implemented in `oxyn-core::primitives::layout (`move_item`, `drag_target`, `drag_shift`, `handle_label`)`, `oxyn-dioxus::ReorderableList, ReorderItem`, `oxyn-gpui::ReorderListEl`.
> Source: Arc UI Reorderable list (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Reorder the agent queue, steps of a plan, or pinned items.

## 2. State machine
```
Rest ─grip down─▶ Dragging(from) ─move─▶ target = round(dy / row) ─up─▶ Rest(reordered)
Handle ─↑ / ↓─▶ moved ±1, focus stays, announced
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| rest | grip, number, title | ↑ ↓ on grip | grip button label |
| dragging | lifted row, shadow | — | — |
| removable | x button | Tab | “Remove X” |

## 4. Micro-interactions
The dragged row follows the pointer and lifts; neighbours slide by one row on a spring; the position number rolls.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Neighbour slide | 220 ms | `ease.out` | instant |
| Number roll | 200 ms | `ease.out` | swap |

## 6. Variants
Free: drag + keyboard, numbers.

**Oxyn plus:** remove buttons, plain (no numbers).

## 7. Edge cases
Only the grip starts a drag (touch-action none), so the rest of the row scrolls. Moving past either end clamps.

## 8. Accessibility
Grip: “Reorder Ship pricing page. Position 1 of 4. Use arrow keys.”; focus stays on the moved handle; a status region announces “moved to position 2 of 4”.

## 9. Props sketch
```rust
ReorderableList { items: Vec<ReorderItem>, label: String, numbered: bool, removable: bool, on_reorder: EventHandler<Vec<String>> }
```

## 10. Related components and references
`composer-queue-stack.md`, `plan-todo.md`. Arc UI Reorderable list — behaviour only.
