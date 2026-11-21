**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Write sweep (`write-sweep`)

Page `/components/agents/write-sweep/` (family text-motion). Reference: an X post the founders saved (R34 (aurora sweep while the agent fills a plan) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `WriteSweep` (Dioxus, `oxyn_dioxus::xpost`, `xpost/moments.rs`) · `XPostView` with `XPostVariant::WriteSweep` (GPUI), rules in `oxyn_core::xpost::sweep`.

## 1. Purpose
Show where an agent is writing so the eye follows the edit instead of hunting for what changed.

## 2. States
Ready · Sweeping (band, "Agent is writing" tag, rows swap as it passes) · Done (changes counted, Undo) · Undone (Redo, Rewrite).

## 3. Micro-interactions
A soft blurred band moves top to bottom; each row it crosses blurs out and settles into its new value; changed rows keep a thin info-coloured marker. The mono tone uses a grey band.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Sweep | 1800 ms default (duration_ms) | linear |
| Row settle | 450 ms | ease-out |
| Autoplay delay | 600 ms | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `aurora` | A soft colour band. |
| `mono` | A grey band for quiet UIs (Oxyn extra). |
| `compare` | Keeps the old value struck through. |

## 6. Responsive and edge cases
Rows wrap the old value under the new one when show_before is on; the band is clipped to the region. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The region is aria-busy while sweeping; the band is decorative and hidden under reduced motion (values still change); Enter runs, Cmd/Ctrl+Z undoes, Escape applies at once. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::WriteSweep, ..)`; presses by id (ws-run, ws-undo, ws-redo), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-doc-cursor, text-reveal, inline-diff.
