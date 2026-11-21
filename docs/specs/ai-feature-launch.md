**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: AI feature launch (`ai-feature-launch`)

Section: Blocks · AI (page `/blocks/ai/ai-feature-launch/`, Pro). Reference: uiarc "Blocks · AI" ("AI feature launch", https://uiarc.dev/components/blocks/ai-feature-launch), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AiFeatureLaunch` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Launch` (GPUI), rules in `oxyn_core::blocks::launch`.

## 1. Purpose
An in-product launch card for an AI feature: a looping mini demo, a three-step tour, benefits, an admin-gated workspace switch and Try it now.

## 2. States
Autoplay (dots fill over 6 s) · Step picked (autoplay stops) · Switch off / saving (press again to cancel) / on · Member (switch locked with a note) · Tried · Learn more.

## 3. Micro-interactions
The demo types its request, then shows the result and a check; the active dot fills as its step plays; benefits fade in with each step.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Step | 6,000 ms (STEP_MS) | linear |
| Typing | 45 ms a character | linear |
| Saving | 1,200 ms (SAVE_MS) | — |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `tour` | Autoplaying tour, admin. |
| `member` | Member: the switch is locked with a note (Oxyn extra). |
| `enabled` | On for the workspace, tour paused (Oxyn extra). |

## 6. Responsive and edge cases
Up to 520 px. Below 440 px of block width the actions span the width and the model credit centres under them; demo lines truncate with an ellipsis. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Dots are a tablist with roving arrow keys, Home and End; the demo is decorative and hidden; the switch is named and its note says who can change it; every simulated action is announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Launch, ..)`; presses by id (fl-dot-<n>, fl-switch, fl-try, fl-learn, fl-dismiss), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
onboarding-tour, upgrade-card.
