**Status:** draft v0.1 (2026-11-11).

# Spec: file-edit chip (chips, inline edit summaries, many-file rollup)

Part of family F ([tools.md](tools.md) — event model, grouping and collapse rules live there). Same
depth as the worked example [composer-send-control.md](composer-send-control.md). Target **3 free +
11 Pro variants**. The diff itself (hunks, split / stacked, keep / undo per hunk) belongs to family
H; this spec owns the **compact summary** of an edit and the jump into that diff.

## 1. Purpose
When an agent edits code (or any document), the user wants three things without opening a diff:
**which files changed, how much, and is it applied yet**. The green/red count chip is "the universal
affordance" (research note 03, "File edit card with +/− counts", table-stakes): Codex prints
`• Edited 2 files (+2 -1)` with `└ a.txt (+1 -1)` children and shows renames as `old → new`; Claude
desktop's `+12 −1` chip opens the diff; Paradev puts `+46 −1` on each workspace row; Zed shows an
accordion bar above the composer with files and lines edited; Antigravity ends a turn with
`2 files changed +24 -0 ›` and a Review button; Cursor's dashboard rows read `N Files +X−Y` (notes 03,
08). Codex's desktop app animates its diff stat (changelog, type undocumented) and OpenCode ships a
560 ms digit odometer (note 08). The chip appears:
- in the **transcript**, as the visible form of an `Edit` tool call (tool-call row → chip);
- as a **rollup** for consecutive edits ([tools.md](tools.md) §3.1.8);
- in **thread / sidebar rows** and **agent dashboards** (`+12 −3` badge [R016], [R053]);
- above the **composer** (changed-files accordion) and in the **turn footer** (review bar).

## 2. Anatomy and formats
```
 chip (inline, 20 px tall)         [✎] auth.rs  +12 −3                       → click: View diff
 row (32 px chat / 24 px ide)      [✎] src/…/auth.rs   +12 −3   Open · View diff   [state glyph]
 rollup (tree-rollup, collapsed)   [✎] Edited 6 files  +9 −9  ›
 rollup (expanded)                 ├ auth.rs        +4 −2
                                   ├ session.rs     +3 −3
                                   ├ + new_test.rs  +2          (created)
                                   ├ − legacy.rs    −2          (deleted, path struck)
                                   └ util.rs → helpers.rs       (renamed, no content change)
                                     and 1 more ›                (rows beyond 5 when > 8 files)
```
**Count format** (proposed; tabular figures, status-text.md §2.3):
| Case | Shows | Notes |
|---|---|---|
| Modified | `+12 −3` | `+` and `−` (U+2212, same advance as `+` in tabular fonts) each in a fixed cell; additions in `color.diff.add.fg`, deletions in `color.diff.del.fg`; 4 px gap. Zed uses a thin space and a figure dash (`+ 1,234` / `‒ 56`, note 08 §1.9); a `zed` token preset reproduces that spacing |
| Only additions / only deletions | `+40` / `−120` | the zero side is omitted (Codex prints `+1 -0`; we drop the zero to save width) |
| Large | `+1,234 −56` locale grouping; ≥ 100,000 → `+120k` | full value in tooltip and accessible name |
| Created | `+` glyph before the name + `+40` | op word "Created" in rows and accessible names |
| Deleted | `−` glyph, path struck through, `−120` | "Deleted" |
| Renamed | `old.rs → new.rs` (common prefix shown once: `src/{old → new}.rs` when it fits) | counts if the content also changed |
| Binary | `binary · 24 KB → 31 KB` | no line counts |
| Mode only | `mode +x` | — |
| Partially applied (per-hunk review) | `+8 −2 of +12 −3` | the "of" part dims |

## 3. State machine
```
                       edit call starts (tools.md §2.2 Running / StreamingOutput)
  (none) ───────────────────────────────────────────▶ Writing{+a −r rolling} ──diff complete──▶ ┐
                                                                                               │
          ┌───────────────── permission = auto-apply ◀──────────────────────────────────────────┤
          │                                                                                     │ permission = ask
          ▼                                                                                     ▼
       Applied ◀───────────────────────── approve (all hunks) ──────────────────────── AwaitingApproval
        │  │ ▲                                                                              │      │
        │  │ └── keep (review accepted) ── Kept                                  reject all │      │ approve some
        │  │                                                                                ▼      ▼
        │  └─ file changed on disk since ─▶ Conflict ──resolve (family H)──▶ Applied     Rejected  PartiallyApplied
        │
        └─ undo / checkpoint restore ─▶ Reverted
  Writing ── write error ──▶ Failed{reason} ──retry──▶ Writing
  Writing ── run stopped ──▶ Cancelled (partial file state shown as Applied-partial if bytes were written)
  Presentation: Rest · Hovered · Focused · Pressed · Expanded (rollup) · Opening (link pressed, ≤ 400 ms)
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (focused) |
|---|---|---|---|---|
| `Writing` | pencil icon replaced by the row loader (or `writing-sweep`); counts roll up as lines stream; "Editing auth.rs" | View diff (live, streaming) | Open file | Enter view diff |
| `AwaitingApproval` | amber left bar, counts static, "Review changes" label; parent tool card auto-expands the diff (tools.md §4.1 rule 2) | family G / H (Approve, Reject, per hunk) | Open file | Enter view diff, Tab into decision |
| `Applied` | ✓ morph (600 ms) then pencil icon; past tense "Edited"; counts final | View diff | Open file, Copy path | Enter view diff, O open, C copy path |
| `Kept` | as Applied + a small "kept" check (review done) | View diff | Undo (family H) | — |
| `PartiallyApplied` | `+8 −2 of +12 −3`, half-filled glyph | View diff | — | — |
| `Rejected` | counts struck through and dimmed, "Rejected" | View diff (what was proposed) | — | — |
| `Reverted` | ↶ glyph, counts dimmed, "Reverted" | View diff | Re-apply (if supported) | — |
| `Conflict` | amber ! glyph, "Changed on disk" | Resolve (family H merge view) | View diff | Enter resolve |
| `Failed` | ✕ danger, "Couldn't write auth.rs: permission denied" | Retry | Copy error | R retry |
| `Cancelled` | ■, counts at the value written so far | View diff | — | — |
| Rollup `Live` | "Editing 3 files…", totals roll, children appear as files are touched | expand | View all diffs | Enter, ←/→ |
| Rollup `Settled` | "Edited 6 files +9 −9" | expand | View all diffs, Review (review-bar) | Enter, ←/→ |
| `Hovered` | underline on file name and links; tooltip with full path, op, exact counts, hunk count | — | — | — |
| `Opening` | link label crossfades to "Opening…" in the same cell, then back | — | — | — |

## 5. Micro-interactions
- **Counters rolling.** While `Writing`, counts move toward their target closing 25 % of the gap per
  50 ms tick (min +1) — the counter rule from [elapsed-timer.md](elapsed-timer.md) §5 — and each digit
  change rolls with `dur.digit.roll` (280 ms, 30 ms right → left stagger). The count slot uses
  `Width::ReserveMagnitude`, so `+9 → +10` widens once with `spring.snap` before the roll. A
  correction downward (e.g. the agent rewrites a hunk) rolls down once; never silently.
- **Settle.** On apply: loader → ✓ (`dur.row.morph`, stroke draw 240 ms), verb rolls "Editing" →
  "Edited" (`dur.tense.roll`); counts finish their roll within 300 ms.
- **New file in a rollup.** The child row enters with `dur.row.enter` and a `dur.highlight.fade`
  (1600 ms) tint; the header total rolls. Order is **stable**: files keep the order they were first
  touched (no reshuffling under the pointer; Amp shows a blue dot when a new order is ready instead
  of reshuffling, note 08 §1.14 — our `review-bar` variant may offer "Sort by size" the same way).
- **Approval.** Entering `AwaitingApproval` fades in a 2 px amber inline-start bar (160 ms); on
  approve it turns success for 600 ms then fades; on reject the counts get a strike line drawn left →
  right (200 ms `ease.out`) and dim to 60 %.
- **Revert.** Counts dim (200 ms) and the ↶ glyph rotates −90° → 0 as it appears (160 ms
  `spring.snap`).
- **Conflict.** ! glyph pops (scale 0 → 1, `spring.snap`), one amber pulse of the chip outline
  (opacity 1 → 0.6 → 1, 600 ms); never loops.
- **Rename.** The common path prefix stays put; the changed segment crossfades old → new (200 ms)
  with a → glyph sliding in (Pro `rename-morph` animates per character).
- **Hover / press.** Chip background tint 80 ms; press scale 0.97 for 80 ms; links underline on hover.
- **Open / View diff.** Pressing a link swaps its label to "Opening…" in the same cell (Arc's PR
  button keeps all labels in one cell so the width never changes, note 12); the diff opens in the host's
  diff surface (family H) with the file scrolled to its first hunk.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Count tick (live) | 50 ms ticks, 25 % of gap, min +1 | — | — | — | 20 Hz while moving |
| Digit roll | 280 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz during roll |
| Slot widen at magnitude change | 200 ms | before roll | `spring.snap` | — | 60 Hz |
| Loader → ✓ + stroke | 160 + 240 ms | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Tense roll | 240 ms | after morph | `ease.out` | — | 60 Hz |
| Rollup child enter + highlight | 200 ms + 1600 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| Approval bar in / success / fade | 160 / 600 / 240 ms | 0 | `ease.out` / — / `ease.in` | — | 60 Hz |
| Reject strike | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Revert glyph | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Conflict pop + pulse | 160 ms + 600 ms | 0 | `spring.snap` / `ease.in-out` | — | 60 Hz |
| Rename crossfade | 200 ms | 0 | `ease.in-out` | — | 60 Hz |
| Hover / press | 80 ms / 80 ms | 0 | `ease.out` | — | 60 Hz |
| Link label swap | 120 ms | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | counts jump to the current value at most every 500 ms (no roll); every other change is an 80 ms opacity crossfade; no strike draw (strike appears), no pulse, no highlight fade (static 2 px bar for 4 s) | — | linear | none | 2 Hz while writing |

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `file-edit-chip/counter` | Compact chip: icon, file name, `+12 −3`; whole chip opens the diff. Also the sidebar / dashboard badge form | all |
| `file-edit-chip/row` | Full row: icon, middle-truncated path, counts, "Open · View diff" links, state glyph | chat, desktop, ide, mobile |
| `file-edit-chip/tree-rollup` | "Edited 6 files +9 −9" header with `├`/`└` children per file, 5 shown then "and N more" (Codex layout, note 08) | all incl. terminal |

### Pro (11)
| Name | One-line idea |
|---|---|
| `file-edit-chip/odometer` | Counts on vertical digit strips with an 18 % top/bottom mask fade (OpenCode AnimatedNumber idea, MIT) |
| `file-edit-chip/block-bar` | Five small blocks after the counts filled green/red by the add/remove ratio |
| `file-edit-chip/writing-sweep` | A soft glow passes over the chip while the file is being written and settles when done ("agent is writing here" idea [R034]) |
| `file-edit-chip/review-bar` | Turn-end footer "2 files changed +24 −0 › Review" that expands into the rollup (Antigravity pattern, note 08) |
| `file-edit-chip/composer-accordion` | Changed-files bar docked above the composer; expands into the rollup with Keep all / Undo all (Zed idea, note 03) |
| `file-edit-chip/hover-hunk` | Hover shows the first hunk (≤ 8 lines) in a popover, with next / previous hunk keys |
| `file-edit-chip/status-letters` | A / M / D / R letter badge per file instead of icons, IDE source-control style (note 05 §3) |
| `file-edit-chip/stack-fan` | Many files as a stack of file-type icons that fans out on hover with per-file counts |
| `file-edit-chip/viewed-progress` | Per-file "Viewed" checkbox and a "2 of 6 viewed" header ring (Conductor / Hoplite review idea, notes 03, 10) |
| `file-edit-chip/rename-morph` | Renames animate character by character from the old path to the new one |
| `file-edit-chip/prose` | Sentence form for consumer chat: "Edited auth.rs — added **12** lines, removed **3**" (Claude Code's wording style, note 08) |

## 8. Edge cases
- **Very fast edits.** An edit that applies inside `delay.row_guard` enters as `Applied` with final
  counts (no roll from 0).
- **Very long edits** (generated files with 10k lines): counts roll with the 25 % rule so they reach
  the target in ≈ 1 s whatever the size; magnitude widen happens at most once per digit boundary.
- **Same file edited many times in a turn.** One chip per file in the rollup, counts summed against
  the turn's starting version, "×3" tag; the transcript keeps per-call chips only in `Verbose`.
- **Counts unknown** (adapter sends no diff): chip shows the op word only ("Edited auth.rs"), no
  zeroes.
- **Generated / ignored / lock files.** Rollups put files matching the app's `low_signal` patterns
  (e.g. lockfiles) last under "and 2 generated files" (proposed).
- **Huge rollups** (100+ files): header + 5 rows + "and 95 more"; expanded list virtualised; "View all
  diffs" opens family H.
- **Long paths.** Middle-truncate keeping the basename and its parent (`src/…/auth/mod.rs`); full path
  in tooltip and accessible name.
- **RTL.** Chip mirrors; paths and counts stay LTR in bidi isolates; `├` tree glyphs mirror only in
  GUI variants (terminal keeps LTR glyphs).
- **Tiny sizes (16–20 px).** Counts only (`+12 −3`) with no file name; at 12–16 px a single dot
  coloured by net change with the counts in the tooltip.
- **Dense IDE.** `row` at 24 px with links as icon buttons (open ↗, diff ±) with tooltips.
- **Touch.** Chip is a 44 px target (padding grows, not the text); links become a trailing ⋯ sheet.
- **Colour-blind users.** Signs always shown; diff colours are tokens and the theme can swap green
  for blue (Claude Code daltonized themes, note 08).
- **Offline / slow file system.** "Open" shows `Opening…` up to 3 s, then an inline "Couldn't open"
  with Retry.

## 9. Accessibility
- **Accessible name:** "<file>, <op>, <n> lines added, <m> lines removed, <state>", e.g. "auth.rs,
  modified, 12 lines added, 3 lines removed, applied". Rollup: "Edited 6 files, 9 lines added, 9
  removed, collapsed".
- **Role.** Counter chip = link-like button ("View diff for auth.rs"); row links are separate buttons
  ("Open auth.rs", "View diff"); rollup header = tree item with `aria-expanded`.
- **Announcements.** Rolling counts are **never** announced per tick. One polite announcement when the
  turn's edits settle ("Edited 6 files"); conflicts and write failures announced once.
- **Keyboard.** Enter view diff; O open file; C copy path; ←/→ collapse/expand rollups; ↑/↓ between
  files; in `viewed-progress`, Space toggles Viewed.
- **Contrast.** Count colours ≥ 4.5:1 against the chip background in both themes (the `+`/`−` sign is
  the non-colour cue); struck-through rejected counts stay ≥ 4.5:1 (strike is decoration).
- **Focus.** Ring follows the chip's radius; focus stays on the chip when its state changes.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct FileEdit {
    pub call: CallId,                     // the Edit tool call (tools.md §9)
    pub path: FilePath,                   // workspace-relative
    pub op: EditOp,                       // Create | Modify | Delete | Rename { from } | Binary { before, after } | Mode
    pub added: Option<u32>,               // None when the adapter gives no diff
    pub removed: Option<u32>,
    pub applied: Option<(u32, u32)>,      // per-hunk partial apply
    pub state: EditState,                 // Writing | AwaitingApproval | Applied | Kept | PartiallyApplied
                                          // | Rejected | Reverted | Conflict | Failed { reason } | Cancelled
    pub diff: Option<DiffRef>,            // handle for family H
    pub edits_in_turn: u16,               // ×N tag
}

pub struct FileEditChip<'a> {
    pub edit: &'a FileEdit,
    pub agent: &'a AgentState,            // Writing only while the run is live
    pub variant: FileEditVariant,         // Counter | Row | TreeRollup | Pro(..)
    pub count_style: CountStyle,          // Default | Zed (thin space + figure dash) | Compact
    pub path_display: PathDisplay,        // Basename | MiddleTruncate { keep_parent: true } | Full
    pub roll: RollPolicy,                 // CountUp { gap: 0.25, tick_ms: 50 } | Instant
    pub labels: FileEditLabels,
    pub on_open: Option<Callback<FilePath>>,
    pub on_view_diff: Option<Callback<DiffRef>>,
    pub motion: MotionLevel,
}

pub struct FileEditRollup<'a> {
    pub edits: &'a [FileEdit],            // first-touch order, never re-sorted live
    pub show: usize,                      // 5 visible when > 8 files (proposed)
    pub low_signal: Vec<Glob>,            // sorted last, e.g. lockfiles
    pub expansion: Expansion,
}
```

## 11. Related components and references
**Related:** [tool-call.md](tool-call.md) (the Edit row hosts the chip) · [tools.md](tools.md)
(edit grouping, approval collapse rule) · family H diff viewer, per-hunk keep / undo, merge editor ·
family G approval footer · sidebar thread rows (family J) · run meter (part 2).

| Reference | What we take | Licence note |
|---|---|---|
| Codex CLI diff render (note 08 §1.1) | `Edited N files (+a -r)` rollup, `└` rows, `old → new` renames, Added / Deleted / Edited | MIT per licensing.md / Apache-2.0 per note 08 — portable with attribution; re-check |
| OpenCode AnimatedNumber + DiffChanges (note 08 §1.5) | 560 ms odometer with mask fade; 11 px counts in success / danger | MIT — port with attribution |
| Zed DiffStat (note 08 §1.9) | thin space + figure dash, comma grouping, tooltip | Mixed licence by crate — study only until checked |
| Cline DiffEditRow (note 08 §1.7) | `+N · -N` chip, 4 px stripe | Apache-2.0 — port with attribution |
| Claude Code / desktop (notes 03, 08) | `+12 −1` chip opens diff; prose counts; daltonized palette | Proprietary — reference only |
| Antigravity, Cursor, Paradev, Zed accordion (notes 03, 08) | footer review bar, `N Files +X−Y`, per-row badges, composer bar | Proprietary — reference only |
| Amp (note 08 §1.14) | no reshuffle; dot when a new order is ready | Proprietary — reference only |
| Thread-row diff badges [R016], [R053] | `+12 −3` on sidebar rows | Study only |
| "Agent is writing here" sweep [R034] | writing-sweep idea | Study only |
| Arc agent run card (note 12) | one-cell label swaps | Paid — study only |
