**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: AI insight card (`ai-insight-card`)

Section: Blocks · AI (page `/blocks/ai/ai-insight-card/`, Pro). Reference: uiarc "Blocks · AI" ("AI insight card"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AiInsightCard` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Insight` (GPUI), rules in `oxyn_core::blocks::insight`.

## 1. Purpose
A KPI card that explains its own change: why it moved, which drivers mattered and the next question to ask.

## 2. States
Idle (Explain) · Streaming (Stop) · Answer (Hide; drivers ranked; markers on the chart) · Stopped · Follow-up answer · Error (Retry) · Chart readout.

## 3. Micro-interactions
Words stream in; when the answer reaches a driver its numbered marker lights up on the sparkline and its bar grows; follow-up chips swap the answer in place.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Stream | 2 words per 34 ms tick | linear |
| Driver bar | 400 ms | ease-out |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `explain` | Explains on open. |
| `idle` | Waiting for Explain, no previous line (Oxyn extra). |
| `error` | The first answer fails, with Retry (Oxyn extra). |

## 6. Responsive and edge cases
Up to 600 px; below 460 px the legend hides and driver bars shorten; the chart redraws at the card width. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Explain has aria-expanded and aria-controls; the chart is a focusable group read point by point with the arrow keys; deltas carry a sign as well as colour; unread drivers are hidden from assistive technology; follow-ups use aria-pressed. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Insight, ..)`; presses by id (ai-explain, ai-follow-<n>, ai-retry), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
generation-stats, ask-your-data.
