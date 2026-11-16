**Status:** built v0.2 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8 – exact pass against the ref-1 capture notes).

# Spec: Decision Tree (`decision-tree`)

Family: Agents. Closes the parity row: affectUI · Decision Tree (Pro). Behaviour-only rebuild; nothing is copied (the
decisions, agents, numbers, notes and every line of demo copy are our own). Related Oxyn components: plan-approval,
decision-card, approval questions.
Components: `DecisionTree` (Dioxus) · `DecisionTreeEl` (GPUI), model in `oxyn_core::parity::dtreefid`.

## 1. Purpose
An agent at a fork it should not take alone shows each way forward as a branch – what it changes, which consequence
worries it, which one it would take – and waits for a person to pick a path or write a different instruction.

## 2. Parts
| Part | Look (Clean, light) |
|---|---|
| Root | 820 px (max 100 %), container-sized decision card: white / #17171A, 1 px #E6E6E6 / #2A2A2E, 14 px radius, soft shadow, 20 × 22 px padding |
| Bar | blue tree glyph, "Decisions" 13/600, "n waiting for you" 12 muted; right: 5 px page dots (current #111827, rest #D4D4D8) and "1 of n" (12, tabular) |
| Question | 190 px card, 12 px radius, #E6E6E6 hairline, 16 px padding; risk pill (10 px uppercase, 999 px radius) over the top-left corner (high #FEE2E2/#DC2626, medium #FEF3C7/#B45309; dark: translucent tints of #F87171 / #FBBF24); round 24 px agent avatar + agent 12/600 and model 11 muted; question 14/600; context 11 muted |
| Branch | 150 × 40 px button, 10 px radius: 18 px number chip, 14 px tool icon (brand marks coloured), label 12/600, "Agent's pick" 11/600 #2563EB / #4C8DFF under it; picked: blue border on #EFF6FF |
| Lines | 1.5 px #D4D4D8 elbows from the card to a rail and on to each branch; the lit / picked path #2563EB (dark #4C8DFF) with a 3 px soft-blue ring on its card; dashed stubs to the consequences |
| Consequences | 11 px muted lines; flagged runs are amber (#B45309 on #FEF3C7) with an amber underline and that open an "Agent's note" card (note 13 px, source line with a warning glyph) |
| Follow-up | sub-question as an 11 px muted label and its own 230 × 44 px option cards (10 px radius, 12/600 label, 11 px consequence) with consequences and app logos; dimmed to 42 % and disabled until its branch is picked |
| Other… | the last branch; opens a two-line textarea (grows to three) with a darkening border; a sent instruction shows as a ringed card with "Sent to the …" |
| Outcome | "✓ Approved · running" chip at the end of the picked path |
| Footer | "Choose a path 1 – 4 · next →" / "Then 1 – 2", 11 px, grey kbd chips 16 px (10 px text, 4 px radius), plus "· note F" when there are flags |
| All decided | "n of n decided" in the bar; centre: 40 px green-tinted rounded-square check badge, "All decided" 14/600, a 12 px muted line |

## 3. Behaviour
- `DtState::pick` takes the branch, then (when it has a follow-up) the follow-up option; a branch without one settles
  at once. A settled decision fades after 0.8 s and leaves the queue after 1.6 s; the next one grows in.
- Hovering a branch turns its route blue and fades the others to 40 %; a pick removes the rest.
- Other… sends on Enter (Shift+Enter: new line); `on_other` gets the text; the agent's short answer follows.
- `auto` (demo): 7.2 s per decision – settle, point at the pick, pick it, answer its follow-up with the agent's pick,
  fade, next – until the first touch, then the tree is yours.
- With reduced motion requested, no animation or transition runs.

## 4. Variants
auto (default) · settled · hover · picked · outcome · other · sent · flagged · no-other · single · narrow · classic
(bordered panel, grey header band, risk tag inside the card, consequences boxed, flatter cards).

## 5. Keyboard and accessibility
Digits choose at the active level and one past the last opens Other…; → ← or J K switch decisions; F steps through
the amber flags; Escape closes Other… or a note. Branches are buttons named after their option (the preferred one says
so), with aria-keyshortcuts; flags take focus and aria-describedby points at a role=tooltip; every pick, instruction and
reply is spoken through a polite status region; connector lines are pure CSS.

## 6. Responsive
A container query: wide, question → branches → consequences left to right. Under 520 px top to bottom: branches hang
down the left below the question, consequences tucked under every branch; the "waiting" count steps aside.

## 7. Known gaps
- The ref-1 capture gives approximate sizes (screenshot reading); dark colours follow its hexes (#0B0B0C page, #111113 stage, #17171A card, #2A2A2E borders, #EDEDEF / #8A8A93 text).
- Connector elbows are square (affectUI rounds them ~8 px) and do not draw themselves out on grow-in.
- The agent's answer to an instruction is a fixed stand-in line (no async `onChoose` reply yet).
- GPUI: a static snapshot – no lines, notes, hover or grow-in.
