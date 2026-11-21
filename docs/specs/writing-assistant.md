**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A).

# Spec: Writing assistant (`writing-assistant`)

Family: Chat. Closes the parity row: affectUI · Writing Assistant. Behaviour-only rebuild; nothing is copied.
Components: `WritingAssistant` (Dioxus) · `WritingAssistantEl` (GPUI), models in `oxyn_core::kit::answers`.
Oxyn extras on the same page: none.

## 1. Purpose
A small AI writing chat: suggestions to start from, streaming replies and voice dictation, in a card with a soft gradient glow.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 380 px, 14 px radius, hairline, min 430 px tall |
| Header | 62 px under a hairline: violet sparkle, name 14.5/600, tagline 12.5 grey; "New chat" + once there are messages |
| Glow | Pink, violet and blue blurred blobs drifting in the lower 62%; quieter (35%) once you chat; dimmer in dark |
| Welcome | "What should we write?" 17/600 and 32 px suggestion chips (hairline, 9 px radius) |
| Messages | User bubble right (grey, 11 px radius, 13.5 px); reply plain 13.5/21; "Writing…" shimmers before it streams |
| Field | Bare input "Ask anything…", a round send when there is text, a mic; dictating: pale-blue tile with three blue level bars |

## 3. Behaviour
- A chip or Enter sends; the reply waits 0.7 s then streams at 22 words a second.
- The mic dictates a sample take into the field and sends it when it ends; pressing it again stops.
- New chat clears back to the welcome.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Welcome |
| chat | After a message |
| dictating | Dictating |
| classic | Inset rounded panel with a stronger glow, 23 px greeting, capsule translucent chips, a white pill field and a round white mic with a ring while listening |

## 5. Keyboard and accessibility
New messages are announced politely; "Writing…" has a label; the mic is an aria-pressed toggle.

## 6. Responsive
Up to 380 px.
