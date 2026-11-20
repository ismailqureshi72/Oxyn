**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Enrich column (`enrich-column`)

Family: AI Components (page `/components/ai/enrich-column/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `EnrichColumn` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Enrich` (GPUI), rules in `oxyn_core::aikit::enrich`.

## 1. Purpose
Run one AI prompt down every row of a table column and see each value with its source.

## 2. States
Idle · Running (cells shimmer, a few at a time) · Cell done (value + source) · Cell failed (Retry) · Done (footer counts).

## 3. Micro-interactions
Shimmering cells settle into their value; the footer counts done / failed / left.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Cell | 900 ms (CELL_MS) | — |
| Concurrency | 3 (industry) or 1 | — |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `industry` | Industry, three cells at a time. |
| `one-at-a-time` | Employee counts one by one (Oxyn extra). |
| `idle` | HQ city, not run yet. |

## 6. Responsive and edge cases
Long values truncate with a tooltip; the table scrolls horizontally on narrow screens. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Table semantics; cell states in text; Cmd/Ctrl+Enter runs. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Enrich, ..)`; presses by id (ec-run, ec-stop, ec-menu, ec-col-<n>, ec-conc-<n>, ec-retry-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
crm, fanout-grid.
