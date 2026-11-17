**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Schedules (`agent-schedules`)

Family: Screens. Reference: affectUI Agent / Schedules (`/screens/agent-schedules`), layout and behaviour only; no source or assets copied.
Components: `SchedulesScreen` (Dioxus) · `SchedulesEl` (GPUI), models in `oxyn_core::parity::opsscr`.

## 1. Purpose
What agents do on a rhythm, when each runs next and how the last runs went.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Schedules with on / total (6 / 7) |
| Around now | A card with agent activity before now (a line), a now marker, a mark per run after it in the schedule's hue (filled while running), hour labels and Next: … |
| Rows (44 px) | Hue square (hollow when paused), name, the rhythm in words, next run and how far off, last result tag, cost per run and a switch |
| Work area (368) | The schedule with Run now, Pause and its cost per run; What it does; Next runs with Skip / Undo; Last runs with Done / Failed |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Marks and rows open the schedule; switches pause or resume; Run now fills the mark and shows Running on the row and in Last runs; Skip strikes a next run through (Undo brings it back).
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **Running now**. Frame: Inset · Flat. Oxyn extras: `hide_paused` (paused schedules hidden) and `month` (a footer with what the schedules cost in a month).

## 5. Keyboard and accessibility
Strip marks are buttons named by schedule and time; switches are role=switch; actions announce in a polite live region. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-schedules/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 760 px the rhythm column hides; under 560 px cost and relative time hide. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
