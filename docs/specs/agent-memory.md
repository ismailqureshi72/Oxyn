**Status:** built v0.1 (2026-11-18, S3: Agent ops I).

# Spec: Agent · Memory (`agent-memory`)

Family: Screens. Reference: affectUI Agent / Memory (`/screens/agent-memory`), layout and behaviour only; no source or assets copied.
Components: `MemoryScreen` (Dioxus) · `MemoryEl` (GPUI), models in `oxyn_core::kit::agentscr`.

## 1. Purpose
What the agents remember about you and each project, where each fact came from and where it was used.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Memory with its count, Saved / To review 2, a search and Remember |
| Groups | Folding heads on a grey bar: About you (purple), then each project in its hue, with counts |
| Rows (40 px) | A glyph for where it came from (thread, you, decision, file), the memory, the source, uses and age; review rows carry Keep |
| Work area (368) | The memory; Applies to, Learned, Used; Where it came from (the question and your answer, Open); Where it was used; Keep (to review) and Forget |

Chrome is monochrome neutral; colour only means something: official app marks, file-type glyphs, project hues (purple, blue, green, orange) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Remember opens a field and saves the line under About you. Forget removes the memory with an Undo toast. Keep moves a memory from To review to Saved. Search and the group heads narrow the list.
prefers-reduced-motion: no glide, spin, waveform or pulse.

## 4. Variants
States: Data · Loading · Empty · **To review**. Frame: Inset · Flat. Oxyn extras: `scope_menu` (a scope menu that narrows to one group) and `compact` (rows without source and uses).

## 5. Keyboard and accessibility
Group heads report aria-expanded; rows are buttons with aria-current; Remember is a dialog (Enter saves, Escape cancels); Forget and Keep announce in a polite live region.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-memory/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button. Under 760 px search and source hide; under 520 px use counts hide. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
