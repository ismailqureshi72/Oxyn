**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Tone tuner (`tone-tuner`)

Family: AI Components (page `/components/ai/tone-tuner/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ToneTuner` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Tone` (GPUI), rules in `oxyn_core::aikit::tone`.

## 1. Purpose
Tune a reply to the right voice before sending: formal or casual, brief or detailed, friendly, confident or empathetic.

## 2. States
Touring (voices cycle until the first touch) · Waiting (debounce) · Rewriting (status shimmers, text dims) · Rewritten · Original · Failed (last version kept, Try again) · Copied.

## 3. Micro-interactions
Changed sentences fade in place while unchanged ones stay put; new words against the original are marked when Changes is on and the header counts them. Pressing an active chip returns to neutral.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Debounce | 220 ms | — |
| Rewrite | 650 ms | — |
| Sentence swap | 180 ms fade | ease-out |
| Tour | every 3.4 s | — |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `tour` | Tours the voices until you touch it. |
| `formal-detailed` | Formal and detailed, no tour (Oxyn extra). |
| `failed` | A failed rewrite keeps the last version. |

## 6. Responsive and edge cases
The message keeps a minimum height so voices never make the card jump. Under 520 px the sliders stack and the chips share one row; under 400 px chip icons and the count hide. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
Chips are toggle buttons with aria-pressed in a group named Voice; sliders are role slider and speak their level name; changes are mark elements. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Tone, ..)`; presses by id (tt-voice-<n>, tt-f-<n>, tt-d-<n>, tt-changes, tt-copy, tt-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
reply-draft, ai-translate.
