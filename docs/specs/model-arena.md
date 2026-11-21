**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Blind compare (`model-arena`)

Section: Blocks · AI (page `/blocks/ai/blind-compare/`, Pro). Reference: uiarc "Blocks · AI" ("Model arena"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `ModelArena` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Arena` (GPUI), rules in `oxyn_core::blocks::arena`.

## 1. Purpose
A blind side by side comparison of two models on one prompt: both answers stream, you vote, and only then are the models revealed.

## 2. States
Ready · Thinking (before the first token) · Writing · Done · Finished first (the faster side) · Stopped · Failed (model B, Retry) · Voting open (both done) · Voted (models revealed with their marks, Next round) · Blind off (names shown early) · Leaderboard open (top five by win rate, updated by each vote).

## 3. Micro-interactions
Each side streams at its model's own pace after its own first token time; the stats line rolls first token time, speed and cost; revealed names fade in where the letter stood.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| First token | 380 to 1,100 ms by model | — |
| Stream | 2 or 3 words per 34 ms tick | linear |
| Reveal | 240 ms fade | ease-out |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `blind` | Blind round running. |
| `named` | Names shown from the start (Oxyn extra). |
| `error` | Model B fails, with Retry (Oxyn extra). |
| `leaderboard` | Idle with the leaderboard open (Oxyn extra). |

## 6. Responsive and edge cases
Up to 800 px. From 560 px of block width the panes sit side by side; below it they become two tabs, each with the side's status, so both statuses stay visible. Below 480 px the Leaderboard label hides (its name stays), samples scroll in one line and the votes form a 2 × 2 grid. Answers scroll inside a fixed height. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Each pane is a labelled region whose name gains the model only after the vote or with Blind off; answers are focusable scroll regions with aria-busy while streaming; votes are a labelled group; the narrow tabs are a tablist with arrow keys; Escape stops a run or closes the leaderboard. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Arena, ..)`; presses by id (ar-run, ar-sample-<n>, ar-vote-<a|tie|b|bad>, ar-next, ar-blind, ar-board, ar-retry, ar-tab-<n>), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
fanout-grid, model-card, eval-suite.
