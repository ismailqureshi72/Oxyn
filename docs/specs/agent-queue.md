**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 4).

# Spec: Agent Queue (`agent-queue`)

Family: Agents. Closes the parity row: affectUI · Agent Queue (Pro). Behaviour-only rebuild; nothing is copied (the
demo's jobs, workers' names, queue name and numbers are our own; the simulation is ours).
Components: `AgentQueue` (Dioxus) · `AgentQueueEl` (GPUI), model in `oxyn_core::parity::queuefid`.

## 1. Purpose
The work waiting for your agents and how fast it moves: a live queue, workers running jobs as growing bars, Done and
Failed, the four numbers, and a depth chart that shows the backlog build up when a rate limit pauses the workers and
drain when they resume.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 756 px (max 100 %), container-sized, 12 px radius, hairline, soft shadow; four bands split by hairlines |
| Header | 52 px: blue list glyph, "Queue" 13/600, the name muted, a pill – Live (blue, pulsing dot), Rate limited · 9s (amber, counting down), Paused (grey) –; right Pause / Resume (28 px outlined) and a − 4 + stepper (1–8) |
| Numbers | four columns split by hairlines: WAITING / RUNNING / THROUGHPUT / WAIT P95 (10 px caps, tracked) over 15 px values with muted units ("4/4", "62/min", "13s"); Waiting turns amber above the backpressure line |
| Queue | 196 px: "QUEUE 12", the first three jobs as 30 px pills (kind icon or tool logo, name, waited time; ↑ high priority, ↻ 2 retry), the rest as a grid of 9 px squares tinted by kind |
| Workers | rows 40 px on a soft grey, 8 px radius: 24 px model tile, name 12.5/600, then the job as a pill with a progress bar tinted by kind (violet LLM, teal tool, slate sub-agent, blue human) with a glowing front edge and the run time in the kind's colour; "Paused" when stopped; "Idle" when free |
| Totals | 92 px: Done (green check tile) and Failed (red × tile, red number) cards |
| Chart | "QUEUE DEPTH", 0 / 12 / top labels, a violet line with a fading fill, a dashed backpressure line, amber stretches for rate limits, a dot at now; "96s ago" … "now" |
| Job card | 340 px over the columns: tile, name, kind · detail, Waiting / Running chip, close; a grey 2 × 2 grid WAITING / POSITION / PRIORITY / ATTEMPT; Cancel and Move to front |

## 3. Behaviour
- Demo: a deterministic simulation on a 40 s pass with 96 s of history: about 1.3 jobs a second, high-priority jobs go
  to the front, a few fail once and come back with a retry badge; 10 s into each pass a 10 s rate limit stops the
  workers, the queue climbs past the line and drains as callers back off.
- Pause / Resume (`on_paused_change`), the stepper (`on_concurrency_change`, takes effect from that moment), a job's
  card with Move to front (`on_promote`) and Cancel (`on_cancel`) – all change the simulation from that moment on.
- Esc closes the card. prefers-reduced-motion: no pulse or glide.

## 4. Variants
default (live) · limited · paused · one-worker · card · narrow (420 px, stacked) · classic (mixed-case labels, inset
panels).

## 5. Keyboard and accessibility
Every job is a button named with its kind, waited or run time, place in line, priority and attempt; the front job is the
one tab stop. Pause has aria-pressed; the stepper's buttons say what they do and the count is announced. The chart is an
image with a text summary (waiting now, peak, average, the line, the rate-limited stretches); rate limits, pauses and
crossing the line are announced politely.

## 6. Responsive
Under 600 px the flow stacks: queue, workers (marks only), Done and Failed side by side; the numbers wrap to two
columns. Under 440 px Pause and the stepper take their own row.

## 7. Known gaps
- Jobs do not glide from the queue onto a worker or fly into Done (they appear in place).
- Arrow-key roving between jobs is not built (Tab reaches the front job; the others are clickable).
- GPUI: a static snapshot; the chart is drawn as bars.
