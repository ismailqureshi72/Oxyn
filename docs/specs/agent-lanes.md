**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 3).

# Spec: Agent Lanes (`agent-lanes`)

Family: Agents. Closes the parity row: affectUI · Agent Lanes (Pro). Behaviour-only rebuild; nothing is copied (the
demo agents, tasks, summaries and the decision's wording are our own).
Components: `AgentLanes` (Dioxus) · `AgentLanesEl` (GPUI), model in `oxyn_core::parity::lanesfid`.

## 1. Purpose
Where every agent is right now: a live timeline with one lane per agent over the last half hour. Tasks grow into
"now", hand-offs are drawn between agents and a decision is made right on the waiting block.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 760 px (max 100 %), container-sized; 12 px radius, hairline, soft shadow |
| Bar | 48 px: blue spark, title 14/600, "● 2 running" grey 12.5, "● 1 needs you" amber (pulsing dot), "Last 30m" chip 24 px grey, 7 px radius |
| Ruler | 30 px: "−25m … −5m" faint 11.5, tabular, every 5 minutes; a blue "now" chip at the right |
| Names column | 175 px: 28 px logo tile (coloured maker mark, 8 px radius), name 14/500 with an "Idle" grey or "Needs you" amber tag, the model 12.5 grey |
| Lanes | 50 px each, dashed grid lines at the ticks |
| Block | 28 px, 7 px radius, up to three 16 px tool logos then the title 13/500, ellipsis: Done grey, Running a blue gradient with a 3 px blue cap at "now" and a soft shimmer, Failed red, Needs you an amber gradient with an outlined "▲ Needs you" chip; a block that started before the window fades in from the left edge |
| Hand-off | a dashed blue curve from the end of one block to the start of the next agent's; it flows while the next block runs |
| Summary | dark tooltip under a clicked block: "Agent · State · 10m", two or three points, optional Open link |
| Decision | a 280 px card under a waiting block: title, the ask, Reject / Approve (Approve filled) |

## 3. Behaviour
- Demo: a 24 s pass that starts over on a clock running a minute per second. The window slides, running blocks grow,
  the Writer's draft finishes and hands over to the Reviewer (a curve and a new running block), the interviews finish,
  and old blocks fade out at the left edge.
- Click a block for its summary; click a waiting block for its decision. Approve turns it into a running block, Reject
  into a failed one (`on_approve(id)`, `on_reject(id)`; with your own scene you change it). Esc closes.
- The counts in the bar and the tags follow the tasks; how many need someone is announced.
- prefers-reduced-motion: no shimmer, pulse or flow.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live pass |
| summary | Settled, a finished block's summary open |
| decision | Settled, the waiting block's decision open |
| narrow | 380 px: the names column shrinks to the logos |
| classic | Outlined blocks, round tiles, clearer grid, a filled bar, 16 px radius |

## 5. Keyboard and accessibility
Tab goes block to block, lane by lane; Enter opens the summary or the decision; Esc closes. Each lane is a list labelled
with the agent; each block is a button naming agent, task, state and time ("Researcher: Market scan – Done, took 10m"),
with aria-expanded. The decision is a labelled dialog and Approve takes focus when you open it. State is never colour
alone (labels in the name, the tag and the chip).

## 6. Responsive
Fills its container up to 760 px. Below 520 px the names column shrinks to the logos, the running count and every other
tick label drop. Blocks shrink with ellipsis; nothing scrolls.

## 7. Known gaps
- Hand-off curves use one cubic between block ends rather than affectUI's exact routing.
- GPUI: a static snapshot; the hand-off is a dashed elbow, no shimmer or flow.
