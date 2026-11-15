**Status:** built v0.1 (2026-11-15, plan #32 affectUI fidelity pass, chunk 2).

# Spec: Model picker (`model-picker`)

Family: Chat. Closes the parity row: affectUI · Model Picker. Behaviour-only rebuild; nothing is copied.
Components: `ModelPickerMenu` (Dioxus) · `ModelPickerMenuEl` (GPUI), models in `oxyn_core::parity::chatfid`.
Oxyn extras on the same page: the plan #28 `ModelMenu` (variant `oxyn-model-menu`); the provider
switcher with search and favourites is its own page, `model-switcher`.

## 1. Purpose
The model menu of an AI chat: compare speed, quality and cost at a glance, switch on extended
thinking and pick its effort, and see what the pointed model costs before choosing it.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Composer | 362 px card, 16 px radius, "Ask anything…" 13.5 px, 30 px rounded-square send |
| Trigger | Full-radius pill 28 px: 20 px letter tile (6 px radius), name 13/600, blue bulb when thinking is on, chevron that flips |
| Menu | Dialog "Model settings", 320 px, 14 px radius, opens under the trigger (left-aligned) |
| Header | MODEL · SPEED · QUALITY · COST, 10.5 px uppercase, tracked |
| Row | 48 px: tile, name 13/600 (+ blue check when chosen), description 12 grey; three 40 px dash bars of five 3 px dashes |
| Dashes | Lit row: filled dashes blue (`#4C9CF8` dark, `#3B82F6` light) and fill in a 30 ms stagger; other rows: filled grey, empty darker |
| Router | "Varies" across the three columns |
| Switches | Extended thinking (bulb), Web search (globe): 26×15 track, blue when on |
| Effort | Unfolds under thinking (grid-rows 260 ms): "Effort" + Low / Medium / High segmented, selected on a raised pill |
| Tray | "~0.9s first token · $15 / M tokens" (numbers roll in), ↑↓ and ↵ kbd chips; Web search on: "Looks things up on the web when it helps" |

## 3. Behaviour
- Pointing at (or arrowing to) a row lights it and rewrites the tray; thinking multiplies the time by ×1.6 / ×2.5 / ×4 (Low / Medium / High): Opus 2.1 s → ~5.3 s at Medium.
- Click or Enter picks and closes; the trigger names the model and "extended thinking".
- Settings survive closing and reopening.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Clean, menu open, Opus lit, Sonnet chosen |
| thinking | Extended thinking on: Effort shown |
| web-search | Tray note |
| classic | Title-case headers, monochrome dot-dashes (4.5 px, round), monochrome check and switch, round send, 20 px menu, tray on its own shelf |
| brand-marks | Oxyn extra: coloured brand marks on rows and trigger |
| closed | The trigger only (bulb on) |
| oxyn-model-menu | Oxyn extra: the plan #28 menu with stats under the pointed row |

## 5. Keyboard and accessibility
- ↓ / Enter / Space open; ↑↓ move; Home / End; Enter picks; Esc closes and returns focus.
- Listbox with `aria-activedescendant`; options name their strengths ("Speed 2, quality 5, cost 5 of 5"); switches are `role=switch`; Effort is a `radiogroup` tabbable only while thinking is on; the tray is `aria-live=polite`.

## 6. Responsive
Composer 362 px, menu 320 px (never wider than the composer); under 340 px descriptions drop and rows are 40 px.
