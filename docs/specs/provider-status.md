**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Provider status (`provider-status`)

Family: AI Components (page `/components/ai/provider-status/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ProviderStatus` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::ProviderStatus` (GPUI), rules in `oxyn_core::aikit::provstatus`.

## 1. Purpose
See at a glance whether the AI providers a product depends on are healthy, how fast they answer and how reliable they were over 30 days.

## 2. States
Checking (spinner, refresh disabled) · Countdown (ring drains to the next check) · Healthy / degraded / outage per row · Incident open (updates newest first) · Check failed (last results kept, alert) · Day readout on an uptime bar.

## 3. Micro-interactions
Health, latency and the header summary crossfade when a check lands; ticks change colour; an incident row opens its updates in place. The ring drains linearly; Check now restarts it.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Check | 900 ms (CHECK_MS) | — |
| Countdown | interval (default 12 s) | linear |
| Value swap | 160 ms crossfade | ease-out |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `incident` | An OpenAI outage resolving over checks. |
| `failed-check` | A failed check keeps the last results. |
| `compact` | All clear, no uptime bars (Oxyn extra). |

## 6. Responsive and edge cases
From 560 px each row is one line; below it (container query) the uptime bar drops under the name, and under 400 px latency hides. Names and model lists end in an ellipsis. The readout flips below the first bar so it never leaves the card. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
A labelled section with a list; incident rows are full-row buttons with aria-expanded and aria-controls; each uptime bar is one tab stop labelled with its 30-day uptime, arrows read days aloud. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::ProviderStatus, ..)`; presses by id (ps-refresh, ps-row-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
status-page, run-meter.
