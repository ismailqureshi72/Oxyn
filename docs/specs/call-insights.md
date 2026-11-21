**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Call insights (`call-insights`)

Section: Blocks · AI (page `/blocks/ai/call-insights/`, Pro). Reference: uiarc "Blocks · AI" ("Call insights", https://uiarc.dev/components/blocks/call-insights), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `CallInsights` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Call` (GPUI), rules in `oxyn_core::blocks::call`.

## 1. Purpose
A sales call distilled into one card: talk time, key moments with their quotes, an AI summary and next steps that lower the deal risk.

## 2. States
Touring (moments cycle every 4 s until you interact) · Moment selected · Playing / paused / ended clip · Seeking · Summary streaming / done (Copy, Copied) · Step done (risk recomputed: High, Medium, Low).

## 3. Micro-interactions
The rail fills up to the selected moment; the quote fades in; the waveform fills as the clip plays; checking a step strikes it through and can drop the risk chip a level.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Tour | 4,000 ms a moment (TOUR_MS) | — |
| Clip | 12 s (CLIP_S) | linear |
| Summary | 2 words per 34 ms tick | linear |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `tour` | Touring the moments, high risk. |
| `low-risk` | Two steps done, risk drops (Oxyn extra). |
| `all-done` | Every step done (Oxyn extra). |

## 6. Responsive and edge cases
Fluid from 360 to 680 px. Below 560 px of block width the talk legend goes to two columns and attendees hide; below 460 px the platform name, speaker titles and owners drop. Markers clamp inside the rail. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Markers are a radio group named with kind and time (arrow keys, Home, End); the waveform is a slider with a time value; risk is a word with an icon; next steps are real checkboxes; selections, playback, copy and risk changes are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Call, ..)`; presses by id (ci-m-<n>, ci-play, ci-copy, ci-step-<n>), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
meeting-recap, live-captions, waveform.
