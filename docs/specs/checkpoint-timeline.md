**Status:** draft v0.1 (2026-11-11).

# Spec: checkpoint timeline (`checkpoint-timeline`)

Part of family H ([checkpoints-git.md](checkpoints-git.md) — checkpoint model, capture layers, the four
actions, branches and family tokens live there). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **4 free + 14 Pro = 18 variants**. The
confirm step embeds [rollback-impact.md](rollback-impact.md); "View changes" opens
[diff-viewer.md](diff-viewer.md); the preview bar is `snapshot-banner` (checkpoints-git.md §6.1).

## 1. Purpose
The agent just broke something three turns ago, or the user wants to try a different approach from an
earlier point. They want to **find the right moment, see what it looked like, and go back — choosing
whether to bring back the files, the conversation, both, or to branch off** — without fear of losing
work they want to keep.

What we saw (notes 03 §1E, 05 §2): Cursor puts checkpoint markers in the chat timeline, shows them only
on turns that changed files, previews files on click, and restores via a Restore Checkpoint button or a
**+** on message hover; Windsurf shows a revert arrow when you hover a prompt and supports named
checkpoints; Verdent shows "Rollback to this point" on hover; Claude Code opens a rewind menu of prior
prompts with Esc Esc on an empty input; Copilot forks a conversation from a checkpoint; Replit lists
agent-created checkpoints with description, time, scope and cost; Lovable groups versions by date with
bookmarks. Research note 05 proposes marker states `available`, `hover-reveal`, `current`,
`restoring`, `restored`, `unavailable`, `no-file-changes` and two placements (between messages, on
message hover) — this spec adopts and extends them.

Anatomy (`inline-marker`, between turns):
```
 ┃ You  · refactor the auth middleware to use sessions
 ┃ Agent · … Edited 3 files +42 −17
 ─ ◆ ── Checkpoint · 3:42 PM · 3 files ──────────────────────────────── ↶ Restore ▾  ─   ← actions reveal on hover/focus
 ┃ You  · now add rate limiting
              ┌──────────────────────────────────────────────┐
              │ ◆ Checkpoint · auto · 3:42 PM                 │  hover preview (200 ms)
              │ Before: "now add rate limiting"               │
              │ auth.rs        +4 −2                          │
              │ session.rs     +30 −15                        │
              │ + rate.rs      +8                             │
              │ Files and chat · on main @ a1b2c3d            │
              │ Preview · View changes                        │
              └──────────────────────────────────────────────┘
 Restore ▾ menu:   Restore files            R
                   Rewind chat              C
                   Restore files and chat   B
                   ─────────────────────────
                   Fork from here           F
                   Preview                  P
```
Rules (proposed):
1. **Labels say what moves.** Never a bare "Restore"; the menu names the scope, the confirm button
   repeats it with counts ("Restore 3 files", "Rewind 2 turns").
2. **Every action goes through the impact sheet** unless the impact is trivially empty (chat-only
   rewind with no tool state change and no external effects) — then it applies directly with the undo
   toast as the safety net.
3. **Every restore is undoable** through an automatic `RestorePoint` (checkpoints-git.md §2.3); the
   toast is a shortcut, not the only path.
4. **A checkpoint marks the state before its turn.** "Restore to 3:42 PM" brings back the files as
   they were **before** the turn that follows the marker (Cursor and Claude attach checkpoints to the
   prompt that started the turn, note 05 §2). The preview's heading says "Before: <prompt>".

## 2. State machine
Per marker (presentation + lifecycle; lifecycle states come from checkpoints-git.md §3):
```
                         turn starts writing / manual save
   (none) ──────────────────────────────────────▶ Capturing ──fail──▶ CaptureFailed
                                                      │ captured
                                                      ▼
  ┌─────────────── pointer leaves / blur ─────── Rest(Available) ──pointer enters / focus──▶ Hovered ──150 ms──▶ Revealed
  │                                                   ▲   ▲                                               │
  │                                                   │   └──── Esc / select elsewhere ── MenuOpen ◀──────┤ click Restore ▾ / Enter
  │                                                   │                                    │              │ hover 200 ms
  │                                                   │                                    │ choose       ▼
  │                                                   │                                    │          Previewing(popover)
  │                                                   │        P / "Preview" ──▶ Previewing(snapshot-banner, read-only views)
  │                                                   │                                    ▼
  │                                                   │                         ImpactPending (skeleton after 150 ms)
  │                                                   │                                    │ impact ready / impact empty
  │                                                   │          cancel ◀────────── Confirming{action, impact}
  │                                                   │                                    │ confirm
  │                                                   │                                    ▼
  │                                                   │                      Restoring{n of m} ──▶ RestoreFailed (nothing changed) ──retry──┐
  │                                                   │                                    │                                           │
  │                                                   │                    ┌───────────────┴────────────┐                              │
  │                                                   │                    ▼                            ▼                              │
  │                                                   │               Restored ("You are here")   PartiallyRestored ──retry failed──────┘
  │                                                   │                    │ toast visible: UndoWindow{remaining}
  │                                                   │                    │  ├── Undo ──▶ Undoing ──▶ Undone (RestorePoint is current)
  │                                                   │                    │  └── window ends ──▶ Restored (undo still in timeline)
  │                                                   │
  │       Fork from here ──▶ Forking ──▶ Forked (toast "Opened fork · Go to fork") │ ForkFailed
  │
  Agent running  ─▶ Blocked("Stop the agent to restore"; Preview and Fork still allowed)
  Rewound later  ─▶ OnBranch (inside "N rewound turns" fold; Bring back · Fork)
  Expired / Unavailable{CloudAgent | NotCaptured} / ChatOnly (file actions hidden) / Offline
```
Timeline-level states (side list, scrubber): `Loading` (skeleton rows after 150 ms) · `Empty` ("No
checkpoints yet — they appear after the agent changes files") · `Ready` · `Filtering` (Auto / Manual /
Milestones) · `LoadFailed` (Retry).

## 3. States table
| State | Shows | Primary | Secondary | Keyboard (focused) |
|---|---|---|---|---|
| `Capturing` | 40 % hairline + diamond; "Saving checkpoint…" only if > 600 ms | — | — | — |
| `Rest` | hairline, 6 px `◆`, "3:42 PM · 3 files" in `fg.muted` (or just the diamond in compact) | — | — | Tab focuses the marker |
| `Hovered` / `Revealed` | hairline darkens; "↶ Restore ▾" and "⋯" slide in from the end | open menu | preview popover | Enter menu, P preview, V view changes |
| `MenuOpen` | menu with four scoped actions + Preview; disabled items show their reason | chosen action | — | ↑/↓, letters R C B F P, Esc close |
| `Previewing` (popover) | files + counts, prompt, capture fidelity, git head | Preview | View changes | Enter preview |
| `Previewing` (banner) | `snapshot-banner` "Viewing checkpoint from 3:42 PM"; file views and transcript at that point, read-only | Back to latest | Restore… | Esc back, ⌘↩ restore… |
| `ImpactPending` | sheet frame with skeleton rows (after `delay.skeleton` 150 ms) | — | Cancel | Esc |
| `Confirming` | [rollback-impact.md](rollback-impact.md) sheet; confirm label = scope + counts | confirm | Cancel, scope switch (segmented: Files / Chat / Both) | Enter confirm (when focused), Esc cancel, ←/→ scope |
| `Restoring` | "Restoring 4 of 7 files" with rolling count; rewind fold animates | — | — | — |
| `Restored` | marker becomes "You are here" in `color.checkpoint.current`; 600 ms accent wash; undo toast | Undo (toast) | View changes | ⌘Z / Ctrl+Z undo while toast visible and focus is not in a text field |
| `UndoWindow` | toast "Restored 3 files to 3:42 PM · Undo" + depleting hairline | Undo | Dismiss | Tab reaches toast (F6 region jump) |
| `Undoing` → `Undone` | toast text swaps "Undoing…" → "Restore undone"; marker returns to `Rest` | — | — | — |
| `PartiallyRestored` | warning toast "Restored 6 of 7 files · rate.rs is locked" | Retry failed | Undo all | R retry |
| `RestoreFailed` | danger toast "Couldn't restore · nothing changed" + reason | Retry | Copy error | R retry |
| `Blocked` | Restore items disabled, reason "Stop the agent to restore" | Stop and restore (proposed: stops the run, then opens the impact sheet) | Preview, Fork | — |
| `Forking` → `Forked` | toast "Opened fork from 3:42 PM · Go to fork" | Go to fork | — | Enter |
| `OnBranch` | dimmed marker inside "3 rewound turns · Bring back" fold | Bring back (restores the branch's head) | Fork | Enter |
| `ChatOnly` | hollow `◇`, "no file changes"; menu has Rewind chat and Fork only | Rewind chat | Fork | C, F |
| `Expired` | dashed diamond, "Files no longer kept"; Rewind chat still offered if the chat position exists | Rewind chat | — | C |
| `Unavailable` | diamond with ⊘, reason ("Not available for cloud runs") | — | — | — |
| `CaptureFailed` | diamond with !, "Checkpoint not saved · <reason>" | Retry (if supported) | — | R |
| `Offline` | actions disabled, "Checkpoint store offline" | Retry | — | R |

## 4. Micro-interactions
- **Marker enter.** The hairline draws from the start edge to the end over 240 ms (`ease.out`) as the
  turn begins; the diamond scales 0 → 1 (`spring.snap`) at the start. A marker never pushes content:
  its 24 px row height is reserved when the turn row is created.
- **Hover reveal.** After `delay.marker.hover` (150 ms) the actions fade in with x −4 → 0
  (`dur.marker.reveal`, 120 ms); leaving hides them after a 300 ms grace so the pointer can travel to
  them. Hover over a **user message** also reveals a ↶ on that message (the `hover-revert` placement).
- **Preview popover.** 200 ms hover delay, 140 ms fade; the file list rows show counts with no roll
  (static); clicking a file opens the diff viewer filtered to that file at the checkpoint → head range.
- **Menu.** Opens with `dur.sheet.enter`; items carry letter hints; disabled items show their reason
  as secondary text, not a tooltip only.
- **Scope switch inside the sheet.** Switching Files / Chat / Both crossfades the impact summary
  (160 ms) and rolls the confirm button counts ("Restore 3 files" → "Restore 3 files and rewind 2
  turns"); the button keeps its width reserved for the longest label (one cell, Arc idea, note 12).
- **Restore — files.** Each later file-edit chip touched by the restore morphs to `Reverted` (↶ rotates
  −90° → 0, 160 ms) in transcript order with `stagger.bulk` (30 ms, cap 8); the "Restoring n of m"
  count rolls with `dur.digit.roll`.
- **Restore — chat.** Turns after the marker fold upward into a single "2 rewound turns · Bring back"
  row (`dur.rewind.fold`, 280 ms, stagger 20 ms cap 6), then the rewound user prompt travels from its
  bubble into the composer (`dur.travel`, ≤ 260 ms) with the caret at its end. If the composer has a
  draft, the prompt is **not** moved; a chip "Rewound prompt · Insert" appears above the composer
  instead (proposed).
- **Restored.** The marker label crossfades to "You are here" and an accent wash fades over 600 ms.
- **Undo toast.** Rises 8 px from the composer edge (200 ms `ease.out`); a 1 px hairline depletes over
  `undo.window` (10 s, proposed); hover or focus pauses it and it resumes with at least 3 s left.
  Undo plays the same choreography backwards (chips un-revert, fold re-expands).
- **Fork.** The marker duplicates: a ghost diamond slides 12 px toward the end edge and fades (200 ms)
  while the toast appears; if the host opens the fork in a new tab, the tab strip owns that animation.
- **Failure.** Toast in danger tone, glyph nudge 2 × 3 px (tools.md error nudge); text never shakes.
- **Auto vs manual.** `Auto` markers are hairline-only at rest; `Manual` markers always show their name
  and a flag; saving a manual checkpoint (⌘S in the timeline) stamps the diamond with a 160 ms scale
  pulse 1 → 1.2 → 1.

## 5. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Marker hairline draw | 240 ms | at turn start | `ease.out` | — | 60 Hz |
| Diamond enter | 160 ms, scale 0 → 1 | 0 | `spring.snap` | — | 60 Hz |
| Actions reveal / hide | 120 ms / 120 ms | 150 ms / 300 ms grace | `ease.out` / `ease.in` | — | 60 Hz |
| Preview popover | 140 ms | 200 ms | `ease.out` | — | 60 Hz |
| Menu / sheet enter | 220 ms, y +12 → 0 | 0 | `spring.snap` | — | 60 Hz |
| Menu / sheet exit | 160 ms | 0 | `ease.in` | — | 60 Hz |
| Impact skeleton | — | 150 ms | — | — | 30 Hz shimmer (skeletons.md) |
| Scope switch | 160 ms crossfade + 280 ms count roll | 0 | `ease.in-out` / `ease.out` | 30 ms digits | 60 Hz |
| Chip revert cascade | 160 ms each | 0 | `spring.snap` | 30 ms, cap 8 | 60 Hz |
| Rewind fold | 280 ms | after files settle | `ease.in-out` | 20 ms, cap 6 | 60 Hz |
| Prompt → composer | ≤ 260 ms | after fold | `ease.in-out` | — | 60 Hz |
| "You are here" wash | 600 ms | 0 | `ease.out` | — | 30 Hz |
| Undo toast enter / exit | 200 ms / 160 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Undo hairline | 10 s (paused on hover/focus) | 0 | linear | — | 10 Hz |
| Manual save pulse | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Fork ghost | 200 ms, x +12, fade | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | hairline and diamond appear instantly; reveals, popovers, sheets and toasts use 80 ms opacity crossfades with no offsets; no cascade (all chips swap together); rewind fold is instant with an 80 ms crossfade; prompt appears in the composer without travel; no wash (static 2 px accent outline for 2 s); undo countdown is text "Undo · 9 s" updated at 1 Hz | — | linear | none | event-driven |

## 6. Variants

### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `checkpoint-timeline/inline-marker` | Hairline marker between turns with hover-revealed Restore ▾ menu and preview popover (anatomy above) | chat, desktop, ide |
| `checkpoint-timeline/hover-revert` | No standing markers; a ↶ appears on hover / focus of each user message (Windsurf / Verdent / Cursor "+" placement, note 03) | chat, desktop, mobile (long-press) |
| `checkpoint-timeline/side-list` | Vertical list in a side panel grouped by day: time, kind glyph, prompt excerpt, `+N −N`; selecting a row previews | ide, desktop |
| `checkpoint-timeline/terminal` | Rewind list opened by Esc Esc on an empty prompt (Claude Code binding, note 05 §2): numbered prompts, then a scope list `1 Restore files · 2 Rewind chat · 3 Both · 4 Fork · esc never mind` | terminal |

### Pro (14)
| Name | One-line idea |
|---|---|
| `checkpoint-timeline/scrubber` | Horizontal track of diamonds under the transcript; drag to preview any checkpoint (orientation from note 05 §2), shares run-replay's track |
| `checkpoint-timeline/tree` | Session tree with rewound branches as side shoots; select any node to preview or bring it back (Pi `/tree` idea, note 03) |
| `checkpoint-timeline/filmstrip` | App-builder history: each checkpoint is a screenshot thumbnail of the preview app (needs host screenshots) |
| `checkpoint-timeline/diff-peek` | Hover shows the first changed hunk of the turn inline under the marker instead of a file list |
| `checkpoint-timeline/bookmarks` | History / Bookmarks tabs; star any checkpoint to keep it past retention (Lovable bookmarks idea, note 05 §2) |
| `checkpoint-timeline/milestone-cards` | Agent-written milestone cards with summary, scope and cost; auto checkpoints fold between them (Replit idea, note 05 §2) |
| `checkpoint-timeline/ghost-rewind` | Rewound turns stay visible as a translucent ghost layer to the side, so the user can read them before bringing them back |
| `checkpoint-timeline/branch-rail` | A thin git-graph-style rail down the transcript's start edge; forks and rewinds branch off visually |
| `checkpoint-timeline/cost-delta` | Each marker shows tokens / cost spent since it (pairs with run-meter) so "go back" shows what is being thrown away |
| `checkpoint-timeline/summarize-from-here` | Adds a fifth action that rewinds the chat but replaces the rewound turns with a model-written summary (Claude idea, note 05 §2) |
| `checkpoint-timeline/composer-dock` | "↶ Rewind" pill above the composer opening the last 5 checkpoints as a popover list |
| `checkpoint-timeline/day-heat` | Side list header with a per-day strip whose cells shade by number of changes; click a day to filter |
| `checkpoint-timeline/compare-pair` | Select any two checkpoints to open the diff viewer between them (not just checkpoint → head) |
| `checkpoint-timeline/tape-rewind` | Expressive rewind: transcript rolls back like tape with a brief motion blur and counter spin, then settles |

## 7. Edge cases
- **Restore while the agent runs.** Restore actions are `Blocked`; "Stop and restore" stops the run
  (send-control Stop semantics), waits for `Cancelled`, then opens the impact sheet. Preview and Fork
  stay available because they change nothing in this thread.
- **User has a draft in the composer** when rewinding chat: the rewound prompt goes to a chip above the
  composer, never overwrites the draft.
- **Very fast agents** writing many turns quickly: markers still enter one per turn; a burst of
  markers inside 1 s merges visually into one hairline with a count badge "3 checkpoints" until hovered.
- **Hundreds of checkpoints.** Inline markers remain one per turn; side list virtualises and groups by
  day; scrubber aggregates diamonds closer than 4 px into a stacked diamond with a count.
- **Same file restored twice** (restore, then restore an older one): each creates its own
  `RestorePoint`; undo always targets the most recent restore only; older ones are reachable in the list.
- **Undo after further edits.** If files changed after the restore (user or agent), Undo opens the impact
  sheet instead of applying directly ("2 files changed since the restore").
- **Partially captured checkpoints.** Preview lists skipped files with the reason; the confirm sheet says
  "1 file can't be restored (too large)".
- **Checkpoint on a different git branch** than the current one: preview shows "Captured on
  `feature/auth`, you're on `main`"; restore writes files into the current branch's working tree and the
  sheet says so (no checkout happens, checkpoints-git.md §2.1 rule 3).
- **Fork without worktree support.** Fork starts on current files with a note "Files in the fork start as
  they are now".
- **Long prompt excerpts.** One line, 60 ch, ellipsis; full text in the preview and accessible name.
- **RTL.** Marker hairline and side list mirror; times, paths and counts stay LTR; the scrubber's time
  axis keeps left → right (run-replay.md §7 convention).
- **Tiny sizes (16–20 px).** Marker is the diamond only; Enter / tap opens the menu; preview disabled.
- **Dense IDE panels.** 20 px marker rows, no labels at rest, preview opens on Enter instead of hover.
- **Touch.** 44 px hit area on markers; long-press for preview; menu becomes a bottom sheet; undo toast
  sits above the keyboard.
- **Slow networks / remote store.** Impact skeleton after 150 ms; if impact takes > 5 s, the sheet offers
  "Restore without summary" with a generic warning (rollback-impact.md `ImpactUnavailable`).
- **Expired mid-flow.** If retention removes files between Confirming and Restoring, the restore fails
  with "No files were restored · this checkpoint has expired" and nothing changes.

## 8. Accessibility
- **Marker role:** a `button` with `aria-haspopup="menu"`; accessible name "Checkpoint, auto, 3:42 PM,
  before 'now add rate limiting', 3 files changed, available". Chat-only: "…, no file changes".
  Current: "…, you are here".
- **Menu items** are named by scope with counts ("Restore 3 files", "Rewind 2 turns", "Restore 3 files
  and rewind 2 turns", "Fork from here"); disabled items keep focusability with
  `aria-disabled` and the reason as description.
- **Side list:** `listbox` grouped by day (`group` with the date as label); scrubber is a `slider`
  whose value text names the checkpoint (as run-replay.md §8).
- **Live region (polite):** "Restored 3 files to 3:42 PM. Undo available for 10 seconds." ·
  "Restore undone." · "Opened fork from 3:42 PM." Failures assertive once. No announcements on hover.
- **Keyboard:** Tab to marker; Enter menu; R / C / B / F / P inside menu; Esc closes; in the
  side list ↑/↓ move, Enter preview, Shift+Enter open menu; ⌘S save manual checkpoint (side list and
  timeline focus only); ⌘Z undo restore while the toast is visible and focus is not in an editable field;
  F6 cycles to the toast region.
- **Focus order after restore:** focus returns to the marker (now "You are here") unless the user was in
  the sheet, in which case it moves to the toast's Undo button; after the toast closes, focus goes back to
  the marker.
- **Contrast:** hairline ≥ 3:1 against the transcript background on focus/hover (rest may be 2:1 as
  decoration since the diamond and text carry it); "You are here" accent text ≥ 4.5:1.
- **Undo is never toast-only:** the `RestorePoint` stays in the side list and menu ("Undo last restore").
- **Reduced motion:** §5 last row.

## 9. Props sketch
```rust
pub struct CheckpointTimeline<'a> {
    pub agent: &'a AgentState,                    // Blocked while not Idle/Done/Error/Cancelled
    pub checkpoints: &'a [Checkpoint],            // checkpoints-git.md §9, in transcript order
    pub current: Option<CheckpointId>,            // "You are here"
    pub store: &'a dyn CheckpointStore,           // list / change_set / impact / restore / undo
    pub variant: TimelineVariant,                 // InlineMarker | HoverRevert | SideList | Terminal | Pro(..)
    pub actions: ActionSet,                       // which of Files | Chat | FilesAndChat | Fork | Preview | Summarize
    pub default_action: RestoreAction,            // menu order + Enter default; per surface, see note below
    pub confirm: ConfirmPolicy,                   // AlwaysSheet | SheetUnlessEmpty (default)
    pub undo_window: Duration,                    // 10 s (proposed)
    pub prompt_return: PromptReturn,              // ToComposer | ChipIfDraft (default) | Never
    pub fork_start: ForkStart,                    // CurrentFiles | NewWorktree (if store caps allow)
    pub filters: CheckpointFilter,                // Auto | Manual | Milestones | All
    pub labels: TimelineLabels,                   // i18n incl. plural rules ("1 file" / "3 files")
    pub on_view_changes: Option<Callback<(CheckpointId, Head)>>, // opens diff-viewer
    pub on_open_fork: Option<Callback<ThreadId>>,
    pub on_stop_run: Option<Callback<()>>,        // "Stop and restore"
    pub motion: MotionLevel,
}

pub enum MarkerPresentation { Rest, Hovered, Revealed, MenuOpen, PreviewPopover }

pub enum TimelineFlow {                           // one per open marker; lifecycle from checkpoints-git.md §3
    Idle,
    ImpactPending { action: RestoreAction },
    Confirming { action: RestoreAction, impact: Impact },
    Restoring { handle: RestoreHandle, done: u32, total: u32 },
    Restored { restore: RestoreId, undo_until: Instant },
    PartiallyRestored { restore: RestoreId, failed: Vec<(FilePath, IoReason)> },
    RestoreFailed { reason: SharedString },
    Undoing { restore: RestoreId },
    Forking, Forked { thread: ThreadId }, ForkFailed { reason: SharedString },
}
```
(`default_action` proposed: **Files** in `ide` surfaces, matching Cursor's files-only default; **Files
and chat** in `chat` surfaces where users think in conversation turns — to validate in the Studio.)

## 10. Related components and references
**Related:** [checkpoints-git.md](checkpoints-git.md) (model, tokens, `snapshot-banner`) ·
[rollback-impact.md](rollback-impact.md) (confirm step) · [diff-viewer.md](diff-viewer.md) (View
changes, compare-pair) · [file-edit-chip.md](file-edit-chip.md) (`Reverted` morph) ·
[run-replay.md](run-replay.md) (checkpoint markers on the replay track; scrubber shares its track) ·
[composer-send-control.md](composer-send-control.md) (Stop) · [composer-queue-stack.md](composer-queue-stack.md)
(rewound prompt chip sits with queued chips) · [run-meter.md](run-meter.md) (cost-delta) ·
[git-chips.md](git-chips.md) (fork into worktree) · [merge-conflict.md](merge-conflict.md) (bringing
back a branch over changed files).

| Reference | What we take | Licence note |
|---|---|---|
| Cursor checkpoints (notes 03 §1E, 05 §2) | markers only on file-changing turns, click to preview, files-only restore, prompt back to composer, "+" on message hover | Proprietary — reference only |
| Claude Code rewind (note 05 §2) | Esc Esc on empty input, scoped actions incl. summarize, hidden code options without edits, expiry error wording | Proprietary — reference only |
| Windsurf, Verdent (note 03 §1E) | revert arrow / "Rollback to this point" on hover; named checkpoints | Proprietary — reference only |
| Kiro, Copilot (note 03 §1E) | restore files + context; fork from checkpoint | Proprietary — reference only |
| Replit (note 05 §2) | milestone checkpoints with description, scope, cost; roll forward | Proprietary — reference only |
| Lovable (note 05 §2) | grouped by date, bookmarks, Back to latest | Proprietary — reference only |
| Pi `/tree` (note 03) | session tree | Not in licensing.md — study only until checked |
| Note 05 §2 recommendations | 7 marker states × 2 placements; 2 orientations | Our own research |
| AI Elements Checkpoint (note 05 §1) | inline icon + trigger row | Not in licensing.md — study only until checked |
| affectUI Agent Checkpoints (note 11) | exists as a Pro component; no behaviour taken | Paid — study only |
| Arc one-cell label swaps (note 12) | confirm button keeps one width | Paid — study only |
