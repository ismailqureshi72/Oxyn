**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 3).

# Spec: Agent Handoff (`agent-handoff`)

Family: Agents. Closes the parity row: affectUI · Agent Handoff (Pro). Behaviour-only rebuild; nothing is copied (the
demo agents' descriptions, tasks and items are our own).
Components: `AgentHandoff` (Dioxus) · `AgentHandoffEl` (GPUI), model in `oxyn_core::parity::handofffid`.

## 1. Purpose
A task passed from agent to agent like a relay: at every handoff, what went along to the next agent and what got lost
on the way. People can push a task on to a later agent and watch it travel down the relay.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 788 px (max 100 %), container-sized; a grid of 2 × agents half columns, 16 px gaps |
| Agent card | spans two half columns; 14 px radius, hairline, soft shadow, 12 px padding: 28 px logo tile (coloured maker mark), name 13.5/600 over the model 11.5 grey, cost 12 grey, status pill 20 px (Idle grey, Working blue with a spinner); description 13 grey, two lines; task chip 32 px grey, 8 px radius: "Up next" + logo + title 600 + "+2" on the first agent, or logo + title + "· Drafting" while working, or "Ready for a task" faint |
| Task window | the chip opens a dialog under the card: logo, title, "Up next" or what it does, close; "Waiting" and the other queued tasks |
| Wires | 60 px: 1.5 px light-blue lines from each agent's middle down, one rounded turn, into its column 11 units left or right of the middle; 7 px hollow dots at both ends |
| Column | centred under the gap between two agents (one agent wide), 236 px tall, fading out over the last 64 px: header 28 px "logo Researcher → logo Writer" grey 12 + a count chip; then the handoff cards, latest on top, 6 px apart |
| Handoff card | 10 px radius, hairline, soft shadow: logo + task 13.5/600; what went along 12.5 grey, one line; the status: spinner "With Writer" / "Handing on", green dot "Looks good", amber triangle "2 claims unchecked" |
| Open card | grows in place (no overlay): every item as a 26 px grey row with its icon (file, link, fact, check, note); a lost item as a red row, struck through with a ⊘; open, a dark note with why and "Add back" |

## 3. Behaviour
- Demo: a 22 s pass that starts over: the Researcher picks up the next task ("Reading sources"), hands it to the Writer
  ("Drafting") and on to the Reviewer ("Checking claims"), who ends with "2 claims unchecked". Costs tick up while
  agents work.
- At a handoff a short light with a glow runs down the line into the column, the card drops in on top, and the light
  runs on up to the next agent. Handoffs present on mount do not run.
- Click a handoff card to open or close it; click a lost item for why and "Add back" (`on_restore(handoff, item)`).
- Drag a card (or double-click it, then click an agent) onto an agent to the right of its column: that agent lifts and
  rings in blue, the ones it has passed step back; the drop gives it the task (`on_assign(task, agent)`). Cards only move
  on down the relay. Esc drops a picked-up card, then closes what is open.
- prefers-reduced-motion: everything appears at once.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live pass |
| handing-on | Frozen just after a handoff ("With Writer", Writer drafting) |
| card-open | A card open with its items, the lost one struck through |
| lost-item | The lost item open: why + Add back |
| stacked | 380 px: agent, handoff, agent in one column |
| classic | Visible borders, crisper corners, the column header in a filled band |

## 5. Keyboard and accessibility
Tab moves through the task chips, the cards and the lost items; Enter / Space opens a card or a task; on a lost item it
shows why and Add back. Each column and each card's items are labelled lists ("Handed from Researcher to Writer"); cards
and lost items are buttons with aria-expanded. Status is never colour alone.

## 6. Responsive
Below about 760 px the status pill shows as an icon only; below 620 px the relay stacks into one column – an agent, its
handoff column (with a blue rail), the next agent. Cards keep their size; columns fade out instead of scrolling.

## 7. Known gaps
- A dragged card does not follow the pointer with a custom ghost (the browser's drag image is used).
- GPUI: a static snapshot; wires are square-cornered, no light, no dragging.
