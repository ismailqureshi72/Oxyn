**Status:** draft v0.1 (2026-11-11).

# Spec: trace / span waterfall (`trace-view`)

Part of family F, part 2 "Orchestration & observability" ([../component-taxonomy.md](../component-taxonomy.md),
overview in [tools.md](tools.md)). Same depth as [composer-send-control.md](composer-send-control.md).
Target **4 free + 14 Pro = 18 variants**. Spans are derived from the shared `RunLog`
([run-replay.md](run-replay.md) §9) and nest along the `RunGraph` ([subagent-tree.md](subagent-tree.md) §9).

## 1. Purpose
After (or during) a run the user — usually a developer or an ops person — asks **"where did the time and
the tokens go, what failed, and why was it slow?"** The trace view lays every model call, tool call,
retrieval and sub-agent out as **spans on a time axis**, nested by parent, with duration, tokens and
(only when the app supplies prices) cost per span. It is the observability twin of the transcript: the
transcript says what was said, the trace says what happened and how long it took.

What we saw: Langfuse observation types event, span, generation, agent, tool, chain, retriever,
evaluator, embedding, guardrail, with Tree / Timeline / Graph views and a dense timeline with
pan / zoom / fit; LangSmith runs → trace → thread, a Trajectory view that flattens a session to messages,
filter scopes Thread / Root run / Single run / Any run; AgentOps Session Waterfall with LLM calls,
actions, tools and errors on a time axis plus a detail pane, coloured by span type (all research note 05);
a cost waterfall where rows append live, each bar starts at the running cost offset, the total rolls,
failed steps auto-expand with INPUT/OUTPUT JSON, and a Cost ↔ Tokens toggle re-scales the bars [R030];
affectUI Trace Waterfall: every LLM call, tool call and sub-agent as a bar on a live timeline, tokens and
cost per step, the slowest call highlighted, branches fold into their parent (research note 11, Pro,
study only); signoff-ui ToolCallTimeline: a waterfall with nested sub-agents [R107].

Anatomy (`waterfall`, live run, 24 px rows):
```
┌ Trace · run 4f2a · 2m 14s · 48.2k tok · $0.46 ─── Filter ▾  Errors only ☐  Cost|Tokens  ⤢ Fit  ▶ Replay ┐
│ name                         │0s        30s        60s        90s       120s   │ dur    in/out    cost   │
│ ▾ ◆ Orchestrator  (agent)    │███████████████████████████████████████████▌now │ 2m 14s 18k/3.1k  $0.21  │
│   ✦ plan        (generation) │███▌                                            │ 6.2s   4.1k/820  $0.04  │
│   ▾ ◆ Researcher (agent)     │    ██████████████████▌                         │ 48s    9.8k/1.2k $0.09  │
│      ⌕ search    (retriever) │     ██                                         │ 2.1s   —         —      │
│      ⚒ fetch_pages (tool)    │       ███████▍ ← slowest                       │ 18.4s  —         —      │
│   ⚒ check_links  (tool) ✕    │                          ███  2 of 31 → 404    │ 3.0s   —         —      │  failed: auto-expanded
│      └ input {…}  output {…} │                                                │                         │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
  type glyph · name · bar (offset = start, length = duration; generation bars show a TTFT tick) · numbers
```

Rules (proposed):
1. **Ten span types + error overlay** with distinct glyphs and tones (Langfuse's list): `agent ◆`,
   `generation ✦`, `tool ⚒`, `retriever ⌕`, `chain ⛓`, `span ▭`, `event •`, `evaluator ✓?`, `embedding ⋰`,
   `guardrail ⛨` (final glyph set comes from family L). Error is an overlay (✕ + danger edge), not a type.
2. **Time axis is wall time from the run start**; paused intervals (awaiting approval) are drawn as a
   hatched band across all rows, not removed, so bars stay honest (matches elapsed-timer.md §2).
3. **Numbers only from data**: tokens from usage events, cost only with a `PriceTable`, TTFT only when the
   adapter reports the first-token time. Missing values show "—".
4. **Errors are never hidden by collapse**: a collapsed parent with a failed descendant shows an error dot;
   failed spans auto-expand once [R030].
5. The view **links to run replay**: any span can "Replay from here".

## 2. State machine
View-level:
```
                 load trace                  spans arrive (live)                 run ends
  Empty ───────────────────────▶ Loading ─────────────────────▶ Live{now edge} ─────────────────▶ Complete
 ("No trace for            (skeleton rows,        │  ▲ follow-now on / off (scroll or zoom detaches)   │
  this run")                show_delay 150 ms)    │  └──── "Jump to now" ─────────────────────────────  │
                                                  ▼                                                     ▼
                                         Live{detached} ── run ends ──▶ Complete          Filtered{query} ⇄ Complete
  Loading ── error ──▶ LoadFailed ("Couldn't load trace · Retry")
  Complete/Live ── select span ──▶ Selected{span} (detail pane open) ── Esc ──▶ previous
  Complete/Live ── zoom/pan ──▶ Zoomed{range} ── Fit ──▶ previous
  Overlays: Truncated ("Showing 2 000 of 14 302 spans · load more") · Offline (live paused, "reconnecting")
```
Per span:
```
  Open (bar grows to now, end cap pulses) ── end event ──▶ Closed{ok}
       │                                     └── error ─▶ Closed{error} (auto-expand once)
       ├── awaiting approval ──▶ Waiting (hatched segment grows) ── answered ──▶ Open
       ├── no events for tool_stall ──▶ Stalled (bar tail ramps to warning)
       └── run cancelled ──▶ Cut (bar ends with ■ cap)
  Orphan: end event without a start → zero-width marker with "?" (adapter bug, debug only)
```

## 3. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Empty` | "No trace for this run" | — | learn how (app link) | — |
| `Loading` | 6 skeleton rows with bars of varied widths (shared sweep) | — | cancel | — |
| `LoadFailed` | danger line + Retry | retry | copy error | Enter retry |
| `Live{now edge}` | open bars grow to a vertical "now" line; axis auto-extends; header totals roll | select span | pause follow | ↑/↓ rows, → expand, ← collapse, Enter select |
| `Live{detached}` | as Live, follow off; "Jump to now ⇥" chip | jump to now | — | `End` jump to now |
| `Complete` | static bars, totals frozen, slowest span marked | select span | export (app) | as above |
| `Filtered{query}` | non-matching rows hidden; ancestors of matches kept dimmed; count "12 of 214" | clear filter | edit filter | `/` focus filter, Esc clear |
| `Selected{span}` | row highlighted; detail pane: type, timings (start offset, duration, TTFT, waiting), tokens in/out/cached, cost, model, input/output (collapsed JSON, copy), error, links | **Replay from here** | open in transcript, copy span id | `r` replay, `o` open, Esc close |
| `Zoomed{range}` | axis shows range; minimap brush | fit | zoom out | `+`/`−` zoom, `0` fit, Shift+scroll pan |
| Span `Open` | bar to now, end cap breathes | select | — | — |
| Span `Waiting` | hatched amber segment | review approval | — | — |
| Span `Stalled` | tail ramps to warning, tooltip "no events 2m" | select | — | — |
| Span `Closed{error}` | danger edge + ✕; auto-expanded once with error line | select | retry (if the app maps it) | — |
| Span `Cut` | bar ends with ■ | — | — | — |
| `Truncated` | footer "Showing 2 000 of 14 302 spans" | load more | filter | — |

## 4. Micro-interactions
- **Rows append live.** New spans insert under their parent with a 160 ms unfold; the bar starts at its
  offset and grows with the shared clock (bars repaint at 10 Hz, the "now" line at 30 Hz while visible).
- **Totals roll.** Header duration, tokens and cost count up with the elapsed-timer count-up rule (25 % of
  the gap per 50 ms, min +1) [R030].
- **Cost ↔ Tokens ↔ Time toggle.** Changing the bar metric re-scales bars with a 260 ms `spring.snap`
  width morph; in Cost/Tokens mode bars are cumulative-offset (each starts where the previous total ended),
  in Time mode they sit on the time axis.
- **Slowest highlight.** On complete (and every 5 s while live), the slowest leaf span gets a "slowest"
  tag that slides in from its bar end; only one at a time.
- **Error auto-expand.** A failed span expands once to show the error line and input/output; a parent with
  a failed hidden child shows a danger dot that pulses once.
- **Hover.** Bar shows a tooltip (name, duration, start offset, tokens); a thin vertical guide line follows
  the pointer across all rows with a time label at the axis.
- **Select.** Detail pane slides in from the trailing edge (240 ms); the selected bar raises with a 1 px
  outline; sibling bars dim to 70 %.
- **Zoom / pan.** Wheel + modifier zooms around the pointer; drag on the minimap brush pans; Fit animates
  to the full range (240 ms `ease.in-out`).
- **Collapse.** Collapsing an agent span folds children into the parent bar, which shows a stacked
  segment pattern of its children's types ("branches fold into their parent" idea).
- **Replay from here.** Pressing it sends the span's start time to run replay; a small playhead glyph
  flies from the bar to the replay control (`dur.travel` 240 ms) when both are visible.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Skeleton show | — | `show_delay` 150 ms | — | — | 30 Hz sweep |
| Row unfold on append | 160 ms | — | `spring.snap` | 20 ms, cap 10 | 60 Hz |
| Live bar growth | continuous | — | linear | — | 10 Hz bars, 30 Hz now line |
| Open end-cap breathe | 1600 ms loop | — | sine | max 8 animate | 30 Hz |
| Header totals count-up | 50 ms ticks, 25 % of gap | — | — | — | 20 Hz while moving |
| Metric toggle re-scale | 260 ms | — | `spring.snap` | 10 ms per row, cap 30 | 60 Hz |
| Slowest tag slide | 200 ms | — | `ease.out` | — | 60 Hz |
| Error dot pulse | 400 ms × 1 | — | `ease.out` | — | 60 Hz |
| Detail pane in / out | 240 ms / 160 ms | — | `ease.out` / `ease.in` | — | 60 Hz |
| Zoom step / Fit | 160 ms / 240 ms | — | `ease.out` / `ease.in-out` | — | 60 Hz |
| Collapse fold | 180 ms | — | `spring.snap` | — | 60 Hz |
| Playhead fly to replay | 240 ms | — | `ease.in-out` | — | 60 Hz |
| Stall tail ramp | 2 s | after `tool_stall` 120 s | linear OKLab | — | 10 Hz |
| **Reduced motion** | rows appear in place (80 ms fade); bars still grow (information) but repaint at 2 Hz without easing; no breathing end caps (static open cap ▸); totals jump every 500 ms; metric toggle and zoom switch instantly; pane appears with 80 ms fade; no fly | — | linear | none | 2 Hz live |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `trace-view/waterfall` | Baseline: nested rows + time-axis bars + numbers columns + detail pane | desktop, ide |
| `trace-view/tree` | Indented typed rows with duration / tokens / cost columns, no time axis (dense) | ide, desktop side panel |
| `trace-view/cost-rows` | Steps as rows whose bars start at the running cost (or token) offset; total rolls; Cost ↔ Tokens toggle [R030] | chat (card), desktop |
| `trace-view/terminal` | Text waterfall with `█▌` bars scaled to terminal width, `✕` errors, selectable rows | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `trace-view/flame` | Flame graph (time × depth), click to zoom into a subtree |
| `trace-view/icicle` | Top-down icicle partition by tokens or cost instead of time |
| `trace-view/graph` | Node graph of agents/tools/models with aggregated vs expanded edges ("llm.chat (3/3)", note 05) |
| `trace-view/trajectory` | Flattened message list (user / AI / tool) with jump-to-span (LangSmith idea, note 05) |
| `trace-view/swimlanes-by-agent` | One horizontal lane per agent; spans placed on their agent's lane; hand-offs drawn |
| `trace-view/critical-path` | Highlights the chain of spans that determined total duration; others fade |
| `trace-view/diff-traces` | Two runs aligned by step; bars show slower / faster deltas |
| `trace-view/latency-histogram` | Per span-type distribution over many runs with p50 / p95 ticks |
| `trace-view/token-sankey` | Tokens flowing from prompt parts → model calls → outputs |
| `trace-view/minimap-dense` | 1 px rows minimap for traces with thousands of spans; brush to zoom |
| `trace-view/live-ticker` | Single-line ticker of spans opening and closing for status bars |
| `trace-view/error-focus` | Opens filtered to failures with their ancestor chain and a retry path |
| `trace-view/cache-overlay` | Overlays cached-token share per generation span (only with cache data) |
| `trace-view/budget-lines` | Horizontal budget threshold lines on the cumulative cost view |

## 7. Edge cases
- **Huge traces** (> 2 000 spans): virtualised rows; bars for spans narrower than 1 px merge into a density
  strip; loading is paged ("load more"); filters run on the full set if the app provides a query, else on
  loaded spans with a note.
- **Very short spans** (< 1 ms or < 1 px): drawn as 2 px ticks; duration "< 1 ms".
- **Very long runs** (hours, days): axis switches to `1h 02m` labels and adaptive tick spacing (1 s … 1 h);
  live bars repaint at 1 Hz once the visible range exceeds 1 h.
- **Clock skew between agents/services**: child spans that start before their parent are clamped to the
  parent's start with a "skew" badge in the detail pane; raw timestamps are kept.
- **Missing end** (run crashed): spans stay `Open` until the run is marked ended, then become `Cut` at the
  last event time.
- **Parallel children**: siblings overlap in time on separate rows (never stacked on one row).
- **Paused intervals** cross all rows as hatching; duration columns show active time and, in the tooltip,
  "waited 40 s".
- **No usage data**: tokens column hidden; no prices: cost column hidden and Cost toggle disabled with
  reason "No prices configured".
- **Huge payloads**: input/output JSON truncated at 64 KB in the pane with "Open full"; secrets the adapter
  marks as redacted stay masked.
- **Long names**: span names truncate middle at 32 ch; full name in tooltip and pane.
- **RTL**: the table mirrors, but the **time axis stays left → right** (time is not text); numbers in LTR
  isolates.
- **Tiny sizes**: under 480 px wide, the waterfall becomes `tree` with an inline mini bar per row.
- **Dense IDE panels**: 20 px rows, numbers columns collapse into a tooltip.
- **Touch**: pinch to zoom, drag to pan, tap selects; rows 44 px.
- **Slow networks**: live mode shows "reconnecting"; bars stop growing at the last event time (they do not
  keep growing on assumption) and resume.

## 8. Accessibility
- The waterfall is a `treegrid`: rows are spans (`aria-level`, `aria-expanded`), columns are name, start,
  duration, tokens, cost. The bar is decorative; its values are in the cells.
- **Labels:** "fetch_pages, tool, started at 7.2 seconds, took 18.4 seconds, slowest span"; "check_links,
  tool, failed: 2 of 31 links return 404".
- **Live region (polite)** in live mode: span failed (**assertive**), run complete with totals. Span starts
  are not announced. Max one announcement per 3 s.
- **Contrast:** bars ≥ 3:1 against the track; type is carried by glyph + label, not tone alone; error uses
  ✕ + text; hatching for waiting has a text equivalent.
- **Focus order:** header controls (filter, errors only, metric toggle, fit, replay) → treegrid → detail
  pane (when open, Esc returns focus to the row).
- **Keyboard:** treegrid arrows, `/` filter, `e` errors only, `+`/`−`/`0` zoom, `r` replay from selected,
  `End` jump to now.
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct TraceView {
    pub source: TraceSource,                // Log(RunLogRef) (run-replay.md §9) | Spans(Vec<Span>) from an external tracer
    pub graph: Option<RunGraphRef>,         // nest by agent (subagent-tree.md §9)
    pub variant: TraceVariant,              // Waterfall | Tree | CostRows | Terminal | Pro(..)
    pub metric: BarMetric,                  // Time (default) | Tokens | Cost (needs price)
    pub price: Option<PriceTable>,          // app-supplied; None → no cost column
    pub filter: TraceFilter,                // types, status, agent, min_duration, text
    pub follow_now: bool,                   // true while live
    pub max_rows: usize,                    // 2 000 before paging
    pub auto_expand_errors: bool,           // true
    pub highlight_slowest: bool,            // true
    pub on_select: Callback<SpanId>,
    pub on_replay_from: Option<Callback<Instant>>, // None hides "Replay from here"
    pub on_open_in_transcript: Option<Callback<SpanId>>,
    pub labels: TraceLabels,
    pub motion: MotionLevel,
}

pub struct Span {
    pub id: SpanId,
    pub parent: Option<SpanId>,
    pub agent: Option<AgentId>,
    pub kind: SpanKind,                     // Agent | Generation | Tool | Retriever | Chain | Span | Event | Evaluator | Embedding | Guardrail
    pub name: SharedString,
    pub start: Instant,
    pub end: Option<Instant>,               // None = Open
    pub waits: Vec<(Instant, Option<Instant>)>,
    pub first_token: Option<Instant>,       // generation TTFT tick
    pub usage: Option<Usage>,               // in, out, cached
    pub model: Option<SharedString>,
    pub status: SpanStatus,                 // Ok | Error(ErrorInfo) | Cut | Stalled
    pub io: Option<SpanIoRef>,              // lazy input/output payloads
}
```
Mapping from `AgentState` (per agent, live): `Thinking/Streaming` open a `Generation` span;
`ToolRunning{tool}` opens a `Tool` span; `AwaitingApproval` adds a wait interval; `Stalled` marks open
spans `Stalled`; `Done` closes the agent span; `Error` closes with `Error`; `Cancelled` cuts open spans.
Sub-agent starts open an `Agent` span under the parent. External tracers (OpenTelemetry-style exporters)
can feed `Spans` directly through an adapter (later).

## 10. Related components and references
Related: [run-replay.md](run-replay.md) (shared log, replay from span) · [run-meter.md](run-meter.md)
(totals in the header) · [subagent-tree.md](subagent-tree.md) (agent nesting) ·
[tool-call.md](tool-call.md) (tool span ↔ tool row) · [fanout-grid.md](fanout-grid.md) (swimlanes share
the axis) · [elapsed-timer.md](elapsed-timer.md) (formats, count-up) ·
[composer-status-strip.md](composer-status-strip.md) Pro `waterfall-peek` · [skeletons.md](skeletons.md) ·
trace observation-type icons (family L).

| Ref | What we take | Licence note |
|---|---|---|
| Langfuse (note 05) | 10 observation types; tree / timeline / graph; pan / zoom / fit | Licence not checked here — study only until checked |
| LangSmith (note 05) | trajectory view; filter scopes | Proprietary — study only |
| AgentOps (note 05) | session waterfall + detail pane; colour by type | Licence not checked — study only |
| Cost waterfall [R030] | live rows at running cost offset; rolling total; failed rows auto-expand; Cost ↔ Tokens | Study only |
| affectUI Trace Waterfall (note 11) | slowest highlighted; branches fold into parent | Paid — study only |
| signoff-ui ToolCallTimeline [R107] | waterfall with nested sub-agents; AG-UI / ACP adapters | MIT per licensing.md — portable with attribution |
