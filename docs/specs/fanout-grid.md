**Status:** draft v0.1 (2026-11-11).

# Spec: fan-out grid and parallel lanes (`fanout-grid`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 14 Pro = 18 variants**. Reads the shared `RunGraph` from
[subagent-tree.md](subagent-tree.md) §9.

## 1. Purpose
Two related moments:
1. **Parallel attempts** — the user runs the same task N times (N agents, N models, or N git worktrees)
   and wants to **watch them side by side, compare the outputs, and pick a winner**. Coding agents do this
   with a worktree per task (Codex app, Conductor, Verdent, Paradev, Emdash, Cursor Agents Window, Copilot
   — research note 03; agentbox "each agent gets its own git worktree" [R027]).
2. **Fan-out / map-reduce** — one agent hands a big job to many workers (100 files, 40 URLs) and the user
   wants **coverage at a glance: how many done, which failed, what is left**.

Plus a **lanes inbox**: when many lanes run, the user only cares about the ones that need them.

What we saw: affectUI Agent Lanes — a lane per agent, tasks grow into "now", hand-offs drawn between
agents, approvals sit on the block; affectUI Fan-out Grid — every task a cell that fills as workers
complete, retries, failures, cost and time left, details on hover; affectUI also lists Parallel Attempts
and Judge Bracket (names only) (research note 11, all Pro, study only). Flo Hoeller's lanes post: header
"2 running · 1 need you", lanes with model names, a lane marked "Idle", a red "Needs you" block
[R061]; his inbox: columns NEEDS YOU / RUNNING / DONE with counts, live clocks, msgs/tools counts, cards
glide between columns, Board↔List toggle [R062]. Vespin: an attention inbox with an unread dot per agent
that needs you [R022]. Claude Code agent view: one row per background session, needs-input / working /
done, last response, time, peek, attach (research note 05). fal.ai shows two lanes, your messages vs the
agent's queued work [R084]. Hoplite advertises "Dozens of agents, one orchestrator" (research note 10).

Anatomy, `columns` (parallel attempts, 3 lanes):
```
┌ 3 attempts · 2 running · 1 needs you ─────────── Compare ▾ · Layout ▾ · Stop all ┐
│ ┌ A · Opus ── worktree a1 ┐ ┌ B · Sonnet ── worktree b2 ┐ ┌ C · GPT ── worktree c3 ┐ │
│ │ ● Editing auth.rs  1:12 │ │ ⚠ Needs approval    ⏸0:40 │ │ ✓ Done 2m 04s          │ │  lane header
│ │ +48 −12 · tests 11/14   │ │ +6 −2 · tests —           │ │ +31 −9 · tests 14/14 ✓ │ │  scorecard row
│ │ ─ live tail ─────────── │ │ [Review]                  │ │ ─ final summary ─────  │ │  lane body (transcript /
│ │ …                       │ │                           │ │ …                      │ │   terminal / diff tail)
│ │ [Focus]                 │ │ [Focus]                   │ │ [Pick ★] [Diff vs A]   │ │  lane actions
│ └─────────────────────────┘ └───────────────────────────┘ └────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────┘
```
Anatomy, `cells` (fan-out of 120 tasks): a grid of 12 px cells (16 px touch), one per task, coloured
by state; header "84 / 120 done · 3 failed · 2 retrying · ~ time left only if supplied".

Rules (proposed):
1. **Same lane header shape everywhere**: identity swatch + label, model, workspace chip, state dot +
   verb + elapsed, scorecard (diff stat, tests, tokens, cost if prices).
2. **Scorecards only show app-supplied or measured facts** (diff stat from git, tests from the runner,
   tokens from usage events). We never compute a "best" score; a judge result appears only when the app
   or a judge agent supplies it.
3. **Pick winner is a commit with consequences**: it shows exactly what happens to the other lanes
   (keep, archive, delete worktrees) and the delete path needs explicit confirmation.
4. **Lanes > visible width** scroll horizontally with a lane minimap; above **16 lanes** the layout
   suggests `cells` (proposed), above 200 tasks cells shrink to 8 px and group by 10.
5. **Each lane is its own busy region**; max 8 animate (nearest viewport centre win).

## 2. State machine
Grid-level:
```
                      dispatch N                      first lane settles                 all settled
  Empty ─────────────────────────▶ Dispatching ───────────────────────▶ Running{k/N} ──────────────────▶ Comparing
  ("No attempts")         (lanes appear staggered,        │  ▲                                          │
                           each Pending → Starting)       │  └ retry lane / add attempt                 │ pick lane X
                                                          │                                             ▼
                                         stop all ─────▶ Stopping ──▶ Stopped           Picking{X} (confirm: keep / archive / delete others)
                                                                                                        │ confirm
                                                                                                        ▼
                                                                          Resolved{winner} (winner expands, others collapse)
                                                                                                        │ undo (≤ 8 s, if no delete)
                                                                                                        ▼
                                                                                                    Comparing
  Banner layer: None ⇄ Offline ⇄ None · None ⇄ BudgetReached (all lanes paused) ⇄ None
```
Per lane: the `subagent-tree` node machine (Pending → Starting → Working ⇄ AwaitingYou / Stalled → Done /
Failed / Interrupted) plus lane-only states `Winner`, `Discarded`, `Archived`, `Merging` (app applying the
winner), `MergeFailed`.

Per cell (fan-out): `Queued` → `Running` → `Done` | `Failed{attempt}` → `Retrying{attempt}` → … |
`Skipped` | `Cancelled`.

Inbox view: each lane card lives in exactly one column, derived from its state:
`NeedsYou` (AwaitingYou, Failed, MergeFailed) · `Running` (Pending…Working, Stalled) · `Done` (Done,
Winner, Interrupted) · optional `Archived`.

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Empty` | hidden, or "No attempts yet" + "Run in parallel" (app action) | app's dispatch | — | — |
| `Dispatching` | lanes slide in with skeleton bodies, header counts roll up | — | stop all | Esc stop all (confirm) |
| `Running{k/N}` | header "3 attempts · 2 running · 1 needs you"; live lanes | focus lane | stop lane, stop all | ←/→ move lane focus, Enter focus, `s` stop lane |
| Lane `Working` | dot + verb + elapsed, live tail (last 6 lines), scorecard updating | focus | stop | Enter |
| Lane `AwaitingYou` | ⚠ amber header, lane body shows the request summary | **Review** | focus | Enter review |
| Lane `Stalled` | warning ramp, "No response for 45 s" | retry lane | stop | Enter |
| Lane `Done` | ✓, final elapsed, scorecard frozen, summary | **Pick ★** (when ≥ 2 settled or N = 1) | diff vs…, focus | `p` pick, `d` diff |
| Lane `Failed` | ✕ header, error line | retry lane | discard | Enter retry |
| `Comparing` | compare bar: choose two lanes → split diff / output diff; scorecard table | open diff | swap sides | `c` compare, Tab cycles lanes |
| `Picking{X}` | confirm sheet: "Use B. Other attempts: Keep / Archive / Delete worktrees (2)" (default **Archive**) | confirm | cancel | Enter confirm, Esc cancel |
| `Resolved{winner}` | winner lane expands; others collapse to 40 px stubs "Archived · A" | open winner / continue | undo (8 s) | `u` undo |
| Lane `Merging` | winner header "Applying to main…" indeterminate | — | cancel (if app supports) | — |
| Lane `MergeFailed` | ✕ "Couldn't apply · 3 conflicts" | open conflicts (family H) | retry | Enter |
| `Stopped` | all lanes interrupted; header "Stopped after 1m 12s" | resume (if app supports) | — | — |
| Cell `Running` | cell breathes (only 8 cells animate; rest still accent) | hover details | — | arrow keys move cell focus |
| Cell `Failed{n}` | danger cell with ✕ glyph at ≥ 12 px; badge "attempt 2" | retry task | open log | Enter retry |
| Inbox card | name, live clock, msgs/tools counts, last line; expands to recent steps + Open | open | stop / review | ↑/↓ within column, ←/→ columns |
| Banner `BudgetReached` | "Budget reached · 3 lanes paused" | extend | stop all | Enter |

## 4. Micro-interactions
- **Dispatch.** Lanes slide in from the dispatch origin (the composer or the parent row) left → right,
  y +8 → 0, 40 ms stagger (cap 6); bodies start as skeletons ([skeletons.md](skeletons.md)) sharing one
  viewport-pinned sweep.
- **Live tail.** New lines push up inside a 6-line window with a 12 px top fade mask; no scroll jank
  (fixed line height).
- **Scorecard numbers.** Diff stat and test counts roll (odometer, 280 ms); tests fill a tick row one tick
  per test [R037].
- **Needs you.** Lane header turns amber with the static waiting glyph; the grid header count rolls; in
  the inbox the card glides to NEEDS YOU (FLIP move, 260 ms `ease.in-out`) [R062].
- **Lane finishes first.** A single 400 ms outline pulse in success tone on the lane frame (no confetti);
  the header shows "1st" rank chip only if the app asks for finish order.
- **Compare.** Pick lane A, then B: the two lanes slide together (others dim to 50 %) and the body morphs
  into a split diff; Esc slides them back.
- **Pick winner.** Pressing Pick ★ lifts the lane (scale 1.02, shadow) and opens the confirm sheet; on
  confirm the winner widens to full width with `spring.snap` while the others collapse to stubs and fade
  to 40 %, 40 ms stagger outward from the winner. Undo reverses within 8 s (not available after delete).
- **Cells.** A cell fills from centre (scale 0.6 → 1) when its task completes; failures flash danger then
  hold; retries draw a ring around the cell; the header counter rolls. Hover shows a tooltip card (task
  label, worker, elapsed, tokens, error line).
- **Hand-offs** (Pro `swimlanes`): a curved connector draws from one lane's block to another's over
  300 ms when work passes between agents.
- **Hover / focus.** Lane frame gets the focus ring; header reveals stop and focus; the lane minimap
  highlights the hovered lane.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Lane enter | 220 ms, y +8 → 0 | — | `ease.out` | 40 ms, cap 6 | 60 Hz |
| Skeleton sweep | `loop.sweep` 2.0 s | — | linear | shared | 30 Hz |
| Live tail line push | 120 ms | — | `ease.out` | — | 60 Hz |
| Scorecard digit roll | 280 ms | — | `ease.out` | 30 ms right → left | 30 Hz |
| Test tick fill | 120 ms | — | `ease.out` | 60 ms | 30 Hz |
| Inbox card glide (FLIP) | 260 ms | — | `ease.in-out` | 40 ms when several | 60 Hz |
| First-finish pulse | 400 ms × 1 | — | `ease.out` | — | 60 Hz |
| Compare slide together | 240 ms | — | `ease.in-out` | — | 60 Hz |
| Pick lift | 120 ms | — | `spring.snap` | — | 60 Hz |
| Winner widen / losers collapse | 260 ms / 200 ms | — | `spring.snap` | 40 ms outward | 60 Hz |
| Undo window | 8 s | — | — | — | — |
| Cell fill | 160 ms, scale 0.6 → 1 | — | `spring.snap` | 10 ms within a batch, cap 20 | 60 Hz |
| Cell failure flash | 200 ms | — | `ease.out` | — | 60 Hz |
| Hand-off connector draw | 300 ms | — | `ease.out` | — | 60 Hz |
| Header count roll | 280 ms | — | `ease.out` | — | 30 Hz |
| **Reduced motion** | lanes and cards appear in place with 80 ms crossfade; inbox cards change column without gliding (column count text updates); no lift, widen or collapse animation (layout switches on the next frame with 80 ms fade); cells change colour instantly; no pulses; digits swap | — | linear | none | 1 Hz for clocks |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `fanout-grid/columns` | Baseline: N lanes side by side with header, scorecard, live tail, Pick ★ and Compare | desktop, ide |
| `fanout-grid/cells` | Map-reduce grid: one cell per task, header totals, hover card | desktop, chat (card), mobile |
| `fanout-grid/inbox-board` | Lanes as cards in NEEDS YOU / RUNNING / DONE columns, with a Board ↔ List toggle [R062] | desktop, mobile (list) |
| `fanout-grid/terminal-split` | Panes split in a TUI, one per lane, header line `A ● 1:12 +48 −12`; Pick by number key | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `fanout-grid/swimlanes` | Horizontal lane per agent on a time axis; blocks grow into a live "now" edge; hand-off connectors [R061] |
| `fanout-grid/judge-bracket` | Tournament bracket when a judge agent compares attempts pairwise (shown only with judge data) |
| `fanout-grid/scoreboard` | Table of attempts × metrics (tests, diff size, time, tokens, cost) with sortable columns |
| `fanout-grid/stacked-cards` | Attempts as a fanned card stack; swipe through on touch, pick with a flick up |
| `fanout-grid/diff-tabs` | One shared diff viewer with tabs per attempt; changed hunks highlight what differs between attempts |
| `fanout-grid/preview-wall` | Live web previews per lane (app builders); click to enlarge into compare-split |
| `fanout-grid/media-contact-sheet` | Generated images/videos per attempt as a contact sheet with star picking (media agents) |
| `fanout-grid/heatmap-cells` | Cells coloured by duration or tokens instead of state; outliers pop |
| `fanout-grid/sankey-reduce` | Workers flow into the reducer; band width = items processed |
| `fanout-grid/ring` | Circular fan-out: workers around a hub, segments fill on completion |
| `fanout-grid/attention-dots` | Minimal list with one unread dot per lane that needs you [R022] |
| `fanout-grid/two-lane-queue` | Two lanes: your queued messages vs the agent's queued work [R084] |
| `fanout-grid/mission-control` | Dashboard block: counts, budget, inbox, and the swimlanes together (Multi-agent ops shell) |
| `fanout-grid/ascii-matrix` | Terminal cells as `▪▫✕` glyph matrix with a progress line |

## 7. Edge cases
- **N = 1.** Renders as a single lane without Compare; Pick hidden.
- **N large (parallel attempts > 16).** Horizontal scroll + minimap; auto-suggest `cells`; Compare limited
  to two lanes at a time.
- **Fan-out > 1 000 tasks.** Cells group into blocks of 100 with a fill level; drilling in shows the block.
  Only visible cells repaint; the header counters are the truth.
- **Unequal lanes.** Lanes have equal width by default; a focused lane can expand to 2× while others
  shrink to a minimum of 220 px.
- **Pick while some lanes still run.** Confirm sheet adds "Stop 2 running attempts"; default stops them.
- **Delete worktrees.** Destructive: the sheet names the worktrees/branches and requires a second press
  (or typing the count for > 5); undo is not offered after delete.
- **Winner apply fails.** Lane goes `MergeFailed`; other lanes are restored from Archived if they were
  archived in the same action (if the app supports it).
- **Identical outputs.** Compare shows "No differences" with the shared diff stat; both stay pickable.
- **No scorecard data.** Scorecard row hidden (never "0 tests").
- **Long labels.** Lane label 20 ch, model 16 ch, workspace chip truncates middle.
- **RTL.** Lane order mirrors (first attempt on the right); diff stats, numbers and code stay LTR.
- **Tiny sizes.** Under 480 px wide, `columns` becomes a vertical list of lane headers (accordion); cells
  stay at ≥ 8 px with a 2 px gap.
- **Dense IDE panels.** Headers only (28 px), bodies hidden until focused.
- **Touch.** Lanes swipe horizontally with snap; Pick requires press-and-hold 400 ms or the sheet.
- **Slow networks.** Lanes keep last tail; an offline banner pauses clocks' liveness styling, not the
  clocks themselves (elapsed-timer.md §2).
- **Very fast lanes.** A lane finishing within `show_delay` (150 ms) appears directly as Done.

## 8. Accessibility
- `columns` is a `group` labelled "3 parallel attempts"; each lane is a `region` labelled "Attempt B,
  Sonnet, needs approval". Cells grid uses `grid` with `gridcell`s ("Task 37, failed, attempt 2"); arrow
  keys move by cell, Page Up/Down by row.
- **Live region (polite):** attempt finished ("Attempt C done in 2 minutes 4 seconds, 14 of 14 tests
  passing"), attempt needs you (**assertive**), fan-out milestones at 25 / 50 / 75 / 100 % and on each new
  failure batch (max one per 5 s), winner chosen, undo available.
- **Contrast:** state cells use tone + glyph at ≥ 12 px (✓ ✕ ↻); below 12 px, a pattern (hatched for
  failed, dotted for retrying) in addition to colour; text ≥ 4.5:1, swatches ≥ 3:1.
- **Focus order:** header (Compare, Layout, Stop all) → lanes left → right (each: header, body, actions)
  → confirm sheet traps focus while open and returns it to the Pick button.
- **Keyboard:** `1–9` focus lane n (when the grid has focus), `p` pick, `c` compare, `u` undo, Esc closes
  compare/sheet. All have visible hints in tooltips.
- **Reduced motion:** §5 last row; inbox column changes are also announced.

## 9. Props sketch
```rust
pub struct FanoutGrid {
    pub graph: RunGraphRef,                  // subagent-tree.md §9; lanes = children of `parent`
    pub parent: AgentId,
    pub mode: FanoutMode,                    // Attempts | MapReduce
    pub variant: FanoutVariant,              // Columns | Cells | InboxBoard | TerminalSplit | Pro(..)
    pub lanes: Vec<LaneInfo>,                // per child: workspace, scorecard
    pub tasks: Option<Vec<TaskCell>>,        // MapReduce: { id, label, worker, state, attempt, elapsed }
    pub scorecard: ScorecardCfg,             // DiffStat | Tests | Tokens | Cost(needs PriceTable) | Judge
    pub price: Option<PriceTable>,           // app-supplied; None → no cost
    pub eta: Option<Duration>,               // only if the app supplies it
    pub pick: Option<PickCfg>,               // None hides Pick; { default_others: Archive, allow_delete, undo: 8 s }
    pub on_pick: Callback<(AgentId, OthersPolicy)>,
    pub on_compare: Callback<(AgentId, AgentId)>,
    pub on_retry: Option<Callback<LaneOrTask>>,
    pub on_stop: Option<Callback<Option<AgentId>>>, // None = stop all
    pub max_animating: u8,                   // 8
    pub suggest_cells_above: u16,            // 16 lanes
    pub inbox_columns: Vec<InboxColumn>,     // NeedsYou | Running | Done | Archived
    pub labels: FanoutLabels,
    pub motion: MotionLevel,
}

pub struct LaneInfo {
    pub agent: AgentId,
    pub workspace: Option<WorkspaceRef>,     // worktree / branch
    pub diff: Option<DiffStat>,              // +48 −12
    pub tests: Option<TestTally>,            // passed, failed, total
    pub lane_state: LaneState,               // Live(AgentState) | Winner | Discarded | Archived | Merging | MergeFailed
}
```
Lane states map from each child's `AgentState` exactly as in [subagent-tree.md](subagent-tree.md) §9;
`Winner/Discarded/Archived/Merging/MergeFailed` are app-driven.

## 10. Related components and references
Related: [subagent-tree.md](subagent-tree.md) (same nodes as a tree) · [tool-call.md](tool-call.md) and
[terminal-output.md](terminal-output.md) (lane bodies) · [file-edit-chip.md](file-edit-chip.md) (diff
stats) · [run-meter.md](run-meter.md) (per-lane usage) · [trace-view.md](trace-view.md) (swimlanes
share the time axis) · [background-jobs.md](background-jobs.md) (lanes left running in background) ·
[composer-status-strip.md](composer-status-strip.md) Pro `lanes-peek` · [skeletons.md](skeletons.md) ·
diff / merge / worktree chips (family H) and approvals (family G) in
[../component-taxonomy.md](../component-taxonomy.md).

| Ref | What we take | Licence note |
|---|---|---|
| affectUI Agent Lanes, Fan-out Grid, Parallel Attempts, Judge Bracket (note 11) | study reference: lanes growing into "now", hand-offs, approval on block; task cells with retries, cost, time left | Paid (Pro, code gated) — study only, nothing copied |
| Agent lanes [R061] | "2 running · 1 need you" header; idle lane; red needs-you block | Study only |
| Agent inbox [R062] | NEEDS YOU / RUNNING / DONE columns; cards glide; Board↔List | Study only |
| Vespin [R022] | unread dot per agent that needs you | Study only |
| agentbox [R027] | worktree per agent; running count pill | Study only |
| Claude Code agent view (note 05) | row per background session; peek and attach | Proprietary — reference only |
| Worktree ADEs (note 03: Codex app, Conductor, Verdent, Paradev, Emdash, Cursor, Copilot) | worktree-per-attempt is the norm | Mixed / unchecked — study only |
| Test runner [R037] | tick row per test | Study only |
| fal.ai queue [R084] | two lanes: user vs agent queue | Check source; study only |
| Hoplite (note 10) | orchestrator over dozens of agents | Proprietary; study only |
