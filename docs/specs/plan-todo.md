**Status:** draft v0.1 (2026-11-11).

# Spec: plans & todos (todo list, plan card, step timeline)

Part of family F ([tools.md](tools.md) — event model, collapse and density rules live there). Same
depth as the worked example [composer-send-control.md](composer-send-control.md). Three components,
one item model: **`todo-list`** (the agent's checklist), **`plan-card`** (a proposed plan the user
approves), **`step-timeline`** (the executed steps with their tool calls nested). Target **4 free + 14
Pro variants** across the three.

## 1. Purpose
For any task longer than a few calls the user wants to know **what the agent intends to do, how far
along it is, and whether the plan changed**. Before execution they want to **review and approve (or
edit) the plan**; during it they want a calm "3/7 · Running tests" they can glance at; afterwards, a
record of what happened step by step. Research note 03 rates the todo checklist and plan document as
table-stakes: Claude Code's task list toggles with Ctrl+T (pending / in_progress / completed); Windsurf
keeps an in-conversation todo list maintained by a background planning agent; Warp auto-creates task
lists with status icons; Devin's Progress tab has a task drawer that filters the activity stream;
Conductor's todos can block merging; Kiro runs `tasks.md` items individually or all; Cursor's plan is an
editable document with a Build button and minimises to a "Plan" pill above the prompt; Lovable versions
plans ("Viewing version 2 of 3"); Jules has Approve plan with an auto-approve timer; Factory combines
"proceed" with an autonomy level in one approval (notes 03, 05, 08). Hoplite shows a collapsible
"Agent tasks" dock above the composer with pending / in-progress / completed / cancelled (note 10).

Where it appears:
- **Plan card** in the transcript when the agent proposes a plan (Plan mode,
  [composer-mode-switch.md](composer-mode-switch.md)), with the approval footer from family G.
- **Todo dock** above the composer while the run executes (collapsed to one line by default).
- **Todo list inline** in the transcript where the agent first writes or rewrites it (`Plan` tool
  calls, [tools.md](tools.md) §2.4).
- **Step timeline** in a side panel (IDE) or as the turn's structure (`tool-stack/step-card`).

## 2. Item model and progress

### 2.1 Item states (shared by all three components)
Research note 05 §5 lists `pending | in_progress | completed | blocked (waiting on #n) |
cancelled/deleted | failed`. We use:

| `ItemStatus` | Glyph (16 px) | Label tone | Notes |
|---|---|---|---|
| `Pending` | ○ hollow | `fg` secondary | not started |
| `Active` | row loader (busy region) | `fg` + `text-shimmer/calm` on the label | present-tense label if the item has a verb form ("Running tests") |
| `Done` | ✓ (stroke draws once) | `fg` secondary; optional strikethrough per variant | past tense if available ("Ran tests") + duration |
| `Skipped` | ⤼ | secondary, italic, "skipped" tag | agent decided it was unnecessary; reason in tooltip |
| `Failed` | ✕ `danger` | `fg`, reason line under it | the run may continue; stays visible |
| `Blocked { on }` | ⧗ `waiting` | secondary + "waiting on #3" | dependency or user input |
| `Cancelled` | — (dash) | secondary, struck | removed by the agent or the run stopped; see §5 for display rules |

### 2.2 Progress (proposed)
- **"3/7"** = `Done` ÷ (all items − `Skipped` − `Cancelled`). Skipped and failed are shown beside it:
  "3/7 · 1 skipped · 1 failed". Sub-items do not count toward the parent list's total (the parent
  counts once).
- A list of one item hides the fraction. A list where every counted item is done shows "Done · 7/7"
  with the success tone for 600 ms, then `fg`.
- The fraction uses tabular digits; the numerator rolls (`dur.count.roll`); the denominator rolls
  when the agent adds or removes items.
- Bars and rings (Pro) show `Done` as the solid fill, `Failed` as a `danger` segment, `Active` as a
  shimmering partial segment; skipped items are removed from the scale.

## 3. State machines

### 3.1 Item
```
          agent writes item               agent starts it                agent completes it
 (none) ───────────────────▶ Pending ─────────────────────▶ Active ───────────────────────▶ Done
                              │  ▲                            │  │
                              │  └──── agent reopens ─────────┼──┼──── (from Done/Failed: "reopened" amber tag)
                              │                               │  ├── error ──▶ Failed ──agent retries──▶ Active
                              ├── dependency / needs user ────┼──┴──▶ Blocked{on} ──unblocked──▶ Active
                              ├── agent decides not needed ───┴────▶ Skipped
                              └── agent removes / run stopped ─────▶ Cancelled
  Presentation overlays: Changed{added | reworded | moved | removed} (highlight, §5) · Hovered · Focused · Editing (plan card, Proposed only)
```

### 3.2 Plan card
```
          agent starts planning           first step arrives               plan complete
 (none) ───────────────────────▶ Drafting ─────────────────▶ Streaming ───────────────────▶ Proposed
                                 (skeleton)                  (steps enter one by one)          │
                                                                                               │ approval required
                                                                                               ▼
                       ┌──────────── Request changes (note) ──────────────────────── AwaitingApproval (family G footer)
                       ▼                                                                │          │
                   Revising ──agent rewrites──▶ Streaming (version n+1; vn → Superseded)  │ Reject   │ Approve (+ autonomy, + user edits)
                                                                                         ▼          ▼
                                                                                     Rejected    Approved ──(≤ 600 ms)──▶ Executing
                                                                                                                   │ (card → "Plan 0/7" pill,
                                                                                                                   │  items drive the todo dock)
                                          Interrupted ◀── run stopped / error ──────────────────────────────────────┤
                                          Done ◀───────────── all counted items Done ───────────────────────────────┘
  User edits allowed in Proposed and AwaitingApproval (inline text, reorder, add, delete). During Executing a user edit
  becomes a steering note (composer-queue-stack.md), not a silent plan change.
  Auto-approve (Pro plan-card/countdown only, opt-in): AwaitingApproval ──timer expires──▶ Approved; hover / focus pauses it.
```

### 3.3 Todo dock (above the composer)
```
  Hidden ──list appears──▶ Collapsed("Tasks 3/7 · Running tests") ◀──toggle──▶ Expanded(list, max 50 % of viewport)
     ▲                         │ all done                     │ run ends with items left
     └── fade after 4 s ── Completed("Done · 7/7") ◀────────────┘── Stopped("Stopped · 4 left") ── stays until next run / dismiss
```

## 4. States table

### 4.1 Todo list / dock
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| Dock `Collapsed` | progress "3/7" + current active label (shimmer) + mini bar | expand | — | proposed ⌘/Ctrl+Alt+T toggles (app-configurable; Claude Code uses Ctrl+T in its CLI) |
| Dock `Expanded` | full list, active item scrolled into view, header with progress | collapse | jump to step in transcript | ↑/↓ items, Enter jump, Esc collapse |
| List `Live` | items with glyphs; one (or more, parallel) active with shimmer | click item → scroll to its step / tool calls | — | ↑/↓, Enter |
| List `Changed` (overlay) | highlight tints on changed items + "Plan updated · 2 added, 1 removed" note | "Show changes" (previous version) | dismiss note | — |
| Dock `Completed` | "Done · 7/7", success tone 600 ms, fades after 4 s | — | reopen list | — |
| Dock `Stopped` | "Stopped · 4 left" | Resume (sends a continue turn, app-defined) | dismiss | Enter resume |
| Item `Failed` | ✕ + reason line | retry item (if adapter supports) | jump to the failing call | R |
| Item `Blocked` | ⧗ "waiting on #3" or "waiting for you" | jump to blocker / approval | — | Enter |

### 4.2 Plan card
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Drafting` | title skeleton + 3 step skeleton bars ([skeletons.md](skeletons.md)), "Planning" with shimmer | — | Stop (composer) | — |
| `Streaming` | title, steps entering one by one, the newest shimmering | — | Stop | — |
| `Proposed` | full plan; inline editing affordances on hover; "Build" / "Approve" CTA (if no approval gate: Build) | Build | Edit, Copy as markdown | ↑/↓ steps, E edit step, Enter build |
| `AwaitingApproval` | plan + family G footer (Approve, Approve with autonomy, Request changes, Reject); version chip "v2" if revised | family G | Edit steps | Tab into footer; family G shortcuts |
| `Revising` | footer collapses to "Revising plan…" with shimmer; old version dims | — | Stop | — |
| `Approved` | ✓ "Approved" for ≤ 600 ms, then the card folds | — | — | — |
| `Executing` | card folded to a pill "Plan · 3/7" (in transcript) and the todo dock takes over | open plan | — | Enter opens |
| `Superseded` | dimmed card, "Replaced by v3" link | view latest | — | Enter |
| `Rejected` | dimmed card, "Rejected" + the user's note | — | — | — |
| `Interrupted` / `Done` | pill shows "Stopped · 4 left" / "Done · 7/7" | open plan | — | Enter |

### 4.3 Step timeline
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| Step `Pending` | ○ marker, label, connector unfilled | — | — | ↑/↓ |
| Step `Active` | loader marker, present-tense label with shimmer, live elapsed, nested tool rows streaming in | expand / collapse nested calls | jump to latest call | Enter, ←/→ |
| Step `Waiting` | amber marker, "Review changes to 2 files", timer ⏸, approval inline (family G) | family G | — | Tab |
| Step `Done` | ✓ marker, past-tense label, duration (`0.9s`), connector above fills, nested calls collapsed | expand | — | Enter |
| Step `Failed` | ✕ marker, reason, nested failing call pinned open | jump to call | retry step (if supported) | R |
| Timeline `Folded` (run end, variant) | one line "7 steps · 1m 12s" + result | expand | — | Enter |

## 5. Micro-interactions
- **Item enter.** `dur.row.enter` (height spring + fade, y +4 → 0); the initial list staggers 40 ms,
  cap 6, then the rest appear together (Arc's run card opens steps on a critically damped spring,
  note 12; affectUI's working panel shows steps one by one, note 11).
- **Becoming active.** ○ crossfades to the loader (120 ms); the label starts `text-shimmer/calm` after
  `dur.shimmer.delay` (600 ms); in the dock, the current label swaps with the verb-cycler fade
  (180 ms, `Width::ReserveLongest` so the dock never jitters).
- **Completing.** Loader finishes its beat, morphs to ✓ (`dur.row.morph`) and the stroke draws
  (240 ms); label rolls to past tense (`dur.tense.roll`); `strike-sweep` variants draw a strike line
  left → right over 200 ms. Numerator rolls. In step-timeline the connector to the next marker fills
  top → bottom (240 ms `ease.in-out`); when several steps complete at once the connectors fill one
  after another with a 60 ms stagger (Arc stepper behaviour, note 12).
- **Skip.** Label fades to secondary italic (160 ms), ⤼ appears, denominator rolls down.
- **Fail.** ✕ morph + 2-cycle 3 px glyph nudge, reason line opens under the item.
- **Agent edits mid-run** (agents often rewrite the whole list each time; we diff by item id, falling
  back to text similarity ≥ 0.6 when the adapter gives no ids — proposed):
  - *Added:* item enters at its position with `dur.row.enter` and a `color.plan.changed` tint that
    fades over `dur.highlight.fade` (1600 ms); a "new" tag stays 4 s.
  - *Reworded:* old text crossfades to new (200 ms); "edited" tag 4 s; hover shows the previous text.
  - *Moved:* FLIP glide to the new position (240 ms `spring.snap`); held until the pointer leaves the
    list, so nothing moves under the cursor.
  - *Removed:* a pending item gets a strike line (200 ms), holds 600 ms, then collapses (180 ms);
    **done items are never removed visually** — they stay with a "removed from plan" tag.
  - *Reopened:* a done item going back to pending/active gets an amber "reopened" tag.
  - A one-line note "Plan updated · 2 added, 1 removed" slides in under the header (160 ms) and fades
    after 4 s; "Show changes" opens the previous version beside the current one.
  - Rewrites arriving within 300 ms of each other are coalesced into one change set.
- **Plan streaming.** Steps enter as they arrive; the newest step's label shimmers until the next one
  starts; the title is laid out first so the card width never changes.
- **Approval hand-off.** Entering `AwaitingApproval`, the family-G footer rises from the card's bottom
  edge (200 ms `ease.out`). On **Approve**: footer collapses into "Approved" (220 ms `spring.snap`),
  the card folds to the "Plan · 0/7" pill (280 ms `ease.in-out`), the todo dock unfolds above the
  composer, and — when the app is in Plan mode — the Build/Approve button flies a small spark to the
  mode chip, which morphs Plan → Agent (the choreography defined in
  [composer-mode-switch.md](composer-mode-switch.md) §"Plan approved → Agent"). On **Request changes**:
  the footer swaps in place to the note field (height follows on `spring.snap`); after submit, the
  card dims to 60 % and a new version streams in below or in place (variant). On **Reject**: card dims,
  footer collapses to "Rejected".
- **User editing (Proposed).** Hover shows a drag handle and an edit pencil per step; editing opens an
  inline text field in the same row (no height jump for one-line steps); edited steps get a "you
  edited" tag that rides along with the approval.
- **Dock toggle.** Expand: list height opens with `dur.expand`, capped at 50 % of the viewport
  (scrolls inside); the active item is scrolled into view without animation if it is far away.
- **Done.** Dock shows "Done · 7/7" in success tone for 600 ms, then holds 4 s and fades out (240 ms).
- **Hover.** Items tint 80 ms; tooltip after 400 ms with duration, owner (sub-agent), reason for
  skipped / failed, and the time it changed.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Item / step enter | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| ○ → loader | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Active label shimmer | 1400 ms sweep / 800 ms rest | 600 ms | `ease.coast` | — | 30 Hz |
| Dock current-label swap | 180 ms | 0 | `ease.out` | — | 30 Hz |
| Loader → ✓ + stroke | 160 + 240 ms | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Tense roll | 240 ms | after morph | `ease.out` | — | 60 Hz |
| Strike sweep | 200 ms | after morph | `ease.out` | — | 60 Hz |
| Connector fill | 240 ms | 0 | `ease.in-out` | 60 ms per connector | 60 Hz |
| Progress numerator / denominator roll | 280 ms | 0 | `ease.out` | 30 ms | 30 Hz |
| Changed highlight fade | 1600 ms | 0 | `ease.out` | — | 30 Hz |
| Reworded crossfade | 200 ms | 0 | `ease.in-out` | — | 60 Hz |
| Moved glide (FLIP) | 240 ms | until pointer leaves | `spring.snap` | 20 ms | 60 Hz |
| Removed: strike / hold / collapse | 200 / 600 / 180 ms | 0 | `ease.out` / — / `ease.in` | — | 60 Hz |
| "Plan updated" note in / hold / out | 160 / 4000 / 240 ms | 0 | `ease.out` / — / `ease.in` | — | 60 Hz |
| Coalesce window for rewrites | 300 ms | — | — | — | — |
| Approval footer in | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Approved → fold to pill | 220 ms + 280 ms | 0 | `spring.snap` / `ease.in-out` | — | 60 Hz |
| Spark to mode chip | per composer-mode-switch.md | — | — | — | 60 Hz |
| Dock expand / collapse | 220 / 280 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Done tone / hold / fade | 600 / 4000 / 240 ms | 0 | — / — / `ease.in` | — | 30 Hz |
| Auto-approve countdown (Pro) | per second | — | digit swap | — | 1 Hz |
| **Reduced motion** | no shimmer (active item marked by the static ◐ glyph and a bold label); morphs, tense changes and swaps become 80 ms crossfades; no strike draw, no connector fill animation (fills instantly), no FLIP (items jump), no spark; changed items get a static 2 px inline-start bar for 4 s instead of a tint fade | — | linear | none | 1 Hz idle |

## 7. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `todo-list/checklist` | Inline list: glyph, label, duration; progress "3/7" header; changed-item highlights | chat, desktop, ide, mobile, terminal (glyphs `○ ◐ ✓ ✕ ⤼`) |
| `todo-list/dock` | Collapsible dock above the composer: one-line "Tasks 3/7 · current item" that expands into the checklist | chat, desktop, ide, mobile (bottom sheet) |
| `plan-card/document` | Plan card with title, numbered steps, inline edit in Proposed, family G footer, folds to a "Plan · 3/7" pill | chat, desktop, ide |
| `step-timeline/vertical` | Vertical markers with filling connectors, verb + target labels, durations, nested tool rows per step | chat, desktop, ide |

### Pro (14)
| Name | One-line idea |
|---|---|
| `todo-list/ring` | Progress ring with segments per item (done, failed, active shimmering) beside the list header |
| `todo-list/capsules` | Items as capsules in a wrap row that fill as they complete, with counts inside (Beautiful UI Task Rows "Capsules" view idea [R106]) |
| `todo-list/owner-lanes` | Each item carries its owner's colour (main agent or sub-agent) and the list can split into lanes per owner |
| `todo-list/strike-sweep` | Completing an item draws a strike line and the row slides 2 px and settles, a tactile "ticked off" beat |
| `todo-list/board` | Pending / Active / Done columns; cards glide between columns as status changes (agent inbox idea [R062]) |
| `plan-card/versioned` | Version chip and history: "v2 of 3", step through versions with a word-level diff of what changed (Lovable idea, note 03) |
| `plan-card/autonomy` | Approval footer offers "Approve and run with: Ask · Auto-edit · Full" in one action (Factory idea, note 03) |
| `plan-card/countdown` | Opt-in auto-approve countdown ring on the Approve button; hover or focus pauses it (Jules idea, note 03) |
| `plan-card/flowchart` | Steps drawn as a small flow graph with branches and dependencies instead of a list (Verdent workflow-chart idea, note 08) |
| `plan-card/plan-vs-actual` | Each step's bar grows over a dashed planned-duration ghost; overruns turn amber and later steps slide right (affectUI Plan vs Actual idea, note 11) |
| `step-timeline/fold-to-line` | When the run ends, all steps fold into one line and a result card opens below (Arc run-card idea, note 12) |
| `step-timeline/stepper` | Horizontal stepper for short plans (≤ 6 steps); collapses to markers + caption under 30 rem (Arc stepper idea, note 12) |
| `step-timeline/metro` | Metro-line drawing where steps are stations and retries or branches split off and rejoin ([R038]) |
| `step-timeline/filter` | Clicking a step filters the transcript / activity to that step's calls (Devin task-drawer idea, note 03) |

## 8. Edge cases
- **Long lists** (30–100 items): checklist shows a window around the active item — 2 done above, active,
  4 pending below — with "12 done" and "+40 more" rows that expand; the dock never exceeds 50 % of the
  viewport.
- **Nested items.** Depth capped at 2 visible levels (deeper levels indent no further and show a "…"
  path); parent auto-completes only if the adapter says so (we never infer).
- **No item ids.** Text-similarity matching (§5); if confidence is low the whole list is replaced with
  one crossfade and a "Plan rewritten" note instead of per-item changes.
- **Agent rewrites the list every call.** Coalescing (300 ms) and id diffing keep unchanged items
  perfectly still; only real changes animate.
- **Several active items** (parallel sub-agents): each shimmers (max 8 animate, loaders.md §2.3);
  the dock shows "3 active" instead of one label.
- **All done but the run continues.** Dock shows "Done · 7/7" without fading until the run ends; new
  items re-open it.
- **Run ends with pending items.** `Stopped · 4 left`; pending items stay ○; no failure tone unless an
  item failed.
- **One-item plans.** No fraction; the dock shows only the label.
- **Very fast plans** (items done within 150 ms of appearing): enter already done, no active phase.
- **Very slow items** (hours): elapsed in Units; after 10 min the active shimmer slows to the calm
  cadence (status-text.md §10 "very slow turns").
- **User edits a step while the agent revises.** The agent's version wins for the item text; the
  user's edit is kept as a pending note with "Your edit wasn't applied · Apply again".
- **Long labels.** Wrap to 2 lines max, then end-truncate with full text in the tooltip; in the dock,
  single line, end-truncated.
- **RTL.** Lists mirror (glyph at inline-start); connectors stay vertical; progress fractions are
  LTR isolates; FLIP moves are vertical only.
- **Tiny sizes (16–20 px rows).** Glyph + label only, durations hidden; dock shows only "3/7".
- **Dense IDE.** 22–24 px rows, 12 px text, dock as a status-bar segment.
- **Touch.** 44 px rows; drag handles appear on long-press in Proposed; dock becomes a bottom sheet
  with a grab handle; swipe down collapses.
- **Offline / reconnect.** The list keeps its last state; on reconnect the snapshot is diffed (no enter
  animations for items that already existed).
- **Terminal surface.** Glyphs `○ ◐ ✓ ✕ ⤼ ⧗`, no shimmer (bold active label), progress as `[3/7]`.

## 9. Accessibility
- **Roles.** Todo list = `role="list"` with `aria-label="Agent tasks, 3 of 7 done"`; items are list
  items whose name is "<label>, <status>[, <duration>]" ("Run tests, in progress"; "Update schema,
  done, 2.1 seconds"; "Lint, skipped: no files changed"; "Deploy, failed: build error"; "Write docs,
  waiting on task 3"). Plan card = `role="region"` labelled with its title and version. Step timeline
  = ordered list; nested calls are a sub-list.
- **Announcements** (composer polite region): plan ready ("Plan ready for review, 7 steps" —
  assertive if approval is required, owned by family G); item becomes active only if it stays active
  ≥ 2 s ("Working on: Run tests"), debounced to once per 10 s; item failed (once); "Plan updated, 2
  added, 1 removed" (once per coalesced change set); "All tasks done".
- **Changed items** carry a text tag ("new", "edited", "reopened", "removed from plan") — the tint is
  never the only cue.
- **Keyboard.** ↑/↓ items; Enter jumps to the item's step or calls; in Proposed: E edit, Alt+↑/↓ move
  step, Delete remove (with undo), Enter (on CTA) build; dock toggle shortcut (§4.1); Esc collapses
  the dock and returns focus to the composer.
- **Contrast.** Secondary (done / pending) labels ≥ 4.5:1; struck labels stay ≥ 4.5:1; glyphs ≥ 3:1;
  the highlight tint keeps text ≥ 4.5:1.
- **Focus.** Arriving or moving items never steal focus; if the focused item is moved, focus moves with
  it; if it is removed, focus goes to the next item.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct Plan {
    pub id: PlanId,
    pub title: Option<SharedString>,
    pub version: u16,                      // v1, v2…; earlier versions kept for "Show changes"
    pub items: Vec<PlanItem>,
    pub state: PlanState,                  // Drafting | Streaming | Proposed | AwaitingApproval { request }
                                           // | Revising | Approved | Executing | Superseded { by } | Rejected | Interrupted | Done
    pub source: PlanSource,                // AgentTodo (tool calls) | PlanMode | User
}

pub struct PlanItem {
    pub id: Option<ItemId>,                // None → text-similarity matching
    pub label: ItemLabel,                  // { text, present: Option<..>, past: Option<..> }
    pub status: ItemStatus,                // Pending | Active | Done | Skipped { reason } | Failed { reason }
                                           // | Blocked { on: BlockedOn } | Cancelled
    pub owner: Option<AgentRef>,           // sub-agent colour
    pub children: Vec<PlanItem>,
    pub calls: Vec<CallId>,                // tool calls attributed to this step (step-timeline)
    pub clock: Option<ChildClockRef>,
    pub change: Option<ItemChange>,        // Added | Reworded { from } | Moved | Removed | Reopened, with at: Instant
}

pub struct TodoList<'a> {
    pub plan: &'a Plan,
    pub agent: &'a AgentState,
    pub variant: TodoVariant,              // Checklist | Dock | Pro(..)
    pub progress: ProgressRule,            // DoneOverCounted (default)
    pub window: ItemWindow,                // { done_above: 2, pending_below: 4 }
    pub coalesce: Duration,                // 300 ms
    pub labels: PlanLabels,
    pub on_jump: Option<Callback<ItemId>>,
    pub motion: MotionLevel,
}

pub struct PlanCard<'a> {
    pub plan: &'a Plan,
    pub agent: &'a AgentState,             // AwaitingApproval { request } links to the family-G footer
    pub variant: PlanCardVariant,          // Document | Pro(..)
    pub editable: bool,                    // true in Proposed / AwaitingApproval
    pub approval: Option<PlanApprovalSlot>,// family G component rendered in the footer
    pub auto_approve: Option<Duration>,    // Pro countdown only; None by default
    pub fold_to_pill: bool,                // true: Executing shows "Plan · 3/7"
    pub on_build: Option<Callback<PlanId>>,
    pub on_edit: Option<Callback<PlanEdit>>,
    pub motion: MotionLevel,
}

pub struct StepTimeline<'a> {
    pub plan: &'a Plan,
    pub calls: &'a [ToolCall],             // nested under steps via PlanItem::calls
    pub variant: StepTimelineVariant,      // Vertical | Pro(..)
    pub fold_on_done: bool,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [tools.md](tools.md) (Plan tool kind, collapse rules) · [tool-call.md](tool-call.md)
(rows nested in steps) · [composer-mode-switch.md](composer-mode-switch.md) (Plan → Agent
choreography) · [composer-queue-stack.md](composer-queue-stack.md) (user edits during execution
become steering notes) · [text-shimmer.md](text-shimmer.md) · [verb-cycler.md](verb-cycler.md) ·
[skeletons.md](skeletons.md) · family G plan approval (decision footer, autonomy options) ·
subagent-tree.md (owner colours) · run-meter.md (step durations, cost).

| Reference | What we take | Licence note |
|---|---|---|
| Claude Code task list (notes 03, 05) | pending / in_progress / completed, Ctrl+T toggle | Proprietary — reference only |
| Cursor plan mode (notes 03, 08) | editable plan, Build, minimise to "Plan" pill | Proprietary — reference only |
| Windsurf, Warp, Devin, Conductor, Kiro (note 03) | background planning todos, auto task lists, task drawer filter, todos gating merge, run per task | Proprietary (Warp AGPL) — reference / study only |
| Lovable, Jules, Factory (note 03) | plan versions, auto-approve timer, approve + autonomy | Proprietary — reference only |
| Hoplite agent tasks dock (note 10) | collapsible dock above the composer, interrupted state | Proprietary — reference only |
| Research note 05 §5 item model | 6 item states, owner colour, "3/7", "+4 more" | Our synthesis |
| Arc agent run card and stepper (note 12) | spring enter, past-tense crossfade, staggered connector fill, fold-to-line | Paid — study only |
| affectUI Plan vs Actual, Agent Steering, working panel (note 11) [R080] | ghost bars and overrun cascade; steering note glides into the plan; steps appear one by one | Paid — study only |
| Beautiful UI Task Rows [R106] [R133] | capsules / list views, nested sub-steps, counts | MIT — may port with attribution |
| Agent inbox board [R062]; metro-line pipeline [R038] | cards glide between columns; metro-line steps | Study only |
| AI Elements Plan / Task / Queue (note 05 §5) | collapsible plan with streaming shimmer | Licence not recorded in licensing.md — study only until checked |
