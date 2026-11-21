**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Image reveal (`image-reveal`)

Family: AI Components (page `/components/ai/image-reveal/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ImageReveal` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Reveal` (GPUI), rules in `oxyn_core::aikit::reveal`.

## 1. Purpose
Show an image forming like a diffusion model samples, so the wait reads as progress rather than a spinner.

## 2. States
Queued (calm shimmer) · Generating (latent mosaic loses noise step by step over a blurred prediction, pill shows n / total) · Decoding (a band lands the crisp image top to bottom) · Done (one sheen, Generated in 5.2s, then the pill fades) · Error (dimmed partial image, Retry) · Loop.

## 3. Micro-interactions
Noise strength and blur follow whole steps, not time, so the picture moves in visible steps. The latent cells take the image's own colours. Sample images are Oxyn vector scenes.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Queue | 900 ms | — |
| Sampling | 4600 ms default, 30 steps | stepped |
| Decode | 560 ms | linear |
| Hold before loop | 2600 ms | — |
| Sheen | 1100 ms once | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `loop` | A looping generation over Oxyn's sample scenes. |
| `grid` | Four small tiles with staggered starts (Oxyn extra). |
| `wide-again` | Wide, with Generate again (Oxyn extra). |
| `error` | Fails at step 19, with Retry. |

## 6. Responsive and edge cases
Fills its container up to 560 px at the given aspect ratio (cover-cropped). The small size keeps only the ring and the count. A grid staggers starts. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
The canvas is role img, labelled with the state and then the alt text; while generating the pill is a progressbar with Step n of total; states are announced, steps are not. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Reveal, ..)`; presses by id (ir-retry, ir-again), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
image-generation, media-generation-placeholder.
