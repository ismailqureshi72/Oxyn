**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Social proof toast (`social-proof-toast`)

Family: AI Components (page `/components/ai/social-proof-toast/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `SocialProofToast` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Proof` (GPUI), rules in `oxyn_core::aikit::proof`.

## 1. Purpose
Show recent sign-ups and purchases on a landing or pricing page without interrupting the visitor.

## 2. States
Waiting · Cycling (newest at the front, older ones step back) · Paused and fanned (hover or focus) · Dismissed toasts · Limit reached / end of feed.

## 3. Micro-interactions
A new toast rises in at the front while the others step back 10 px and shrink 5 % each (their text hidden so nothing overlaps); hover or focus fans them out; dismiss buttons appear on hover or focus and are always visible on touch.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Interval | 4000 ms default | — |
| Entrance | 360 ms rise | ease-out |
| Stack and fan | 320 ms | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `feed` | A sample SaaS feed. |
| `single` | One at a time, no header (Oxyn extra). |
| `deep` | Five stacked, no fan (Oxyn extra). |

## 6. Responsive and edge cases
360 px wide; full width on narrow screens. One line per row with an ellipsis, so a toast never changes height. Real marks only for named brands (Slack, Linear, Stripe); fictional people and companies use initials. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
A labelled region; new toasts are not announced; each dismiss button names its toast; Escape dismisses the front toast; the pause keeps the time left. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Proof, ..)`; presses by id (sp-pause, sp-resume, sp-dismiss-<n>, sp-dismiss-front), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
toast, notification-stack.
