**Status:** v0.2 (2026-11-12), implemented in plan #20: `oxyn-core::run` (ToolCall, row_view), Dioxus `ToolCallRow` (4 free), GPUI `ToolCallRowEl` (row).

# Spec: tool call (rows, cards, group rows, MCP badge)

Part of family F ([tools.md](tools.md) — event model, grouping, collapse and density rules live
there and are not repeated here). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **4 free + 14 Pro variants**.

## 1. Purpose
The tool-call row is the most repeated element in an agent transcript: one per file read, search,
command, web fetch or MCP call. The user glances at it to see **what the agent touched and whether it
worked**, opens it to check **exactly what was sent and returned**, and acts on it when it fails
(**retry**, read stderr) or runs long (**stop**). It has to be calm at rest (dozens settle per turn),
legible at 16–20 px in an IDE, and loud only on failure.

Every coding agent has one (research note 03, "Tool-call row / card": table-stakes). Codex shows
cyan verb labels in a `•`/`└` tree; Claude Code blinks a bullet at 600 ms until it resolves and hangs
the result off a `⎿` gutter; OpenCode puts its Soft Radar spinner on tool cards; Paradev uses a
monospaced header / input / output with left icons and a braille spinner (notes 03, 08). affectUI's
Tool Call Card is "a single line — live timer, status, and summary; folds open to highlighted
arguments and the result with Copy; retry on errors; a group sums up multiple calls" (note 11 §15).
Our row covers all of that plus approval hand-off, MCP identity, stalls and redaction.

## 2. Anatomy
```
 row (collapsed, 32 px chat / 24 px ide / 44 px mobile)
 ┌──────────────────────────────────────────────────────────────────────────────────────────┐
 │ [icon|loader 16] 8 [Verb] 4 [target]  6 [args preview, dim, truncates first]  [MCP badge] [0.8s] [›] │
 └──────────────────────────────────────────────────────────────────────────────────────────┘
 error line (Failed only, always visible):  ✕ exit 101 · "error[E0425]: cannot find value `x`"   [Retry]
 detail (expanded):
 ┌ Input ─────────────────────────────── Copy ┐   Structured args → JSON tree; command → `$ cmd`
 │ { "path": "src/auth.rs", "range": [1,80] } │   Long input: 12 lines, then "Show all (84 lines)"
 ├ Output ────────────────────────────── Copy ┤   Text / JSON / file list / terminal (terminal-output.md)
 │ …                                          │   200 lines max, then "Open full output"
 └ stderr (Failed) ──────────────────── Copy ┘   last 5 lines visible, rest behind "Show all"
 group row:  [loader|✓] Read 6 files, searched 2 patterns  · 3.4s  [›]   → children are tool-call rows
```
- **Icon per kind** from [tools.md](tools.md) §2.4 (read page, edit pencil, search magnifier, shell
  `›_`, web globe, MCP server mark or plug). While the call is live the icon slot holds the row
  loader (default `loader/ring-comet` at 16 px, or the semantic activity loader, loaders.md §3.4);
  on settle the loader morphs into the state glyph, then back to the kind icon after the 600 ms
  success hold (failed rows keep ✕).
- **Arguments preview** (proposed): kinds with a natural target show only the target. `Mcp` and
  `Generic` show up to 2 `key: value` pairs, string values truncated to 24 chars, objects as `{…}`,
  arrays as `[3]`; secrets redacted (`token: ••••`). The preview truncates before the verb or target.
- **Duration**: elapsed-timer `Decimal` under 10 s (`0.8s`), Units above (`1m 05s`); appears at 1 s
  for live calls; paused (⏸) while awaiting approval; never "0s".
- **MCP server badge**: 16 px server mark (or initial in a rounded square) + server name in 11 px;
  status dot when the server is not `connected`. States from research note 05 §6:
  `connected` (no dot), `connecting`, `needs-auth` (key glyph + "Needs sign-in"),
  `tools-fetch-failed`, `failed`, `disabled`, `pending-approval`, `rejected`. Hover: transport
  (stdio / HTTP), scope (project / user), tool count.

## 3. State machine (row UI)
The call lifecycle is [tools.md](tools.md) §2.2. The row adds **presentation** states on top: hover,
focus, pressed, expanded, retrying, copied.

```
                 call state changes (tools.md §2.2) drive the left column; user input drives the right
  ┌────────────── call ───────────────┐                    ┌────────── presentation ──────────┐
  Pending ─▶ StreamingInput ─▶ ┬─▶ Running ─▶ StreamingOutput     Rest ──hover──▶ Hovered ──press──▶ Pressed
                               └─▶ AwaitingApproval ─▶ Approved ┘   │ ▲             │ release = toggle
                                                                     │ └─ leave ─────┘
  StreamingOutput ─▶ Succeeded │ Failed │ Cancelled │ AuthNeeded     Collapsed ◀──Enter / click──▶ Expanded
  AwaitingApproval ─▶ Denied                                          (Auto: system-set; User: pinned, tools.md §4.1)
  Failed / Cancelled ──Retry──▶ Retrying(attempt n+1, ≤ 300 ms) ─▶ Pending
  Failed (agent auto-retry) ──backoff──▶ RetryScheduled{in} ─▶ Pending          Copy ─▶ Copied (1200 ms) ─▶ Rest
  Overlays: Stalled{soft,hard} · Disabled(reason) for actions (e.g. Retry unavailable: "Run ended")
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (row focused) |
|---|---|---|---|---|
| `Pending` | ○ muted icon, present verb, target, no timer | — | expand (empty Input) | Enter expand |
| `StreamingInput` | ○ + args preview shimmering; target appears as soon as it parses | — | expand (live args) | Enter |
| `AwaitingApproval` | amber hand glyph, "Waiting for you", ⏸ timer, detail auto-expanded; approval footer from family G inside the card | family G (Allow / Deny) | expand / collapse | Tab into approval |
| `Approved` | ✓ flash (amber) 400 ms | — | — | — |
| `Running` | loader, present verb, timer from 1 s | Stop (shell, MCP with cancel support) | expand | Enter, Esc stop |
| `StreamingOutput` | as Running; output pane grows (tail) when expanded; count in label for multi-result tools ("Fetched 3 of 5") | Stop | expand | Enter, Esc |
| `Succeeded` | ✓ (success tone 600 ms → kind icon), past verb, duration, short result summary ("12 matches") | expand | copy output | Enter, C copy |
| `Failed` | ✕ danger, past verb + reason, **error line always visible**, Retry | Retry | expand stderr, copy error | R retry, Enter |
| `Cancelled` | ■ desaturated, "Stopped after 4s" | Retry | expand partial output | R |
| `Denied` | ⊘, target struck through, "Denied: <reason>" | — | "Change rule" (family G) | Enter expand |
| `AuthNeeded` | key on the MCP badge, "Needs sign-in to <server>" | Connect (opens family J connect flow) | — | Enter connect |
| `RetryScheduled{in}` | ✕ + "Retrying in 3s · 2/3" countdown | Retry now | Cancel retry | R now, Esc cancel |
| `Retrying` | ✕ morphs to loader; attempt pill "2/3" | — | — | — |
| `Stalled` (overlay) | loader slows/stops, `warn` tone, "No output for 2m" | Stop | keep waiting | Esc |
| `Hovered` | row tint, chevron brightens; tooltip after 400 ms | — | — | — |
| `Expanded` | detail panes (Input, Output, stderr), each with Copy | collapse | copy, open full output | Enter collapse, Tab into panes |
| `Copied` | copy glyph → ✓ for 1200 ms | — | — | — |
| Group `Live` / `Settled` / `SettledWithErrors` | see tools.md §3.2 | expand | — | Enter, ←/→ collapse/expand (tree semantics) |

## 5. Micro-interactions
- **Enter.** `dur.row.enter` (height spring + fade, y +4 → 0); burst stagger 40 ms cap 6.
- **Args stream.** In `StreamingInput` the preview text appends left to right with no reflow of the
  verb (reserved slot); `text-shimmer/calm` sweeps it after 600 ms.
- **Start.** ○ → loader crossfade 120 ms; timer slot fades in at 1 s (`Width::ReserveMagnitude`).
- **Settle.** Loader finishes its beat, morphs to ✓ / ✕ / ■ / ⊘ (`dur.row.morph`), ✓ strokes once
  (`dur.check.draw`); verb rolls to past tense (`dur.tense.roll` 240 ms); duration freezes. The result
  summary fades in after the roll (120 ms).
- **Failure.** ✕ + 2-cycle 3 px glyph nudge; the error line opens under the row with
  `dur.row.enter`; Retry button fades in 120 ms. The row never shakes.
- **Retry.** Pressing Retry: button press scale 0.94 for 80 ms; ✕ crossfades to the loader; the
  error line collapses (180 ms); an attempt pill "2/3" pops in (scale 0.8 → 1, `spring.snap`).
  Agent-scheduled retries show a countdown with digits swapping each second.
- **Approval hand-off.** Entering `AwaitingApproval` expands the detail and slides the family-G footer
  in from the card's bottom edge (200 ms `ease.out`); on approve the footer collapses into a one-line
  "Allowed once" note (affectUI's approval card morphs into a resolved row, note 11 §17) and the row
  continues as Running; on deny it morphs to `Denied`.
- **Expand / collapse.** `dur.expand` / `dur.collapse`; chevron 90° in 160 ms. Expanding a row
  never scrolls the transcript unless the row's header would leave the viewport (then the header is
  kept in place, content grows downward).
- **Copy.** Glyph swaps to ✓ with a scale 0.9 → 1 and 4 px blur → 0 (Arc's JSON viewer copy feedback,
  note 12), reverts after 1200 ms.
- **MCP badge.** On `needs-auth` the key glyph pops in once (scale 0 → 1, `spring.snap`); after a
  successful connect the badge dot fades out and the row re-runs (Running) without re-entering.
- **Group merge.** See tools.md §5.1: the first row's label crossfades to the group label; count rolls.
- **Hover.** Background tint 80 ms; tooltip after 400 ms: raw tool name, server, full target, start
  time, exact duration, attempt, provenance ("Ran automatically · Auto-edit", from
  composer-permission-selector.md).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Row enter | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| ○ → loader | 120 ms crossfade | 0 | `ease.out` | — | 60 Hz |
| Args shimmer | 1400 / 800 ms | 600 ms | `ease.coast` | — | 30 Hz |
| Timer appear | 120 ms | at 1.0 s | `ease.out` | — | 30 Hz |
| Decimal timer resolution | 100 ms | — | — | — | 10 Hz while visible and < 10 s |
| Loader → glyph | 160 ms + 240 ms stroke | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Verb tense roll | 240 ms | after morph | `ease.out` | — | 60 Hz |
| Success tone hold → icon | 600 ms + 240 ms | — | `ease.in` | — | 30 Hz |
| Error line open | 200 ms | after nudge | `spring.snap` | — | 60 Hz |
| Error nudge | 300 ms, 2 × 3 px | after morph | linear | — | 60 Hz |
| Retry press / pill pop | 80 ms / 160 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Approval footer in / resolve | 200 ms / 220 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Expand / collapse | 220 ms / 280 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Chevron rotate | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Copy ✓ / revert | 160 ms / after 1200 ms | 0 | `spring.snap` | — | 60 Hz |
| Tooltip | 120 ms | 400 ms | `ease.out` | — | 60 Hz |
| **Reduced motion** | 80 ms opacity crossfades for every change; instant heights; no shimmer, no nudge, no stroke draw, no pill pop; timer and countdown digits swap | — | linear | none | 1 Hz idle |

## 7. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `tool-call/row` | Default single line: icon/loader, verb, target, preview, duration, chevron; detail opens below | chat, desktop, ide, mobile |
| `tool-call/card` | Bordered card with header row and separate Input / Output / stderr panes; better for MCP and long results | chat, desktop |
| `tool-call/terminal` | TUI: `•` bullet that blinks at 600 ms while live (Claude Code's cadence, note 08) and turns ✓/✕ coloured, dim `└` result line, `(exit N)` | terminal |
| `tool-call/minimal` | Text only, no icon: "Read auth.rs · 0.8s"; glyph for failure; for dense IDE panels and 16–20 px rows | ide |

### Pro (14)
| Name | One-line idea |
|---|---|
| `tool-call/chip` | Inline pill chip in prose ("⌕ 12 matches"); click opens a popover with the full call (Beautiful UI tool chips idea [R106]) |
| `tool-call/rail` | Vertical rail with a marker per call; the rail fills downward as calls finish (step-card look, note 12) |
| `tool-call/json-tree` | Expanded args and result as a collapsible JSON tree with search and copy-path (Arc JSON viewer idea, note 12) |
| `tool-call/args-stream` | Arguments type out key by key during `StreamingInput` with a caret, then compact into the preview |
| `tool-call/semantic-loader` | The kind icon itself animates (magnifier sweeps, page scans, pencil writes) instead of a separate loader |
| `tool-call/border-beam` | A light travels the card border while running and stops on settle ([border-beam.md](border-beam.md)) |
| `tool-call/deck` | Group rows render as a fanned deck of cards that spreads open on expand |
| `tool-call/duration-bar` | A thin bar under the row shows this call's duration against the turn's longest call |
| `tool-call/mcp-branded` | Large server mark, tool name in mono, read/write access glyph and scope chip (MCP list idea, note 12) |
| `tool-call/retry-ladder` | Attempts as a row of dots (✕ ✕ ✓) with the backoff countdown drawn as a shrinking arc |
| `tool-call/cost-chip` | Adds tokens and cost for the call (only when the app supplies a price table, elapsed-timer.md §4) |
| `tool-call/result-peek` | Hover shows a rich preview: file snippet, favicon + page title, first rows of a result table |
| `tool-call/split-io` | Expanded detail is side by side, Input left and Output right, for wide IDE panels |
| `tool-call/ascii-box` | Box-drawing frame (`┌─ Read ─┐`) with a block-glyph progress fill, for TUI and retro themes |

## 8. Edge cases
- **Very fast calls.** Inside `delay.row_guard` (150 ms) the row renders final: no ○, no loader, no
  tense roll (it enters in past tense).
- **Very slow calls.** Units timer; stall overlay at `tool_stall` 120 s; Stop stays available.
- **Huge arguments** (pasted files, base64): preview shows `{…}` / `"…" (48 KB)`; Input pane shows 12
  lines then "Show all (84 lines)"; binary or > 1 MB values show size + "Download".
- **Huge output.** Output pane renders 200 lines, then "Open full output" (opens a panel or file);
  JSON trees page arrays over 100 items.
- **Streaming JSON args that never close.** The preview keeps the last parsable state; the Input pane
  shows raw text with a "partial" tag.
- **Error with no stderr.** The error line shows the protocol error kind ("Timed out after 30s",
  "Tool error"); never an empty line.
- **Exit codes that are not failures.** Per-kind `ok_exit_codes`: search exit 1 = "No matches"
  (neutral, Codex's rule, note 08).
- **Unknown tool names.** `Generic` kind, raw name shown in mono, JSON detail.
- **MCP server disappears mid-call.** Row → `Failed { kind: Protocol }`, badge shows `failed`;
  Retry is disabled with reason "Server disconnected" until it reconnects.
- **Retry after the run ended.** Retry runs the call again as a new user-initiated turn (the adapter
  decides); if unsupported, Retry shows `Disabled("Run ended")` with the reason in its tooltip.
- **Long targets / RTL.** Middle-truncation for paths, LTR isolates for paths and code in RTL layouts;
  row mirrors.
- **Tiny sizes.** At 16–20 px: icon + verb only (`terse` pack), duration and badge on hover, error as
  a ✕ glyph with the line below.
- **Touch.** Whole row toggles; Retry / Stop are 44 px trailing buttons; Copy appears in the expanded
  panes, not on hover.
- **Offline.** Running rows freeze with the stall overlay; Retry disabled with "Offline".

## 9. Accessibility
- **Role.** Row = button (`aria-expanded`, `aria-controls` → detail). Detail panes are regions labelled
  "Input", "Output", "Error output". Group rows use tree semantics (`role="treeitem"`, ←/→).
- **Accessible names per state:** Pending "Read auth.rs, queued"; StreamingInput "Preparing read";
  AwaitingApproval "Run cargo test, waiting for your approval"; Running "Running cargo test, 12
  seconds"; Succeeded "Read auth.rs, succeeded, 0.8 seconds"; Failed "Ran cargo test, failed, exit code
  101"; Cancelled "Stopped cargo test after 4 seconds"; Denied "Denied: write outside workspace";
  AuthNeeded "Linear needs sign-in"; RetryScheduled "Retrying in 3 seconds, attempt 2 of 3". MCP badge
  is part of the name ("via Linear").
- **Live region.** None on the row (tools.md §8). Failures are announced once by the composer's polite
  region; retries announce "Retrying cargo test".
- **Keyboard.** Enter / Space toggle; R retry; C copy the focused pane; Esc stop or cancel a scheduled
  retry; Tab moves into panes and their Copy buttons, Shift+Tab back to the row.
- **Contrast.** Verb and target ≥ 4.5:1; args preview (dim) ≥ 4.5:1 in default themes (it is real
  text); glyphs ≥ 3:1; the error line in `danger` ≥ 4.5:1 on its tint.
- **Focus.** 2 px ring around the full row (inset in dense lists so rows don't overlap); focus stays on
  the row when it expands or settles.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct ToolCallRow<'a> {
    pub call: &'a ToolCall,               // tools.md §9; state, kind, server, args, clock, output
    pub agent: &'a AgentState,            // for run-level context (e.g. Retry disabled after Cancelled run)
    pub variant: ToolCallVariant,         // Row | Card | Terminal | Minimal | Pro(..)
    pub density: ToolDensity,             // from the stack
    pub expansion: Expansion,             // Auto (system rules) | User(bool) (pinned)
    pub args_preview: ArgsPreview,        // TargetOnly | Pairs { max: 2, str_max: 24 } | None
    pub duration: DurationDisplay,        // Auto (Decimal < 10 s, then Units) | Hidden
    pub detail: DetailRenderers,          // per kind: Text | Json | Terminal | FileList | Custom(fn)
    pub output_cap: OutputCap,            // lines: 200, input_lines: 12, stderr_tail: 5
    pub ok_exit_codes: OkExitCodes,       // per kind, e.g. Search → [0, 1]
    pub retry: RetryPolicy,               // UserOnly | AgentBackoff { max: 3, base: 1 s, factor: 2 }
    pub labels: ToolCallLabels,           // i18n + screen reader templates
    pub on_retry: Option<Callback<CallId>>,
    pub on_stop: Option<Callback<CallId>>,
    pub on_connect: Option<Callback<McpServerId>>,
    pub motion: MotionLevel,
}

pub struct ToolGroupRow<'a> {
    pub calls: &'a [ToolCall],            // same class, from ToolStack::layout()
    pub class: GroupClass,                // Explore | Web | Mcp(server) | Parallel
    pub expansion: Expansion,
    pub cap: usize,                       // 50, then "Show N more"
    pub labels: GroupLabels,              // composed templates with CLDR plurals
}
```

## 11. Related components and references
**Related:** [tools.md](tools.md) (model, grouping, density) · [file-edit-chip.md](file-edit-chip.md)
(edit calls render as chips) · [terminal-output.md](terminal-output.md) (shell detail) ·
[plan-todo.md](plan-todo.md) (plan calls) · loaders ([loader-glyph.md](loader-glyph.md),
[loader-vector.md](loader-vector.md)) · status-text tense roll · family G approval footer · family J
MCP server list (connect flow).

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI exec cell (note 08 §1.1) | verb labels, `└` tree, `(exit N)`, search exit 1 neutral, secret masking in `/mcp` | MIT per licensing.md / Apache-2.0 per note 08 — portable with attribution; re-check |
| Claude Code tool row (note 08 §1.3) | 600 ms blink while unresolved, settle to green/red, `⎿` gutter, Ctrl+O | Proprietary — reference only |
| OpenCode / Kilo (note 08 §1.5–1.6) | Soft Radar on tool cards | MIT — port with attribution |
| Paradev tool header / input / output (note 03) | left icons + mono header layout | Proprietary — reference only |
| affectUI Tool Call Card, Approval Card (note 11 §15, §17) | one-line call, fold-open args/result, retry, group sum, resolved-row morph | Paid — study only |
| Arc agent run card and JSON viewer (note 12) | spring enter, past-tense crossfade, copy ✓ feedback, JSON tree | Paid — study only |
| MCP server states (note 05 §6; Claude Code `mcp list`, Cursor) | badge status set | Reference only (we define our own set) |
| Beautiful UI tool chips [R106] [R133] | chip summary pattern | MIT — may port with attribution |
| Wensity AI Agent UI [R119] | per-tool verb/icon renderers | Pro tier — study only |
| "Tool calls should look this good" post [R072] | motion quality bar | Study only |

## Plan #28 parity (2026-11-14)

Closed with [tool-call-card](tool-call-card.md) (open on error, capped code, group summary), [tool-call-transcript](tool-call-transcript.md) (approval / denied, list-table-JSON-text output, auto-collapsing group), [tool-chips](tool-chips.md), [json-viewer](json-viewer.md), [expandable-card](expandable-card.md) and the shared [tool-kind-map](tool-kind-map.md).
