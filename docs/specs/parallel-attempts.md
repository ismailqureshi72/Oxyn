**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 9 – against the ref-1 capture notes).

# Spec: Attempt race (`parallel-attempts`)

Family: Agents. Closes the parity row: affectUI · Parallel Attempts (Pro). Behaviour-only rebuild; layout, sizes,
colours, states, motion and keyboard follow the ref-1 capture notes (approximate); models, approaches, scores and copy
are our own. Related Oxyn components: fanout-grid, decision-tree.
Components: `ParallelAttempts` (Dioxus) · `ParallelAttemptsEl` (GPUI), model in `oxyn_core::kit::parallelfid`.

## 1. Purpose
One task tried several ways at once. Every attempt races as a row; when they finish, the highest score is marked
Best and used, and you can switch to another attempt.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | one card up to 720 px: white / #17171A, 1 px #E5E7EB / #2A2A2E, 16 px radius, 16 px padding |
| Bar | blue stack mark, "Best of N" 13/600, the task 12 muted (ellipsis); right: "n of N done" 12 muted + total cost 12/600 tabular |
| Row | 48 px min, 12 px radius, hairline; 20 px model tile (vetted brand mark) · name 13/600 + " · approach" muted, over the step (blue while running) or the outcome note (muted) · 6 px bar · score block · use button |
| Bar fill | blue with a shine and a soft glow while running; grey when done; green #16A34A / #34D399 on the row in use; red #DC2626 / #F87171 where a failed attempt stopped |
| Score block | while running "n%" + cost; done: bold score, "Best" pill (#DCFCE7 / green 16 %) before it on the best one; "Failed" in red; below, took · cost 11 muted |
| Use button | 26 px, 8 px radius, "Use this one"; on the row in use a green tick + "In use", no frame |
| In-use row | soft green wash and a green edge; other finished rows step back to 80 % opacity until hovered or focused; failed rows sit at 58 % |

## 3. Behaviour
- `pa_at(scripts, ms)` plays the scripted race: progress, step and cost per attempt at that moment; `pa_len` is the
  race length. `pa_best` = highest score among finished attempts, `pa_finished` and `pa_cost` feed the bar.
- The best attempt is in use unless `picked` (controlled) or a click on Use says otherwise; `on_pick(id)` fires.
- Live demo: rows enter staggered, bars grow with the shine, then scores land and the race loops after a rest; a pick
  from an earlier loop does not carry over.
- Step text gives way to the outcome note in place; row height stays fixed.
- prefers-reduced-motion: no stagger, shine or glides.

## 4. Variants
live (loops) · start · running (one already failed) · done · own pick · best of 5 · narrow (380 px) · classic.
Controls: style, state (5), attempts (3 / 5), width.

## 5. Keyboard and accessibility
Tab walks the Use buttons of attempts that have finished; Enter or Space takes the focused one. The rows are a radiogroup labelled by the
heading; each Use control is a radio named with the model and its score (aria-checked on the one in use). Bars are
decorative: progress, failure and score are all in text.

## 6. Responsive
Container query: under 560 px the bar drops under the name, the approach and the task hide, the score sits beside the
name and Use moves to its own line.

## 7. Known gaps
- Percent and cost step with the clock rather than a number-ticker roll.
- The ref's "Model logos" example is covered by the brand tiles only; there is no logo-less example.
- GPUI: a static snapshot – no shine, stagger or hover step-forward.
