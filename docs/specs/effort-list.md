**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Effort list (`effort-list`)

Family: Thinking Selector. Reference: a user-supplied screenshot of a "change thinking effort" list over a chip (layout
idea only; wording is our own). Components: `EffortList` (Dioxus) · `EffortListEl` (GPUI), models in
`oxyn_core::parity::{thinkfid, effmenufid}`.

## 1. Purpose
The quietest effort control: a chip in the composer's footer showing the level, opening a plain list above it.

## 2. Parts
| Part | Look |
|---|---|
| Chip | 30 px, 9 px radius, hairline; level name 14.5/500 and a caret that flips when open |
| List | 300 px card above the chip, 14 px radius, soft shadow; heading "Thinking effort" 13 muted; rows 36 px, 17 px text |
| Chosen row | a check at the right end |
| Highlight | hover / keyboard row tinted; an inset ring when the list has keyboard focus |

## 3. Behaviour
Scale: Minimal, Low, Medium, High (`effort_scale_short`). Clicking the chip opens the list on the chosen level and
focuses it; a pick sets the level, closes the list and returns focus to the chip. Controlled (`value`) or uncontrolled
(`default_value`); `on_change`. `default_open` previews it open without taking focus.

## 4. Variants
open (default) · closed · minimal.

## 5. Keyboard and accessibility
Menu button pattern: the chip has `aria-haspopup="menu"` and `aria-expanded`; ↑/↓ on the chip open the list. In the
list ↑/↓ move (wrapping, `menu_move`), Home/End jump, Enter/Space pick, Escape or Tab close. Rows are `menuitemradio`
with `aria-checked`; the highlight uses `aria-activedescendant` with page-unique ids.

## 6. Responsive
300 px list capped at the viewport minus 32 px.

## 7. Known gaps
A press outside closes a menu the user opened (a start-open preview draws no catcher, so it never covers the page). GPUI: static snapshot.
