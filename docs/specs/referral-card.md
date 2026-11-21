**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Referral card (`referral-card`)

Family: AI Components (page `/components/ai/referral-card/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ReferralCard` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Referral` (GPUI), rules in `oxyn_core::aikit::referral`.

## 1. Purpose
Run a give-and-get referral program from one card: share the link, invite by email, see who joined and what was earned.

## 2. States
Idle · Chips (valid addresses) · Invalid address / too many (hint turns into an error) · Sending · Sent (rows appear as Invited, Just now) · Send failed (chips kept) · Funnel moving (simulated) · Milestone unlocked.

## 3. Micro-interactions
Enter, comma, space or Tab turns an address into a chip; Backspace on an empty field removes the last; stages change with a crossfade; milestone segments fill and turn green when reached.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Send | 900 ms | — |
| Simulated steps | 2.6 s, 4.8 s, 7 s, 9.4 s | — |
| Stage swap | 160 ms crossfade | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `program` | Friends sign up and subscribe; the milestone unlocks. |
| `compact` | No list, no simulation (Oxyn extra). |
| `send-fails` | The first send fails and keeps the chips. |

## 6. Responsive and edge cases
Up to 560 px; under 470 px (container query) share buttons take their own row and emails hide behind names. Chips scroll sideways inside a fixed-height field. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
Share buttons are named icon buttons; the hint is a polite status tied with aria-describedby and sets aria-invalid on errors; stages are icon plus word. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Referral, ..)`; presses by id (rf-copy, rf-share-<x|slack|email>, rf-send, rf-chip-<n>, rf-sample), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
upgrade-prompt, invite-dialog.
