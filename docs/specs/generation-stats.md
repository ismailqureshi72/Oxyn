**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Generation stats (`generation-stats`)

Family: AI Components (page `/components/ai/generation-stats/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `GenerationStats` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::GenStats` (GPUI), rules in `oxyn_core::aikit::genstats`.

## 1. Purpose
Show how a streaming reply performs: time to first token, speed with a sparkline, output tokens and cost.

## 2. States
Queued · Waiting for first token · Streaming (counting up) · Done (settled, expandable latency breakdown, session totals) · Stopped (counts frozen) · Error (what arrived is kept) · Inline chip.

## 3. Micro-interactions
Numbers count up with the stream and settle; the sparkline grows right; expanding shows queue / TTFT / generation bars.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Queue | 60 ms | — |
| TTFT | 420 ms | — |
| Loop gap (demo) | 2500 ms | — |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `streaming` | Counting up while the reply streams. |
| `breakdown` | Finished, with the latency breakdown and session totals (Oxyn extra). |
| `inline` | A compact chip under a message (Oxyn extra). |
| `error` | Failed part way; what arrived is kept. |

## 6. Responsive and edge cases
Tabular figures keep widths stable while counting. Inline chip wraps under the message on narrow columns. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Done summary is announced once (not every tick); the breakdown is a disclosure. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::GenStats, ..)`; presses by id (gs-run, gs-expand, gs-stop, gs-retry, gs-model-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
composer-context-meter, model-card.
