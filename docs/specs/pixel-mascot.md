**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Pixel mascot (`pixel-mascot`)

Family: AI Components (page `/components/ai/pixel-mascot/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `PixelMascot` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Pixel` (GPUI), rules in `oxyn_core::aikit::pixel`.

## 1. Purpose
A pixel-art assistant for playful products and retro themes: Oxyn's own robot on a 24 × 24 grid.

## 2. States
Idle (bob, blink, glance) · Thinking (bubble of dots, blinking antenna) · Writing (keyboard, tapping arms) · Done (arms up, twinkling stars) · Error (crossed red eyes, red antenna, one shake) · Loop · Paused.

## 3. Micro-interactions
Sprites step on a fixed clock in whole pixels; a status change shuts the eyes for two steps first, so it never cuts.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Sprite step | 120 ms (STEP) | stepped |
| Bob | 2 steps up, 8 steps period | stepped |
| Change | 2 steps eyes shut | stepped |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `loop` | Plays every status in turn. |
| `writing-violet` | Writing, violet (Oxyn extra). |
| `scales` | Done at 1×, 2×, 3× and 5× (Oxyn extra). |

## 6. Responsive and edge cases
A fixed square of 24 × scale px; crisp at every whole scale (crispEdges, merged runs). Inline at scales 1 and 2. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
One SVG with role img and a label that follows the status; optional polite announcements; each status has its own shape. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Pixel, ..)`; presses by id (ms-mood-<n>, ms-auto, ms-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
ai-mascot, ghost-mascot, cube-mascot.
