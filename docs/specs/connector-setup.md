**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Connector setup (`connector-setup`)

Family: AI Components (page `/components/ai/connector-setup/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ConnectorSetup` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Connector` (GPUI), rules in `oxyn_core::aikit::connector`.

## 1. Purpose
Connect the tools an assistant may use, inline: redirect, authorise, first sync, then the scopes granted.

## 2. States
Available · Redirecting · Authorising · Syncing (progress hairline) · Connected (scopes, Disconnect with inline confirm) · Denied (Retry).

## 3. Micro-interactions
The row morphs through each stage in place; Disconnect asks inline and confirms.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Redirect | 600 ms | — |
| Authorise | 900 ms | — |
| First sync | 2200 ms | linear hairline |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `connecting` | Notion connects and syncs. |
| `denied` | A denied grant with Retry. |
| `compact` | No search, nothing connecting (Oxyn extra). |

## 6. Responsive and edge cases
Search filters apps; compact variant hides search. Brand marks are the official coloured logos. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Each row is a group with status text; progress is announced at start and end only. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Connector, ..)`; presses by id (cs-connect-<n>, cs-open-<n>, cs-disc-<n>, cs-yes-<n>, cs-no, cs-retry-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
integrations, agent-tool-calls.
