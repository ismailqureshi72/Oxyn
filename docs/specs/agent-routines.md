**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Agent routines (`agent-routines`)

Section: Blocks · Agents (page `/blocks/agents/agent-routines/`, Pro). Reference: uiarc "Blocks · Agents" ("Agent routines"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AgentRoutines` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Routines` (GPUI), rules in `oxyn_core::blocks::routines`.

## 1. Purpose
Scheduled agents in one card, each read as a sentence: when it runs, what it reads, where it posts, which model, and how its last runs went.

## 2. States
On / paused · Counting down · Running (thin progress line, current step shimmers; due schedules run by themselves) · Failed at a step · Open (report, instruction, frequency) · New routine (added open).

## 3. Micro-interactions
The countdown ticks every second; a run fills its line in the routine's series colour and slides a new tick into the strip. Run now shows on hover or focus.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Step | 1100 ms | linear |
| Preview clock | Monday 08:58:40 at start | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `list` | Four routines; the stand-up digest comes due in the preview. |
| `open` | The first routine open with its report (Oxyn extra). |
| `failing` | Run Error triage now to see it fail (Oxyn extra). |

## 6. Responsive and edge cases
Up to 680 px; rows keep three lines; below 540 px the last run leaves line two; below 420 px Run now becomes an icon button and the marks slot narrows. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Names are buttons with aria-expanded and roving arrow keys; marks are replaced by one sentence per row; the tick strip is one group named with its counts; a running routine is a progressbar whose value text names the step. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Routines, ..)`; presses by id (ro-open-<n>, ro-run-<n>, ro-toggle-<n>, ro-freq-<n>-<f>, ro-new), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-schedules, background-jobs.
