**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Ghost mascot (`ghost-mascot`)

Family: AI Components (page `/components/ai/ghost-mascot/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `GhostMascot` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Ghost` (GPUI), rules in `oxyn_core::aikit::mascot`.

## 1. Purpose
A soft floating assistant for calm, friendly surfaces: Oxyn's own ghost with a scalloped, rippling hem.

## 2. States
Idle (breath, float, blinks, a glance) · Listening (comes forward, wide eyes) · Thinking (eyes up, a swirl curls above, the sheet turns translucent) · Writing (eyes on the line being drawn) · Done (happy squint, sparkle) · Error (one shake, worried brows) · Loop (no status: each status for 2.8 s) · Paused (frozen frame).

## 3. Micro-interactions
The hem is two travelling waves over three scallops; the shadow shrinks as the ghost rises; listening opens a small mouth; writing draws a line with a tiny pen.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Status blend | 420 ms (BLEND_MS) | ease-in-out |
| Loop step | 2800 ms per status (STEP_MS) | — |
| Blink | 140 ms every 3.6 s | linear |
| Error shake | 700 ms, decaying | sine |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `loop` | Plays every status in turn. |
| `listening-ink` | Listening, ink (Oxyn extra). |
| `sizes` | Thinking with a size ladder (Oxyn extra). |

## 6. Responsive and edge cases
A fixed square; under 56 px a tighter framing. Tones: soft (raised sheet with a hairline), ink, violet. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
role img labelled with name and status; optional polite announcements; status changes the face and marks. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Ghost, ..)`; presses by id (ms-mood-<n>, ms-auto, ms-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
ai-mascot, cube-mascot, pixel-mascot.
