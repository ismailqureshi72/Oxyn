**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: PII redaction (`pii-redaction`)

Family: AI Components (page `/components/ai/pii-redaction/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `PiiRedaction` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Pii` (GPUI), rules in `oxyn_core::aikit::pii`.

## 1. Purpose
Strip personal data before a prompt reaches the model and restore it for the reader afterwards.

## 2. States
Edit · Scanning (entities light up one by one) · Found (counts by type) · Redacted ([EMAIL_1] placeholders) · Sending · Reply (restored, or placeholders kept with Show original off).

## 3. Micro-interactions
Each name, email, phone and card is marked with its type colour as the scan reaches it; Redact swaps marks for placeholders in place.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Scan step | 120 ms per entity (SCAN_STEP_MS) | step |
| Model think | 700 ms | — |
| Reply stream | 34 ms per word tick | linear |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `scan` | Scanning the sample message. |
| `categories` | Choose what to redact (Oxyn extra). |
| `edit` | Edit the message, then Scan. |

## 6. Responsive and edge cases
Long messages wrap; placeholders never break mid-token. Category toggles exclude a type from redaction. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Marks carry their type in text for screen readers; Show original is a switch; counts are in a status line. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Pii, ..)`; presses by id (pii-scan, pii-redact, pii-send, pii-orig, pii-reset, pii-type-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
composer-input-shell, privacy screens.
