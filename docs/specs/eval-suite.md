**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Eval suite (`eval-suite`)

Section: Blocks · Agents (page `/blocks/agents/eval-suite/`, Pro). Reference: uiarc "Blocks · Agents" ("Eval suite"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `EvalSuite` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Evals` (GPUI), rules in `oxyn_core::blocks::evals`.

## 1. Purpose
Run a new version of an LLM feature against its baseline, case by case, and see the regressions and fixes with the cases behind them.

## 2. States
Not run · Running (cells fill round robin, score counts) · Stopped · Finished · Judge failed (error scenario) · Filter All / Failed / Regressions / Improved · Case open (input, expected, both outputs, rubric) · Marked as expected / Issue opened (Undo).

## 3. Micro-interactions
Cells pop in with their glyph; regressions are the only solid cells; category bars show a ghost tick at the baseline; history bars let you read earlier runs.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Case scored | 90 ms each (48 cases) | linear |
| Cell in | 180 ms | ease-out |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `ready` | Ready to run 48 cases. |
| `regressions` | Finished, regressions only, one open (Oxyn extra). |
| `judge-fails` | The judge stops halfway (Oxyn extra). |

## 6. Responsive and edge cases
Up to 760 px; below 680 px the cells go under each label; summary goes to two columns at 620 px and one at 480 px; outputs stack below 560 px; below 480 px categories, triage words and filter counts hide. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Each category row is one image with a full text summary; pass, fail and error have their own glyphs; case rows are buttons with aria-expanded; the filter is a radio group with arrow keys; results and triage are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Evals, ..)`; presses by id (ev-run, ev-f-<n>, ev-open-<n>, ev-expected-<n>, ev-issue-<n>, ev-undo-<n>, ev-hist-<n>), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
evals, eval-board.
