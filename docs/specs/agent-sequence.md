**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 4).

# Spec: Agent Sequence (`agent-sequence`)

Family: Agents. Closes the parity row: affectUI · Agent Sequence (Pro). Behaviour-only rebuild; nothing is copied (the
demo run's task, messages, branch, PR number and times are our own).
Components: `AgentSequence` (Dioxus) · `AgentSequenceEl` (GPUI), model in `oxyn_core::parity::seqfid`.

## 1. Purpose
One agent run as a sequence diagram: a column for you, each agent and each tool, every message an arrow from one
lifeline to the next. Shows who called whom, in what order, what ran in parallel and what had to be retried.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 800 px (max 100 %), container-sized, straight on a soft #FAFAFA canvas (no border or shadow), 20 px padding, 14 px radius, clipped |
| Header | the run's name 13.5/600; right: "N working" (blue) while running or "N failure(s)" (red), "N messages", elapsed m:ss – muted, tabular |
| Chips | one per participant, centred on its lifeline: white, hairline, 7 px radius, 26 px tall, a 3 × 3 pixel mark (full in ink for a person, a pattern in the agent's colour) or a tool's logo, then the name |
| Gutter | 40 px of muted 11 px times, one per message, on the arrow's line |
| Lifelines | a dashed light-grey line down each column; the diagram grows with the run |
| Message | a 1 px line in the sender's colour, solid for a call and dashed for a reply, fading in from the sender; a self-message is a small bracket back to its own lifeline |
| Tag | the label, centred over its arrow (beside a self-message's bracket), 19 px, 5 px radius, tinted in the sender's colour (grey for a person); a failure is red with a cross where it lands |
| Bar | 5 px rounded capsule on the receiver's lifeline from a call until its reply, the participant's colour at a third; open bars shimmer while running |
| Frame | consecutive messages with the same tag ("par", "retry") share a white box, 10 px radius, hairline, the tag in 10 px caps at its top left |
| Cards | a click on a chip: kind, Working / Idle, model, what it does, and Sent · Received · Tasks · Busy. A click on a tag: sender → receiver, the label, Call / Reply / Note, its frame, Failed, what went along (mono) and "Sent 0:05 · Answered in 1s". Lower rows open their card upwards |

## 3. Behaviour
- The demo plays a 15-message run, a message every 0.75 s; each new arrow draws itself from the sender while its tag
  fades in, and the receiver's bar grows. After the last message it holds, then loops (16 s); a new pass re-mounts.
- With your own `messages` everything shows; `running` keeps open bars shimmering and the working count in the header.
- `at` freezes the demo; `open` / `open_participant` open a card at the start; `on_select(Some(id) | None)`.
- Escape closes the open card. Another tag dims the rest while a message card is open.
- prefers-reduced-motion: no drawing, shimmer or twinkle.

## 4. Variants
live (default) · settled · message-card · participant-card · narrow (420 px) · classic (a white card with a border and
shadow, pill chips and tags, tinted frames).

## 5. Keyboard and accessibility
Messages are an ordered list ("Messages"); each tag is a button named "Sender to Receiver: text, status, m:ss, opens a
dialog". Participants are buttons with `aria-expanded`; cards are dialogs with a close button reachable by Tab. A
polite live region says the newest message. Colour never carries meaning alone: replies are dashed, failures say so and
carry a cross, frames are labelled.

## 6. Responsive
Under 560 px the chips show only their marks, the gutter narrows to 36 px and times shrink to 10 px; tags may take 40 %
of the width before they ellipsize.

## 7. Known gaps
- No light travelling at the tip of a drawing arrow and no landing ripple (the line is revealed with a clip).
- Tags are not measured: a long tag over a short arrow can overlap a neighbour column's tag.
- Message tags use a roving tabindex without arrow-key movement yet.
- GPUI: a static snapshot – no drawing, shimmer or cards; chips and tags route `part-<n>` / `msg-<n>`.
