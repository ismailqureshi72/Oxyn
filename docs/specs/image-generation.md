**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Image generation (`image-generation`)

Family: AI Components (page `/components/ai/image-generation/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ImageGeneration` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::ImageGen` (GPUI), rules in `oxyn_core::aikit::imagegen`.

## 1. Purpose
Generate images from a prompt and pick one: four variations form, a filmstrip picks one, Upscale doubles it and Vary makes four more from it.

## 2. States
Idle · Forming (dot-matrix wave, then the coming colours, then a blur into the sharp image) · Ready · Upscaling (progress) · Failed variation (others still finish) · Stopped (Escape keeps what finished).

## 3. Micro-interactions
The dot matrix ripples while waiting, picks up the palette of the coming image, then blurs into it. Filmstrip selection moves a ring; upscale fills a hairline and the size label doubles. Model and aspect are chips in the prompt bar.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Forming per variation | ≈ 2.4 s, staggered 300 ms | ease-out |
| Upscale | 1400 ms (UPSCALE_MS) | linear |
| Blur into image | 400 ms | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `dot-matrix` | Four variations forming on the dot matrix. |
| `blur-up` | Blur-up placeholder in 16:9 (Oxyn extra). |
| `batch-grid` | All four in a grid with scan lines (Oxyn extra). |
| `failed` | The third variation fails; the others finish. |

## 6. Responsive and edge cases
Aspect 1:1, 4:5 and 16:9 keep the canvas size and letterbox. Long prompts wrap to three lines in the bar. A failed variation shows Retry in its own tile. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
The canvas is an img with a label that changes per state; the filmstrip is a listbox with arrow keys; Escape stops a running batch. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::ImageGen, ..)`; presses by id (ig-gen, ig-stop, ig-upscale, ig-vary, ig-download, ig-pick-<n>, ig-model-<n>, ig-aspect-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
media-generation-placeholder, media-analysis-scan.
