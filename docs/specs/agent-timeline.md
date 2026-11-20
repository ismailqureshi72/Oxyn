**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Agent timeline (`agent-timeline`)

Family: AI Components (page `/components/ai/agent-timeline/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AgentTimeline` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Timeline` (GPUI), rules in `oxyn_core::aikit::timeline`.

## 1. Purpose
Replay one orchestrated run on a shared time axis: the orchestrator on top, its subagents below, so a builder sees who worked when, what each waited on, where handoffs happened and what the run cost.

## 2. States
Idle at zero · Playing (playhead moves, header counts time, tokens and cost) · Paused · Finished (header settles into a summary, Replay) · Live (following a run in progress, no playback controls) · Span selected (detail popover with kind, agent, start, duration, tokens).

## 3. Micro-interactions
Spans grow from their left edge as the playhead passes them; thinking spans are a tint of the agent colour, tool calls solid, waiting hatched. Handoff connectors draw between rows when their source span ends. Hover or focus on a span lifts it and shows its readout; the header numbers roll, never jump. Speed chip cycles 1x / 2x / 4x.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Playhead | derived from the host clock, 1x = real run time | linear |
| Span grow | follows the playhead | linear |
| Header numbers | 160 ms roll per change | ease-out |
| Finish summary | 220 ms crossfade | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `replay` | The run replays from zero. |
| `finished` | A finished run, ready to replay, with cost per span (Oxyn extra). |
| `waterfall` | One row per span in start order (Oxyn extra). |
| `live` | Following a run in progress: no playback controls. |

## 6. Responsive and edge cases
Long agent names truncate with an ellipsis and keep the full name in the label. Under 640 px the label column narrows to marks only and the axis keeps 4 ticks. The waterfall variant lists one row per span for narrow panels. RTL mirrors the label column but keeps time running left to right. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
The chart is a labelled group; each span is a button naming agent, kind, start and duration. Space toggles play, Home restarts, Left/Right step the playhead. Finish is announced in the polite live region. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Timeline, ..)`; presses by id (tl-play, tl-restart, tl-speed, tl-seek-<ms>, tl-span-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
trace-view, run-replay, subagent-tree, fanout-grid.
