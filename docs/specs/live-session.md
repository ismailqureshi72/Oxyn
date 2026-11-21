**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Live session (`live-session`)

Section: Blocks · Agents (page `/blocks/agents/live-session/`, Pro). Reference: affectUI "Blocks · Agents" ("Live Session"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `LiveSession` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::LiveSession` (GPUI), rules in `oxyn_core::blocks::session`.

## 1. Purpose
Work beside a team of agents while they work: talk to them, watch the plan move and see how full the shared context gets, on one screen.

## 2. States
Running (the conversation streams in, the current step is marked) · Typing (an agent is writing) · Waiting on you (an agent asks; quick answers under its message; your step is current) · Paused (the session clock freezes) · Narrowed (an agent picked: the stream fades the rest, the plan dims other steps) · Message open (details under it) · Done (every step finished).

## 3. Micro-interactions
Messages slide in; the running step shimmers; the context bar segments grow with the conversation. Picking an agent fades the others rather than hiding them, so the thread keeps its shape. The @ menu opens above the composer and filters as you type.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Script | 7 messages over 13.4 s of session time | — |
| Typing line | 1.2 s before a message | — |
| Reply to you | 1.5 s | — |
| After your answer | copy 2.4 s, publish 5 s | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `live` | The session plays from the first message. |
| `your-turn` | Opens on the question that waits for you (Oxyn extra). |
| `writer` | Narrowed to the Writer, plan only (Oxyn extra). |

## 6. Responsive and edge cases
Side by side from 680 px of block width (container query); below it the plan and context go under the stream. Agent tabs scroll in their own row; long messages wrap; the stream has a fixed height and its own thin scrollbar. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
A region named after the task; agent tabs are a tablist with roving focus (arrow keys, Home, End); messages are focusable list items labelled with who wrote to whom and when; the plan is an ordered list with aria-current=step on the running step; the context bar is a meter with a value text; Escape lets go. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::LiveSession, ..)`; presses by id (ls-agent-<n>, ls-msg-<n>, ls-answer-<n>, ls-pause), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
collaboration-stream, agent-plan, context-visualizer.
