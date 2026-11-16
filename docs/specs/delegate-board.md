**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8).

# Spec: Delegate Board (`delegate-board`)

Family: Agents. Closes the parity row: affectUI · Delegate Board (Pro). Behaviour-only rebuild from a single
cross-reference line in the affectUI notes (no capture); agents, tasks and copy are our own. Related Oxyn component:
background-jobs. Components: `DelegateBoard` (Dioxus) · `DelegateBoardEl` (GPUI), model in
`oxyn_core::parity::delegatefid`.

## 1. Purpose
Give a task to the agent you choose: unassigned work in a tray, each agent with its skills and free slots, the right
ones lit when you pick a task, and the job queuing, running and finishing in that agent's column.

## 2. Parts
| Part | Look (Clean, light) |
|---|---|
| Root | 780 px (max 100 %), container-sized, white, #E6E6E9 hairline, 12 px radius |
| Bar | 48 px: blue board glyph, "Delegate" 13/600, "n to hand out · m with agents" muted |
| Tray | 210 px column on #FAFAFA: UNASSIGNED caption, task cards (title 13/600, skill chip, ~minutes); a picked card gets a blue border and 3 px ring |
| Agent column | auto-fit ≥150 px cards: 26 px brand tile, name / model, busy/capacity; skill chips; while a task is picked a Hand over button (digit chip, "Best fit" on the least busy agent with the skill) or a disabled "Can't do …"; columns without the skill fade to 50 % |
| Job | 12 px row: status dot (grey queued, blue pulsing running, green done), title with ellipsis, status word, × to take back a queued job |
| Footer | key hint |

## 3. Behaviour
- `db_fits` (skill match), `db_load` (open jobs), `db_best` (most free slots, first on a tie) and `db_statuses`
  (jobs start when handed over or when a slot frees, `capacity` at a time, est × `ms_per_s`).
- Hand over by clicking the agent's button, pressing its digit, or dragging the card onto the column.
- A queued job can be taken back (it returns to the tray). Hand-overs announce in a polite status region.

## 4. Variants
default · selected · busy · full (best fit moves) · done · narrow (380 px) · classic.

## 5. Keyboard and accessibility
Tray cards are toggle buttons; digits 1–n hand the selected card over; Escape clears. Columns are named groups; the
Hand over button carries aria-keyshortcuts; drag and drop is never the only way.

## 6. Responsive
Under 620 px the tray goes on top and the agent columns stack.

## 7. Known gaps
- Needs capture: affectUI's layout, sizes and motion are unknown – this follows the one-line description and the
  family's Clean / Classic language.
- No drop-target highlight while dragging; no reordering inside a column.
- GPUI: a static snapshot – no drag and drop.
