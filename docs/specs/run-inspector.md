**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Run inspector (`run-inspector`)

Section: Blocks · Agents (page `/blocks/agents/run-inspector/`, Pro). Reference: affectUI "Blocks · Agents" ("Run Inspector"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `RunInspector` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::RunInspector` (GPUI), rules in `oxyn_core::blocks::inspector`.

## 1. Purpose
One multi-agent run on one screen: who started whom, what each agent called, what it cost and where it failed.

## 2. States
Playing (agents and calls appear as they start; running bars pulse) · Finished · Agent picked (others fade; the numbers sum its part) · Call open (input and output, or the error, under its row) · Failed call (red, the agent reads Running · 1 failed call) · Replay.

## 3. Micro-interactions
Cards join the tree as their agents start; each call is a bar on its agent's own time axis. The supervisor sits beside the orchestrator with a dashed border.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Run | 24 s at 1× (TOTAL) | linear |
| Speed | 0.5× to 3× | — |
| Card in | 240 ms | ease-out |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `live` | The run plays from the start. |
| `failed-call` | Finished, with the failed test run open (Oxyn extra). |
| `fast` | Three times speed, no supervisor (Oxyn extra). |

## 6. Responsive and edge cases
Below 640 px of block width the numbers wrap into two rows of three, the supervisor goes under the orchestrator, sub-agents indent less and call bars hide (the duration stays). Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Each card's calls are a list; a row is a button whose name carries the call, its duration and any failure, with aria-expanded and aria-controls; the numbers are a description list labelled with their scope; the title line is a polite live region. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::RunInspector, ..)`; presses by id (ri-agent-<n>, ri-call-<n>, ri-replay), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-tree, trace-waterfall, run-meter.
