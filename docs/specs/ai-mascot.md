**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: AI mascot (`ai-mascot`)

Family: AI Components (page `/components/ai/ai-mascot/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AiMascot` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Mascot` (GPUI), rules in `oxyn_core::aikit::mascot`.

## 1. Purpose
Give an assistant a face that shows what it is doing, for empty states, chat headers and loading moments. Oxyn's own character: a rounded head with an antenna orb.

## 2. States
Idle (breath, float, blinks, a glance) · Listening (comes forward, wide eyes) · Thinking (eyes up, the companion moves) · Writing (eyes on the line being drawn) · Done (happy squint, sparkle) · Error (one shake, worried brows) · Loop (no status: each status for 2.8 s) · Paused (frozen frame).

## 3. Micro-interactions
The orb rests on the antenna, pulses while listening, circles the head (passing behind it) while thinking, turns red on an error and becomes a sparkle when done. Eyes move with the gaze; blush shows when happy.

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
| `thinking-violet` | Thinking, violet (Oxyn extra). |
| `sizes` | Done at 112 px with a 24 / 40 / 64 ladder (Oxyn extra). |

## 6. Responsive and edge cases
A fixed square of `size` px that never reflows its parent; under 56 px a tighter framing with larger eyes. Tones: ink, soft, violet. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
role img labelled with name and status; optional polite announcements; every status changes the face, not only motion or colour. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Mascot, ..)`; presses by id (ms-mood-<n>, ms-auto, ms-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
ghost-mascot, cube-mascot, pixel-mascot, ai-spark.
