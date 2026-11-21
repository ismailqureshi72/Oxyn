**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Upgrade prompt (`upgrade-prompt`)

Family: AI Components (page `/components/ai/upgrade-prompt/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `UpgradePrompt` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Upgrade` (GPUI), rules in `oxyn_core::aikit::upgrade`.

## 1. Purpose
Turn a hit limit into a clear choice: what was used, what the next plan costs and unlocks, upgrade now or later.

## 2. States
Offer (limit reached or credits left) · Monthly / yearly · Upgrading · Upgraded (new limits, Continue) · Later (reminder, See Pro again) · Declined (nothing charged, Try again).

## 3. Micro-interactions
The price rolls when billing switches; the meter refills to the new limit after upgrading; every state shares one footprint so the card never jumps.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Upgrade | 1200 ms | — |
| Price roll | 200 ms | ease-out |
| Meter | 240 ms | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `limit` | 50 of 50 AI credits, yearly billing. |
| `under` | 38 of 50, monthly (Oxyn extra). |
| `seats` | All 3 seats used, card declined (Oxyn extra). |

## 6. Responsive and edge cases
Up to 440 px; the billing switch wraps under the price when narrow; feature lines stay single-line with an ellipsis; buttons stay side by side. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
A labelled section with a heading; the usage bar is role meter with a value text; billing is a radio group with arrow keys; results are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Upgrade, ..)`; presses by id (up-int-<n>, up-upgrade, up-later, up-again, up-continue, up-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
pricing-table, trial-banner.
