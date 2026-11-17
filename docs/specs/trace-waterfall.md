**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-3 capture notes).

# Spec: Trace Waterfall (`trace-waterfall`)

Family: Agents. Closes the parity row: affectUI · Trace Waterfall (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-3 capture notes (approximate); the trace, names and copy are our
own. Related Oxyn components: trace-view.
Components: `TraceWaterfall` (Dioxus) · `TraceWaterfallEl` (GPUI), model in `oxyn_core::parity::tracefid`.

## 1. Purpose
Show where an agent run spent its time and money: every agent, model call and tool call as a bar on one time axis,
nested under whoever called it, live while the run goes and inspectable after.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | card 600 px: white / #17171A, 1 px hairline, 12 px radius, soft shadow |
| Header | violet spark, "Trace" 13/600, the task muted (ellipsis); right: "Fold finished" (once a branch ends) and a pill – blue "Live" with a pulsing dot, green "Done" |
| Stats | six figures in a row – time taken, spend, tokens, then the counts of model calls, tool calls and errors (uppercase 9.5 px muted labels, 13.5/600 values; errors red when > 0; a light shimmer while live) |
| Head | "SPAN" and the axis ticks (0s / 15s / …) between hairlines |
| Row | 26 px: chevron (branches; turns when folded), kind glyph (tool calls with a known service show its mark in its official colours), name 12/600, muted model / tool, "+n" when folded, a warning icon when failed; the lane holds an 8 px bar (agents slate, model calls violet, tool calls teal, failed red) and "duration · cost"; the slowest finished call has an amber SLOWEST chip |
| Marks | faint gridlines per tick; a glowing blue now line while live; a folded branch draws a tick on its bar for each hidden call |
| Details | 300 px window under the row: name, kind · model/tool, status pill, ✕; DURATION / STARTED / COST (+ TOKENS IN / OUT for model calls); Input and Output (Error, red-tinted, when failed) |

## 3. Behaviour
- At time t a span is hidden before its start, running until its end (drawn to t), then done or failed. Totals are
  read at t (cost and tokens pro rata for running calls). The axis is the smallest round range (5, 10, 15, 30, 45,
  60, 90, 120, 180 s …) that holds the elapsed time; ticks re-space and bars rescale with a glide.
- Clicking a row (or Enter / Space on the active row) opens its details; clicking again or Escape closes. The chevron
  (or Right / Left) folds a branch; "Fold finished" folds every ended branch at once.
- Live demo: `now` drives a looping replay; rows slide in, running bars grow, lit by a moving sheen and a bright leading edge.
  prefers-reduced-motion turns off sheen, shimmer, slides and glides.

## 4. Variants
live · early · mid · folded · done · details · failed · narrow · classic. Controls: style, state (7), width.
Extras beyond the reference: "Fold finished" in the header; Home / End in the tree; known tool services show their
mark in its official colours.

## 5. Keyboard and accessibility
The tree is a single tab stop (role tree, aria-activedescendant); Up / Down / Home / End move, Right / Left unfold /
fold, Enter / Space open details, Escape closes. Rows are treeitems with aria-level, aria-expanded and aria-selected.
The pill is a status region; failure is said in words as well as red.

## 6. Responsive
Under 560 px the totals wrap to three columns, the axis hides and each row puts its bar under the name; under 360 px the
muted details and labels drop.

## 7. Known gaps
- Rows leaving on fold disappear at once (no bottom-first close-up); bars do glide when the axis changes.
- The details window opens under the row, not from the bar's exact position.
- GPUI: a static snapshot – no sheen, details window or keyboard tree.
