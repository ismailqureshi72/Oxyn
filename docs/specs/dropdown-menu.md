**Status:** v1 — implemented (plan #23).

# Spec: dropdown menu (with gliding highlight) and context-menu mode

> Implemented in `oxyn-core::primitives::menu` (`Menu`, `MenuItem`, `ItemKind`, `MenuKey`,
> `MenuOutcome`, `Density`, `MenuAnchor`), `button::Glide` (highlight), `nav::Typeahead`,
> `placement::place`; `oxyn-dioxus::DropdownMenu` (`context` mode for right-click);
> `oxyn-gpui::MenuEl`. Sources: affectUI Dropdown Menu, Arc Dropdown menu and Context menu
> (behaviour only); Beautiful UI `glide-menu` (MIT, © 2026 Shane Levine — the one-highlight idea).

## 1. Purpose
Actions and choices behind a trigger: thread actions, "Move to", model and view options, a right-click
menu on a message or file. Agent apps put a lot here (rename, branch, export, delete), so it must be
fast from the keyboard and calm under the pointer.

## 2. State machine
```
Closed ─click─▶ Open(no highlight)                  Closed ─Enter / Space / ↓─▶ Open(first row)
Closed ─↑─▶ Open(last row)                          Context: right-click ─▶ Open at the pointer
                                                    Context: Shift+F10 / ContextMenu key ─▶ Open under the target, first row
Open ─↑ ↓ Home End─▶ highlight moves (skips disabled, dividers, labels; wraps)
Open ─A–Z─▶ highlight the next row starting with the typed prefix (500 ms buffer; repeats cycle)
Open ─→ / hover a submenu row─▶ Submenu open (first row highlighted from the keyboard)
Submenu ─← / Esc─▶ back to the parent row
Open ─Enter / Space / click─▶ action: Activated(id) + Closed (focus to trigger)
                              checkbox: Toggled(id, v), stays open (keep_open)
                              radio: Toggled(id, true), siblings in the group unchecked, closes
Open ─Esc─▶ Closed (focus to trigger)     Open ─Tab─▶ Closed (focus moves on)     outside press ─▶ Closed
Searchable (Oxyn plus): typing filters across submenus (flattened); first match highlighted.
```

## 3. States table
| Row / state | Shows | ARIA |
|---|---|---|
| action | icon, label, right-aligned shortcut | `menuitem` |
| checkbox | check (draws itself) in the icon column | `menuitemcheckbox` + `aria-checked` |
| radio | dot in the icon column | `menuitemradio` + `aria-checked` |
| danger | red text; highlight tinted red | `menuitem` (label says what it does) |
| disabled | `fg.disabled`, skipped | `aria-disabled=true` |
| divider / label | hairline / small caps heading | `separator` / `presentation` |
| submenu | chevron; opens beside its row | `aria-haspopup=menu`, `aria-expanded` |
| highlighted | one soft highlight under the row | focus stays on the menu (roving) |
| empty | "Nothing here yet" (configurable) | — |
| title / header | heading row at the top | menu `aria-label` |
| compact density (Oxyn plus) | 28 px rows | — |
| context mode (Oxyn plus) | opens at the pointer, clamped to the viewport | wrapper `aria-haspopup=menu` |

## 4. Micro-interactions
- The menu fades and scales in (0.96 → 1) from the corner it is attached to; submenus fade in beside
  their item; rows fade in with a short stagger (Arc).
- **One highlight** glides between rows for the pointer (`spring.snap`) and jumps for the keyboard
  (Arc); red-tinted on a danger row (affectUI).
- The check on a choice draws itself; the radio dot pops in.
- Context mode grows from the pointer even when clamped away from an edge; reopening fades the old
  menu out in place (Arc).

## 5. Timing table
| Transition | Duration | Delay | Easing | Reduced motion |
|---|---|---|---|---|
| Open | 180 ms, scale 0.96 | 0 | `ease.out` | 80 ms fade |
| Close | 120 ms | 0 | `ease.in` | 80 ms |
| Row stagger | 20 ms per row, cap 80 ms | — | `ease.out` | none |
| Highlight glide (pointer) | 214 ms | 0 | `spring.snap` | jump |
| Highlight (keyboard) | jump | 0 | — | jump |
| Submenu open on hover | 140 ms fade | 80 ms intent | `ease.out` | instant |
| Check draw / dot pop | 240 / 200 ms | 0 | `ease.out` | drawn |
| Type-ahead window | 500 ms | — | — | — |

## 6. Variants
Source: `trigger`, `items` (`icon`, `logo`, `checked`, `type` radio / checkbox, `danger`, `shortcut`,
`divider`, `disabled`, `keepOpen`, `submenu`), `align` start / end, `side` bottom / top, `title`,
`header`, `open` / `defaultOpen` / `onOpenChange`, `width`, `empty`, `aria-label`.

**Oxyn plus:** searchable filter for long menus, compact / comfortable density, context-menu mode
(right-click / Shift+F10 / ContextMenu key), second-line hints per row, platform-aware shortcut
glyphs.

## 7. Edge cases
Long labels: the menu widens to content up to 340 px, then labels ellipsize (shortcuts never
truncate). Menus near the bottom open upward (`placement::place`). Submenus near the right edge open
to the left. A filter with no matches shows the empty text and keeps the field. Activating a disabled
row (pointer) does nothing. Items changing while open keep the highlight on the same index if it is
still focusable.

## 8. Accessibility
`role=menu` with roving focus (the menu element holds focus; rows are `tabindex=-1`), full WAI menu
keyboard, type-ahead, `aria-checked` on choice rows, `aria-haspopup` + `aria-expanded` on submenu
rows and on the trigger. Esc restores focus to the trigger; Tab closes and moves on.

## 9. Props sketch
```rust
DropdownMenu { items: Vec<MenuItem>, trigger, icon, align: "start|end", side: "bottom|top", title,
               width, empty, open, highlight, searchable, density: "compact|comfortable",
               context, children, now, on_action, on_toggle, on_open_change }
MenuItem::action(id, label).icon(..).shortcut("Mod+D").danger().disabled()
MenuItem::checkbox(id, label, on) · MenuItem::radio(id, label, group, on) · MenuItem::submenu(id, label, items)
MenuItem::divider() · MenuItem::label(text)
```

## 10. Related components and references
`button-groups.md` (split and overflow menus), `select.md`, `composer-tools-menu.md`,
`command-palette.md`. Context menu (Arc) is this component's `context` mode. affectUI
`/components/dropdown-menu`; Arc `/components/dropdown-menu`, `/components/context-menu` —
behaviour only; Beautiful UI `glide-menu` — MIT with attribution.
