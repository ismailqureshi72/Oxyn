**Status:** v1 — implemented (plan #27).

# Spec: dock

> Implemented in `oxyn-core::primitives::layout (`Dock`, `DockItem`)`, `oxyn-dioxus::Dock`, `oxyn-gpui::DockEl`.
> Source: Arc UI Dock (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Tools or modes switched all day: canvas tools, agent views, quick actions.

## 2. State machine
```
Item ─press─▶ selected   Group ─press─▶ shown member selected + tray toggles
Group ─↑─▶ tray open, focus member ─←→─▶ move ─Enter─▶ choose, close ─↓ / Escape─▶ close
Alt + ← → ─▶ move the focused item
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| item | icon | ← → Home End | `aria-pressed` |
| selected | tint | — | pressed |
| group | corner mark, tray | ↑ ↓ Escape | `aria-haspopup`, `aria-expanded` |
| badge | count | — | in the name |

## 4. Micro-interactions
A label glides between items on hover and focus; the tint springs to the chosen item; a group’s glyph crossfades when its member changes; trays grow out of the dock with members staggering in.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Label | 120 ms fade | linear | instant |
| Tint | 260 ms | overshoot | instant |
| Tray | 220 ms + 30 ms stagger | `ease.out` | instant |

## 6. Variants
Free: items, groups, shortcuts, badges.

**Oxyn plus:** `magnify`, vertical `left` dock, reorder toggle.

## 7. Edge cases
Labels never show on touch. Closed trays are inert. Reorder keeps an open tray attached to its group.

## 8. Accessibility
`toolbar` with roving focus; names include shortcut and badge (“Pen, P”; “Shapes: Ellipse, O, selected”).

## 9. Props sketch
```rust
Dock { items: Vec<DockItem>, value: Option<String>, label, magnify: bool, reorder: bool, position: "bottom"|"left",
      on_change: EventHandler<String> }
```

## 10. Related components and references
`app-shell.md`, `button-groups.md`. Arc UI Dock — behaviour only.
