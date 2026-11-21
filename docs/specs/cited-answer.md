**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family A).

# Spec: Cited answer (`cited-answer`)

Family: Chat. Closes the parity row: affectUI · Cited Answer. Behaviour-only rebuild; nothing is copied.
Components: `CitedAnswerCard` (Dioxus) · `CitedAnswerEl` (GPUI), models in `oxyn_core::kit::answers`.
Oxyn extras on the same page: none (the plan #27 `SourceCitation` stays on its own page).

## 1. Purpose
Checking which sentence comes from which source: point at a citation to light up its sentence, click to read the passage it rests on.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Card | 460 px, 12 px radius, hairline; last row 48 px under a hairline |
| Header | Three overlapping 16 px letter tiles in their tints + "3 sources · 14 pages read" 12.5 grey |
| Text | 15/23 px; markers: 17 px squares, 5 px radius, chip fill, 10.5 px number; lit: blue with white number |
| Lit sentence | Tinted (#E7EDFB / #25304A), weight 500; every other sentence at 38% |
| Chips | 32 px, 9 px radius, hairline: tile, name, index; lit: chip fill; open: 3 px blue ring with a 1.5 px gap |
| Quote | Hairline panel, 12 px radius: tile, "site · date", index tag, Close ✕; title 15/600; passage 14/21 grey with a 2.5 px left rule and the matching words marked (tint, 600); Open source (outlined, ↗) and Copy quote (ghost → green "Copied") |
| Last row | Status ("Cache Notes · cited in 1 sentence", "<name> · <title>" on hover, "Quote copied" in green, or "5 of 6 sentences backed by a source") and an outlined Follow up → |

## 3. Behaviour
- Hover or focus a marker or chip: that source lights; leaving returns to the open or chosen one.
- Click: unfold or fold its quote (one at a time); Copy quote shows "Copied" and "Quote copied".
- Static: nothing streams.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Clean, source 2 lit |
| quote | Quote unfolded |
| none-lit | Nothing lit |
| classic | Text on a white card over a grey tray with the chips and last row; the quote sits above the chips; Open source filled with the arrow after the label |

## 5. Keyboard and accessibility
Tab through markers, chips, quote actions, Follow up. Markers and chips are buttons with aria-expanded/aria-controls; the quote is a named region; the last row is a polite live region.

## 6. Responsive
Up to 460 px, from 300 px: tighter under 400, index tags drop under 360, Copy quote keeps only its icon under 280; chips stay on one line.
