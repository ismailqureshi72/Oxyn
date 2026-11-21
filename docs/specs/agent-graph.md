**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 3).

# Spec: Tool network (`agent-graph`)

Family: Agents. Closes the parity row: affectUI · Agent Graph (Pro). Behaviour-only rebuild; nothing is copied (the demo
agents, models, tools and calls are our own).
Components: `AgentGraph` (Dioxus) · `AgentGraphEl` (GPUI), model in `oxyn_core::kit::graphfid`.

## 1. Purpose
Agents and the tools they call as a living network: the lead in the middle, the other agents on an inner orbit, tools and
you on the outer one; a pulse along the link for every call and its answer, agents that breathe while they work and turn
amber when they need you; a click opens a node's card with its latest calls.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 756 px (max 100 %), container-sized |
| Bar | 32 px above the panel: three-dot network glyph (blue) + title 13/600; chips "Writer needs you" (amber, clock) / "Coder failed" (red, alert), 24 px pills; right: "5 agents 7 tools 72 calls/min $0.66", numbers 600 in ink, words grey 12.5 |
| Panel | white, hairline, 12 px radius, 0.6 × 756 px tall (fixed `height` possible); no scroll |
| Nodes | rounded squares (27 % radius): lead 56 px with a soft blue halo, agents 48 px, tools and you 32 px; white with a hairline and a soft shadow, coloured brand marks; you a dark tile with a person mark, its label above |
| Labels | agents: name 12.5/600 + model 11.5 grey ("Needs you" amber, "Error" red); tools and you: name 12.5 grey |
| Status | working: a soft ring breathes; done: green check badge (15 px, top right); waiting: amber fill and stroke, a shine sweeping across, clock badge; error: the same in red with "!" |
| Links | one even 1.5 px light-grey line each with one rounded right-angled turn (12 px), turning on the side that clears every node |
| Pulses | a short line with a glow along the link: blue call, green answer, amber for you, red error (the link flashes) |
| Card | dialog beside its node (towards the middle), 244 px, 12 px radius, shadow: mark tile 24 px, name + model, status pill, close; what it does; "Latest calls" (→ mark, name, · label, a spinner or "1.4s"); footer cost + "5 calls in the last minute" |
| Legend | centred under the panel, 11 px grey with 12 px samples: Call · Answer · Needs you · Failed |

## 3. Behaviour
- Demo: a 24 s pass that starts over. The lead splits the release into four jobs; the Researcher and the Analyst finish
  (checks), the Coder's sandbox check fails (red, "Coder failed") and retries, the Writer ends up asking you (amber,
  "Writer needs you"); Stripe slides in while the Coder needs it and leaves again. Calls per minute and costs follow.
- A new call sends a pulse from caller to callee; when it is answered a green pulse runs back, a failure a red one. Pulses
  keep their place on re-render (negative animation delay from the call's time).
- Hover or focus on a node lights its neighbourhood: its links and linked nodes stay, everything else fades to 30 % and
  turns grey.
- Click / Enter / Space opens or closes the node's card (`on_select(id)`, empty on close); Esc closes it and returns
  focus to the node. A chip opens its node's card.
- First draw: the lead pops in, then the agents, then the outer nodes; links draw themselves.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live pass |
| card | The lead's card while it works |
| needs-you | Settled: Writer waiting (its card open), Coder failed, two agents done |
| narrow | 380 px: taller than wide, smaller nodes, short labels, the card docked to the bottom |
| classic | Blue links that thicken with traffic, the bar on a grey band, legend "Call · Answer · Busier link · Needs you" |

## 5. Keyboard and accessibility
Tab moves between the chips and the nodes; a focused node lights its neighbourhood; Enter / Space opens or closes its card;
Esc closes and returns focus. Every node is a button named "Coder, agent, failed: The sandbox check timed out, 3 calls in
the last minute" with aria-expanded and aria-haspopup="dialog"; the card is a dialog whose close button takes focus. A
visually hidden list mirrors the graph as text (each node, its status, whom it called how often in the last minute and
its cost). The chips sit in a polite live region; status is never colour alone (badge + text line).

## 6. Responsive
The layout is computed for a wide (756 × 454) and a narrow (360 × 432) box, x as a share of the width, so it fits the
container. Below 520 px the narrow one shows: taller than wide, nodes at 80 %, short labels, no model line, the card
docked to the bottom of the graph, "calls/min" dropped from the bar. Nothing scrolls or is cut off down to 320 px.

## 7. Known gaps
- Nodes do not glide on a spring when nodes come and go (the demo keeps a place for the tool that comes and goes).
- Links that share a side are not spread 8 px apart; they overlap.
- GPUI: a static snapshot; links drawn as straight runs with a square corner, no pulses.
