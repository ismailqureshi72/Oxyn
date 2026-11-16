**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8).

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
| Root | 820 px (max 100 %), container-sized; no panel – the bar and tree sit on the page |
| Bar | blue tree glyph 18 px, "Decisions" 15/600, "n waiting for you" 13 muted; right: 5 px page dots (current #18181B, rest #D4D4D8) and "1 of n" |
| Question | 205 px card, 12 px radius, #E6E6E9 hairline, 16 px padding; risk tag floating over the top-left corner (high #FEE4E2/#B42318, medium #FEF0C7/#B54708); 24 px agent tile (7 px radius) + agent 13/600 and model 11.5 muted; question 16/650 at 1.4; context 13 muted |
| Branch | 183 × 36 px button, 10 px radius: 18 px number chip (#F1F1F3, 5 px radius), 14 px tool icon (brand marks coloured), label 14/600, "Agent's pick" 12/600 #2F6FED under it |
| Lines | 1.5 px #D8DCE3 elbows from the card to a rail and on to each branch; the lit / picked path #4C7DF0 with a 3 px soft-blue ring on its card; dashed stubs to the consequences |
| Consequences | 12.5 px muted lines; flagged runs are amber pills that open an "Agent's note" card (note 13 px, source line with a warning glyph) |
| Follow-up | question pill (8 px radius, 13.5/600) and its own 283 px option cards with consequences and app logos; dimmed to 42 % and disabled until its branch is picked |
| Other… | the last branch; opens a two-line textarea (grows to three) with a darkening border; a sent instruction shows as a ringed card with "Sent to the …" |
| Outcome | "✓ Approved · running" chip at the end of the picked path |
| Footer | "Pick a branch 1 – 4 · next →" / "Then 1 – 2", kbd chips 18 px, plus "· note F" when there are flags |

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
A container query: wide, question → branches → consequences left to right. Under 640 px top to bottom: branches hang
down the left below the question, consequences tucked under every branch; the "waiting" count steps aside.

## 7. Known gaps
- Needs capture: the affectUI dark and classic screenshots were described in notes only; colours for dark are ours.
- Connector elbows are square (affectUI rounds them ~8 px) and do not draw themselves out on grow-in.
- The agent's answer to an instruction is a fixed stand-in line (no async `onChoose` reply yet).
- GPUI: a static snapshot – no lines, notes, hover or grow-in.
