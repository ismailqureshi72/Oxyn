**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Run Detail (`agent-run-detail`)

Family: Screens. Reference: affectUI Agent / Run Detail (`/screens/agent-run-detail`), layout and behaviour only; no source or assets copied.
Components: `RunDetailScreen` (Dioxus) · `RunDetailEl` (GPUI), models in `oxyn_core::parity::opsscr`.

## 1. Purpose
One run, step by step on one time axis: what it thought, which tools it called, what failed and what it cost.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | The task with a Running (or Stopped) tag and Stop (or Resume) |
| Sub line | The agent's mark, agent · model · start time |
| Tiles | Time · Cost · Tokens · Steps (failed in red) |
| Trace | Legend (Thinking purple, Tool call blue, Sub-agent grey, Failed red); rows with the official tool mark or a glyph, sub-agent steps indented under a tree line, tinted bars on a 0–1m 10s axis with their length, and a now line while running |
| Work area (348) | The step: kind, status, Started, Took, Tokens, Cost; Input and Output with Copy |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Rows open the step; ↑/↓, Home and End move through them; Stop ends the run at the failure with a banner and Retry.
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **Stopped**. Frame: Inset · Flat. Oxyn extras: `errors_only` (failures and the steps around them) and `cost_labels` (bars labelled with cost).

## 5. Keyboard and accessibility
The trace is a listbox with aria-activedescendant; each option names its kind and length for screen readers. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-run-detail/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 760 px the label column narrows; under 560 px tiles go two by two and every other tick hides. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
