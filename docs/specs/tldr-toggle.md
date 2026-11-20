**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: TL;DR toggle (`tldr-toggle`)

Family: AI Components (page `/components/ai/tldr-toggle/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TldrToggle` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Tldr` (GPUI), rules in `oxyn_core::aikit::tldr`.

## 1. Purpose
Read an article at the depth you want: Full, Summary or Key points, with a length control.

## 2. States
Full · Summarising · Summary · Key points (each jumps to its paragraph) · Failed (Try again).

## 3. Micro-interactions
Switching streams the summary in place; a key point jumps to Full and highlights its paragraph briefly.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Think | 700 ms | — |
| Paragraph highlight | 1600 ms (JUMP_MS) | ease-out fade |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `summary` | A medium summary streams in. |
| `key-points` | Five key points that jump to their paragraph. |
| `full` | The full article. |
| `failed` | Couldn't summarize, with Try again. |

## 6. Responsive and edge cases
Segmented control stays one row down to 320 px. Saved-time readout updates per length. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Mode and length are radio groups with arrow keys; key points are buttons naming the paragraph. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Tldr, ..)`; presses by id (td-mode-<n>, td-len-<n>, td-jump-<n>, td-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
streaming-markdown, citations-sources.
