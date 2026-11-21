**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Trial banner (`trial-banner`)

Family: AI Components (page `/components/ai/trial-banner/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TrialBanner` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Trial` (GPUI), rules in `oxyn_core::aikit::trial`.

## 1. Purpose
Keep a trial's end in view without nagging: days left, what is lost, and the upgrade one click away.

## 2. States
Active · Warning (last days, names the end day) · Expired · Upgraded (Manage billing) · Adding a card · Card failed · Snooze menu · Folded into the header pill · Pill popover (the card).

## 3. Micro-interactions
The ring follows days left / trial length and takes the tone colour; copy swaps with a short rise; snooze and dismiss fold into the pill, which reopens the card.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Add payment | 1100 ms | — |
| Copy swap | 160 ms | ease-out |
| Note | shown 4 s | — |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `banner` | Nine days left as a slim banner. |
| `warning-card` | Two days left as a sidebar card. |
| `expired` | Expired. |
| `pill` | The header pill; it opens the card (Oxyn extra). |

## 6. Responsive and edge cases
Banner: under 600 px (container query) the buttons drop under the copy and the sub line may take two lines. Card: up to 340 px. The days are always in the text. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
Labelled regions; the ring is decorative; the snooze menu is a menu with arrow keys and Escape; results are announced, including simulated ones. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Trial, ..)`; presses by id (tb-compare, tb-pay, tb-manage, tb-snooze, tb-snooze-<n>, tb-dismiss, tb-pill), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
upgrade-prompt, banner.
