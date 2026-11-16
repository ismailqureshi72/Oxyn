**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 6).

# Spec: Cost Waterfall (`cost-waterfall`)

Family: Agents. Closes the parity row: affectUI · Cost Waterfall (Pro). Behaviour-only rebuild; nothing is copied (the
run, its steps, models' numbers and copy in the demo are our own). Related Oxyn components: trace-view and run-meter.
Components: `CostWaterfall` (Dioxus) · `CostWaterfallEl` (GPUI), model in `oxyn_core::parity::costfid`.

## 1. Purpose
A run's spend, step by step: model calls, tools and sub-agents stack across to the total, each with its model, tokens
in and out, price and duration; the priciest step is flagged and one toggle swaps money for tokens.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 756 px (max 100 %), container-sized, white, hairline, 12 px radius (continuous where supported) |
| Bar | 48 px: blue sparkle, "Run cost" 13/600, the task muted; "Measure by" Cost \| Tokens (grey track, white pressed pill); a Done (green) or Live (blue, pulsing dot) pill |
| Stats | three cells split by hairlines: TOTAL COST, TOTAL TOKENS (+ "in · out"), MOST EXPENSIVE (a button: name + its cost); 9.5 px uppercase labels, 16/600 values (a shine while live) |
| Head | STEP · axis ticks (nice 1·2·5 steps, max a little past the total) · IN · OUT · COST · TIME |
| Row | 32 px: chevron (sub-agents), an 18 px icon tile (brand mark, sparkle for sub-agents, a globe for tools), name, model muted, "+4" when folded, a TOP tag; a 14 px bar that starts where the previous step ended (LLM violet with a darker cap, tools teal, sub-agents grey, failed red); right-aligned numbers, cost 600, a failed time red with a warning glyph |
| Details | under the row: two 10 px-radius boxes – Prompt / Response (Task / Result, Input / Output; Error for a failure) |

## 3. Behaviour
- `cost_rows` walks the tree: a sub-agent's steps start where it starts; folded ones hide theirs (`+n`). Cost and
  tokens are each step's own; sub-agents add their children up; missing numbers show "–"; a negative cost is a credit.
- Cost \| Tokens re-bases the axis and every bar (CSS transitions glide them); Most expensive unfolds its sub-agents,
  opens the step and focuses its row; a click or Enter / Space opens a row's details; chevrons fold.
- `live` (demo): steps arrive `step_ms` apart; the newest runs (sheen + glow) until the next, and Live turns Done.
- prefers-reduced-motion: everything appears at once.

## 4. Variants
live (default) · settled · tokens · details · failed-step · folded · medium (640 px) · narrow (380 px) · classic (grey
header band, sentence-case stats, saturated bars, a stronger frame).

## 5. Keyboard and accessibility
Tab: the switch, Most expensive, then the rows as one roving stop; ↑ ↓ Home End move; → opens / ← folds a sub-agent;
Enter / Space opens details; Escape closes them. The rows are a tree (treeitem, aria-level, aria-expanded, a full
label); the switch is two toggle buttons with aria-pressed; the run's state is a status.

## 6. Responsive
Wide: steps, waterfall, numbers. Under 720 px In and Out step aside; under 520 px each row stacks – name, cost, time,
then the bar across the width.

## 7. Known gaps
- ArrowRight on an open sub-agent does not jump to its first step, nor ArrowLeft to the parent.
- The axis does not glide to a new round value; bars re-base instantly when the total changes while live.
- GPUI: a static snapshot – no details, chevrons or live sheen.
