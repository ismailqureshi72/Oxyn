**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Ask your data (`ask-your-data`)

Family: AI Components (page `/components/ai/ask-your-data/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AskYourData` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Ask` (GPUI), rules in `oxyn_core::aikit::ask`.

## 1. Purpose
Answer a plain question about a database with a chart, while showing the SQL that produced it so the answer can be checked.

## 2. States
Empty (suggestions) · Writing SQL (panel open, tokens stream) · Running (rolling rows scanned) · Done (chart builds, reading streams, panel folds unless kept open) · No match (suggestions) · Stopped (Enter or Escape while busy) · Error (Try again).

## 3. Micro-interactions
The SQL streams four words a tick and is tinted by token type; the panel folds once the query starts. Bars rise from the baseline, lines draw, the table swaps in place. Arrow keys move a readout across categories with a guide line.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Think before SQL | 450 ms | — |
| SQL stream | 4 words per 34 ms tick | linear |
| Query | 1300 ms (RUN_MS), rows ease out | ease-out |
| Chart build | 460 ms (BUILD_MS) | ease-out |
| Reading stream | 3 words per tick | linear |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `signups` | Weekly signups by plan as stacked bars. |
| `revenue` | MRR by plan as lines, latest values in the legend (Oxyn extra). |
| `no-match` | A question that matches no table offers suggestions. |
| `failed` | The query times out, with Try again. |

## 6. Responsive and edge cases
Under 520 px (container query) the source keeps only its mark, the view switch takes its own row and every other x label hides. Wide SQL and the table scroll inside their own wells with thin scrollbars; the stage keeps its height so the card does not jump. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
The SQL row is a button with aria-expanded and aria-controls; the chart is a focusable group whose readout is announced; the stage is aria-busy while working; the run is narrated in the polite region. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Ask, ..)`; presses by id (ay-ask, ay-stop, ay-sql, ay-copy, ay-chart-<n>, ay-sug-<n>, ay-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
genui-blocks, enrich-column, run-meter.
