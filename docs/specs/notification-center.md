**Status:** v1 — implemented (plan #27).

# Spec: notification center

> Implemented in `oxyn-core::primitives::chrome (`NotificationCenter`, `Note`, `NoteView`)`, `oxyn-dioxus::NotificationCenter`, `oxyn-gpui::NotificationCenterEl`.
> Source: Arc UI Notification center (behaviour only, rebuilt from the public docs; no source used).

## 1. Purpose
Approvals waiting, runs finished, mentions and system notices in one place.

## 2. State machine
```
Closed ─bell─▶ Open(view) ─All / Unread─▶ view
Row ─press─▶ expanded + read   ─x─▶ removed (focus → next)
Mark all read · Clear
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| unread | bold, dot | Enter | label “…, unread” |
| read | muted | Enter | — |
| expanded | body | Enter | `aria-expanded` |
| empty | “You’re all caught up” | — | — |

## 4. Micro-interactions
The bell tilts when the panel opens; the badge and summary roll; rows expand their own height and cascade in.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Tilt | 520 ms | `ease.out` | none |
| Row expand | 260 ms | `ease.out` | instant |
| Cascade | 220 ms + 30 ms | `ease.out` | instant |

## 6. Variants
Free: bell + panel, views, bulk actions.

**Oxyn plus:** kind icons (approval · run · mention), `inline` panel for sidebars.

## 7. Edge cases
Unread view hides rows as they are read; focus moves to the next row. The panel caps at the viewport.

## 8. Accessibility
Bell label carries the unread count, badge `aria-hidden`; summary `aria-live=polite`; view toggles `aria-pressed`.

## 9. Props sketch
```rust
NotificationCenter { items: Vec<Note>, open: bool, inline: bool, view: "all"|"unread", on_open: EventHandler<String> }
```

## 10. Related components and references
`notification-stack.md`, `toaster.md`, `inbox-sidebar` (sidebar.md). Arc UI Notification center — behaviour only.
