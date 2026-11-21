**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: AI setup wizard (`ai-setup-wizard`)

Section: Blocks · AI (page `/blocks/ai/ai-setup-wizard/`, Pro). Reference: uiarc "Blocks · AI" ("AI setup wizard"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `AiSetupWizard` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Wizard` (GPUI), rules in `oxyn_core::blocks::wizard`.

## 1. Purpose
Turn on an AI assistant in three steps: connect sources and watch them sync, choose model, tone and guardrails, then test with a question and go live.

## 2. States
Step 1: source off / syncing (count and progress) / synced / failed (Retry) / disconnected · Step 2: model card, tone, guardrails · Step 3: question asked, answer streaming (Stop), citations · Live (summary, Edit setup back to draft) · Continue disabled until a source has synced.

## 3. Micro-interactions
Sync counts climb with a progress line; the answer streams three words a tick and each citation number appears once its sentence is complete; the header pill flips from Draft to Live.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Sync | 2,400 ms a source (SYNC_MS) | linear |
| Answer | 3 words per 34 ms tick | linear |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `connect` | Step 1, nothing connected. |
| `test` | Step 3 with sources synced (Oxyn extra). |
| `error` | Intercom's token expired, with Retry (Oxyn extra). |
| `live` | Live, with the summary (Oxyn extra). |

## 6. Responsive and edge cases
Up to 600 px. Below 480 px of block width step labels hide behind their numbers and model cards stack; below 400 px the footer meta and citation names hide. The step area keeps one minimum height. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Steps are an ordered list with aria-current; each sync is a progressbar named after its source; model cards are a radio group with arrow keys; guardrails are switches; Enter asks or stops; every change is announced, including that it is simulated. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Wizard, ..)`; presses by id (wz-step-<n>, wz-connect-<n>, wz-retry-<n>, wz-model-<n>, wz-tone-<n>, wz-guard-<n>, wz-next, wz-back, wz-ask-<n>, wz-stop, wz-live, wz-edit), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
connector-setup, model-picker, citations-sources.
