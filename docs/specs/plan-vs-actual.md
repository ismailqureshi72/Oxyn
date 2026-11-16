**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-2 capture notes).

# Spec: Plan vs Actual (`plan-vs-actual`)

Family: Agents. Closes the parity row: affectUI · Plan vs Actual (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-2 capture notes (approximate); the steps, times and copy are our
own. Related Oxyn components: plan-todo, trace-view.
Components: `PlanVsActual` (Dioxus) · `PlanVsActualEl` (GPUI), model in `oxyn_core::parity::planfid`.

## 1. Purpose
Lay an agent's run against its plan: each step's real bar over its dashed planned bar, so overruns, knock-on delays
and the drift of the expected finish are visible at a glance.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | card up to 720 px: white / #171717, 1 px hairline, 12 px radius, soft shadow, 16 px padding |
| Header | violet tool mark, title 13/600, the task 12 muted; right: "Time actual / plan", "Cost actual / plan" (actual bold), and a status pill (amber "+n behind", green "n ahead", grey "on plan") |
| Axis | "STEP" uppercase 10 px muted, then m:ss ticks with faint gridlines down the chart |
| Row | 28 px: status glyph (violet check / spinning arc / hollow circle), title 12 px (muted while waiting), then the lane: a dashed planned ghost (10 px, 5 px radius, light violet) under the solid actual bar (violet; the part past the plan amber), elapsed m:ss and a delta pill (+n amber / −n green) |
| Marks | a violet line (dot on top) at the current time; dashed planned-finish line; an amber (or green) band between the planned finish and the ETA |
| Footer | legend Plan / Actual / Over plan; right: "Plan m:ss" grey pill and "ETA m:ss" pill (amber when late, green when early) |

## 3. Behaviour
- A step starts when everything it depends on has ended; a running step that passes its plan keeps growing; waiting
  steps are placed at their expected start, so an overrun pushes everything after it right; the ETA is the latest
  expected end. Cost is summed per step.
- Selecting a step (click, Enter, Space) tints its row, rings its bar, joins it with curves to its prerequisites and to
  the steps that wait for it, and shows its cost; Escape clears.
- Live demo: `now` advances; running bars grow with a sheen, waiting steps glide to their new start, the ETA moves;
  the run loops.
- prefers-reduced-motion: no sheen, spin or glides.

## 4. Variants
live · early · mid · selected · done (behind) · ahead · narrow · classic.
Controls: style, state (6), width.

## 5. Keyboard and accessibility
Tab through steps; each is a toggle button whose name says took / planned or the expected start; Enter / Space selects;
Escape clears. The header status is a polite status region; chart marks are aria-hidden; overruns are in text, never
colour alone.

## 6. Responsive
Rows keep 28 px at every width. Under 560 px the name column narrows to 96 px and the task, glyphs and axis labels hide.

## 7. Known gaps
- The axis does not glide wider as the run passes it; it is sized to the larger of plan and ETA up front.
- GPUI: a static snapshot – no selection links or sheen.
