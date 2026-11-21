**Status:** built v0.2 (2026-11-16, plan #32 affectUI fidelity pass, family B part 8 – exact pass against the ref-1 capture notes).

# Spec: Assign board (`delegate-board`)

Family: Agents. Closes the parity row: affectUI · Delegate Board (Pro). Behaviour-only rebuild; layout, sizes, colours,
states, motion and keyboard follow the ref-1 capture notes (approximate); agents, tasks, rates and copy are our own.
Related Oxyn component: background-jobs.
Components: `DelegateBoard` (Dioxus) · `DelegateBoardEl` (GPUI), model in `oxyn_core::kit::delegatefid`.

## 1. Purpose
Hand tasks to agents yourself – or let Auto do it: a pile of unassigned work beside each agent's model, rate,
busy-until time and queue, with the money waiting in queues.

## 2. Parts
| Part | Look (Clean, light / dark) |
|---|---|
| Root | one card up to 720 px: white / #17171A, 1 px #E6E6E6 / #2A2A2E, 14 px radius, soft shadow, 16 × 18 px padding |
| Bar | blue sparkle, "Delegate" 13/600, a muted hint that changes with the mode; right: "Auto" 12 px + a 32 × 18 switch (grey off, blue #2563EB / #4C8DFF on, white knob) |
| Pile | 170 px, #F5F5F5 / #101012 panel, 12 px radius: "Unassigned" 12 muted + count badge; task cards 44 px min, 10 px radius, white, title 12/500 (two lines max) over a tool mark + "~15 min" (11 muted); empty: "Nothing left to hand out" |
| Agents | "Agents" 12 muted and "$1.96 in queues" (11, tabular) right-aligned; rows: 22 px round avatar (vetted brand mark), name 12.5/600 (+ "n queued" in Auto), "model · $0.08/min" 11 muted, status pill right ("Busy until 10:33" grey #F1F1F1, "Free" green #16A34A on #ECFDF5, a blue "Until 10:56" preview while a task hovers over or is aimed at the agent) |
| Queue | chips 22 px, 999 px radius, 11 px: the running one blue-tinted (#EFF6FF / rgba(76,141,255,.15)) with a pulsing dot, queued ones white with a hairline; past three a "+N" chip with a tooltip listing the rest |
| Footer | 11 px muted hint with 16 px kbd chips |

## 3. Behaviour
- `db_load_min` = minutes left on the current job + every queued estimate; busy-until = start clock + load
  (`db_clock`); `db_cost` = Σ load × rate. `db_free_first` picks the agent with the least load; `db_auto_plan` hands the
  pile out one task at a time to whoever is free first at that moment.
- Manual: drag a card onto an agent (the agent lights blue and previews its new time; the card tilts and fades while
  dragged), or Enter on a card to pick it up, ↑ ↓ to choose, Enter (or a digit) to hand over. A waiting chip can be
  clicked to put it back in the pile (`on_assign(task, None)`).
- Auto: switching on hands out one task a second until the pile is empty; switching off keeps what was handed out.
- `suggestion` lights an agent with its preview time and marks the task in the pile.
- prefers-reduced-motion: no lifts, pulses or fades.

## 4. Variants
manual (default) · suggest · queued (+N) · auto · done · narrow (380 px) · classic.

## 5. Keyboard and accessibility
Enter picks up / hands over, ↑ ↓ choose the agent, 1 – 9 hand over at once, Escape puts it back. Cards are toggle
buttons (aria-pressed); each agent is a named group whose digit button carries aria-keyshortcuts; Auto is a role=switch;
hand-overs are announced in a polite status region; the +N chip is focusable.

## 6. Responsive
Fills up to 720 px; under 560 px the pile sits above the agents and the header hint drops out.

## 7. Known gaps
- Pointer (touch) drag is not built – HTML drag and drop plus the keyboard path only.
- The clock does not run: busy-until times are counted from a fixed start.
- GPUI: a static snapshot – no drag, preview or tooltip.
