**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Live visitors (`live-visitors`)

Family: AI Components (page `/components/ai/live-visitors/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `LiveVisitors` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Visitors` (GPUI), rules in `oxyn_core::aikit::visitors`.

## 1. Purpose
Show a site's live audience at a glance: how many are on it, where they are, which pages they read and who converted.

## 2. States
Live (pulsing dot) · Paused (holds the snapshot) · Connecting (placeholders in the same space) · Reconnecting (keeps the last data).

## 3. Micro-interactions
Digits roll on their own strips in tabular numerals; arrivals ping on a dotted map and fade over eight ticks, conversions ring stronger; page bars resize and reorder; conversions slide in with a flag.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Stream tick | 1600 ms | — |
| Digit roll | 280 ms | ease-out |
| Ping ring | 900 ms | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `live` | A simulated marketing site stream. |
| `reconnecting` | Reconnecting keeps the last data. |
| `compact` | No map, three pages (Oxyn extra). |

## 6. Responsive and edge cases
At most 640 px; under 520 px (container query) the lists stack; the map keeps its ratio. Paths and cities end in an ellipsis; counts keep fixed columns. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
The count is read once as text and the rolling digits are hidden; Pause is a toggle with aria-pressed; status is a dot plus a word. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Visitors, ..)`; presses by id (lv-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
status-page, run-meter.
