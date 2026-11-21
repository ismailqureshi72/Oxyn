**Status:** built v0.1 (2026-11-21, C1f: AI & Agent blocks). Pro.

# Spec: Knowledge base (`knowledge-base`)

Section: Blocks · Agents (page `/blocks/agents/knowledge-base/`, Pro). Reference: uiarc "Blocks · Agents" ("Knowledge base"), behaviour only, under our own design; no source, artwork or copy taken. Every company, person and amount is fictional; companies and models that are named carry their official coloured marks.
Components: `KnowledgeBase` (Dioxus, `oxyn_dioxus::blocks`, styles `blocks.css`, prefix `oxbk-`) · `BlocksView` with `BlockVariant::KnowledgeBase` (GPUI), rules in `oxyn_core::blocks::kb`.

## 1. Purpose
A retrieval knowledge base in one card: watch sources ingest, then test a question and see which chunks come back and why.

## 2. States
Sources: Parse → Chunk → Embed → Index per source, Indexed, Failed at a step (reason, Retry), Added (ingests live) · Test retrieval: suggestions, results (top 3/4/6), no match, Rerank on (moves shown), result open (neighbours).

## 3. Micro-interactions
Each pipeline segment fills in turn while chunks climb; reranking reorders the same results in place and marks how far each moved; matching words are highlighted.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the block), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Pipeline step | 1300 ms | linear |
| Retry | resumes at embed | — |

Reduced motion: shimmer, pulses and slides stop; every state keeps its words and icons.

## 5. Variants
| Variant | What it shows |
|---|---|
| `sources` | Sources ingesting, one failing at embed. |
| `refunds` | Test retrieval on a refund question (Oxyn extra). |
| `reranked` | The same results reranked (Oxyn extra). |

## 6. Responsive and edge cases
One column from 760 px down to 320 px; below 560 px the pipeline goes under each source and its step labels drop; below 440 px the summary hides and the question field takes the full width. Layout follows the block's own width (container queries), never the viewport. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus data series and the official coloured brand and model marks.

## 7. Accessibility
Each pipeline is a progressbar whose value text names the step; results are buttons with aria-expanded named with score and rank change; matched words are mark elements; ingestion, failures and searches are announced. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`BlocksView::new(BlockVariant::KnowledgeBase, ..)`; presses by id (kb-view-<n>, kb-retry-<n>, kb-remove-<n>, kb-add, kb-sug-<n>, kb-rerank, kb-hybrid, kb-k-<n>, kb-open-<n>, kb-clear), keys through `BlocksState::key`. The view advances with `advance(ms)`.

## 9. Related
source-cards, citations-sources.
