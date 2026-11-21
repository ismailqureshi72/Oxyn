**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 4).

# Spec: Run monitor (`agent-monitor`)

Family: Agents. Closes the parity row: affectUI · Agent Monitor (Pro). Behaviour-only rebuild; nothing is copied (the
demo run, its steps, the command and the agent's role are our own).
Components: `AgentMonitor` (Dioxus) · `AgentMonitorEl` (GPUI), model in `oxyn_core::kit::monitorfid`.

## 1. Purpose
One agent at work as a card: its current step with a clock and the bill so far, a time strip and the finished steps
with each tool's logo, the exact command when it needs an OK, and a fold into a small bar.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 340 px (max 100 %), 12 px radius, hairline, soft shadow, 12 px padding, 10 px gaps |
| Header | 20 px maker mark, name 13/600 over the role 11 muted (ellipsis), cost mono 11.5 with the last digit faint (shimmers while busy), state pill 20 px (dot + label: blue Running / Paused, amber Needs you, green Finished, red Failed), Fold away chevron |
| Well | 36 px grey, 8 px radius: 20 px tool tile, the current step (rolls in from below), clock right; waiting: amber, the ask's title and the exact command as a mono chip; done: "Finished in 13s" green + the outcome |
| Time strip | 4 px segments, 3 px gaps, widths by time: grey done, blue gradient running, amber waiting (breathing), red failed; all green once finished |
| Finished steps | newest first, the last three, 26 px rows: 18 px logo, label 12, time faint right ("Failed" red) |
| Details | a card under a clicked row: logo, step, tool, close; a 4-cell grid Started / Took / Share / State; one sentence; the command or place as mono |
| Foot | Pause / Cancel (running), Decline Esc + Allow ↵ filled (waiting), Try again filled (failed); right "4/6" faint and Log ↗ |
| Folded | 44 px bar: mark, name, "+n", the last four tool tiles stacked (current full, earlier faded), state pill with the clock or total time |

## 3. Behaviour
- Demo: a 24 s pass: four steps run, the card asks before publishing (Enter allows, Esc declines; it goes on by itself
  after a few seconds), the release publishes, the announcement posts and the run finishes green.
- Pause stops the clock and the demo (Resume picks up where it stopped); Cancel stops the run; Decline cancels it.
- Click a row or its strip segment for details; Esc closes them first. Fold away / the bar toggle the fold.
- Callbacks: `on_allow`, `on_decline`, `on_pause(bool)`, `on_cancel`, `on_retry`, `on_open_log`.
- prefers-reduced-motion: no shimmer, roll or breathing.

## 4. Variants
default (live) · waiting · details · failed · folded · classic (rounder card, bordered tiles, the foot on a tray).

## 5. Keyboard and accessibility
The card is a focusable region named "Claude Opus 4.1: Running – Writing the changelog"; Enter / Esc allow / decline
while it waits. The strip is one image labelled with every step and its time; finished steps are a list of buttons with
aria-expanded; details are a labelled dialog (focus goes to Close). The ask is announced.

## 6. Responsive
A fixed 340 px card that shrinks with its container; labels, the role and the command ellipsize.

## 7. Known gaps
- Only the last three finished steps show (no scrolling list with a fade).
- Hovering a row does not light its strip segment.
- GPUI: a static snapshot; no shimmer, roll or breathing; glyph icons for Pause / Cancel.
