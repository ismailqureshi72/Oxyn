**Status:** v1 — implemented (plan #27); the draft below is kept as the design record.

# Spec: composer queue stack (queued messages · steer notes · lanes · stash)

Part of the composer family ([composer.md](composer.md)). Same depth
as the worked example [composer-send-control.md](composer-send-control.md). Target **6 free + 14 Pro
variants**, driven by one `QueueState` (stack level) and one `ChipState` (item level).

## 1. Purpose
The agent is working and the user has the next thought. They want to **line it up** without
interrupting, **nudge** the current run, change their mind, or put a draft aside. The queue stack is
the visible list of everything waiting to reach the agent, docked **above** the composer.

The behaviour is converging across tools: Enter while working queues the message above the composer
[R075][R077][R081][R082]; queued items sit under a "Queued N" count with Edit / Steer / Delete on hover
and drag to reorder [R083]; queued items can be edited and reordered [R078]; an "Up next" badge and
**two lanes** — your messages vs the agent's own queued work [R084]; steer appends to the last tool
result without interrupting [R076]; a steer note glides into the plan step that picks it up, with
**Now** and ✕ on each note [R080]; and every exit has its own motion — queue = a chip unfolds upward,
**stash** = the text shrinks into a tab with a count [R079]. Codex CLI renders queued messages under
its status row as `↳ first` / `↳ second` with `alt + ↑ edit` (research note 08, MIT).

Two item kinds:
- **Message** — a full user turn, delivered when the current run ends (or now, on demand).
- **Steer note** — guidance for the *current* run, delivered at the next tool boundary without
  stopping it (Hermes `/steer` semantics [R076]). Created by ⌘/Ctrl+Enter in `WorkingWithDraft`
  (send control) or by the chip action **Steer**.

Rules:
1. The composer **never moves**; the stack grows upward from the composer's top edge.
2. Queue order is delivery order. Position 1 always carries the **Up next** badge.
3. Nothing in the queue is sent while the user has stopped the run or the run errored (see §2
   "run end policy"). The user resumes explicitly. *(Proposed; prevents "I hit Stop and it kept going".)*
4. Every destructive action (remove, clear) is undoable for 5 s.
5. The stack is per thread and survives reload.

## 2. State machine

**Stack level**
```
            first item queued                        > max_expanded items
  Hidden ─────────────────────▶ Open(n) ───────────────────────────▶ Overflow(n)  (collapsed "+k more")
    ▲                             │  ▲                                   │
    │◀──── last item delivered ───┤  └────── items drop ≤ max ───────────┘
    │      or removed             │
    │                             │ run ends
    │            ┌────────────────┼──────────────────────┬─────────────────────────┐
    │            ▼ Done           ▼ Cancelled (Stop)     ▼ Error                   ▼ Offline
    │        Draining ─grace 600 ms─▶ send Up next   Paused("Stopped")      Paused("Run failed")   Held("Offline")
    │            │ (each turn ends → next)            │ Resume / Send next / Clear             │ reconnect → Open
    └────────────┘                                    └───────────────▶ Open / Draining
  Restored(n): after reload → Paused("Restored") until the user resumes
```

**Item (chip) level**
```
 Entering ──▶ Waiting ──(becomes position 1)──▶ UpNext ──picked up──▶ Delivering ──▶ Delivered (read-receipt fade) ──▶ gone
    │            │ ▲                                                       │
    │            │ └── save / Esc ── Editing (text back in composer, ghost keeps slot)
    │            ├── drag ──▶ Dragging ──drop──▶ Waiting (new index)
    │            ├── Steer ──▶ SteerPending ──next tool boundary──▶ Steered (glides into plan/transcript)
    │            ├── Send now ──▶ Interrupting (stop current run) ──▶ Delivering
    │            ├── Stash ──▶ Stashed (tab)  ──restore──▶ Waiting
    │            └── ✕ ──▶ Removed (undo 5 s) ──▶ gone
 Delivering ── fails ──▶ Failed (retry / edit / remove)
```

| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Hidden` | nothing; composer top edge is flat | — | — | — |
| `Open(n)` | header "Queued n"; oldest chip (Up next) sits nearest the composer, newer chips above it | per chip | header: collapse ▾, clear all | ↑ in empty composer edits the newest; `Alt+↑/↓` moves focus between chips |
| `Overflow(n)` | Up next chip + "+k more" row; header count | expand | clear all | Enter on "+k more" expands |
| `Draining` | header "Sending next…" with 600 ms countdown ring on Up next | cancel (hold the queue) | — | Esc holds |
| `Paused(reason)` | header "Queue paused · Stopped" / "· Run failed" / "· Restored 2 messages"; chips at 70 % opacity | **Send next** | Resume all, Clear | Enter on header = Send next |
| `Held(Offline)` | header "Waiting for connection"; chips normal | — | Clear | — |
| chip `Waiting` | 1–2 line clamp of text, kind glyph (✉ message / ⤷ steer), attachment count | edit (click text) | hover/focus row: **Send now**, **Steer**, **⋯** (move to top, stash, remove), drag handle | Enter edit, `⌘/Ctrl+Enter` steer, `⌘/Ctrl+Shift+Enter` send now, Delete remove, `Alt+Shift+↑/↓` reorder |
| chip `UpNext` | as Waiting + "Up next" badge | edit | same | same |
| chip `Editing` | ghost chip "Editing…" dashed, text is back in the composer; composer send shows **Save** | save (Enter) | cancel (Esc) restores original | Enter save, Esc cancel |
| chip `Dragging` | lifted chip, gap opens at target | drop | Esc cancels drag | arrows move, Space drop |
| chip `SteerPending` | ⤷ glyph pulses; label "Steer · at next step" | — | undo (until delivered) | Delete undo |
| chip `Steered` | glides out toward the plan step / transcript, becomes an inline "↳ Steered: …" note | — | — | — |
| chip `Interrupting` | "Stopping current run…" | — | — | — |
| chip `Delivering` | chip slides into the transcript as the user bubble | — | — | — |
| chip `Delivered` | (Conductor-style read receipt, note 08) bubble stays at 60 % until the agent's first event, then 100 % | — | — | — |
| chip `Stashed` | lives in the stash tab, not in delivery order | restore | delete | — |
| chip `Removed` | collapsed row "Removed · Undo" for 5 s | undo | — | `⌘/Ctrl+Z` undo |
| chip `Failed` | danger edge, "Couldn't send · Retry" | retry | edit, remove | Enter retry |

**Run end policy** (`on_done` / `on_stop` / `on_error`, all configurable):
| Run ended by | Default | Why |
|---|---|---|
| `Done` | **auto-send Up next** after a 600 ms grace (Esc holds) | the user queued it to run next |
| `Cancelled` (user Stop) | **pause**; header offers Send next / Resume / Clear | Stop means "wait" — proposed |
| `Error` | **pause** with "Run failed"; steer notes that were never delivered become messages | the plan they targeted no longer exists |
| `AwaitingApproval` | hold; steer notes keep waiting | nothing is delivered over an open approval |
| Offline | hold; resume on reconnect (Messages only; Steer notes expire if the run ended meanwhile) | |

## 3. Micro-interactions
- **Queue exit (chip arrives).** On Enter-to-queue, the draft text lifts off the input baseline and
  the chip **unfolds upward** from the composer's top edge: height 0 → auto with the text landing on
  its baseline [R079]. Existing chips shift up together. The input clears in the same commit.
- **Header count.** "Queued 2 → 3" rolls the digit (odometer, OpenCode-style AnimatedNumber, note 08).
- **Up next badge.** When chip 1 changes, the badge slides from the old chip to the new one (shared
  element), it does not blink off/on.
- **Hover/focus actions.** Actions fade + slide in 4 px from the trailing edge; they are always
  visible on touch and in `queue-stack/minimal`.
- **Edit.** Text flies back down into the composer (reverse of the queue exit); the chip leaves a
  dashed ghost so its slot is kept [R083]; saving flies it back up into the ghost.
- **Drag reorder.** Lift: scale 1.02, shadow elevation +1, 1° tilt only in `expressive`; others shift
  with a spring to open the gap; drop settles with no overshoot. Auto-scroll when within 24 px of the
  stack edge (in overflow scroll mode).
- **Steer.** The chip turns into a ⤷ note; when the agent reaches the next tool boundary the note
  **glides into the plan step or transcript row that consumed it** [R080] and leaves an inline
  "↳ Steered" marker. If the plan component isn't mounted, it glides into the transcript.
- **Send now.** Header flashes "Stopping current run…"; the status strip drops "Interrupted" into the
  transcript (see [composer-status-strip.md](composer-status-strip.md)); then the chip delivers.
- **Delivery.** Up next chip slides down-and-into the transcript as the user bubble; the remaining
  chips settle down one slot. Read-receipt fade on the bubble until the agent's first event (note 08).
- **Stash.** Text **shrinks into a tab** at the composer's top-left with a count badge [R079]
  ("Stash 2"); the tab is not part of the delivery order. Clicking the tab unfolds the stashed list.
- **Remove.** Chip collapses to a 28 px "Removed · Undo" row, which collapses fully after 5 s.
- **Overflow collapse.** When n > `max_expanded` (default 3), chips 2..n fold behind chip 1 as a
  stacked-card edge (2 px offsets, max 2 visible edges) plus "+k more". Expanding fans them up.
- **Two lanes.** Left/top lane "You" (user messages and steer notes), right/bottom lane "Agent"
  (agent's own queued work items, read-only text, can be skipped or moved up if the adapter allows)
  [R084]. Lane headers carry their own counts.

## 4. Timing (defaults; every variant may override)
| Transition | Duration | Easing / spring | Notes |
|---|---|---|---|
| Chip unfold upward (enter) | 200 ms + 40 ms stagger (cap 4) | `ease.out` | same numbers as send-control timing table |
| Other chips shift up | 200 ms | `spring.snap` | |
| Header count roll | 300 ms | `ease.out` (`cubic-bezier(0.22,1,0.36,1)`) | OpenCode's AnimatedNumber uses 560 ms; we use 300 ms for a 1-digit change (proposed) |
| Up next badge slide | 180 ms | `spring.snap` | shared element |
| Hover actions in | 120 ms, x +4 → 0 | `ease.out` | out 80 ms |
| Edit fly-down / fly-up | 200 ms | `ease.in-out` | ghost fades in 120 ms |
| Drag lift | 120 ms | `ease.out` | scale 1.02 |
| Gap open while dragging | 160 ms | `spring.soft` | |
| Drop settle | 140 ms | `spring.snap` (no overshoot) | |
| Steer glide to plan step | 320 ms | `ease.in-out` | path-arc, opacity 1 → 0 at the end |
| Delivery into transcript | ≤ 260 ms | `ease.in` | matches send-control's draft → bubble |
| Read-receipt fade 0.6 → 1 | 160 ms | `ease.out` | on agent's first event |
| Draining grace | 600 ms | linear ring | Esc holds |
| Stash shrink to tab | 220 ms | `ease.in` | tab count pops 1.0 → 1.15 → 1.0, 180 ms |
| Remove collapse / undo window | 160 ms / 5 s | `ease.in` | |
| Overflow fold / fan | 200 ms + 30 ms stagger | `spring.soft` | |
| Paused dim | 160 ms | `ease.out` | opacity 1 → 0.7 |
| **Reduced motion** | all moves become 0–80 ms opacity crossfades; no lift/tilt (drag shows an outline at the target index); steer and delivery simply fade out of the stack and in at their destination; counts change without rolling | — | — |

## 5. Variants

### Free (6)
| Name | Idea | Surfaces |
|---|---|---|
| `queue-stack/chips` | Baseline: "Queued N" header, chips unfold upward, hover actions, drag reorder [R083] | chat, desktop |
| `queue-stack/compact` | One-line rows `↳ text…` with `Alt+↑ edit` hint, no cards (Codex-style, note 08) | ide, terminal |
| `queue-stack/lanes` | Two lanes: You vs Agent work, each with counts and Up next [R084] | desktop, ide |
| `queue-stack/stash-tab` | Adds the Stash exit: text shrinks into a tab with a count [R079] | chat, desktop |
| `queue-stack/steer-notes` | Steer notes styled as ⤷ notes that glide into the plan step [R080] | desktop, ide |
| `queue-stack/minimal` | Text-only "2 queued · Edit · Clear", actions always visible; for 28 px rows | ide |

### Pro (14)
`card-deck` (stacked-card overflow that fans like a hand), `timeline-rail` (chips on a vertical rail
with ETA ticks), `kanban-mini` (Queued / Steering / Delivered columns), `split-flap` (header count
as split-flap digits), `ghost-type` (queued text retypes itself into the transcript on delivery),
`magnetic-reorder` (chips snap magnetically while dragging), `per-item-model` (model / effort chip per
queued item [R078]), `scheduled` (send at a time or "after CI passes"), `branch-queue` (each item can
target a new thread branch), `voice-notes` (queued voice clips with mini waveform), `swipe` (mobile:
swipe right = send now, left = remove), `priority-lanes` (urgent / normal lanes), `orbit-badge`
(Up next badge with a comet while draining), `ascii` (terminal stack with box-drawing borders).

## 6. Edge cases
- **Long text.** Chips clamp to 2 lines (1 line in `compact`) with an end fade; full text on hover
  tooltip after 500 ms and in edit. Attachments show as a count chip ("📎 3"), never thumbnails.
- **Overflow.** `max_expanded` default 3; stack height capped at min(40 % of viewport, 240 px); beyond
  the cap the list scrolls inside with a top fade mask. Header always pinned.
- **Many items.** Above 50 items the list virtualises; drag reorder still works with keyboard moves.
- **Very fast agent.** If the run ends < 600 ms after queueing, the chip may deliver before its enter
  animation ends: interrupt the enter and go straight to delivery (never queue the animations).
- **Very slow agent / long runs.** Queue may sit for hours: show "queued 2h ago" relative time on hover;
  steer notes older than the current step's start are flagged "may be stale".
- **Steer with no tool boundary.** Pure-streaming runs have no boundary; steer note waits until the
  turn ends, then becomes a message (label changes to "Will send next"). Apps whose adapter can't
  steer hide the Steer action (`can_steer = false`).
- **Edit while it becomes Up next and the run ends.** Editing **locks** that chip: draining skips to the
  next item or waits, never sends a half-edited message.
- **Send now on an item with an open approval.** Ask: "Stop and discard the pending approval?" —
  consequential, so a confirm (proposed).
- **Error mid-queue.** Failed chip stays in place with retry; later chips wait (order is preserved).
- **Persistence.** Stored per thread via the app's `QueueStore` (drafts + order + kind + attachments
  refs). On restore the stack is `Paused("Restored")`; nothing is auto-sent after a reload or crash.
  Attachments whose upload expired show "Re-attach file".
- **Multiple windows / devices.** Last writer wins per item id; a chip edited elsewhere shows "Changed
  on another device" and reloads its text.
- **RTL.** Actions mirror to the leading edge; drag handle moves to the trailing edge; lanes swap sides.
- **Tiny sizes / dense IDE.** At 28 px rows the header merges into one line "Queued 3 ▾"; drag handle
  hidden, reorder via `Alt+Shift+↑/↓` only.
- **Touch.** Actions always visible (no hover); long-press 350 ms starts drag; 44 px hit areas.
- **Slow networks.** Queue writes are local first; "Saving…" never blocks typing.

## 7. Accessibility
- The stack is a `list` labelled "Queued messages, 3"; each chip is a `listitem` with its position:
  "Message 1 of 3, up next: <first 80 chars>".
- **Labels per action:** "Edit queued message", "Send now — stops the current run", "Steer the current
  run with this message", "Move up", "Move down", "Stash", "Remove queued message", "Undo remove".
- **Live region (polite):** "Message queued, 3 in queue", "Sending next message", "Queue paused because
  you stopped the run", "Steer note delivered", "Message removed. Undo available". Delivered items are
  announced once, not their full text.
- **Keyboard reorder** announces "Moved to position 2 of 3". Drag has a full keyboard path
  (Space to lift, arrows, Space to drop, Esc to cancel).
- **Focus order:** composer → (Shift+Tab) stack header → chips top-to-bottom in delivery order →
  each chip's actions. After remove, focus goes to the next chip (or the composer). After edit/save,
  focus returns to the chip.
- **Contrast:** paused chips at 70 % opacity must still meet 4.5:1 text contrast (adjust the token, not
  the rule); Up next badge uses text, not colour alone.
- **Reduced motion:** see timing table.

## 8. Props (sketch)
```rust
pub struct QueueStack {
    pub state: QueueState,              // derived from ComposerState (AgentState + queue + draft)
    pub items: Vec<QueuedItem>,
    pub variant: QueueVariant,          // Chips | Compact | Lanes | StashTab | SteerNotes | Minimal | Pro(..)
    pub max_expanded: u8,               // 3
    pub max_height: Length,             // min(40% vh, 240 px)
    pub on_done: DrainPolicy,           // AutoSend { grace: 600 ms } (default) | Hold
    pub on_stop: DrainPolicy,           // Hold (default)
    pub on_error: DrainPolicy,          // Hold (default)
    pub can_steer: bool,                // adapter capability
    pub allow_stash: bool,
    pub agent_lane: Option<Vec<AgentWorkItem>>, // Some = two-lane variant
    pub store: Option<Arc<dyn QueueStore>>,     // persistence per thread
    pub undo_window: Duration,          // 5 s
    pub labels: QueueLabels,
    pub motion: MotionLevel,
}

pub struct QueuedItem {
    pub id: ItemId,
    pub kind: ItemKind,                 // Message | SteerNote
    pub text: String,
    pub attachments: Vec<AttachmentRef>,
    pub overrides: Option<TurnOverrides>, // model / effort per item (Pro `per-item-model`)
    pub state: ChipState,
    pub queued_at: SystemTime,
}

pub enum QueueState { Hidden, Open, Overflow { hidden: usize }, Draining { next: ItemId },
                      Paused(PauseReason), Held(HoldReason) }
```
Mapping from `AgentState`: `Queued { count }` = `items.len()` of kind Message; `Done` triggers
`on_done`; `Cancelled` → `on_stop`; `Error` → `on_error`; `Steering` marks the oldest `SteerPending`
note as `Steered`; `AwaitingApproval` holds draining.

## 9. Related components
Send control (`Queued(n)`, ⌘↵ steer, ↑ edits last queued, [composer-send-control.md](composer-send-control.md)) ·
status strip ("Interrupted" on send-now, [composer-status-strip.md](composer-status-strip.md)) ·
input shell (edit mode, [composer-input-shell.md](composer-input-shell.md)) · plan / action-plan
component (steer target) · transcript user bubble (delivery target) · model picker and effort control
(per-item overrides).

## 10. References (study only unless MIT/Apache)
| Ref | What we take | Licence note |
|---|---|---|
| Glaze manual [R083] | "Queued" count, hover Edit / Steer / Delete, drag reorder | Check source; study only |
| fal.ai queue & steering [R084] | "Up next" badge, two lanes | Check source; study only |
| renzobianchi "The exit is the confirmation" [R079] | chip unfolds upward; stash shrinks into a tab with a count; composer never moves | Check source; study only |
| affectUI Agent Steering [R080] | steer note glides into the plan step; Now and ✕ per note | Paid — study only |
| Replit Queue [R078] | edit, reorder, per-task model and settings | Check source; study only |
| Hermes [R075][R076][R088] | queue above the composer; steer appends to last tool result; `busyAction` | Check source; study only |
| Amp [R077], TUFFUI [R081], AI Patterns [R082], T3 Code [R089] | Enter while working = queue; queued items send-now/edit; "Enter to queue" hint | Check source; study only |
| Codex CLI status row (research note 08) | `↳` queued rows, `alt + ↑ edit` | MIT — may port with attribution |
| OpenCode AnimatedNumber (note 08) | odometer count roll | MIT — may port with attribution |
| Conductor read-receipt fade (note 08) | queued message faded until read | Not in sources.csv; study only |
| Kanban drag lift [R040] | lift / gap / settle choreography for reorder | Study only |


## Plan #27 implementation — queue stack

Implemented in `oxyn-core::composer::parts`, `oxyn-dioxus::{QueueStack}` (styles in `PARTS_CSS`, bundled into `PRIMITIVES_CSS`), `oxyn-gpui::{QueueStackEl}`.

### 27.1 Purpose
Hold messages typed while the agent works and send them in order.

### 27.2 State machine
```
push ─▶ queued   turn_end (every turn_ms while busy, not paused) ─▶ first item sent
send_now(id) ─▶ sent now (interrupts)   shift(id, ±1) ─▶ reordered   edit(id) ─Enter─▶ saved | Escape
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| queued | rows above the composer | Tab | list |
| collapsed | “+n more” | Enter expands | — |
| editing | inline field | Enter / Escape | labelled |
| paused | paused note | — | polite status |

### 27.4 Micro-interactions
Rows pop in; the head row marks itself next while the agent runs.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Row in | 180 ms | `--ox-ease` | instant |

All motion in the parts drops to 1 ms under `prefers-reduced-motion`, except `.oxyn-timer`.

### 27.6 Variants
Free: queue, send now, remove.

**Oxyn plus:** edit, Alt+↑/↓ reorder, pause, collapse after n rows.

### 27.7 Edge cases
Idle agent: the next item sends at once. Editing a row keeps its place.

### 27.8 Accessibility
A labelled list; sends, removals and moves are announced politely.

### 27.9 Props sketch
```rust
QueueStack { items: Vec<String>, busy, turn_ms: u64, paused, max_rows: usize, on_send: EventHandler<(String, bool)> }
```

### 27.10 Related components and references
`composer-send-control.md`.


## Plan #27 implementation — agent steering

Implemented in `oxyn-core::composer::parts`, `oxyn-dioxus::{AgentSteering}` (styles in `PARTS_CSS`, bundled into `PRIMITIVES_CSS`), `oxyn-gpui::{SteeringEl}`.

### 27.1 Purpose
Guide a running agent without stopping it (affectUI Agent Steering, rebuilt from its behaviour; no source used).

### 27.2 State machine
```
Working(step k, every step_ms) ─note (Enter)─▶ Queued ─next step─▶ Picked up
note (Ctrl/⌘+Enter or interrupt on) ─▶ Interrupting ─▶ Picked up now
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| working | current step, spinner | — | status |
| queued note | “at next step” | — | polite |
| picked up | check, time ago | — | announced |
| interrupting | stop marker | — | announced |

### 27.4 Micro-interactions
Notes pop in and show “Waiting for next step” until picked up; the spinner turns.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Note in | 160 ms | `--ox-ease` | instant |
| Spinner | 0.8 s loop | linear | static |

All motion in the parts drops to 1 ms under `prefers-reduced-motion`, except `.oxyn-timer`.

### 27.6 Variants
Free: notes at next step.

**Oxyn plus:** interrupt toggle, Ctrl/⌘+Enter, earlier notes for demos.

### 27.7 Edge cases
Empty notes are ignored. Steps loop in the demo.

### 27.8 Accessibility
Interrupt is a toggle (`aria-pressed`); the current step is a status region.

### 27.9 Props sketch
```rust
AgentSteering { now_ms, step_ms: u64, steps: Vec<String>, notes: Vec<(String, u64)>, interrupt: bool, on_steer: EventHandler<(String, bool)> }
```

### 27.10 Related components and references
affectUI Agent Steering — behaviour only.
