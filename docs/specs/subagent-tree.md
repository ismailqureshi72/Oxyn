**Status:** draft v0.1 (2026-11-11).

# Spec: sub-agent tree (`subagent-tree`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
family overview in [tools.md](tools.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **5 free + 14 Pro = 19 variants**. This file
also defines the shared **`RunGraph`** model (§9) that [fanout-grid.md](fanout-grid.md),
[acting-indicator.md](acting-indicator.md), [trace-view.md](trace-view.md), [run-replay.md](run-replay.md)
and [run-meter.md](run-meter.md) read. If [tools.md](tools.md) defines a family-wide type with the same
job, that one wins and this section becomes an alias.

## 1. Purpose
A parent agent delegates: it spawns a researcher, a test-runner, a reviewer, and they may spawn their own
helpers. The user wants to know **who is working on what, who is stuck, who needs me, and what came back**
without opening every child. The tree answers that in one glance and lets the user **focus** one child
(read its transcript, steer it, stop it) and come back.

What we saw: Claude Code shows a subagent panel below the prompt and gives each subagent one of eight
colours (red, blue, green, yellow, purple, orange, pink, cyan), and `/tasks` keeps finished subagents
listed as done (research notes 05, 08); Claude Code subagent status lines refresh about every 30 s from
short summaries [R024]; Paradev wraps a subagent as a collapsible tool row and lists running subagents in
a top-bar activity panel; Amp gives a named expert subagent ("Oracle") its own identity; Factory's
subagents inherit autonomy; Codex routes cross-thread approvals with "o to open thread" (all research
note 03); Hoplite keeps a durable Subagents panel that separates active work from recent results, with
child states running / needs approval / completed / failed / interrupted, and a blocked child exposing
the same review path (research note 10); Antigravity shows a conversation as working while its
subagents run (research note 08); AG-UI carries `SUBAGENT_STARTED / FINISHED / ERROR` events (research
note 05); signoff-ui nests sub-agents inside its tool-call timeline [R107].

Anatomy (panel layout, 32 px rows desktop, 28 px IDE):
```
┌ Agents · 3 running · 1 needs you ─────────────────────────────── ⌄ collapse all ┐
│ ● Orchestrator      Planning the migration                 2m 14s   ▾           │  depth 0 (parent)
│ ├─● Researcher      Reading 14 files                       0:48     ▸ 1         │  depth 1, collapsed, 1 child
│ ├─⚠ Test runner     Needs approval: run `cargo test`       ⏸ 0:31   Review      │  depth 1, awaiting you
│ │   └─✓ Fixture gen  Returned · 3 fixtures                 12s                  │  depth 2, done + result line
│ └─✓ Reviewer        Returned · 2 findings  ↗               1m 05s               │  result bubbled up
│   + 4 nested agents (depth limit 3)                                             │  truncated row
└─────────────────────────────────────────────────────────────────────────────────┘
  dot · name (colour swatch) · verb or result · elapsed · expand / action
```

Rules (proposed):
1. **Every node row is the same shape**: state dot, colour swatch + name, verb (live) or one-line result
   (finished), elapsed, trailing slot (child count, action, result arrow).
2. **Roll-up**: a collapsed node shows the most urgent state among its descendants as a badge (priority
   `AwaitingYou` > `Failed` > `Stalled` > `Working` > `Done`). A parent never looks "done" while a child
   still runs.
3. **Parent stall only when all are silent**: a parent waiting on children is `Waiting on 3 agents`, not
   stalled (same rule as [composer-status-strip.md](composer-status-strip.md) §6).
4. **Each row is its own busy region**; at most **8 rows animate**, the rest show the still "working" dot
   ([loaders.md](loaders.md) §2.3).
5. **Display depth ≠ runtime depth**: the tree shows depth 0–2 expanded by default and folds deeper levels
   into a "+n nested agents" row; the runtime depth limit belongs to the app and is only *reported*.
6. Colour is identity, never state: state uses dot shape + tone + text; the 8 identity colours never
   collide with warning/danger tones because a swatch never carries state.

## 2. State machine
Per node (driven by that node's `AgentState` in the `RunGraph`):
```
                spawn event                start event            first token / tool
  (absent) ───────────────▶ Pending ──────────────────▶ Starting ─────────────────▶ Working{verb}
                     (queued by parent,      (≤ 500 ms "submitted"        │  ▲   │ tool starts/ends: verb swaps
                      hollow dot)             pulse, no spinner)           │  │   ▼
                                                                          │  └── Streaming / ToolRunning
                      AwaitingApproval / question                         ▼
        AwaitingYou{request} (timer paused, ⚠) ◀──────────────────── Working ──── no event for stall_soft ──▶ Stalled
                  │ answered / approved / denied ─────────────────────▶ Working ◀── event ─────────────────────┘
                  │
  Working ── children spawned ──▶ WaitingOnChildren{n} ── all children settle ──▶ Working
  Working ── finish ──▶ Returning (result chip forms, 240 ms) ──▶ Done{result, elapsed}
  Working ── error ──▶ Failed{kind}          Working ── stop (user or parent) ──▶ Interrupted{after}
  any live ── parent ended first ──▶ Orphaned (still running, detached badge) ── settles ──▶ Done/Failed
  Pending ── runtime depth limit hit ──▶ Refused{DepthLimit(n)} (row explains; no spawn)
  Done ── app parks session ──▶ Hibernating (R053) ── wake ──▶ Starting
  any ── transport lost ──▶ Offline overlay (last known state, 60 % opacity) ── reconnect ──▶ previous

Tree-level (independent of node states):
  Empty ("No sub-agents yet") ⇄ Populated{Collapsed | Expanded} ── focus child ──▶ Focused{id}
  Focused{id} ── Esc / breadcrumb / child removed ──▶ Populated
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| Tree `Empty` | nothing (panel hidden) or "No sub-agents yet" in a pinned panel | — | — | — |
| `Pending` | hollow dot ○ (muted), name, "Queued" | — | cancel spawn (if app allows) | Delete cancel |
| `Starting` | dot pulses once (submitted pulse), verb "Starting" | focus | stop | Enter focus |
| `Working{verb}` | ● breathing in accent, verb ("Reading 14 files"), elapsed ticking | focus child (opens its transcript) | stop child, steer (in focus view) | Enter focus, Esc (in row) nothing, `s` stop with confirm |
| `ToolRunning` | tool glyph in dot slot, verb from tool ("Running cargo test") | focus | stop | as Working |
| `WaitingOnChildren{n}` | ● still (not breathing), "Waiting on 3 agents", elapsed continues | expand | stop all | → expand, ← collapse |
| `AwaitingYou{request}` | ⚠ amber, "Needs approval: run `cargo test`", ⏸ timer | **Review** (opens the approval card in place or in focus view) | open thread | Enter review, proposed `⌘/Ctrl+Shift+A` jumps to the oldest pending |
| `Stalled` (soft / hard) | tone ramps to warning; hard: "No response for 45 s" | retry child (if adapter supports) | stop | Enter retry |
| `Returning` | result chip forms on the row and travels to the parent (§4) | — | — | — |
| `Done{result}` | ✓, one-line result "Returned · 2 findings", final elapsed | open result | focus transcript, copy result | Enter open result |
| `Failed{kind}` | ✕ danger, "Failed · tool error" | retry (if supported) | details, dismiss | Enter retry |
| `Interrupted{after}` | ■ muted, "Interrupted after 31s" | — | details | — |
| `Orphaned` | live state + ⇢ "detached" badge (parent ended) | focus | stop | — |
| `Refused{DepthLimit}` | ⊘ "Not started · depth limit 3" (muted, no motion) | — | learn more (app link) | — |
| `Hibernating` | ☾ "Hibernating" (R053) | wake (if app supports) | — | Enter wake |
| Offline overlay | last known row at 60 % + "offline" in tooltip | — | — | — |
| Truncated row | "+ 4 nested agents" or "+ 12 more" | expand inline (or focus parent) | — | Enter expand |
| Tree `Focused{id}` | breadcrumb `Orchestrator › Test runner`, child transcript, tree shrinks to a rail with the focused row highlighted | back | next/prev sibling | Esc / Alt+← back, Alt+↑/↓ siblings |

Tree keyboard (ARIA tree pattern): ↑/↓ move, → expand / move to first child, ← collapse / move to
parent, Home/End, `*` expand all siblings, type-ahead by name, Enter focus, Space toggles a row's
detail line.

## 4. Micro-interactions
- **Spawn.** A new child row unfolds under its parent (height 0 → 32 px) while its connector line draws
  from the parent's rail down to the new row; siblings below slide down. Rows spawned within 100 ms of
  each other stagger 30 ms (cap 6, `stagger`). The new row's dot starts with the submitted pulse.
- **Verb swap.** Same as the status strip: 180 ms crossfade with 4 px slide, width tween `spring.snap`.
  Summary text from the adapter (cheap status lines, ~30 s cadence per [R024]) animates **only the
  changed words** (diff by word; unchanged prefix stays still); minimum 2 s between swaps per row.
- **Roll-up badge.** When a hidden descendant changes urgency, the collapsed parent's badge does one
  120 ms scale 1 → 1.15 → 1 pop; amber "needs you" badges add a single 400 ms ring pulse. No looping.
- **Collapse / expand.** Chevron rotates 90° (`dur.fast`); children's height animates with
  `spring.snap`; connectors retract with the rows. Collapsing a running branch keeps its roll-up badge
  live.
- **Focus a child.** The focused row's name slides into the breadcrumb (`dur.travel` 240 ms,
  `ease.in-out`) while the content pane crossfades to that child's transcript; the tree narrows to a
  rail (icons + dots only) in layouts under 720 px. Back reverses the travel.
- **Result bubbling up.** On `Done`, a result chip ("2 findings") condenses from the row's verb text,
  then travels along the connector to the parent row (240 ms, `ease.in-out`, max 260 ms per
  `dur.travel`) and lands as a "↗ returned" marker on the parent and a receipt line in the parent's
  transcript ("Reviewer returned · 2 findings"). The child's row settles to ✓ + result line. Many children
  finishing together: chips queue at 80 ms intervals, max 4 animated, the rest appear in place.
- **Needs you.** The row's dot stops breathing and does the waiting tip (hourglass-style turn every 2 s,
  as in the status strip); the row background takes a 6 % amber wash. The tree header count rolls
  ("1 needs you").
- **Done / failed.** Dot morphs to ✓ (scale 0.6 → 1, `spring.snap`) or ✕ with a 2-cycle 3 px nudge,
  then goes still. Finished rows dim text to secondary after 4 s (still ≥ 4.5:1).
- **Hover.** Row reveals trailing actions (focus, stop) and a tooltip: model, started at, tokens, worktree
  or branch if the app supplies them.
- **Drag** (Pro `subagent-tree/canvas` only): pan the node canvas; rows themselves are not reorderable
  (order is spawn order).

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Row spawn unfold | 200 ms | — | `spring.snap` | 30 ms, cap 6 | 60 Hz during |
| Connector draw | 160 ms | with unfold | `ease.out` | follows rows | 60 Hz |
| Submitted pulse (Starting) | 400 ms × 1 | — | `ease.out` | — | 60 Hz |
| Dot breathing (Working) | 1600 ms loop | — | sine | phase-locked to shared clock | 30 Hz (max 8 rows) |
| Verb / summary swap | 180 ms, 4 px | min 2 s between swaps | `ease.out` + `spring.snap` width | word diff 30 ms | 60 Hz |
| Roll-up badge pop | 120 ms | — | `spring.snap` | — | 60 Hz |
| Needs-you ring | 400 ms × 1 | — | `ease.out` | — | 60 Hz |
| Waiting tip | 2000 ms loop (300 ms turn) | — | `ease.in-out` | — | 30 Hz |
| Collapse / expand | 180 ms; chevron `dur.fast` 120 ms | — | `spring.snap` | — | 60 Hz |
| Focus breadcrumb travel | 240 ms | — | `ease.in-out` | — | 60 Hz |
| Pane crossfade on focus | 160 ms | — | `ease.out` | — | 60 Hz |
| Result chip travel | 240 ms (cap 260) | 120 ms condense first | `ease.in-out` | 80 ms, cap 4 | 60 Hz |
| Done / failed morph | 200 ms / 160 ms + 2 × 3 px nudge 240 ms | — | `spring.snap` / linear | — | 60 Hz |
| Dim finished rows | 300 ms | after 4 s | `ease.out` | — | 30 Hz |
| Stall ramp / snap-back | 2 s / 200 ms | after 5 s silent (soft) | linear OKLab / `ease.out` | — | 30 Hz |
| **Reduced motion** | rows appear/disappear with an 80 ms opacity crossfade, no unfold or connector draw; no breathing (static ●), no tip (static ⏸); result chip does not travel: parent marker and receipt crossfade in; numbers swap; focus switches pane instantly with 80 ms fade | — | linear | none | 1 Hz (elapsed only) |

## 6. Variants

### Free (5)
| Name | Idea | Surfaces |
|---|---|---|
| `subagent-tree/indented` | Baseline: indented rows with chevrons, dot + verb + elapsed, roll-up badges | desktop, ide, chat panel |
| `subagent-tree/rail` | Tree with drawn connector lines (├ └) and the result-travel animation along them | desktop, ide |
| `subagent-tree/dock` | Compact panel docked under the composer listing only live children; finished ones fold into "3 done" (Claude Code subagent panel idea, note 05) | chat, desktop |
| `subagent-tree/inline-wrapper` | A sub-agent shown as one collapsible tool row in the parent transcript that expands into its own steps (Paradev idea, note 03) | chat, ide |
| `subagent-tree/terminal` | Box-drawing tree with braille spinner per live row, `✓`/`✗`/`⚠` glyphs, 1 Hz repaint | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `subagent-tree/canvas` | Node-and-edge canvas; pan/zoom; edges light up while results travel |
| `subagent-tree/orbit` | Children orbit the parent avatar as small dots; a child's dot flies inward when it returns |
| `subagent-tree/org-chart` | Top-down org chart for wide, shallow swarms (one orchestrator, many workers) |
| `subagent-tree/ranks` | Columns by depth ("rank 1–4"), rows by agent, for big swarms (Hoplite orchestrator idea, note 10) |
| `subagent-tree/sunburst` | Radial depth rings; segment size = elapsed or tokens; hover to focus |
| `subagent-tree/avatar-stack` | Collapsed to a stack of coloured avatars with a roll-up badge; expands into `indented` |
| `subagent-tree/expert-cards` | Named expert sub-agents (e.g. an "Oracle") get a card with portrait mark and role (Amp idea, note 03) |
| `subagent-tree/metro` | Children drawn as metro lines branching from the parent line; stations are steps [R038] |
| `subagent-tree/heat` | Rows tinted by cost or tokens share (needs `RunMeter` data; prices from the app only) |
| `subagent-tree/breadcrumb-only` | No tree; a breadcrumb with a dropdown per level for dense IDE headers |
| `subagent-tree/activity-popover` | Top-bar button "3 agents" that opens the tree as a popover (Paradev activity panel idea, note 03) |
| `subagent-tree/notch` | Desktop island showing "2 running · 1 needs you" while the window is unfocused |
| `subagent-tree/growth` | Children sprout along a curve with staggered leaf pops (pixel-tree idea [R004]) |
| `subagent-tree/ascii-graph` | Terminal graph with ASCII edges and a live "now" column of states |

## 7. Edge cases
- **Long names / verbs.** Name max 24 ch, verb truncates middle ("Editing src/…/auth.rs"); full text in
  the tooltip. Model-written summaries over 80 ch are clipped.
- **Wide swarms.** More than **8 children** under one parent: first 8 by urgency (needs you, failed,
  running, then done by recency) and a "+ 12 more" row. More than **50 nodes** total: rows virtualised;
  only visible rows tick.
- **Deep trees.** Display depth 3 by default (proposed); deeper levels fold into "+ n nested agents".
  Indent caps at 3 levels × 16 px; deeper indent shows a depth number chip (`d5`) instead of more indent.
  Runtime refusals (`Refused{DepthLimit}`) are shown once per parent, not per attempt.
- **Cycles / missing parents** from a buggy adapter: a node whose parent is unknown attaches to the root
  with an "unlinked" badge; cycles are broken at the second visit (debug log).
- **Very fast children** (< 150 ms `show_delay`): no row animation; a finished child appears directly as
  ✓ in the "done" fold.
- **Very slow children** (hours): elapsed compacts to `1h 02m`; status summaries keep refreshing at the
  adapter's cadence; stall thresholds use `tool_stall` (120 s) while a tool runs.
- **Parent stopped.** Stop on the parent stops all descendants by default; children that keep running
  (app says they are detached) become `Orphaned`.
- **Approval from a deep child.** Root header "1 needs you" plus the ancestors' roll-up badges form a path;
  Review opens the card without changing focus when the layout has room, otherwise focuses the child.
- **RTL.** Indentation and connectors mirror; elapsed and numbers stay LTR in bidi isolates; breadcrumb
  separator flips (‹).
- **Tiny sizes (16–20 px rows).** Dot (8 px) + name only; verb in tooltip; elapsed hidden under 200 px
  width; actions behind a ⋯ menu.
- **Dense IDE panels.** 28 px rows, 12 px dots, no result travel (instant marker), connectors 1.5 px.
- **Touch.** Rows 44 px hit target; swipe-left reveals Stop; long-press opens details.
- **Slow networks.** Rows keep last known state; the tree header shows "offline" once, rows dim to 60 %;
  no stall ramps while the transport reports reconnecting.
- **Same colour twice** (more than 8 agents): colours repeat; names and a 2-letter monogram in the
  swatch disambiguate.

## 8. Accessibility
- Role `tree` with `treeitem`s, `aria-level`, `aria-setsize`, `aria-posinset`, `aria-expanded`. GPUI
  maps to the platform tree role.
- **Labels:** "Test runner, sub-agent of Orchestrator, needs your approval: run cargo test, waiting 31
  seconds"; "Reviewer, done after 1 minute 5 seconds, returned 2 findings"; truncated row: "4 more
  nested agents, collapsed".
- **Live region (polite):** sub-agent started (batched: "3 sub-agents started"), finished with result,
  failed (**assertive**), needs approval (**assertive**), all done. At most one announcement per 5 s
  per tree; verb changes are not announced.
- **Contrast:** names and verbs ≥ 4.5:1; identity swatches ≥ 3:1 against the row and always paired with
  the name; state is carried by glyph + text (●, ⚠, ✓, ✕, ■, ⊘), never colour alone.
- **Focus order:** tree header (collapse all) → rows in tree order → row actions via Tab inside the
  focused row. Focusing a child moves focus to the child transcript's first message heading; Back
  returns focus to the same row.
- **Reduced motion:** §5 last row; meaning never depends on the travel animation (receipt line carries it).

## 9. Props sketch
Shared model (proposed, `oxyn-core`; adapters fill it from AG-UI `SUBAGENT_*`, ACP sessions, or app
events):
```rust
pub struct RunGraph {
    pub root: AgentId,
    pub nodes: IndexMap<AgentId, AgentNode>,   // spawn order
    pub depth_limit: Option<u8>,               // reported by the app; display only
}

pub struct AgentNode {
    pub id: AgentId,
    pub parent: Option<AgentId>,
    pub name: SharedString,                    // "Researcher"
    pub role: Option<SharedString>,            // "expert", "worker", app-defined
    pub model: Option<SharedString>,
    pub colour: AgentColour,                   // 8 identity tokens color.agent.{red..cyan}
    pub state: AgentState,                     // architecture.md enum, per node
    pub clock: RunClockRef,                    // child clock (elapsed-timer.md §2)
    pub summary: Option<SharedString>,         // cheap status line from the adapter [R024]
    pub result: Option<ResultSummary>,         // { line, count, payload_ref }
    pub usage: Option<Usage>,                  // tokens in/out/cached (run-meter.md)
    pub workspace: Option<WorkspaceRef>,       // branch / worktree chip (family H)
    pub detached: bool,                        // survives parent → Orphaned
    pub refused: Option<RefuseReason>,         // DepthLimit(n) | Budget | Policy
}

pub struct SubagentTree {
    pub graph: RunGraphRef,
    pub variant: TreeVariant,                  // Indented | Rail | Dock | InlineWrapper | Terminal | Pro(..)
    pub display_depth: u8,                     // 3
    pub max_children: u8,                      // 8, then "+n more"
    pub max_animating: u8,                     // 8 (loaders.md §2.3)
    pub focused: Option<AgentId>,
    pub on_focus: Callback<AgentId>,
    pub on_stop: Option<Callback<AgentId>>,    // None hides Stop
    pub on_review: Callback<RequestRef>,       // opens the approval card (family G)
    pub on_retry: Option<Callback<AgentId>>,
    pub result_travel: bool,                   // true; false in dense/IDE
    pub dim_finished_after: Duration,          // 4 s
    pub labels: TreeLabels,
    pub motion: MotionLevel,
}
```
Mapping per node: `Idle`→`Pending`; `Submitted`→`Starting`; `Thinking/Streaming`→`Working`;
`ToolRunning`→`ToolRunning`; `AwaitingApproval`→`AwaitingYou`; `Queued/Steering`→`Working` (+ steer
blip); `Stalled`→`Stalled`; `Done`→`Returning`→`Done`; `Error`→`Failed`; `Cancelled`→`Interrupted`.
`WaitingOnChildren`, `Orphaned`, `Refused` and `Hibernating` are derived from the graph, not from
`AgentState` (a gap in architecture.md: it has no notion of child runs; proposed addition above).

## 10. Related components and references
Related: [tool-call.md](tool-call.md) (inline-wrapper reuses the tool row) · [plan-todo.md](plan-todo.md)
(todo owner = sub-agent colour) · [fanout-grid.md](fanout-grid.md) (siblings side by side) ·
[acting-indicator.md](acting-indicator.md) (which agent drives a surface) · [trace-view.md](trace-view.md)
(sub-agent spans nest the same way) · [run-replay.md](run-replay.md) · [run-meter.md](run-meter.md) (per
node usage) · [composer-status-strip.md](composer-status-strip.md) ("+2 agents" and lanes peek) ·
[elapsed-timer.md](elapsed-timer.md) (child clocks) · [loaders.md](loaders.md) (row busy regions) ·
approvals family G ([../component-taxonomy.md](../component-taxonomy.md)).

| Ref | What we take | Licence note |
|---|---|---|
| Claude Code subagents, `/tasks` (notes 05, 08) | 8 identity colours; panel below the prompt; finished ones listed as done | Proprietary — reference only |
| Claude Code status-line update [R024] | summarised status lines ~30 s; animate only the diff | Study only |
| Paradev (note 03) | sub-agent as a collapsible tool wrapper; activity panel of running subagents | Licence not checked; study only |
| Amp Oracle (note 03) | named expert gets its own identity | Proprietary; study only |
| Hoplite Subagents panel (note 10) | durable panel, active vs recent, child states, blocked child uses the same review path; ranked orchestrator view | Proprietary; study only |
| Antigravity (note 08) | parent shows working while subagents run | Proprietary; study only |
| AG-UI events (note 05) | `SUBAGENT_STARTED / FINISHED / ERROR` for the adapter | Protocol; implement from spec, licence to check before copying code |
| signoff-ui ToolCallTimeline [R107] | nested sub-agents inside a timeline | MIT per licensing.md — portable with attribution |
| Threadbear [R053] | Hibernating as a state | Study only |
| Pixel tree [R004], metro pipeline [R038] | Pro `growth`, `metro` | Study only |
| affectUI Agent Tree (note 11) | name only (Pro, code gated) | Paid — study only |
