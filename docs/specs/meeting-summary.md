**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Meeting summary (`meeting-summary`)

Family: AI Components (page `/components/ai/meeting-summary/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `MeetingSummary` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Meeting` (GPUI), rules in `oxyn_core::aikit::meeting`.

## 1. Purpose
Turn a call into notes: Summary, Decisions and Action items stream in, every line links to its transcript quote, and action items can be checked off or sent to Linear or Notion.

## 2. States
Summarizing · Streaming · Done · Brief · Sending · Sent · Send failed (Retry) · Error (Try again).

## 3. Micro-interactions
A short summarising pass, then lines stream in section by section. A reference chip on each line opens its quote. Checking an action item strikes it; Send shows a spinner then a check with the destination.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Summarising pass | 1000 ms (THINK_MS) | — |
| Stream | 34 ms per word tick | linear |
| Send | 800 ms (SEND_MS) | — |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `streaming` | Summarizing, then the notes stream in. |
| `brief` | Brief notes: summary and action items (Oxyn extra). |
| `send-retry` | A failed Notion send with Retry. |
| `error` | Couldn't summarize, with Try again. |

## 6. Responsive and edge cases
Long attendee lists collapse to +N. Quotes wrap in a popover that flips above near the bottom edge. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Sections are headings; action items are checkboxes; send status goes to the live region. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Meeting, ..)`; presses by id (ms-brief, ms-regen, ms-check-<n>, ms-quote-<n>, ms-linear-<n>, ms-notion-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
streaming-markdown, citations-sources.
