**Status:** built v0.1 (2026-11-18, S3: Agent ops I).

# Spec: Agent · Activity (`agent-activity`)

Family: Screens. Reference: affectUI Agent / Activity (`/screens/agent-activity`), layout and behaviour only; no source or assets copied.
Components: `ActivityScreen` (Dioxus) · `ActivityEl` (GPUI), models in `oxyn_core::parity::agentscr`.

## 1. Purpose
A plain record of what agents did in your apps, and a way to take it back while you still can.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Activity, then filter tabs (All, On its own, You approved, Undone), each with its count, and a search |
| Days | Today 6 / Yesterday 3 heads with counts on a grey bar |
| Rows (54 px) | Time, the app's official mark on a white tile (Gmail, Stripe, Slack, GitHub, Google Calendar, Google Drive), the sentence, a dot with On its own or You approved and the task, then Undo (while the window is open), Final with a lock, or an Undone tag; undone rows are struck through |
| Work area (388) | The mark, sentence and account; When / Task / Cost; What happened (fields and the body as a quote); Why it could (how it was allowed, the permission and a Permissions link); Taking it back (what undo does, a live countdown, Undo) |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Tabs filter and search matches the sentence, task or app. Undo (row or work area) marks the step undone, strikes it through and moves the counts. The countdown ticks from the step's window and Undo disables when it closes.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States: Data · Loading · Empty · **Undone** (the first step taken back). Frame: Inset · Flat. Oxyn extras: `cost` (each step's cost on its row) and `clock` (the countdown as 17:42).

## 5. Keyboard and accessibility
Each Undo button names the step it reverses; the countdown is role=timer; rows report aria-current; the search clears on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-activity/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 760 px the search hides; under 520 px the time column hides and Undo keeps its icon. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
