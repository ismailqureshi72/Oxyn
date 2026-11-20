**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Human handoff (`human-handoff`)

Family: AI Components (page `/components/ai/human-handoff/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `HumanHandoff` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Handoff` (GPUI), rules in `oxyn_core::aikit::handoff`.

## 1. Purpose
Hand a support chat from the AI assistant to a person without making the customer repeat themselves (Arc "Agent handoff"; slug human-handoff because agent-handoff is the agent-to-agent component).

## 2. States
Assistant · Transferring (summary passed to the team, typical wait) · Connected (person joined, typing) · Resolved · Unavailable (nobody online: keep chatting or leave an email) · Editable summary (with queue place).

## 3. Micro-interactions
A thin timeline marks each stage; the person's typing dots appear before their first message. The summary card can be edited before transfer.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Transfer | 2600 ms (TRANSFER_MS) | — |
| Typing | 1400 ms (TYPING_MS) | — |
| Assistant reply delay | 600 ms (BOT_MS) | — |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `assistant` | The assistant answers; Talk to a person hands over. |
| `connected` | The person has joined and taken over. |
| `editable-summary` | Edit the summary before the transfer, with a queue place (Oxyn extra). |
| `unavailable` | Nobody online: try again or keep chatting. |

## 6. Responsive and edge cases
Long summaries scroll; the stage timeline wraps labels under 390 px. Cancel transfer returns to the assistant. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Messages are a log; stage changes are announced; buttons say what they do (Talk to a person, Cancel, Keep chatting). Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Handoff, ..)`; presses by id (ho-human, ho-cancel, ho-keep, ho-retry, ho-resolve, ho-editsum, ho-send, ho-chip-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
message-bubble, agent-handoff.
