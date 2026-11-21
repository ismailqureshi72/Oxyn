**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: AI spark (`ai-spark`)

Family: AI Components (page `/components/ai/ai-spark/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AiSpark` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Spark` (GPUI), rules in `oxyn_core::aikit::spark`.

## 1. Purpose
A compact animated mark that says an AI is at work, for buttons, pills, avatars and inline status.

## 2. States
Idle (breath, one diagonal reaching) · Thinking (a ripple round the rays) · Writing (a heartbeat and a slow turn) · Done (gathers and settles) · Error (short rays, dim diagonals, red) · Loop · Paused. Variants: mark, pill (status word), avatar (round tile).

## 3. Micro-interactions
Every status is a function of time and of the ray's angle, so loops have no seam; a change blends from the pose on screen. The pill label crossfades.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Status blend | 420 ms | smoothstep |
| Idle breath | 6.4 s | sine |
| Thinking ripple | 1.1 s | sine |
| Writing beat | 0.9 s | gaussian pulses |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `loop` | The mark with its pill and avatar. |
| `pill-thinking` | The pill, thinking (Oxyn extra: ink tone). |
| `error` | Couldn't finish: short rays, dim diagonals, red. |

## 6. Responsive and edge cases
One footprint for every status; crisp from 12 px up (100-unit geometry). Tones: violet, ink, muted. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
A polite status region announcing changes once (AI is thinking) with aria-busy while working; decorative copies turn announce off. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Spark, ..)`; presses by id (ms-mood-<n>, ms-auto, ms-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
ai-mascot, thinking-indicator.
