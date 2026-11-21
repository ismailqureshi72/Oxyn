**Status:** built v0.1 (2026-11-21, C1g: AI blocks II). Pro.

# Spec: Prompt bench (`prompt-studio`)

Section: Blocks · AI (page `/blocks/ai/prompt-bench/`, Pro). Reference: uiarc "Blocks · AI" ("Prompt studio"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `PromptStudio` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::Prompt` (GPUI), rules in `oxyn_core::blocks::prompt`.

## 1. Purpose
Edit, test and ship a prompt template: variables as chips with sample values, a test run on the chosen model, and versions to compare and promote.

## 2. States
Editing (System or User tab) · Variable empty (dashed chips, field invalid, Run asks to fill it) · Running (Writing, Stop) · Reply (latency, tokens, cost) · Stopped · Run failed (Retry) · Version menu (production marked, Compare, Promote) · Promoted (Undo) · Compare (word diff against production) · Model menu (model and effort).

## 3. Micro-interactions
Chips take their variable's series colour; the diff marks added words with + and green, removed words with − and a strikethrough; the reply streams three words a tick.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| First token | 410 to 720 ms by model | — |
| Stream | 3 words per 34 ms tick | linear |

Reduced motion: fades, shimmer and count-ups stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `editing` | Latest version, test run streaming. |
| `compare` | Word diff against production (Oxyn extra). |
| `error` | The run fails, with Retry (Oxyn extra). |

## 6. Responsive and edge cases
Up to 760 px, down to 320 px. Below 560 px of block width the variable fields go to two columns and below 340 px to one; below 480 px the test cases and Run each take a full row and the shortcut hint hides. Template text wraps anywhere. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
System and User are tabs with a tabpanel; version and model rows are radios in labelled groups; every field has a visible label in its colour; the diff uses ins and del with hidden Added and Removed words; Ctrl/Cmd+Enter runs from anywhere; Escape closes a menu or leaves compare. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::Prompt, ..)`; presses by id (ps-tab-<n>, ps-case-<n>, ps-run, ps-retry, ps-ver, ps-ver-<n>, ps-promote, ps-undo, ps-compare, ps-model, ps-model-<n>, ps-effort-<n>, ps-copy), keys through `BlocksState::key`. The view advances with `advance(ms)`; text fields show the samples' preset text.

## 9. Related
prompt-editor, model-picker, diff-view.
