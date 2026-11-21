**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-1 capture notes).

# Spec: Model knockout (`judge-bracket`)

Family: Agents. Closes the parity row: affectUI · Judge Bracket (Pro). Behaviour-only rebuild; layout, sizes, colours,
states, motion and keyboard follow the ref-1 capture notes (approximate); contenders, answers, scores and copy are our
own. Related Oxyn components: parallel-attempts, decision-tree.
Components: `JudgeBracket` (Dioxus) · `JudgeBracketEl` (GPUI), model in `oxyn_core::kit::judgefid`.

## 1. Purpose
Pick the best of many answers by pairwise judging: a judge model compares two answers at a time in a single-elimination
bracket, and the champion's route through the rounds is the result.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | one card up to 720 px: white / #17171A, 1 px hairline #E5E7EB / #2A2A2E, 14 px radius, soft shadow |
| Bar | gavel mark, title 13/600, the task 12 muted (ellipsis); right: blue "Judging" pill (#EFF6FF / blue 15 %) or "<name> wins", then "Round · n/N" 11 muted tabular |
| Round heads | uppercase 10 px, letter-spaced, muted, one per column |
| Match | a button holding two 30 px slots (8 px radius): brand mark, name 12 px, score right (tabular); the winner keeps full colour and a hairline, the loser fades and its outline turns dashed; undecided slots are dashed and empty; elbows join each pair into the next round |
| Winner path | once the final is decided, the champion's slots and elbows turn blue, one round at a time, with a cup beside the final score |
| Answers panel | under the bracket, #FAFAFA / #131315, 12 px radius: match name + "Judged by …" / "… is comparing…" / "Not judged yet"; two answer columns (name, blue Winner tag, score, meta line, answer, one mono code line) and the judge's reason; a Use button on the decided final when `on_pick` is set |

## 3. Behaviour
- `jb_rounds`, `jb_matches`, `jb_slots` and `jb_winner` derive the whole bracket from the seed order and the verdicts
  (keyed by round and match); `jb_current` is the first undecided match whose two slots are known, `jb_champion` the
  final's winner, `jb_on_path` marks the champion's route, `jb_progress` the "Round · n/N" text.
- Brackets of two, four, eight or sixteen; seeds pair with their neighbour in the first round.
- Clicking a match opens its answers; the match being judged is open by default and shimmers.
- Live demo: verdicts land one at a time, the preferred contender moves on, the other fades but stays; at the end the
  path draws in blue, the cup appears and the demo loops.
- prefers-reduced-motion: no shimmer or draw; everything simply appears.

## 4. Variants
live · judging · match open · not judged · decided (with Use) · four contenders · narrow · classic.
Controls: style, state (5), contenders (8 / 4), width.

## 5. Keyboard and accessibility
Tab match by match, then the answers and Use; → / ← to the match a match feeds and back; ↑ / ↓ within a round; Enter
or Space opens and closes a match; Escape returns to the match being judged. Each match is a button named with both
contenders and its result; the winner/loser difference is carried by colour, score and dashed outline together;
progress is a polite status region.

## 6. Responsive
Rounds share the width with fixed slot heights. Under 560 px each round becomes its own list, a match per row, and the
two answers stack.

## 7. Known gaps
- The answers panel does not animate its height when another match opens.
- 16 contenders are supported by the model and the view, but the gallery demo shows 8 and 4 only.
- GPUI: a static snapshot – click to open a match, no shimmer, no path draw.
