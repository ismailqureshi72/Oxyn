**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Model card (`model-card`)

Family: AI Components (page `/components/ai/model-card/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ModelCard` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::ModelCard` (GPUI), rules in `oxyn_core::aikit::model_card`.

## 1. Purpose
Decide whether a model fits a job: what it reads and writes, its context and prices, an estimated monthly cost at your volume, and how it scores against rivals.

## 2. States
Details · Benchmarks (row hover/focus shows score, rank and gap) · Compare (side-by-side table against a rival with the better value checked, tally on top) · Picker pane (Use model) · Loading · Error (Couldn't load, Retry).

## 3. Micro-interactions
Switching model rolls every number in place; the card never changes size because every panel shares one grid cell. The volume slider moves the monthly estimate live. Copy model id flashes Copied for 1.2 s.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Value roll on model switch | 180 ms | ease-out |
| Panel swap (tabs, compare) | 160 ms crossfade | ease-out |
| Copied flash | 1200 ms | step |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `details` | Details of Claude Sonnet 4.5 at 100M tokens a month. |
| `compare` | GPT-5 against a rival with the better values checked. |
| `picker-pane` | Shown as the side panel of a model chooser, with a Use model button (Oxyn extra). |
| `error` | Couldn't load, with Retry. |

## 6. Responsive and edge cases
Long model names wrap to two lines in the header; the tab row scrolls horizontally under 390 px instead of wrapping onto prices. Prices use tabular figures. Compare collapses to one column of pairs under 640 px. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Tabs are a tablist with arrow keys; the volume slider is a native range with a text value; compare marks the better value with text (Better) as well as the check. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::ModelCard, ..)`; presses by id (mc-compare, mc-copy, mc-model-<n>, mc-rival-<n>, mc-bench-<n>, mc-vol-<n>, mc-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
composer-model-picker, llm-cost-calculator, generation-stats.
