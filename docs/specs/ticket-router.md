**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Ticket router (`ticket-router`)

Family: AI Components (page `/components/ai/ticket-router/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TicketRouter` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Router` (GPUI), rules in `oxyn_core::aikit::router`.

## 1. Purpose
Watch support messages from Gmail, Intercom, Slack and GitHub get labelled with intent, sentiment and confidence and move into team lanes; low confidence goes to review.

## 2. States
Paused · Routing (one ticket at a time) · Finished · Strict threshold (more to review).

## 3. Micro-interactions
The incoming ticket shows its label chips, then slides into its lane; lane counts tick up.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Per ticket | 2000 ms (TICKET_MS) | ease-out slide |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `routing` | Tickets routed live. |
| `strict` | A stricter threshold sends more to review (Oxyn extra). |
| `paused` | Paused before the first ticket. |

## 6. Responsive and edge cases
Lanes scroll horizontally under 768 px; long subjects truncate. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Lanes are lists with counts in their headings; each routed ticket is announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Router, ..)`; presses by id (rt-pause, rt-restart, rt-speed, rt-th-<n>, rt-review-<t>-<lane>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
inbox-triage, kanban.
