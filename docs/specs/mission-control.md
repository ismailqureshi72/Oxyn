**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Fleet overview (`mission-control`)

Section: Blocks · Agents (page `/blocks/agents/fleet-overview/`, Pro). Reference: affectUI "Blocks · Agents" ("Mission Control"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `MissionControl` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::MissionControl` (GPUI), rules in `oxyn_core::blocks::mission`.

## 1. Purpose
The home of an agent product: what happened since you last looked, who runs under whom, and what waits on you, with one filter over all three.

## 2. States
All / Running / Waiting / Failed (one filter for curve, fleet and inbox) · Lead picked (its sub-agents beside it) · Inbox card open (the ask or error with its answers) · Approved or retried (back to running, then done) · Declined or dismissed (done) · Retry fails again (error scenario) · Nothing waiting (empty inbox).

## 3. Micro-interactions
The curve stacks done, needs-you and failed events per bucket; answers settle in place after a short run and join the curve. Running times tick live.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Answer settles | 6 s (SETTLE_MS) | — |
| Curve | 8 h in 16 buckets | — |
| Card open | 160 ms fade | ease-out |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `all` | Every agent, three leads and their sub-agents. |
| `waiting` | Only the agents that wait on you (Oxyn extra). |
| `failing-retry` | Failed only, and a retry that fails again (Oxyn extra). |

## 6. Responsive and edge cases
Below 860 px of block width the inbox goes under the fleet; below 620 px leads and sub-agents stack and the header filter takes its own row. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
The filter is a radio group labelled Show agents with counts; the summary is a labelled region and the curve one image named with the summary; inbox cards are buttons with aria-expanded and aria-controls; answers are real buttons and every result is announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::MissionControl, ..)`; presses by id (mi-f-<n>, mi-lead-<n>, mi-open-<n>, mi-ok-<n>, mi-no-<n>, mi-retry-<n>, mi-dismiss-<n>), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-fleet, agent-inbox, agent-recap.
