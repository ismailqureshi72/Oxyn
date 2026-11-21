**Status:** built v0.2 (2026-11-18, S2: Workspace parity; v0.1 screens 1).

# Spec: Kanban (`kanban`)

Family: Screens. Reference: affectUI Kanban (`/screens/kanban`), layout and behaviour only; no source or assets copied.
Components: `KanbanScreen` (Dioxus) · `KanbanEl` (GPUI), models in `oxyn_core::kit::workscr`.

## 1. Purpose
An issue board by priority: see what is open where, and move work between lanes.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Board persona: Inbox 3 · Activity 2 · My issues · Board · Projects, then the Problems group |
| Top bar | 48 px: Inbox / Overview and a star; on the right Filter (labels), Sort by ⌄ and the Board | List segmented switch |
| Tab row | Three tabs with counts (every task 11, open 9, completed 2) on the left, the black New task on the right |
| Lanes | Four 276 px lanes on the grey inset, 14 px radius: LOW / MEDIUM / HIGH PRIORITY and COMPLETED (11.5 px uppercase, centred) with a priority glyph (bars in blue, amber, red; a green check) and a count chip |
| Card | White, 12 px radius: priority bars and the title, two lines of detail (clamped), then the status (Ready for review blue, In progress purple, Needs attention amber, Fixed green), a label chip and comment / link counts |

Chrome is monochrome neutral; colour only means something: official brand marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Cards drag between lanes with a 3° tilt and a lifted shadow; the lane under the pointer lights; dropping moves the card (to Completed it turns Fixed; out of it, In progress) and announces the move. From the keyboard Space or Enter picks a card up, ← / → carry it lane by lane, Space puts it down. Tabs, the label filter and Sort by (Manual, Most comments, Title) narrow and order the lanes. New task opens a field (Enter adds to Low priority).
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States: Data · Loading (lane and card skeletons) · Empty (no tasks, with New task) · **Dragging** (a card held over High priority). Frame: Inset · Flat. Oxyn extras: label chips (Bug, Feature, Design, can be hidden), keyboard carrying, the List view grouped by lane, the label filter.

## 5. Keyboard and accessibility
Cards are buttons with aria-roledescription "Draggable task" and say how to pick them up; moves are read out in a polite region; lanes are labelled sections with their counts; the list view is a role=table. Escape closes any open menu or popover and clears a search; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/kanban/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and top-bar actions keep their icons with hidden labels. Lanes keep 276 px and the board scrolls sideways; under 620 px the tab row wraps and lanes narrow to 260 px. prefers-reduced-motion drops the tilt. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; drag, menus, typing and charts' crosshair belong to the host.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
