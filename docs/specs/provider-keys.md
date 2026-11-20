**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Provider keys (`provider-keys`)

Family: AI Components (page `/components/ai/provider-keys/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ProviderKeys` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Keys` (GPUI), rules in `oxyn_core::aikit::keys`.

## 1. Purpose
Bring your own API keys: one row per provider with its mark, the masked key, status and last use.

## 2. States
Empty · Keys (active, near limit, invalid) · Revealed (5 s) · Adding (test before save: testing, valid, invalid) · Rotating · Removed (Undo 6 s).

## 3. Micro-interactions
Reveal shows the key with a countdown ring; Copy flashes Copied; Remove collapses the row and offers Undo.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Reveal | 5000 ms (REVEAL_MS) | linear ring |
| Undo window | 6000 ms (UNDO_MS) | — |
| Key test | 900 ms (TEST_MS) | — |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `keys` | Three keys: active, near the limit and invalid. |
| `add` | Adding a key, tested before save (Oxyn extra spend readout off). |
| `empty` | No keys yet. |

## 6. Responsive and edge cases
Keys are always masked except while revealed; long provider names truncate. Every key is fictional sample data. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Reveal, Copy, Rotate, Remove are labelled per provider; test results go to the live region; Undo is reachable before it expires. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Keys, ..)`; presses by id (pk-add, pk-cancel, pk-test, pk-save, pk-reveal-<n>, pk-copy-<n>, pk-rotate-<n>, pk-remove-<n>, pk-undo), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
api-keys-webhooks, composer-model-picker.
