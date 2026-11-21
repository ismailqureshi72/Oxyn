**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Cube mascot (`cube-mascot`)

Family: AI Components (page `/components/ai/cube-mascot/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `CubeMascot` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Cube` (GPUI), rules in `oxyn_core::aikit::mascot`.

## 1. Purpose
A geometric assistant for technical products: an isometric cube with a face painted on its front side.

## 2. States
Idle · Thinking (a quarter turn at a time; the face moves to the side that arrives in front) · Writing (lines printed on the floor) · Done (sparkle, happy eyes) · Error (shake, worried brows) · Loop · Paused.

## 3. Micro-interactions
Eight real 3D corners are turned and projected each frame; the three visible sides are sorted by depth and shaded top, front and side; the face is mapped onto its side.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Status blend | 420 ms (BLEND_MS) | ease-in-out |
| Loop step | 2800 ms per status (STEP_MS) | — |
| Blink | 140 ms every 3.6 s | linear |
| Error shake | 700 ms, decaying | sine |
| Quarter turn | hold 0.5 s, turn 0.8 s | ease-in-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `loop` | Plays every status in turn. |
| `thinking-ink` | Thinking, ink (Oxyn extra). |
| `sizes` | Writing with a size ladder (Oxyn extra). |

## 6. Responsive and edge cases
A fixed square; under 56 px a tighter framing. Tones: violet, ink, soft. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
role img labelled with name and status; optional polite announcements. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Cube, ..)`; presses by id (ms-mood-<n>, ms-auto, ms-pause), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
ai-mascot, ghost-mascot, pixel-mascot.
