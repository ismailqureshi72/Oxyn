**Status:** built v0.1 (2026-11-16, variants 2: Attachments family).

# Spec: Drop lanes (`drop-lanes`)

Family: Attachments. Reference: none (an Oxyn original).
Components: `DropLanes` (Dioxus) · `DropLanesEl` (GPUI), models in `oxyn_core::kit::attachfid`.

## 1. Purpose
Decide at drop time what a file is for – this message only, the whole project, or just its look.

## 2. Parts
| Part | Look |
|---|---|
| Tray | loose files as chips (icon + name), "Drag a file to a lane, or press it" |
| Lanes | three dashed lanes (`LANES`): This message · Project files · Style reference, each with a count and a one-line hint; they grow while a file is held and ring blue under a drag |
| Filed | small rows in each lane with × to send the file back |

## 3. Behaviour
Drag a chip onto a lane, or press a chip (pressed) and then a lane heading. Filed files leave the tray; × returns them. `on_file((name, lane))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
documents · media · code.

## 5. Keyboard and accessibility
Chips are toggle buttons (`aria-pressed`); lane headings are buttons, enabled while a file is held. Moves are said in a status line.

## 6. Responsive
560 px card; lanes stack under 480 px.

## 7. Known gaps
Lanes are fixed (not a prop yet). GPUI: static snapshot (`lane-<i>`).
