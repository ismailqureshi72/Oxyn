**Status:** built v0.1 (2026-11-18, S3: Agent ops I).

# Spec: Agent · Tasks (`agent-tasks`)

Family: Screens. Reference: affectUI Agent / Tasks (`/screens/agent-tasks`), layout and behaviour only; no source or assets copied.
Components: `TasksScreen` (Dioxus) · `TasksEl` (GPUI), models in `oxyn_core::kit::agentscr`.

## 1. Purpose
Every task the agents run, grouped by project, with what each one needs from you.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Tasks with its count, Active / Done tabs and New task |
| Groups | A folding head per project (chevron, project-hued folder, name, count) |
| Rows (40 px) | A progress ring toned by state, name, what it is doing, a Needs you tag, steps n/m, files and age |
| Work area (368) | Brief; a decision card (Needs you, 1 decision waiting, the question, Decide); Status, Project, Started; Steps; So far; Artifacts; Decided |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Rows open the task; group heads fold. Decide shows the answers inline and picking one sets the task back to work with one more step done. New task opens a field and starts a task at the top of Active. Done lists finished tasks.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States: Data · Loading · Empty · **Done tab**. Frame: Inset · Flat. Oxyn extras: `cost` (a cost column) and `folded` (groups start folded).

## 5. Keyboard and accessibility
Group heads report aria-expanded; rows are buttons with aria-current; rings are hidden and the steps are text; Decide reports aria-expanded; New task is a dialog (Enter starts, Escape cancels).

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-tasks/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 760 px age hides; under 520 px the status sentence, steps, files and cost hide. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
