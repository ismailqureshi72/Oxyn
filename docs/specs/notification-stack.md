**Status:** v1 — implemented (plan #26).

# Spec: notification stack

> Implemented in `oxyn-core::primitives::toast` (`NotificationStack`, `Notice`), `oxyn-dioxus::NotificationStack`, `oxyn-gpui::NotificationStackEl`.
> Source: affectUI Notification Stack (behaviour only, no source used).

## 1. Purpose
Several standing notices that each need a moment: maintenance windows, invites, release notes. Stacking keeps them in one place; the user deals with the front one and the next takes its place.

## 2. State machine
```
Stack(n) ─dismiss front─▶ Leaving(220 ms) ─▶ Stack(n − 1)   (next card moves forward)
Stack(n) ─action─▶ on_action(id) (card stays)
Stack(0) ─▶ "You're all caught up"
After a dismiss ─Undo─▶ the notice returns to the front   Oxyn plus
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| stacked | front card + `depth − 1` edges behind | Tab to its buttons | region; front `article` labelled by its title |
| leaving | front card drops 12 px and fades | — | — |
| empty | caught-up line | — | `status` |
| undo | Undo button | Enter | — |

## 4. Micro-interactions
Cards behind scale down 5 % per level and lift 10 px; when the front one leaves the rest move forward on a spring. Patterns fade toward the bottom right.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Dismiss | 220 ms | `ease.out` | instant |
| Move forward | 320 ms | `ease.out` | instant |

## 6. Variants
Free: badge, title, text, action (internal or external), dismiss.

**Oxyn plus:** stack depth, priority ordering, Undo, background patterns (grid, dots, lines), `1 of 3` count.

## 7. Edge cases
Long text wraps; the stack grows with the front card. One notice draws no edges. Priority ties keep insertion order.

## 8. Accessibility
Only the front card is focusable and read; cards behind are `aria-hidden` and inert. The dismiss button is named "Dismiss <title>"; external actions show an external icon.

## 9. Props sketch
```rust
NotificationStack { notices: Vec<Notice { id, title, text, badge, action, external, pattern, priority }>,
                    depth: usize, by_priority, undo, label,
                    on_dismiss: EventHandler<String>, on_action: EventHandler<String> }
```

## 10. Related components and references
`toaster.md`, `alert.md`. affectUI `/components/notification-stack` — behaviour only.
