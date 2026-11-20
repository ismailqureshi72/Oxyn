**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Speech player (`speech-player`)

Family: AI Components (page `/components/ai/speech-player/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `SpeechPlayer` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Speech` (GPUI), rules in `oxyn_core::aikit::speech`.

## 1. Purpose
Listen to an AI reply: Read aloud swaps the action row for a compact player in the same slot, and the text follows along word by word.

## 2. States
Idle (action row with Read aloud) · Preparing · Playing · Paused · Ended · Failed (Couldn't generate audio, Retry).

## 3. Micro-interactions
The spoken word is highlighted while upcoming words stay dimmed; clicking a word jumps there. Switching voice mid-sentence resumes from the same word. Speed chip cycles 1x / 1.5x / 2x.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Prepare | 640 ms (PREP_MS) | — |
| Word highlight | follows playback position | step |
| Row ↔ player swap | 160 ms crossfade | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `reading` | Reading with Ava, word by word. |
| `plain` | Inside a chat column, full-contrast text (Oxyn extra). |
| `idle` | The action row before Read aloud. |
| `failed` | Couldn't generate audio, with Retry. |

## 6. Responsive and edge cases
The player keeps the action row height so the message does not jump. The bar is scrubbable by pointer and arrow keys. Plain variant keeps full-contrast text for chat columns. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Play/Pause names change with state; the bar is a slider with time as value text; words are not each tabbable (one jump target via the slider). Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Speech, ..)`; presses by id (sp-read, sp-play, sp-speed, sp-stop, sp-copy, sp-seek-<ms>, sp-word-<n>, sp-voice-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
message-bubble, composer-voice-input.
