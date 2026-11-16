**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Running Now (`running-now`)

Family: Agents. Closes the parity row: affectUI · Running Now (Pro). Behaviour-only rebuild; layout, sizes, colours,
states, motion and keyboard follow the ref-2 capture notes (approximate); the tasks and copy are our own.
Related Oxyn components: background-jobs, desktop-overlays.
Components: `RunningNow` (Dioxus) · `RunningNowEl` (GPUI), model in `oxyn_core::parity::runningfid`.

## 1. Purpose
A quiet live status of background agent tasks for an app bar or sidebar: one pill with the counts that matter, opening
to the tasks with their progress, step, Pause / Resume and Open.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Pill | 24 px, full radius, hairline; a violet dot + bold count + "running", an amber dot + bold count + "needs you" (only groups that aren't zero; "All done" when nothing is left), chevron |
| Popover | 284 px, 12 px radius, hairline, larger shadow, 8 px padding; opens below the pill (right-aligned), or to the right in a sidebar |
| Heading | uppercase 10 px muted title + count |
| Task row | 40 px: a 16 px progress ring (violet running with a sweep, grey frozen when paused, amber when it needs you, green, full and ticked once finished), title 12/600 over the step 11 muted (ellipsis), "n/N" 11 muted tabular, then a 24 px icon button (Pause / Resume) or an amber "Needs you" tag + Open |
| Footer | hairline, list icon + "Open tasks" + chevron (only with `on_open_all`) |

## 3. Behaviour
- `rn_done_at` advances a running task's steps with the clock; `rn_counts` gives (running, needs you); `rn_label` the
  pill text. Row order and height never change as step text updates.
- Pause freezes a task ("Paused · step"), Resume starts it again; the counts follow at once. Opening a needs-you task
  fires `on_open` and, in the demo, sends it back to running.
- Escape or a click outside closes the popover; a task that finishes or starts needing you is announced politely.
- prefers-reduced-motion: no sweep and no scale on open.

## 4. Variants
open (app bar) · closed · paused · done · sidebar (opens right) · own words · classic.
Controls: style, state, placement (header / sidebar).

## 5. Keyboard and accessibility
The pill (aria-expanded, aria-haspopup=dialog) toggles the popover; the popover is a labelled dialog; each row's main
area is a button (opens the task), Pause / Resume buttons are named with the task; Escape closes and returns focus to
the pill.

## 6. Responsive
The pill shrinks to its counts; the popover stays inside its container width (max-width 100 % of the viewport less
16 px).

## 7. Known gaps
- No green "step completed" flash on the ring.
- GPUI: a static snapshot, popover always open.
