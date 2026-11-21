**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A).

# Spec: Message actions (`message-actions`)

Family: Chat. Closes the parity row: affectUI · Message Actions. Behaviour-only rebuild; nothing is copied.
Components: `MessageActionBar` (Dioxus) · `MessageActionsEl` (GPUI), models in `oxyn_core::kit::answers`.
Oxyn extras on the same page: the plan #27 `ResponseFeedback` (variant `oxyn-response-feedback`; also its own page).

## 1. Purpose
The quiet bar under an AI message: copy, regenerate with versions, rate with a reason, read aloud.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 544 px, 14 px radius, hairline; message padding 20/24 |
| Message | User bubble right (78%, 10 px radius, grey), model line (16 px brand tile + "Sonnet 5.5" 13 grey), answer 16/24 |
| Bar | 56 px band under a hairline (#FAFAFA / #1A1B1E), 32 px icon buttons 8 px radius, 8 px gaps; dim (62%) until the card is pointed at |
| Tooltips | Above the button, 12/600 on a white pill: Copy/Copied, Regenerate/Writing…, Thanks!, Read aloud/Stop |
| Versions | ‹ 2 / 2 › unfolds after Regenerate from two versions on; arrows disabled at the ends |
| Reasons | "What went wrong?" 14/600 + ✕, chips 32 px 8 px radius (chosen grey, the rest at 50%), Send 36 px solid (disabled grey) → green "Thanks!" |
| Read aloud | Blue four-bar equalizer on a pale-blue square |

## 3. Behaviour
- Copy: filled green check for 1.6 s.
- Regenerate: the icon spins 1.2 s while the text blurs, then the new version lands and the switcher grows.
- Thumbs toggle; thumbs down opens the reasons; Send needs a reason.
- Read aloud plays for the length of the answer, or until Stop.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Clean |
| feedback | Reasons open |
| read-aloud | Equalizer |
| classic | Message card and a separate bar card with an 8 px gap, chosen reason blue (#DBEAFE / #1D4ED8) |
| oxyn-response-feedback | Oxyn extra |

## 5. Keyboard and accessibility
A toolbar "Message actions"; aria-pressed thumbs and Read aloud; thumbs down aria-expanded; versions read "Version 2 of 3"; a status region announces each change; tooltips are decoration.

## 6. Responsive
Up to 544 px; touch: 36 px buttons, no fade, no hover tooltips; below 320 px Send wraps.
