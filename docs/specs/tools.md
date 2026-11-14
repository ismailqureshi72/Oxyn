**Status:** v0.2 (2026-11-12), implemented in plan #20: `oxyn-core::run::stack` (grouping, density, worked-for fold), Dioxus `ToolStack` (3 free).

# Spec: tool calls, plans & orchestration (family F overview)

Overview of family **F** in [../component-taxonomy.md](../component-taxonomy.md) (target **50+
variants**). Same depth as the worked example [composer-send-control.md](composer-send-control.md).
Motion tokens are the shared ones from [composer.md](composer.md) §"Motion tokens" and the family-C
tokens from [status-text.md](status-text.md) §4; family-F tokens are added in §5 below. Loader and
busy-signal rules are the ones in [loaders.md](loaders.md) §2.3 and §3; nothing here overrides them.

Specs in this family:

| Spec | Component(s) | Variants (free + Pro) | Writer |
|---|---|---|---|
| this file | tool event model, grouping, collapse, density; `tool-stack` (composite transcript container) | 3 + 7 | part 1 |
| [tool-call.md](tool-call.md) | `tool-call` (row, card, group row, MCP badge) | 4 + 14 | part 1 |
| [file-edit-chip.md](file-edit-chip.md) | `file-edit-chip` (chip, row, rollup) | 3 + 11 | part 1 |
| [terminal-output.md](terminal-output.md) | `terminal-output` (inline block, live tail) | 3 + 11 | part 1 |
| [plan-todo.md](plan-todo.md) | `todo-list`, `plan-card`, `step-timeline` | 4 + 14 | part 1 |
| subagent-tree.md | `subagent-tree` | reserved | part 2 (parallel) |
| fanout-grid.md | `fanout-grid` | reserved | part 2 |
| acting-indicator.md | `acting-indicator` ("who's acting") | reserved | part 2 |
| trace-view.md | `trace-view` (tree, waterfall, graph, trajectory) | reserved | part 2 |
| background-jobs.md | `background-jobs` (tray, dock, pill) | reserved | part 2 |
| run-replay.md | `run-replay` (scrubber) | reserved | part 2 |
| run-meter.md | `run-meter` (tokens, cost, TTFT) | reserved | part 2 |

All values marked *proposed* are Oxyn's own starting points, to be tuned in the Studio. Numbers
attributed to other products come from the research notes named next to them (note numbers refer to
the project research folder; `[Rxxx]` IDs are in [../research/sources.csv](../research/sources.csv)).

## 1. Purpose
Between the user's message and the answer, an agent reads files, searches, edits, runs commands,
calls MCP servers and updates its plan. Family F is **how that work is shown**: one row or card per
tool call, file-edit chips with `+12 −3` counts, live terminal output, and the todo list / plan the
agent is following. The user is trying to answer four questions without reading everything:
1. **What is it doing right now?** (the live row, its verb and elapsed time)
2. **What did it touch?** (files read, files edited, commands run)
3. **Did anything go wrong?** (a failed command, a denied call, a stalled tool)
4. **Is it following the plan?** (todo items done / active / left)

Every coding agent we studied ships a version (research note 03 §1, rows "Tool-call row / card",
"File edit card", "Terminal / command output block", "Todo / plan checklist" — all rated
table-stakes): Codex CLI coalesces reads into `• Explored └ Read …` and commands into
`Running → Ran` with a 5-line output preview; Claude Code blinks a `⏺` bullet at 600 ms until the
call resolves, hangs results off a `⎿` gutter and expands with Ctrl+O; Warp expands diffs while
awaiting approval and collapses them to headers once applied; Antigravity folds tool groups into
`Worked for 31s ›` between prose paragraphs (all research note 08). Kits ship one card per pattern
(affectUI Tool Call Card [note 11 §15], Beautiful UI Tool Chips and Task Rows [R106], Arc's agent
run card [note 12]). Family F makes these **one event model with one set of grouping, collapse and
density rules**, so a tool row, a file chip, a terminal block and a plan item all read the same
state, share one clock and degrade the same way.

Where family F appears:
- **Transcript**, between assistant prose paragraphs (default: `tool-stack/transcript`).
- **Composer region**: the todo dock and plan pill above the composer
  ([composer.md](composer.md)); the status strip shows the pinned tool verb
  ([composer-status-strip.md](composer-status-strip.md)).
- **Side panels** (IDE): plan, tasks, terminal, changes, trace (family J chrome hosts them).
- **Lists and dashboards**: sub-agent rows, background jobs, run history (part 2 specs).

## 2. The tool event model (`oxyn-core::run`, proposed)

### 2.1 Where it sits relative to `AgentState`
[../architecture.md](../architecture.md) has one aggregate `AgentState` with `ToolRunning { tool,
verb }` and `AwaitingApproval { request }`. That is enough for the composer and loaders, but a
transcript needs **every call**, several of which may run at once. So `oxyn-core` keeps, beside the
`AgentState`, a **`RunLog`**: an append-only list of `ToolCall` records per turn, built by the same
adapters (`Event -> StateTransition` also emits `ToolEvent`s). Rules (proposed):
1. `AgentState` is **derived** from the `RunLog` while tools are active: any call `Running` or
   `StreamingOutput` ⇒ `ToolRunning { tool: <most recently started>, verb }`; any call
   `AwaitingApproval` ⇒ `AwaitingApproval { request }` (approval wins over running).
2. Components get `&AgentState` **plus** the records they render (`&ToolCall`, `&FileEdit`,
   `&TerminalBlock`, `&Plan`). They never keep their own copy of a call's state.
3. Each call has a **child clock** of the run's `RunClock` ([elapsed-timer.md](elapsed-timer.md)),
   so its duration pauses while it waits for approval and never drifts from the status strip.
4. `Stalled` for a call is **derived** (no output and no state change for `tool_stall`, 120 s,
   from [composer-status-strip.md](composer-status-strip.md)); the app never sets it.

### 2.2 Call lifecycle (state machine)
```
          adapter: call created                 args begin streaming
  (none) ─────────────────────▶ Pending ───────────────────────────────▶ StreamingInput
                                   │ args complete in one event               │ args complete
                                   ▼                                          │
                          needs approval? ◀───────────────────────────────────┘
                           │ yes                         │ no
                           ▼                             ▼
                  AwaitingApproval ──approve──▶ Approved ──(≤ 400 ms)──▶ Running
                     │      │                                              │ first output chunk
                     │ deny └──timeout / run stopped──▶ Cancelled          ▼
                     ▼                                              StreamingOutput
                  Denied{reason}                                           │
                                     ┌─────────────────┬──────────────────┼─────────────────┐
                                     ▼                 ▼                  ▼                 ▼
                               Succeeded{summary}  Failed{kind,code}  Cancelled{by}   AuthNeeded (MCP)
                                                       │ retry (agent or user)              │ auth ok
                                                       └──▶ Pending (attempt n+1)           └──▶ Running
  Overlays (derived, never set by the app):
    Stalled{soft | hard}  · Backgrounded (shell detached to the jobs tray) · QueuedForAutoRun (permission level)
  Fast path: Pending → Succeeded inside delay.row_guard (150 ms) renders straight in the final state.
```

| State | Meaning | Visual cue (every variant) | Primary action | Keyboard (row focused) |
|---|---|---|---|---|
| `Pending` | call created, not started (queued behind another, or args not yet sent) | hollow ○ muted, verb in present tense, no timer | — | Enter expands (empty input pane) |
| `StreamingInput` | arguments arriving token by token | ○ + args preview shimmers (`text-shimmer/calm`), target fills in as it parses | — | Enter expands to live args |
| `AwaitingApproval` | blocked on the user | amber hand/shield glyph, `color.loader.waiting`, timer **paused** ⏸; diff/command expanded | owned by family G approval component | Tab into approval buttons |
| `Approved` | transient (≤ 400 ms) | ✓ flash in `waiting` tone, then Running | — | — |
| `Running` | executing, no output yet | row loader animates (busy region rules), present verb, timer after 1 s | Stop (shell, long MCP) | Enter expand, Esc stop (shell) |
| `StreamingOutput` | output chunks arriving | as Running + live tail (terminal) or count roll ("Read 3 of 6") | Stop | as Running |
| `Succeeded` | finished OK | loader morphs to ✓ (`success`, 600 ms) then `fg`; past tense + duration | expand | Enter expand, C copy |
| `Failed` | error, non-zero exit, timeout | ✕ in `danger`; error line visible without expanding (stderr tail) | Retry | Enter expand, R retry |
| `Cancelled` | stopped by user, run end, or approval timeout | ■ glyph, desaturated, "Stopped after 4s" | Retry | R retry |
| `Denied` | user or policy said no | slashed ⊘, strikethrough target, reason line ("Denied: outside workspace") | Edit rule (family G) | — |
| `AuthNeeded` | MCP server needs sign-in | key badge on the server chip, "Needs sign-in" | Connect | Enter connect |
| `Stalled` (overlay) | no output for `tool_stall` | loader slows then stops, tone ramps to `warn`, "No output for 2m" | Stop | Esc stop |

### 2.3 Adapter mapping (from research note 05 §0)
| Oxyn state | AI SDK tool part (v6) | AG-UI events | Claude Code / MCP |
|---|---|---|---|
| `Pending` | — | — | `pending` |
| `StreamingInput` | `input-streaming` | `TOOL_CALL_START` / `TOOL_CALL_ARGS` | — |
| `Running` | `input-available` | `TOOL_CALL_END` (awaiting result) | — |
| `AwaitingApproval` | `approval-requested` | interrupt / `CUSTOM` | permission prompt |
| `Approved` | `approval-responded` | — | — |
| `Succeeded` | `output-available` | `TOOL_CALL_RESULT` | — |
| `Failed` | `output-error` | `RUN_ERROR` | `failed` |
| `Denied` | `output-denied` | — | rejected |
| `AuthNeeded` | — | — | `needs-auth` |
| `Cancelled` | — | `RUN_FINISHED` early | Esc interrupt |
`StreamingOutput` has no direct equivalent in these schemas; adapters enter it on the first partial
result they can see (shell stdout, MCP progress) and otherwise skip it.

### 2.4 Tool kinds
The adapter maps each raw tool name to a `ToolKind` through an overridable `ToolKindMap` (a regex or
exact name per kind; unknown names fall to `Generic`; `mcp__<server>__<tool>`-style names and AG-UI
/ ACP metadata set `server`). Verbs come from the activity table in
[status-text.md](status-text.md) §5.3, so the row label and the status strip always agree.

| `ToolKind` | Icon (family L, in-house) | Present → past (neutral pack) | Target shown | Groups with | Default detail renderer |
|---|---|---|---|---|---|
| `Read` | page | Reading → Read | file basename (full path in tooltip) | Explore | file snippet with line range |
| `List` | folder | Listing → Listed | directory | Explore | file tree (family J) |
| `Search` | magnifier | Searching → Searched | query in quotes, ≤ 32 chars | Explore | match list `path:line` |
| `Edit` / `Write` / `Create` / `Delete` | pencil | Editing → Edited (Created / Deleted / Renamed) | file + `+12 −3` | Edit (→ [file-edit-chip.md](file-edit-chip.md)) | diff (family H) |
| `Shell` | prompt `›_` | Running → Ran | command head (first token + 1 arg) | never grouped (proposed) | [terminal-output.md](terminal-output.md) |
| `Web` (search, fetch, browse) | globe | Searching the web / Fetching / Browsing → past | query or domain + favicon | Web | result list / page excerpt |
| `Mcp` | server mark or plug | server's own verb if declared, else "Calling" → "Called" | `server · tool` | same server, consecutive | JSON tree (args, result) |
| `Plan` (todo write) | checklist | Planning → Updated plan | "3/7" | none (updates the plan card) | [plan-todo.md](plan-todo.md) |
| `Subagent` | branch | Delegating → Delegated | agent name + colour | none | subagent-tree.md |
| `Media` | spark | Producing → Produced | asset kind | none | family D placeholder |
| `Generic` | cog | Working → Ran `name` | tool name | none | JSON tree |

## 3. Grouping consecutive calls

### 3.1 Rules (proposed)
Agents often fire ten reads in a row; ten rows bury the prose. Codex coalesces reads and searches
into one `• Explored` cell (note 08 §1.1); affectUI's card "sums up multiple calls" (note 11 §15);
Beautiful UI collapses edits and calls into chips with a summary ("4 tool calls, 2 messages")
[R106]. Our rules:

1. **Group classes.** `Explore` = Read + List + Search; `Web` = web search + fetch; `Mcp(server)` =
   calls to one MCP server; `Edit` = file edits (rendered as the file-edit rollup). `Shell`, `Plan`,
   `Subagent`, `Media` and `Generic` are **never grouped** by default: each command deserves its own
   exit code, and plan/sub-agent calls render elsewhere.
2. **Adjacency, not time.** Calls group when they are consecutive in the transcript with nothing
   between them but other calls of the same class. A group **breaks** on: assistant prose, a
   reasoning block, an approval, a call of another class, a sub-agent boundary, or a turn boundary.
3. **Minimum 2.** A single call is a plain row. The second call of a class **merges** the first row
   into a group header (§4 "merge" interaction).
4. **Parallel calls** (started within 50 ms of each other) form a group regardless of class while any
   of them runs: header "Running 3 tools" (the status-text rule: count, never a list). When all
   finish, the group re-sorts into class groups if the classes differ (no animation; it settles while
   collapsed).
5. **Labels are composed, not concatenated.** Template per class with CLDR plurals: Explore →
   "Read 6 files, searched 2 patterns"; while live → "Reading 3 files…" (count rolls). Mixed outcomes
   append the worst: "Read 6 files · 1 failed".
6. **A failure surfaces.** A failed call inside a collapsed group auto-expands the group once and
   pins the failed row visible; the other rows stay collapsed under "5 more".
7. **Caps.** Expanded groups show 50 rows, then "Show 120 more" (proposed); search groups list unique
   files, not raw matches.
8. **Edits roll up.** Consecutive edits become one `file-edit-chip/tree-rollup` ("Edited 6 files
   +9 −9"); the same file edited twice in one group shows once with summed counts and an "×2" tag.

### 3.2 Group header states
| Group state | Header shows | Loader | Timer |
|---|---|---|---|
| `Live` (any child Pending / Running / Streaming) | present verb + rolling count: "Reading 4 files…" | one loader for the whole group (the group is the busy region; children show still frames) | group elapsed |
| `Waiting` (any child AwaitingApproval) | "Waiting for you · 1 of 5" | waiting motion | paused |
| `Settled` (all children terminal, ≥ 1 succeeded, none failed) | past verb summary + total duration | ✓ morph then none | frozen |
| `SettledWithErrors` | summary + "· 1 failed" in `danger` | ✕ | frozen |
| `Cancelled` | "Stopped · read 3 of 6 files" | ■ | frozen |

## 4. Collapse rules and density

### 4.1 Collapse rules (proposed; apply to rows, groups, terminal blocks, diffs, plan steps)
1. **User intent wins.** Once the user expands or collapses something by hand, the system never
   changes it again for that turn (Conductor keeps thinking and tool blocks open when the next action
   arrives, note 08 §1.15; we make that the user's choice, not the default).
2. **Expanded while it needs you.** `AwaitingApproval` expands the call's detail (diff, command,
   arguments) automatically; once resolved and applied it collapses to its header (Warp's rule,
   note 08 §1.10).
3. **Errors open.** `Failed` expands enough to show the error line and up to 5 lines of stderr; it
   never collapses by itself (R030: failed steps auto-expand).
4. **Live shell shows a tail.** A running `Shell` call shows its last 5 lines (Codex's preview
   length, note 08 §1.1); on success it collapses to the header + exit badge after
   `delay.auto_collapse` (1200 ms) **only if the system opened it**.
5. **Never jump the reader.** Auto-collapse of content above the viewport is applied with scroll
   anchoring (the visible line stays put); auto-collapse of content the pointer is over, or that holds
   focus or a text selection, is deferred until the pointer leaves / focus moves.
6. **Turn fold.** When the turn ends, the `tool-stack/worked-for` variant (and Compact density) folds
   all of the turn's tool rows into one `Worked for 31s ›` row (Antigravity, note 08 §1.11); errors and
   unresolved approvals stay visible below the fold row.
7. **Expand all / collapse all** for the turn or thread: `E` (Warp uses `E`; Codex app has expand /
   collapse all — note 08); per row Enter / Space.

### 4.2 Density levels
Claude desktop has a Normal / Thinking / Verbose transcript density switch and Cursor has compact chat
and tool-call density settings (notes 03, 08); Antigravity's Knowledge Work mode replaces tool
groups with one live progress line that clears when the response finishes (note 08 §1.11). We define
four levels; density is an app/user setting (`ToolDensity`), not a variant.

| Density | Per call | Groups | Shell | Edits | Turn end | Row height (chat / ide / mobile / terminal) |
|---|---|---|---|---|---|---|
| `Hidden` | not shown; one live progress line ("Reading auth.rs") under the last message | — | — | — | line clears; only errors and approvals remain | 24 / 20 / 32 px / 1 row |
| `Compact` | grouped only; groups always collapsed | yes | header + exit badge, no tail | rollup chip only | fold to `Worked for …` | 28 / 22 / 40 px / 1 row |
| `Normal` (default) | one row per ungrouped call | yes, collapsed | 5-line live tail | per-file chips | no fold (variant may fold) | 32 / 24 / 44 px / 1 row |
| `Verbose` | every call expanded: args + output | no grouping | 20-line tail | inline diffs expanded | no fold | 32 / 24 / 44 px / 1 row |

Row heights are proposed; 44 px on mobile is the touch minimum, and IDE rows match the 24 px dense
row used by `status-text/inline-ide`.

## 5. Micro-interactions and timing (family-wide)

### 5.1 Micro-interactions
- **Row enter.** Height opens on `spring.snap` (critically damped, nothing bounces) and the row fades
  in with y +4 → 0; Arc's agent run card opens new steps this way (note 12). Rows that arrive in a burst
  stagger 40 ms, cap 6, then appear together.
- **Verb tense roll.** Present → past on completion uses `dur.tense.roll` (240 ms); the target text
  stays put while the verb part rolls (status-text.md §5.4). Arc crossfades ~100 ms out / 160 ms in
  with a 2 px shift (note 12); our 240 ms total is in the same range.
- **Loader → glyph morph.** The row loader finishes its beat (≤ 200 ms), then morphs to ✓ / ✕ /
  ■ / ⊘ in 160 ms `spring.snap`; ✓ draws its stroke once (240 ms). Same as loaders.md §4.
- **Merge into group.** When a second same-class call arrives: the existing row's label crossfades
  into the group label, the new call does **not** add a row (it lands in the hidden list), and the
  header count rolls 1 → 2 (`dur.digit.roll`). No height change.
- **Expand / collapse.** Height on `spring.snap` 220 ms, content opacity 150 ms `ease.out` starting
  40 ms in (Cline uses a 250 ms max-height + 150 ms opacity expand, note 08 §1.7); collapse fades the
  content first (100 ms) then closes the height (180 ms `ease.in`). Chevron rotates 90° in 160 ms.
- **Failure.** ✕ morph + a 2-cycle 3 px nudge of the glyph only (loaders.md), error line slides in
  under the row; text never shakes.
- **Numbers roll, words swap** (status-text.md §7): counts in group headers, `+N −N`, elapsed.
- **Hover.** Row background tint 80 ms; after 400 ms a tooltip with full target, raw tool name,
  server, start time, exact duration, attempt.

### 5.2 Family-F motion tokens (proposed; added to `oxyn-core` motion presets)
| Token | Value | Used for |
|---|---|---|
| `dur.row.enter` | 200 ms height (`spring.snap`) + 160 ms fade, y +4 → 0 | new rows, new plan items, new files in a rollup |
| `dur.row.morph` | 160 ms `spring.snap` | loader → ✓ / ✕ / ■ / ⊘ |
| `dur.check.draw` | 240 ms `ease.out` | ✓ stroke draw |
| `dur.expand` / `dur.collapse` | 220 ms `spring.snap` (+150 ms content fade, 40 ms delay) / 100 ms fade + 180 ms `ease.in` | detail panes, groups, terminal blocks |
| `delay.row_guard` | 150 ms | a call that resolves inside this never shows a running state |
| `delay.auto_collapse` | 1200 ms | system-opened blocks collapse after success |
| `stagger.row` | 40 ms, cap 6 | bursts of rows |
| `dur.highlight.fade` | 1600 ms `ease.out` | "changed by agent" tint on plan items, new rollup files |
| `dur.count.roll` | = `dur.digit.roll` (280 ms) | group counts, `+N −N`, "3/7" |
| Error nudge | 300 ms, 2 × 3 px | failed glyph |
| **Reduced motion** | rows appear with an 80 ms fade, no height spring (instant height); tense change → 80 ms crossfade; counts swap without rolling; ✓ appears without stroke draw; no shimmer on args or active items; highlight tint is a static 2 px left bar for 4 s instead of a fade | — |

### 5.3 Timing table (family defaults)
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Row enter | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| Args shimmer (StreamingInput) | 1400 ms sweep / 800 ms rest | 600 ms | `ease.coast` | — | 30 Hz |
| Approved flash → Running | 400 ms hold | 0 | `ease.out` | — | 60 Hz |
| Verb tense roll | 240 ms | after loader beat | `ease.out` | — | 60 Hz |
| Loader → glyph morph | 160 ms + 240 ms stroke | ≤ 200 ms (beat) | `spring.snap` | — | 60 Hz |
| Success tone hold → `fg` | 600 ms hold, 240 ms fade | — | `ease.in` | — | 30 Hz |
| Expand / collapse | 220 ms / 280 ms total | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Auto-collapse after success | — | 1200 ms | as collapse | — | — |
| Group count roll | 280 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz |
| Stall tone ramp (per call) | 2000 ms | `tool_stall` 120 s | linear in OKLab | — | 30 Hz |
| Turn fold to `Worked for …` | 280 ms (rows collapse into one) | after the status line's tense roll | `ease.in-out` | 20 ms per row, cap 6 | 60 Hz |
| Error nudge | 300 ms | after morph | linear | — | 60 Hz |
| **Reduced motion** | 80 ms opacity crossfades; instant heights; no shimmer, no rolls, no stroke draw | — | linear | none | 1 Hz idle |

### 5.4 Busy signal and frame budget
- Each running row is its own `BusyRegion::ToolRow(id)`; a **collapsed group** is one region (its
  header animates, children are still). At most **8 rows animate** at once (the lists rule in
  loaders.md §2.3); rows nearest the viewport centre win; the rest show the still "working" frame.
- Rows repaint only for their loader lease and the 1 Hz timer; a settled transcript repaints nothing.
- Off-screen rows pause (loaders.md §2.1) and resume at the clock's phase.

## 6. Variants

### 6.1 `tool-stack/*` (composite transcript containers, owned by this file)
**Free (3)**
| Name | Idea | Surfaces |
|---|---|---|
| `tool-stack/transcript` | Default: rows and groups inline between prose, Normal density rules, errors pinned | chat, desktop, ide, mobile |
| `tool-stack/worked-for` | As transcript, but each finished turn folds its tools into one `Worked for 31s ›` row that re-expands | chat, desktop |
| `tool-stack/terminal` | TUI tree: `•` row bullets, dim `└` children, `Explored` / `Ran` verbs, 5-line previews (Codex layout, note 08) | terminal |

**Pro (7)**
| Name | One-line idea |
|---|---|
| `tool-stack/progress-line` | `Hidden` density as a designed variant: one shimmering live line that clears at turn end (Knowledge Work idea, note 08) |
| `tool-stack/side-rail` | Transcript shows only small chips; the full call list lives in a docked side rail that scrolls in sync |
| `tool-stack/step-card` | The whole turn is one card: steps stream in, the live one shimmers, everything folds to one line at the end (Arc run-card idea, note 12) |
| `tool-stack/lanes` | Parallel calls render as side-by-side lanes that merge back into one column when they finish |
| `tool-stack/digest` | End-of-turn digest card: "Read 14 files · ran 3 commands · edited 2 files +24 −0" with jump links |
| `tool-stack/filmstrip` | A horizontal scroller of call chips under the message; the selected chip opens its detail below |
| `tool-stack/focus-follow` | Auto-scrolls to and outlines the current call; the user's scroll pauses follow (follow-the-agent idea from Zed and Devin, note 03) |

### 6.2 Family F variant catalog
Names follow [../variants.md](../variants.md). **F** = free, **P** = Pro. Part-2 rows are
**reserved minimums** (3 free + 8 Pro each, proposed); their owning specs set the final names and may
change the counts.

| Component | Spec | F | P | Free names | Pro names |
|---|---|---|---|---|---|
| `tool-stack` | this file | 3 | 7 | transcript, worked-for, terminal | progress-line, side-rail, step-card, lanes, digest, filmstrip, focus-follow |
| `tool-call` | tool-call.md | 4 | 14 | row, card, terminal, minimal | chip, rail, json-tree, args-stream, semantic-loader, border-beam, deck, duration-bar, mcp-branded, retry-ladder, cost-chip, result-peek, split-io, ascii-box |
| `file-edit-chip` | file-edit-chip.md | 3 | 11 | counter, row, tree-rollup | odometer, block-bar, writing-sweep, review-bar, composer-accordion, hover-hunk, status-letters, stack-fan, viewed-progress, rename-morph, prose |
| `terminal-output` | terminal-output.md | 3 | 11 | block, tail, tui | follow-hud, split-streams, test-ticks, progress-native, rate-sparkline, pty-live, runnable-snippet, window-chrome, section-fold, hold-to-kill, dock-to-jobs |
| `todo-list` / `plan-card` / `step-timeline` | plan-todo.md | 4 | 14 | todo-list/checklist, todo-list/dock, plan-card/document, step-timeline/vertical | todo-list/ring, todo-list/capsules, todo-list/owner-lanes, todo-list/strike-sweep, todo-list/board, plan-card/versioned, plan-card/autonomy, plan-card/countdown, plan-card/flowchart, plan-card/plan-vs-actual, step-timeline/fold-to-line, step-timeline/stepper, step-timeline/metro, step-timeline/filter |
| `subagent-tree` | subagent-tree.md | 5 | 14 | indented, rail, dock, inline-wrapper, terminal | see subagent-tree.md §6 |
| `fanout-grid` | fanout-grid.md | 3 | 8 | *reserved* | *reserved* |
| `acting-indicator` | acting-indicator.md | 3 | 8 | *reserved* | *reserved* |
| `trace-view` | trace-view.md | 3 | 8 | *reserved* | *reserved* |
| `background-jobs` | background-jobs.md | 3 | 8 | *reserved* | *reserved* |
| `run-replay` | run-replay.md | 3 | 8 | *reserved* | *reserved* |
| `run-meter` | run-meter.md | 3 | 8 | *reserved* | *reserved* |
| **Part 1 subtotal** | | **17** | **57** | | **74** |
| **Part 2 reserved** | | **21** | **56** | | **77** |
| **Family F total** | | **38** | **113** | | **151** (target 50+) |

Counting rule (same as loaders.md §6): a variant differs in structure or motion. Density, colour,
icon set, size and speed are tokens or settings and are **not** counted.

## 7. Edge cases (family level)
- **Very fast agents.** Calls resolving inside `delay.row_guard` (150 ms) render final; a burst of 40
  reads in 300 ms produces one group header whose count rolls once to the final number (rolls are
  coalesced: a new target mid-roll continues from the current position, elapsed-timer.md §5).
- **Very slow tools (minutes to hours).** Timer switches to Units format; the per-call stall overlay
  starts at `tool_stall`; after 10 min the row loader drops to `loop.calm` (loaders.md §7).
- **Hundreds of calls in a turn.** Rows are virtualised beyond 200 in the transcript (proposed);
  group caps (§3.1.7); only 8 loaders animate.
- **Out-of-order events.** A result for an unknown call id creates the row in its final state; a
  late `TOOL_CALL_ARGS` after a result is ignored for display (kept in the trace).
- **Missing durations.** Providers that send no start event: duration shows "—", never "0s".
- **Long targets.** Middle-truncate paths (`src/…/auth.rs`), keep the basename; queries end-truncate
  at 32 chars; full text in the tooltip and accessible name.
- **RTL.** Rows mirror (icon at inline-start, duration at inline-end); paths, commands, code and
  `+12 −3` sit in LTR bidi isolates; chevrons point inline-end when collapsed.
- **Tiny sizes (16–20 px rows).** Icon only, no target, `terse` verb pack, duration hidden until hover;
  groups show the count badge only.
- **Dense IDE panels.** `tool-call/minimal` + Compact density; 24 px rows; hit area still ≥ 24 px.
- **Touch.** 44 px rows; expand on tap anywhere in the row; Retry / Stop as 44 px trailing buttons;
  hover tooltips become a long-press sheet.
- **Slow networks / reconnects.** When the stream drops, running rows keep their last state and the
  stall overlay applies; on reconnect the adapter replays the `RunLog` snapshot and rows reconcile by
  id without re-entering (no enter animation for rows that already existed).
- **Sensitive values.** Arguments and output pass through a redaction hook (`Redactor`) before render;
  masked values show as `••••` with a "hidden" tag (Codex masks MCP secrets, note 08 §8).
- **Restored threads.** Reopening an old thread renders every call settled with no animation.

## 8. Accessibility (family level)
- **Structure.** The tool stack is a list (`role="list"`) of rows; a group is a list item containing
  a button (`aria-expanded`, `aria-controls`) and a nested list. Each row's accessible name is
  "<past or present verb> <target>, <state>, <duration>", e.g. "Read auth.rs, succeeded, 0.8 seconds".
- **Live announcements.** Tool rows **never** own a live region (loaders.md §8). The composer's single
  polite region announces: a failed call once per call ("Command failed: cargo test, exit 101"),
  approvals assertively (owned by family G), stalls at `stall_hard` only. Verb changes are debounced
  to once per 10 s; pinned tool verbs are announced only if they last ≥ 2 s (status-text.md §11).
- **`aria-busy`** sits on the running row's detail pane, not on the loader; removed on any terminal state.
- **Meaning never by colour alone.** Every state has a glyph (○ ✓ ✕ ■ ⊘ ⏸ key) and a word; diff
  counts carry `+`/`−` signs; the theme can swap diff green for blue (Claude Code's daltonized
  themes, note 08 §1.3) through tokens.
- **Contrast.** Row text ≥ 4.5:1; glyphs and loaders ≥ 3:1; dimmed (settled) rows stay ≥ 4.5:1 for
  the target text.
- **Keyboard.** ↑/↓ moves between rows when the stack has focus; Enter / Space expand; `E` expand or
  collapse all in the turn; R retry; C copy; Esc stops a running shell; Home / End jump to the first /
  last row of the turn. Focus never moves on its own when rows arrive.
- **Reduced motion.** §5.2; state is always carried by glyph and text.

## 9. Props sketch (shared types in `oxyn-core::run`)
```rust
/// One per tool invocation; built by adapters, owned by the RunLog.
pub struct ToolCall {
    pub id: CallId,
    pub kind: ToolKind,                 // Read | List | Search | Edit | Shell | Web | Mcp | Plan | Subagent | Media | Generic
    pub name: SharedString,             // raw tool name from the protocol
    pub server: Option<McpServerRef>,   // MCP server id, display name, brand mark, transport, status
    pub target: Option<Target>,         // Path(..) | Query(..) | Url(..) | Command(..) | Label(..)
    pub args: ArgsState,                // Streaming(partial) | Complete(Value) | Redacted
    pub state: ToolCallState,
    pub clock: ChildClockRef,           // child of RunClock: started, ended, paused intervals
    pub output: Option<OutputRef>,      // text, JSON, terminal buffer, file list
    pub attempt: Attempt,               // { n: u8, max: u8, next_retry_at: Option<Instant> }
    pub parent: Option<CallId>,         // sub-agent nesting
    pub provenance: Provenance,         // Asked | AutoApproved(PermissionLevel) | Rule(RuleId) | User
}

pub enum ToolCallState {
    Pending, StreamingInput, AwaitingApproval { request: ApprovalId }, Approved,
    Running, StreamingOutput,
    Succeeded { summary: Option<SharedString> },
    Failed { kind: ToolErrorKind, code: Option<i32> },   // Exit | Timeout | Protocol | Tool | Offline
    Cancelled { by: CancelledBy },                          // User | RunStopped | ApprovalTimeout
    Denied { reason: Option<SharedString> },
    AuthNeeded,
}

/// What every family-F component receives.
pub struct ToolStack<'a> {
    pub agent: &'a AgentState,          // aggregate state (architecture.md)
    pub calls: &'a [ToolCall],          // this turn's calls from the RunLog
    pub variant: ToolStackVariant,      // Transcript | WorkedFor | Terminal | Pro(..)
    pub density: ToolDensity,           // Hidden | Compact | Normal (default) | Verbose
    pub grouping: GroupingRules,        // classes, min 2, parallel window 50 ms, caps 50
    pub collapse: CollapseRules,        // auto_collapse 1200 ms, expand_on_approval, errors_open
    pub kind_map: ToolKindMap,          // raw name → ToolKind (overridable)
    pub stall: StallThresholds,         // shared: soft 5 s, hard 45 s, tool 120 s
    pub redactor: Option<Redactor>,
    pub labels: ToolLabels,             // i18n templates incl. plural rules
    pub motion: MotionLevel,            // None | Subtle | Expressive
}

impl ToolStack<'_> {
    /// Pure: the RunLog slice → rows and groups (§3). Tested with snapshot fixtures.
    pub fn layout(&self) -> Vec<StackItem> { /* … */ }
}
```

## 10. Related components and references
**Related:** status strip pinned verb ([composer-status-strip.md](composer-status-strip.md)) ·
status-text tense rules ([status-text.md](status-text.md)) · loaders and busy regions
([loaders.md](loaders.md)) · elapsed timer `Decimal` format ([elapsed-timer.md](elapsed-timer.md)) ·
skeletons for tool cards ([skeletons.md](skeletons.md)) · border beam on running cards
([border-beam.md](border-beam.md)) · permission selector auto-run provenance
([composer-permission-selector.md](composer-permission-selector.md)) · mode switch Plan → Agent
([composer-mode-switch.md](composer-mode-switch.md)) · family G approvals · family H diff viewer ·
family J MCP list and terminal tabs.

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI `exec_cell`, `diff_render` (note 08 §1.1) | `• Explored` coalescing, `Running → Ran`, 5-line preview, `(exit N)`, search exit 1 not a failure | licensing.md lists MIT, note 08 lists Apache-2.0: either way portable with attribution; re-check before porting |
| Claude Code tool row (note 08 §1.3) | 600 ms blink until resolved, `⎿` result gutter, Ctrl+O expand, daltonized diff themes | Proprietary — reference only |
| Warp diff choreography (note 08 §1.10) | expanded while awaiting approval, collapsed after apply; `E` toggles all | AGPL — study only |
| Antigravity (note 08 §1.11) | `Worked for 31s ›` fold, live progress line mode, count of running tools without flicker | Proprietary — reference only |
| Cline (note 08 §1.7) | 250 ms + 150 ms expand timing, 500 ms thinking grace | Apache-2.0 — port with attribution |
| OpenCode / Kilo (note 08) | Soft Radar on tool cards, phase sync, offscreen pause | MIT — port with attribution |
| AI SDK v6, AG-UI, MCP / Claude Agent SDK statuses (note 05 §0) | state mapping table §2.3 | Protocols — we implement the mapping, no code copied |
| affectUI Tool Call Card (note 11 §15) | one-line call with timer, fold-open args + result, retry, group summary | Paid — study only |
| Arc agent run card (note 12) | spring row enter, past-tense crossfade, fold-to-line at end | Paid — study only |
| Beautiful UI Tool Chips / Task Rows [R106] [R133] | chip summaries, nested sub-steps, counts | MIT — may port with attribution |
| Cost waterfall post [R030] | failed steps auto-expand | Study only |

## Plan #28 parity (2026-11-14)

New in plan #28: [tool-call-card](tool-call-card.md), [tool-call-transcript](tool-call-transcript.md), [tool-chips](tool-chips.md), [tool-kind-map](tool-kind-map.md).
