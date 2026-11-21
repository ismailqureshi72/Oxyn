**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A).

# Spec: AI answer (`ai-answer`)

Family: Chat. Closes the parity row: affectUI · AI Answer. Behaviour-only rebuild; nothing is copied.
Components: `AnswerCard` (Dioxus) · `AnswerCardEl` (GPUI), models in `oxyn_core::kit::answers`.
Oxyn extras on the same page: the plan #27 `AiAnswer` (variant `oxyn-ai-answer`; its own spec is `citations-sources`).

## 1. Purpose
A streamed answer that shows where it came from: the sources it read, citation chips that preview them, and the actions people take after reading – copy, regenerate, rate, follow up.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 480 px in the demo (fills up to 640 px), 12 px continuous radius, 1 px hairline, 20 px padding |
| Question | h3 17/600, 24 px line |
| Sources | Three 72 px cards (coloured brand tile 16 px, name 12 grey, index, title 12.5/600 two lines) and a "+N more" tile (not a button) |
| Status | Blue spark + "Reading 8 sources" → "Writing" (shimmer) → "Answer" |
| Body | 15/23 px; bullets with bold lead-ins; citation chips: 20 px pills, chip fill, 11.5 px grey label ("Notion +1") |
| Chip preview | 290 px tooltip under the chip: per source a tile, domain, index and title (links) |
| Related | RELATED 11 px uppercase tracked, 40 px follow-up rows with a chevron, hairlines |
| Footer | Hairline on top, 30 px icon buttons (Copy, Regenerate, Good, Bad), right "Answered in 2.9s"; while writing: one Stop pill |
| Feedback | Thumbs down: "What went wrong?" and four 28 px reason chips; a reason or thumbs up: "Thanks for the feedback"; Copy: green filled check and "Copied" |

## 3. Behaviour
- Sources pop in one by one (300 ms apart) while it reads (1.4 s), then the text streams at 38 words a second with a block caret.
- Stop freezes the text ("Stopped at N words"); Regenerate restarts the stream and clears the vote.
- Pointing at or focusing a chip opens its preview; Esc or leaving closes it.
- Copy shows the check and "Copied" for 1.6 s.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Clean, streaming from the start |
| finished | Finished |
| feedback | Thumbs down with the reasons |
| classic | Source cards in one grey band with dividers, a blue round agent badge, title-case Related, arrows, a grey footer band, 18 px radius |
| oxyn-ai-answer | Oxyn extra: the plan #27 answer |

## 5. Keyboard and accessibility
Tab: source cards, chips, follow-ups, footer. The body is a polite live region (aria-busy while streaming); chips are buttons named "Source: <titles>" with role=tooltip previews; thumbs are aria-pressed, thumbs down aria-expanded.

## 6. Responsive
Up to 640 px; three cards and "+N", two below 480 px, one below 340 px; the tooltip stays inside the card (max 80% of it).
