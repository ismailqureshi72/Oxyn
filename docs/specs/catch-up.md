**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Catch up (`catch-up`)

Section: Blocks · AI (page `/blocks/ai/catch-up/`, Pro). Reference: uiarc "Blocks · AI" ("Catch up", https://uiarc.dev/components/blocks/catch-up), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `CatchUp` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::CatchUp` (GPUI), rules in `oxyn_core::blocks::catchup`.

## 1. Purpose
A catch me up digest: one button reads your tools for a range and streams back a ranked summary to clear line by line.

## 2. States
Idle · Reading (each source: waiting, spinner, check with how much it read) · Streaming lines · Ready (n of m left) · Stopped · GitHub skipped (error scenario, Retry) · Line open (link preview) · Marked as read · All caught up.

## 3. Micro-interactions
Sources read in turn; lines fade in one by one in section order (Needs you, Decisions made, Shipped, FYI); read lines leave the list.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Source read | 550 ms each (READ_MS) | — |
| Line in | 260 ms apart (ITEM_MS) | ease-out |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `digest` | Three day digest. |
| `idle` | Before the first read, since yesterday (Oxyn extra). |
| `partial` | GitHub skipped, with Retry (Oxyn extra). |

## 6. Responsive and edge cases
Up to 600 px. The range control wraps under the title on narrow cards; meta lines truncate with an ellipsis; item actions are always visible. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Sections are labelled regions with a heading and count; each sentence starts with its source for screen readers; lines not yet streamed are absent; the range is a radio group with arrow keys; reading, ready, read counts and all caught up are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::CatchUp, ..)`; presses by id (cu-run, cu-range-<n>, cu-open-<n>, cu-read-<n>, cu-all), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
inbox-digest, source-cards.
