**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Mission timeline (`mission-timeline`)

Section: Blocks · Agents (page `/blocks/agents/mission-timeline/`, Pro). Reference: affectUI "Blocks · Agents" ("Mission Timeline"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `MissionTimeline` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::MissionTimeline` (GPUI), rules in `oxyn_core::blocks::mission`.

## 1. Purpose
Mission control on one clock: every agent on its own lane over the last 30 minutes, so you see what happened when as well as what needs you.

## 2. States
Same filter, inbox and answers as Mission control · Running blocks grow toward now · Finished blocks stop where they ended · Waiting and failed blocks carry their colour and word and open their inbox card.

## 3. Micro-interactions
Lanes share one time axis ending now; a running block has a soft leading edge. Leads come first with their sub-agents indented beneath.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Span | 30 min (SPAN_MIN) | linear |
| Answer settles | 6 s | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `all` | Every agent on its lane. |
| `running` | Only running agents (Oxyn extra). |
| `bare` | No axis ticks, failed only (Oxyn extra). |

## 6. Responsive and edge cases
Names keep a fixed column and the status column sits at the end; below 620 px the status column hides (it stays in the lane label) and the name column narrows. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Each lane is a list item labelled with agent, status, task and running time; the filter is a radio group with counts; status is always a word with an icon. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::MissionTimeline, ..)`; presses by id (mi-f-<n>, mi-open-<n>, mi-ok-<n>, mi-no-<n>, mi-retry-<n>, mi-dismiss-<n>), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-lanes, mission-control, trace-waterfall.
