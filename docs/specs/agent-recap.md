**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 4).

# Spec: Agent Recap (`agent-recap`)

Family: Agents. Closes the parity row: affectUI · Agent Recap (Pro). Behaviour-only rebuild; nothing is copied (the
demo night's events, names, counts and times are our own).
Components: `AgentRecap` (Dioxus) · `AgentRecapEl` (GPUI), model in `oxyn_core::parity::recapfid`.

## 1. Purpose
A catch-up card for when you come back: a tiny timeline of the night with a spike per event, then grouped items –
waiting on you, broken, finished – each with its action in place.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 804 px (max 100 %), container-sized, 16 px padding, 12 px radius, hairline, soft shadow, 12 px gaps |
| Header | "While you were away" 15/600; "Since 6:40 PM" 12 muted + a "13h 30m" chip; right: ✓ 3 done · 👤 2 need you · △ 1 failed (outline icons, bold counts) |
| Chart | 96 px: faint grid (eighths and a middle line), a smooth grey line with a fading fill – a soft peak per event, taller for bigger counts –; a 6 px dot on each peak (dark done, amber needs, red failed, grey undone); the two ends' times under it |
| Hover | a crosshair snapping to the nearest event, its dot grows, a dark tooltip "2:40 AM · Calendar sync to the CRM", its card's border darkens; pointing at a card does the same |
| Board | three equal lanes 8 px apart, grey, 10 px radius: icon + "Needs you" / "Failed" / "Done" + a count chip; each keeps room for `limit` cards |
| Card | 88 px, white, 8 px radius: 16 px logo + title 12.5/500; detail 11.5 muted (red for a failure) + a count chip + the time; a foot – Review + Approve / Merge (outlined), Retry, or a green Done tag + Undo |
| Working | "Working…", buttons disabled, a spinner in the action; then a green Approved / Merged / Fixed tag, a green check for the logo, the card dimmed, Undo |
| Undone | a done card undone: grey Undone tag and Redo |

## 3. Behaviour
- On open the night replays over 1.8 s (`replay`; 0 shows it all): the line draws left to right, dots pop on as it
  passes, cards fade in at their moment and the counts tick up.
- Approve / Merge / Retry work for a second, then settle (`on_action(id, "primary")`); Review lights the card
  (`on_action(id, "review")`); Undo / Redo (`on_undo`, `on_redo`). Counts and lane chips follow; results are announced.
- prefers-reduced-motion: everything shows at once.

## 4. Variants
default (replay) · pointed · acted · narrow (400 px, stacked lanes) · classic (the chart in its own panel, lanes without
a background, solid primary buttons, amber failure text).

## 5. Keyboard and accessibility
Tab moves through each card's buttons lane by lane; focusing a card lights its dot. Lanes are labelled sections, cards a
list; the chart is decorative and the header says the counts in words; tags name every state.

## 6. Responsive
Under 640 px the lanes stack and stop reserving room for three cards.

## 7. Known gaps
- The chart is pointed at by event zones (each event owns the stretch to its neighbours' midpoints), not by the exact
  pointer x; keyboard focus on the chart itself is not built.
- GPUI: a static snapshot; the chart is thin columns, no replay or tooltip.
