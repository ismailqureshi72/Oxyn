**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 2).

# Spec: Agent Crew (`agent-crew`)

Family: Agents. Closes the parity row: affectUI · Agent Crew (Pro). Behaviour-only rebuild; nothing is copied.
Components: `AgentCrew` (Dioxus) · `AgentCrewEl` (GPUI), model in `oxyn_core::parity::crewfid`.

## 1. Purpose
A lead and its sub-agents as one live tree of cards: every agent stays on screen, each card says what it does right
now, how full its context is, its tools and the last thing it was handed. Handoffs run as a light along the rounded
connectors and light up the card they reach; a click opens the agent's details with steering; a switcher swaps projects.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 760 px (max 100 %), container-sized; no background of its own |
| Header | 28 px, 18 px under it: folder glyph + title 13/600 + chevron (the project switcher); right: "3 working" blue, "1 needs you" amber, "8 agents" grey, 12.5 px |
| Switcher | listbox under the title, 272 px, 12 px radius, rows 34 px: folder, name, "2 working" / "1 needs you", check on the chosen one |
| Card | 205 px wide, 10 px radius, hairline; lead and sub-agents 112 px tall, helpers 79 px. Row 1: 3×3 pixel mark (12 px, agent colour), name 13/600 over the model 11 grey, status badge (18 px, 5 px radius; helpers show the icon only). Activity 12.5 px, one line. Context: 3 px bar in the agent colour (orange from 80 %, red from 95 %) + "136k / 200k" 11 px. Footer (lead and sub-agents): up to four tool tiles (three and "+N" past four), "↳ Researcher Root cause…" |
| Badges | Working (blue tint, spinner) · Idle (grey, dot) · Paused (grey, bars) · Needs you (amber, clock) · Failed (red, !) · Done (green, check) |
| Layout | Wide: lead centred over up to three sub-agents side by side (38 px gaps, 40 px below the lead); helpers hang under their sub-agent, indented 28 px, 14 px apart; the tree centres on its bounding box |
| Connectors | 1.5 px in the parent's colour at 55 %: lead → trunk → bus → each sub-agent with 6 px rounded turns; parent → helper an elbow from 19 px in, entering at the helper's title row; a 3 px open circle where a connector leaves a card |
| Colours | lead blue; sub-agents purple, cyan, green, pink, orange in turn; helpers keep their sub-agent's |
| Details | dialog under the card (above it where the crew ends), 205 px, 12 px radius, shadow: name 14/600 + close; task (3 lines); model tag with the maker's coloured logo; Running/Ran · Cost · Tokens; tool pills; last handoff; steering buttons (Message always; Pause + Stop while working; Resume + Stop when paused; Decline + Approve when it needs you; Retry when failed; the positive one ink-filled) |

## 3. Behaviour
- Demo: scripted passes per project (checkout ≈ 36 s, launch ≈ 25 s, postgres ≈ 23 s) that start over.
- A handoff runs a light hop by hop along the connectors (520 ms a hop, up then down through the tree) and the card it
  reaches lights up with a Gradient Floor in its colour, rising from the edge the light came in by. Handoffs present on
  mount do not run.
- Working agents' pixel marks twinkle in a wave and their activity shimmers; context bars glide.
- A click or Enter/Space on a card opens its details (calls `on_select`); Esc closes the message field, then the details.
- Steering: the component applies Pause/Resume/Stop/Approve/Decline/Retry on top of the data (demo) and reports
  `on_steer(agent, action, message)`; Message opens a field, Enter sends.
- Switching projects restarts the clock; the cards rise in and the connectors draw themselves.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live run, Fix the checkout crash |
| working-details | Lead's details while it works (Message / Pause / Stop) |
| needs-you | Settled with the lead waiting (Message / Decline / Approve, "1 needs you") |
| launch | The launch post crew from the switcher |
| narrow | 380 px: every level indented |
| classic | Elevated cards (9 px radius, shadow), grey footer band, filled badges |

## 5. Keyboard and accessibility
A tree with one tab stop; arrows walk the cards, Home/End go to the lead and the last card; cards are treeitems named
"Coder: Working, Editing cart.ts, context 38%" with aria-haspopup="dialog" and aria-expanded. The details are a dialog
whose close button takes focus. The switcher is a button with a listbox (↑ ↓ Enter, Esc). A polite live region reads
each handoff ("Lead handed to Coder: Guard empty carts, add tests") and each steering action ("Paused Coder").

## 6. Responsive
Wide at 760 px; below that every level is indented and cards take the remaining width. Rows are fixed, so changing
activity or context never moves a card. Below 420 px the "N agents" count drops. Long names, activities and tasks end in
an ellipsis with the full text on hover. Context values outside 0–1 or NaN are clamped; a NaN cost is left out.
