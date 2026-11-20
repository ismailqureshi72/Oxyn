**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: LLM cost calculator (`llm-cost-calculator`)

Family: AI Components (page `/components/ai/llm-cost-calculator/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `LlmCostCalculator` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Cost` (GPUI), rules in `oxyn_core::aikit::cost`.

## 1. Purpose
Estimate monthly spend: set tokens per request, requests per day and the cache hit rate, and eight models re-rank by monthly cost.

## 2. States
Loading prices · Ready · Preset (support chat, coding agent, …) · Batch on.

## 3. Micro-interactions
Rows re-rank with a FLIP move when the order changes; bars show each model's share of the most expensive. Arrow keys nudge sliders, Shift for big steps.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Re-rank move | 220 ms | ease-out |
| Bar width | 180 ms | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `support-chat` | A support chat workload. |
| `coding-agent` | A coding agent with heavy caching and the batch switch (Oxyn extra). |
| `loading` | Loading prices. |

## 6. Responsive and edge cases
Costs use compact notation above $10k. Under 640 px the sliders stack above the ranking. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Sliders are native ranges with value text; the ranking is a list with each row's rank and cost in its label; summary in a status line. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Cost, ..)`; presses by id (lc-preset-<n>, lc-batch, lc-sel-<n>, lc-set-<k>-<v>, lc-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
model-card, billing-usage.
