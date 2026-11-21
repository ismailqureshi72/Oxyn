**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Proactive card (`proactive-card`)

Page `/components/chat/proactive-card/` (family inbox). Reference: an X post the founders saved (R41 (inline action card in a feed) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ProactiveCard` (Dioxus, `oxyn_dioxus::xpost`, `xpost/moments.rs`) · `XPostView` with `XPostVariant::ProactiveCard` (GPUI), rules in `oxyn_core::xpost::proactive`.

## 1. Purpose
Let an assistant offer help it noticed (a renewal, a deadline) inside the feed the person is already reading, without a modal.

## 2. States
Offer · Snooze menu · Snoozed (one line, Undo) · Working (steps, progress) · Result (Review draft, Dismiss) · Failed step (Try again) · Removed (Undo).

## 3. Micro-interactions
Steps appear one by one with a shimmer on the running one and a check when done; the result swaps in place; snoozing folds the card into a dashed one-line row.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Step | 1100 ms each (STEP_MS) | — |
| Card in | 180 ms | ease-out |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `offer` | The offer between two posts. |
| `alone` | The card on its own (Oxyn extra). |
| `step-fails` | The second step fails and offers Try again. |

## 6. Responsive and edge cases
Feed posts and the card wrap their text; actions wrap. The source mark (Google Calendar) is the official coloured mark. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The card is an article named by its title; the step list is aria-busy while working; the snooze menu uses arrows, Enter and Escape; every outcome is announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::ProactiveCard, ..)`; presses by id (pc-accept, pc-later, pc-snooze-<n>, pc-close, pc-undo, pc-review, pc-retry), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-inbox, notification-center, decision-card.
