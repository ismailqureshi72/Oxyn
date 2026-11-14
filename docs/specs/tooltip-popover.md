**Status:** v1.1 — implemented (plan #23, affectUI / Arc gaps closed in plan #26).

# Spec: tooltip and popover

> Implemented in `oxyn-core::primitives::overlay` (`TooltipGroup`, `Entrance`, `TooltipContent`,
> `PopoverStack`, `PopoverTrigger`, `CloseReason`, `enter_progress`) and
> `oxyn-core::primitives::placement` (`place`, flip, collision padding, transform origin, arrow,
> drift), `oxyn-dioxus::{Tooltip, TooltipProvider, Popover}`, `oxyn-gpui::{TooltipEl, PopoverEl}`.
> Sources: affectUI Tooltip and Popover, Arc Tooltip and Popover (behaviour only).

---

## A. Tooltip

### 1. Purpose
A short label for one control — the name of an icon button, a shortcut, why a button is disabled.

### 2. State machine (one delay group per window)
```
Closed ─pointer enter─▶ Waiting ─delay (500 ms)─▶ Open(fade)
Closed ─pointer enter while another is open or within 300 ms of one closing─▶ Open(instant)
Closed ─keyboard focus─▶ Open (no delay; instant when the group is warm)
Open ─pointer leave / blur / press / Esc─▶ Closed (at once) ─▶ Warm for 300 ms ─▶ Cold
Disabled: never opens. Opening one closes the group's other tooltip.
```

### 3. States table
| State | Shows | ARIA |
|---|---|---|
| closed | — | — |
| open | dark label (inverted in dark theme: light), optional shortcut keycaps | `role=tooltip`; trigger `aria-describedby` |
| rich (Oxyn plus) | bold title over the text | same |
| side top / bottom / left / right × align start / center / end | placed `offset` (6 px) from the trigger, flipped if it would leave the viewport | — |

### 4. Micro-interactions
Fades in and drifts 3 px from the trigger's side; disappears at once. Neighbours within 300 ms open
instantly without the fade (affectUI, Arc). Changing content rises in with a blur while the bubble
springs to its new size (Arc).

### 5. Timing table
| Item | Value | Reduced motion |
|---|---|---|
| Hover delay | 500 ms (affectUI `delay`; Arc 250 ms; Oxyn plus: configurable) | same |
| Neighbour window | 300 ms | same |
| Fade + 3 px drift | 120 ms `ease.out` | fade only, 80 ms |
| Hide | instant | instant |

### 6. Variants
Source: `content`, `shortcut`, `side`, `align`, `offset`, `delay`, `disabled`, `open`,
`onOpenChange`. **Oxyn plus:** configurable delay, rich title + body, side preference with flip
(placement module), shortcut chips with platform glyphs.

### 7. Edge cases
Tooltips on disabled buttons: the button stays focusable when it has a reason, so the tooltip still
works. Touch: long-press opens, release closes. Long content wraps at 260 px. Near edges `place()`
flips to the other side, then clamps inside the collision padding.

### 8. Accessibility
Tooltip text is a description, not the name (icon buttons carry their own `aria-label`). Esc hides it
without moving focus. It never contains interactive content (use a popover).

### 9. Props
`TooltipProvider { children }`, `Tooltip { content, title, shortcut, side, align, offset, delay, disabled, open, on_open_change, children }`

---

## B. Popover

### 1. Purpose
A small floating card attached to a trigger: share settings, a filter, details on hover, a confirm.

### 2. State machine (stack)
```
Closed ─trigger press / Enter / Space─▶ Open (focus moves into the card when focus_on_open)
Open ─Esc─▶ Closed, focus back to the trigger   (nested: the innermost closes first)
Open ─press outside─▶ Closed (focus goes where the press went)
Open ─trigger press─▶ Closed
Hover intent (Oxyn plus): Closed ─rest on trigger 150 ms─▶ Open ─leave trigger and card 300 ms─▶ Closed
Opening a top-level popover closes other top-level popovers; children keep their parents open.
```

### 3. States table
| State | Shows | ARIA |
|---|---|---|
| open | card: title, quiet line, any content | `role=dialog` (configurable), labelled by the title |
| arrow (Oxyn plus) | 10 px arrow toward the trigger | — |
| unstyled | content draws its own surface | — |
| flipped | opens on the other side, animating from that side | — |

### 4. Micro-interactions
Fades and scales in (0.97 → 1) from the side and corner it is attached to, settling 5 px toward its
trigger; after a flip, from the new side. Leaves in 120–140 ms. Transitions (not keyframes) so a
reopen mid-close reverses from where it is (Arc).

### 5. Timing table
| Item | Value | Reduced motion |
|---|---|---|
| Open | 180 ms (`dur.popover.open`), scale 0.97, 5 px | 80 ms fade |
| Close | 120 ms (`dur.popover.close`; Arc 140 ms) | 80 ms fade |
| Hover-intent open / close | 150 ms / 300 ms | same |

### 6. Variants
Source: `open`, `side`, `align`, `offset`, `anchor`, `title`, `description`, `width`, `focusOnOpen`,
`unstyled`, `role`, `onClose`; Arc `modal`, `sideOffset`, `collisionPadding`, `onOpenChange`.
**Oxyn plus:** arrow on / off, hover-intent trigger mode, nested popovers.

### 7. Edge cases
Content taller than the viewport scrolls inside the card. A popover whose trigger scrolls out of
view closes. Nested: an outside press inside a parent closes only deeper children
(`PopoverStack::press`).

### 8. Accessibility
Trigger has `aria-haspopup`, `aria-expanded`, `aria-controls`. Focus moves in on open (or stays on
the trigger for hover mode) and returns on Esc. Esc stops propagation so only the innermost closes.

### 9. Props
`Popover { trigger, trigger_label, children, open, side, align, offset, title, description, width, focus_on_open, unstyled, role, arrow, mode: "press|hover", on_close }`

## 10. Related components and references
`dropdown-menu.md` (menus use the same placement and dismissal), `desktop-overlays.md`, `button.md`
(disabled reasons). affectUI `/components/tooltip`, `/components/popover`; Arc `/components/tooltip`,
`/components/popover` — behaviour only.

## Plan #26 additions
- **Tooltip (affectUI):** the playground now exposes `align`, `offset` and `disabled` (all already
  props). Shared delay group, keyboard focus and Escape as before.
- **Popover (Arc UI):** `modal` dims the page behind a press-to-close backdrop and marks the panel
  `aria-modal`; `collision_padding` (default 10 px) keeps the panel that far from the viewport
  edges.
