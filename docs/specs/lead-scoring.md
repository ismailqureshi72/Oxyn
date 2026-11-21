**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Lead scoring (`lead-scoring`)

Section: Blocks · AI (page `/blocks/ai/lead-scoring/`, Pro). Reference: uiarc "Blocks · AI" ("Lead scoring", https://uiarc.dev/components/blocks/lead-scoring), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `LeadScoring` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Leads` (GPUI), rules in `oxyn_core::blocks::leads`.

## 1. Purpose
Inbound leads ranked by an AI fit score with a reason, the signals and breakdown behind it, and a push to the CRM.

## 2. States
Intro (rings count up) · Sorted by score or newest · Rescored (rings count up again, one lead moves) · Lead open (signals, breakdown) · Destination HubSpot or Airtable · Pushing · Pushed (In HubSpot) · Push failed (Retry).

## 3. Micro-interactions
Rings count up over 900 ms and their stroke follows the score; tiers read Hot, Warm or Cold in words and colour; open rows fade in.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Ring count-up | 900 ms (RING_MS) | ease-out |
| Push | 1,100 ms (PUSH_MS) | — |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `ranked` | Ranked list, rings counting up. |
| `open` | Top lead open with its breakdown (Oxyn extra). |
| `error` | Push fails, with Retry (Oxyn extra). |

## 6. Responsive and edge cases
Fits 360 to 640 px. Below 460 px of block width the role, arrival time and Rescore label leave the row, details start at the card edge and the push button fills the row. Names and reasons truncate with an ellipsis; scores keep fixed slots. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Rows are buttons named with company, person, score, tier and reason, with aria-expanded and aria-controls; Up and Down move between rows (roving tab stop), Enter or Space opens, Left and Right switch the CRM; rings are decorative. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Leads, ..)`; presses by id (ld-sort-<n>, ld-rescore, ld-open-<n>, ld-dest-<n>, ld-push, ld-retry), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
crm-card, data-enrichment.
