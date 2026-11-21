**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Done shockwave (`done-shockwave`)

Page `/components/feedback/done-shockwave/` (family feedback). Reference: an X post the founders saved (R39 (shockwave) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `DoneShockwave` (Dioxus, `oxyn_dioxus::xpost`, `xpost/moments.rs`) · `XPostView` with `XPostVariant::DoneShockwave` (GPUI), rules in `oxyn_core::xpost::shock`.

## 1. Purpose
Mark the end of a long task with one unmistakable moment, so someone who looked away notices it finished.

## 2. States
Idle · Running (steps, progress) · Done (two rings, check, "Deployed in 3.6s") · Failed (one short red pulse, Retry) · Reduced motion (still halo).

## 3. Micro-interactions
Two rings leave the status mark 160 ms apart and roll across the card with a glow, fading as they grow; the mark pops into a check. A failure sends one short ring a quarter of the distance.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Task | 3600 ms at 1× | linear |
| Ring | 900 ms (WAVE_MS) | ease-out cubic |
| Echo delay | 160 ms (ECHO_MS) | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `done` | A deploy that lands with two rings. |
| `soft` | Faint rings for a busy page (Oxyn extra). |
| `fails` | Fails with one short red pulse and Retry. |
| `still` | Reduced motion: a still halo. |

## 6. Responsive and edge cases
Rings are clipped to the card; buttons wrap under the title. Strength scales the ring opacity. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
Rings are decorative; the status line says what happened and a failure is an alert; steps list their state in text. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::DoneShockwave, ..)`; presses by id (sw-run, sw-retry, sw-reset), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
toaster, alert, loaders.
