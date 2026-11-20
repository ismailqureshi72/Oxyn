**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: AI translate (`ai-translate`)

Family: AI Components (page `/components/ai/ai-translate/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AiTranslate` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Translate` (GPUI), rules in `oxyn_core::aikit::translate`.

## 1. Purpose
Read a message in your language: pick one and the translation streams in place, sentence by sentence over the dimmed original.

## 2. States
Original · Thinking · Translating · Translated (Show original toggle) · Side by side · Failed (Try again).

## 3. Micro-interactions
Each sentence replaces its dimmed original as it lands; the detected-language chip sits by the language picker.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Think | 450 ms | — |
| Stream | 34 ms per tick | linear |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `german` | Translating to German. |
| `side-by-side` | Japanese beside the original (Oxyn extra). |
| `failed` | Couldn't translate, with Try again. |

## 6. Responsive and edge cases
Japanese and German lengths differ; the card grows smoothly. Side by side stacks under 640 px. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Language picker is a listbox; Show original is a switch; the status line is polite live. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Translate, ..)`; presses by id (tr-menu, tr-lang-<n>, tr-orig, tr-side, tr-retry), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
message-bubble, streaming-markdown.
