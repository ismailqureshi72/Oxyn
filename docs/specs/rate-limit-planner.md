**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Load planner (`rate-limit-planner`)

Family: Agents. Closes the parity row: affectUI · Rate Limit Planner (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-2 capture notes (approximate); the runs, numbers and copy are our
own. Related Oxyn components: background-jobs.
Components: `RateLimitPlanner` (Dioxus) · `RateLimitPlannerEl` (GPUI), model in `oxyn_core::kit::ratefid`.

## 1. Purpose
Plan when agent runs start so the day's token load stays under an API's per-minute limit, and see at a glance where
it would not.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | card 560 px: white / #171717, 1 px hairline, 12 px radius, soft shadow |
| Header | title 13.5/600, muted "n runs · tokens · limit / min"; right: status pill (green under, amber close, red over with the time range), Undo after a move, "Spread out" outline button |
| Chart | 110 px: the day's load (06:00 → 06:00) as a violet line over a violet-to-clear area; dashed red limit line with its value at the left, a dotted safety line (buffer); the part above the limit hatched red; a now line with its label; a dark peak chip "Peak 115k · 16:35" (red when over) |
| Axis | 3-hourly clock ticks |
| Row | 28 px: two-bar glyph in the run's tone, name 12/600 (muted when under way), start time (or "Running"; red for the run causing an overrun), then the bar from start to fade-out in its tone with its peak at the right; remove ✕ and ⋯ appear on hover / selection |
| Edit window | 236 px popover: Name, Starts (time), Peak k/min, Climbs for, Halves every, black Done |
| Add run | muted text button with a plus |
| Footer | hairline, insight text, black "Move to hh:mm" pill when a fix exists |

## 3. Behaviour
- A run climbs linearly to its peak over its climb time, then halves every half-life; the load is the sum plus a base.
  Under < 90 % of the limit, close 90–100 %, over above it.
- Spread out: biggest movable run first, each tries every 5-minute start from now to an hour before the chart ends
  and keeps the lowest peak (nearest to where it was on a tie). Runs under way never move. Undo restores.
- The run to fix is the selected one if it adds to the first overrun, else the run adding most at it; its nearest
  fitting start is offered as "Move to". Drag a bar (pointer, 5-minute snap, never before now), or arrow keys on a
  focused bar. ✕ removes; ⋯ edits; Add run appends a default run at now and opens its edit window.
- Motion: bars glide to new starts (spring-ish, 380 ms) except while dragged; the pill and peak chip cross-fade;
  the edit window scales in. Reduced motion: none of it.

## 4. Variants
close · spread · over · selected · edit · narrow · classic. Controls: style, state (5), width.
Extras beyond the reference: Undo after Spread out / Move to; Shift+arrow 30-minute steps; the buffer line is a prop.

## 5. Keyboard and accessibility
Tab through rows, bars and buttons; bars are sliders (value text = start time; a started run is aria-disabled);
arrows move by 5 min (Shift 30); Escape closes the edit window, then clears the selection. The pill is a status
region; the chart is aria-hidden; the footer states the overrun in words.

## 6. Responsive
Rows keep 28 px. Under 520 px the name column narrows to 96 px, glyphs and the now label hide, the axis keeps every
other tick and the footer wraps.

## 7. Known gaps
- The load curve does not morph between states (it redraws); the bars do glide.
- No measured-load overlay or "above plan" insight (the reference's actual-vs-plan examples).
- GPUI: a static snapshot – columns for the load, no drag, hatching or edit window.
