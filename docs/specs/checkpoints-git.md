**Status:** draft v0.1 (2026-11-11).

# Spec: checkpoints, diff & git (family H overview)

Overview of family **H** in [../component-taxonomy.md](../component-taxonomy.md) (target **30+
variants**). Same depth as the worked example [composer-send-control.md](composer-send-control.md).
Motion tokens are the shared ones from [composer.md](composer.md) §"Motion tokens", the family-F
tokens from [tools.md](tools.md) §5.2 and the loader rules in [loaders.md](loaders.md); family-H
tokens are added in §5 below. The compact `+12 −3` edit summary is family F
([file-edit-chip.md](file-edit-chip.md)); family H owns everything that **opens** from it.

Specs in this family:

| Spec | Component(s) | Variants (free + Pro) | Writer |
|---|---|---|---|
| this file | checkpoint model, capture layers, shared diff model, family tokens; `snapshot-banner` | 3 + 5 | part 1 |
| [checkpoint-timeline.md](checkpoint-timeline.md) | `checkpoint-timeline` (markers, hover preview, restore / rewind / fork, undo restore) | 4 + 14 | part 1 |
| [rollback-impact.md](rollback-impact.md) | `rollback-impact` (impact sheet before a restore) | 3 + 9 | part 1 |
| [diff-viewer.md](diff-viewer.md) | `diff-viewer` (split / unified / stacked, per-hunk keep / undo / comment, file list, minimap) | 4 + 16 | part 1 |
| [merge-conflict.md](merge-conflict.md) | `merge-conflict` (3-way editor) | 3 + 20 | parallel writer |
| [git-chips.md](git-chips.md) | `git-chips` (branch / worktree / sync chips) | 3 + 11 | parallel writer |
| [pr-card.md](pr-card.md) | `pr-card` (PR with CI checks) | 3 + 13 | parallel writer |
| [review-card.md](review-card.md) | `review-card` (review findings, batched comments) | 3 + 12 | parallel writer |

All values marked *proposed* are Oxyn's own starting points, to be tuned in the Studio. Numbers
attributed to other products come from the research notes named next to them (note numbers refer to
the project research folder; `[Rxxx]` IDs are in [../research/sources.csv](../research/sources.csv)).

## 1. Purpose
An agent changes real things: files, the conversation, its own plan, and sometimes the world outside
the workspace. Family H is **how the user looks at those changes, keeps or undoes them, and turns
them into shared history**. The user is trying to answer:
1. **What did it change?** (diff viewer, file list, counts)
2. **Can I go back?** (checkpoint timeline: restore files, rewind chat, fork)
3. **What will going back cost me, and what can't be undone?** (rollback impact)
4. **Is it ready to share?** (branch / worktree chips, PR card with CI, review card, merge conflicts)

Every coding agent we studied ships some of this (research note 03 §1E–1F, rows "Per-turn checkpoint
+ restore" and "Review pane / diff viewer", both table-stakes): Cursor shows checkpoint markers in the
chat timeline, previews files on click and restores **files only, messages kept**; Claude Code's
rewind menu offers restore code, restore conversation, both, or summarize from here; Kiro restores
files **and** context while its CLI rewind forks without touching files; Copilot forks a conversation
from a checkpoint; Replit's rollback dialog lists impacted files, agent memory and tasks with an
opt-in database checkbox (notes 03, 05 §2). Research note 03's recommendation is the core of this
family: **pick one model and label it clearly — restore files vs rewind conversation vs fork.**

Where family H appears:
- **Transcript:** checkpoint markers between turns; inline diffs under edit tool calls (expanded
  while awaiting approval, collapsed once applied — Warp's rule, note 08 §1.10, adopted in
  [tools.md](tools.md) §4.1).
- **Review surfaces:** a docked or full-screen diff viewer with a file list and review bar; review
  findings ([review-card.md](review-card.md)).
- **Composer region:** changed-files accordion ([file-edit-chip.md](file-edit-chip.md)), branch /
  worktree chips ([git-chips.md](git-chips.md)), PR status pill ([pr-card.md](pr-card.md)).
- **Side panels / history:** checkpoint list, version history for app builders.

## 2. The model (`oxyn-core::history`, proposed)

### 2.1 Checkpoint vs git commit
Two kinds of "going back" live side by side and must never be confused in the UI.

| | **Checkpoint** | **Git commit** |
|---|---|---|
| Made by | the agent host, automatically or on request ("Save checkpoint") | the user or the agent via git |
| Granularity | one per turn that wrote files (plus manual / pre-risk ones) | whatever the author chose |
| Captures | files **and** chat position **and** tool state (§2.2) | tracked files only |
| Lives | local to the session; expires under the host's retention | the repository; shared when pushed |
| Restoring | Restore files / Rewind chat / Both / Fork (§2.4) | checkout, revert, reset — git operations, owned by [git-chips.md](git-chips.md) and [pr-card.md](pr-card.md) |
| Glyph & label (proposed) | `◆` diamond, "Checkpoint · 3:42 PM" | `●` commit dot, short SHA `a1b2c3d` in mono |
| Examples | Cursor checkpoints are separate from Git; Claude Code snapshots expire with a cleanup period; Replit and Lovable keep their own history (note 05 §2) | — |

Rules (proposed):
1. **A checkpoint is never labelled as a commit.** If the host backs checkpoints with a shadow git
   repository (OpenCode's `/undo` `/redo` are git-backed, note 03), the SHA is an implementation
   detail shown only in a tooltip ("stored as a git object").
2. A checkpoint **may point at** the commit that was `HEAD` when it was captured (`git_head`), so the
   timeline can show "on `main` @ a1b2c3d". A commit **may be made from** a checkpoint ("Commit this
   state…") — that hands off to the git side and creates a normal commit.
3. Restoring a checkpoint **never runs a destructive git command** (no `reset --hard`, no force
   push). It writes files; git sees ordinary working-tree changes.

### 2.2 What a checkpoint captures
| Layer | Captured (proposed) | Fidelity | Restored by | Not captured |
|---|---|---|---|---|
| **Files** | before-image of every file the agent created, modified, deleted, renamed or chmod-ed in the turn; content hash of each file at capture time | `Exact`, or `Partial { skipped }` when a file exceeds `capture.max_file` (8 MB, proposed) or is excluded by the host's ignore rules | Restore files | files changed by shell commands **unless** the host diff-scans the workspace after each command (Claude Code documents that bash and manual edits are not tracked, note 05 §2); anything outside the workspace root |
| **Chat position** | message id + `RunLog` sequence number ([run-replay.md](run-replay.md) §9) of the last event before the turn | `Exact` | Rewind chat | — |
| **Tool state** | plan / todo snapshot ([plan-todo.md](plan-todo.md)), mode, permission level and session-scoped approvals, model + effort, working directory and environment of agent terminals, memory entries the agent wrote (if the host exposes them) | per item; the host declares which items it supports | Rewind chat (restored with the chat, proposed) | running processes, open network connections, background jobs ([background-jobs.md](background-jobs.md)) — they can be stopped, not rewound |
| **External effects** | **references only**: every tool call since the checkpoint that may have touched the world (network, database, package install, deploy, git push, sent messages) with its classification | n/a | **never** — listed in [rollback-impact.md](rollback-impact.md) as "can't be undone" | the effects themselves |

A checkpoint's capture card states its fidelity in words: "Files and chat · 3 files" / "Chat only
(no file changes)" / "Files partly captured · 1 file too large".

### 2.3 Checkpoint kinds
| Kind | Created | Marker (proposed) | Notes |
|---|---|---|---|
| `Auto` | before each turn that writes files (Cursor shows checkpoints only on turns that changed files, note 05 §2) | small `◆` on a hairline | default |
| `ChatOnly` | before each turn that wrote no files (host option, default on) | hollow `◇` | file actions hidden (Claude hides code options when no edits were captured, note 05 §2) |
| `Manual { name }` | user "Save checkpoint" (⌘S in the timeline, proposed) or `/checkpoint name` | `◆` + flag + name | never auto-pruned before `Auto` ones |
| `PreRisk { reason }` | automatically before a tool call classified high-risk (rollback-impact.md §2) | `◆` + shield | "Before `drop table`" |
| `Milestone { summary }` | agent-created at a stable state with a written summary (Replit's agent-created checkpoints carry a description, scope and cost, note 05 §2) | `◆` + summary line | summary only when the app supplies one |
| `RestorePoint { of }` | automatically right before a restore, so the restore itself can be undone | `↶◆` | hidden in compact lists; shown in the undo toast |

### 2.4 The four actions
| Action (label) | Files | Chat | Tool state | Thread | Example |
|---|---|---|---|---|---|
| **Restore files** | back to the checkpoint | kept; a system note "Files restored to 3:42 PM" is appended | kept | same | Cursor (note 05 §2) |
| **Rewind chat** | kept | turns after the checkpoint move to a rewound branch (§2.5); the rewound prompt returns to the composer (Cursor does this on restore, note 05 §2) | restored | same | Claude "Restore conversation" |
| **Restore files and chat** | back | rewound | restored | same | Claude "Restore code and conversation", Kiro Restore |
| **Fork from here** | untouched in this thread; the fork starts either on the current files or — if the host supports worktrees — on a new worktree at the checkpoint ([git-chips.md](git-chips.md)) | new thread starts at the checkpoint | copied into the fork | **new** | Copilot Fork Conversation, Kiro CLI rewind (note 03) |

"Summarize from here" (Claude, note 05 §2) is a Pro variant action
([checkpoint-timeline.md](checkpoint-timeline.md) `summarize-from-here`), not a core action, because
it depends on a model call.

### 2.5 Branches after a rewind
A rewind **hides, never deletes**: rewound turns move to a branch (`BranchId`) shown as a folded
"3 rewound turns · Bring back" row at the rewind point, until the host's retention drops them.
New turns after a rewind form the live branch (Replit: new work after a rollback forms an alternate
branch, with roll-forward, note 05 §2). Pro variant `checkpoint-timeline/tree` draws the whole tree
(Pi's `/tree` navigator idea, note 03).

### 2.6 Shared diff model
Every family-H view of change — the hover preview, the impact sheet's file list, the diff viewer, the
merge editor, the PR card's file count — reads one `ChangeSet` (§9): files with an op, stats and
hunks, each hunk with an origin (agent turn / user / unknown) and a review state. Diff computation
(line diff, word diff, moved-block detection) runs in `oxyn-core` off the UI thread or comes from the
host (e.g. a git patch); views never diff text themselves.

## 3. State machine (checkpoint lifecycle)
```
            turn starts writing (or manual save / pre-risk trigger)
  (none) ─────────────────────────────────────────────▶ Capturing ──capture fails──▶ CaptureFailed{reason}
                                                            │ captured
                                                            ▼
     ┌──────────────────────────────────────────────── Available ◀──────────── preview closed ──────┐
     │ retention passed / store lost                     │  │  │                                    │
     ▼                                                    │  │  └── preview ──▶ Previewing{read-only, snapshot-banner}
  Expired (actions hidden, "Expired · files no longer kept")  │                                      │ restore from banner
                                                          │  └── action chosen ──▶ ImpactPending ──▶ Confirming{impact}
                                                          │                                          │ cancel ─▶ Available
                                                          │                                          │ confirm
                                                          │                                          ▼
                                                          │                                 Restoring{done n / m}
                                                          │                   ┌──────────────┼──────────────────┐
                                                          │                   ▼              ▼                  ▼
                                                          │               Restored       PartiallyRestored   RestoreFailed{reason}
                                                          │       ("You are here")      {failed files}        (nothing changed; Retry)
                                                          │                   │ undo (RestorePoint)
                                                          │                   ▼
                                                          │               Undoing ──▶ Available (restore point becomes current)
                                                          │
                                                          └── agent running ──▶ Blocked{"Stop the agent to restore"} ──run ends──▶ Available
  Rewound turns' checkpoints ──▶ OnBranch{rewound} (dimmed; "Bring back" restores the branch)
  Store unreachable (remote / cloud agent) ──▶ Offline (actions disabled with reason; preview from cache if any)
```

`snapshot-banner` (owned here) has its own small machine:
`Hidden → Entering → Shown{checkpoint} → (Restore pressed → hands off to Confirming) | (Back to latest → Leaving → Hidden)`;
`Shown` also reacts to `Offline` ("Preview may be out of date") and to the live run moving on
("Agent made 2 new changes · Back to latest" counter rolls).

## 4. States table (family level)
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Capturing` | marker drawn at 40 % opacity with a 6 px `loader-glyph` dot (lease from the turn's busy region, not its own) | — | — | — |
| `Available` | marker, time, kind glyph, file count | open actions (hover / focus) | preview | Enter menu, P preview |
| `Previewing` | `snapshot-banner` "Viewing checkpoint from 3:42 PM · Back to latest · Restore…"; workspace views read-only | Back to latest | Restore… | Esc back, ⌘↩ restore… |
| `Confirming` | [rollback-impact.md](rollback-impact.md) sheet | confirm (label echoes action) | cancel | Enter confirm when focused, Esc cancel |
| `Restoring` | progress "Restoring 4 of 7 files"; transcript fold animation for chat rewinds | — | — (cancel only before the first write, proposed) | — |
| `Restored` | marker becomes "You are here" (accent); undo toast | Undo | View changes | ⌘Z while toast visible (§5) |
| `PartiallyRestored` | warning: "Restored 6 of 7 files · auth.rs is locked" | Retry failed | Undo all | R retry |
| `RestoreFailed` | danger line, "Nothing was changed" | Retry | Copy error | R retry |
| `Blocked` | actions disabled with reason "Stop the agent to restore" | Stop and restore (proposed) | — | — |
| `OnBranch` | dimmed marker inside the "rewound turns" fold | Bring back | Fork from here | Enter |
| `Expired` | marker with dashed outline, "Files no longer kept"; Rewind chat still allowed if the chat position exists | Rewind chat | — | — |
| `CaptureFailed` | marker with ! "Checkpoint not saved · disk full" | Retry capture (if host supports) | — | — |
| `Offline` | actions disabled, reason "Checkpoint store offline" | — | Retry | — |

## 5. Micro-interactions and timing (family-wide)

### 5.1 Micro-interactions
- **Marker reveal.** Markers sit as a 1 px hairline with a 6 px diamond; actions reveal on hover or
  focus after `delay.marker.hover`, so a pointer passing over the transcript does not flicker them.
- **Preview.** A popover follows the run-replay preview timing (200 ms hover, 140 ms fade) so the two
  timelines feel the same.
- **Restore choreography.** Files: affected file-edit chips in later turns morph to `Reverted`
  ([file-edit-chip.md](file-edit-chip.md) §5). Chat: turns after the point fold into the "rewound
  turns" row with the tool-stack turn-fold motion ([tools.md](tools.md) §5.3), then the rewound prompt
  flies into the composer with `dur.travel` (≤ 260 ms, composer.md).
- **Undo toast.** Enters from the composer edge; a 1 px countdown hairline depletes over
  `undo.window`; hover or focus pauses it.
- **Hunk resolve.** Keep / Undo morph a hunk's actions into a one-word chip ("Kept", "Undone") while
  the line tint fades to neutral (diff-viewer.md §4).
- **Numbers roll, words swap** (status-text.md): file counts, `+N −N`, "3 of 7 viewed".

### 5.2 Family-H tokens (proposed; added to `oxyn-tokens` / `oxyn-core` motion presets)
**Colour** (diff colours are tokens so themes can swap them; Claude Code's daltonized themes swap diff
green for blue, note 08 §1.3):

| Token | Default (proposed) | Used for |
|---|---|---|
| `color.diff.add.bg` / `color.diff.del.bg` | `success` / `danger` at 14 % alpha (dark), 10 % (light) | added / removed line rows |
| `color.diff.add.word` / `color.diff.del.word` | same hues at 34 % (dark), 26 % (light) | word / char highlights on top of the row tint (Claude has separate word-level colours, note 08 §1.3) |
| `color.diff.add.fg` / `color.diff.del.fg` | `success` / `danger` text step (≥ 4.5:1 on surface) | counts, `+`/`−` gutter signs (shared with file-edit-chip) |
| `color.diff.gutter.add` / `.del` | hue at 22 % (dark), 18 % (light) | line-number gutter cells |
| `color.diff.stripe` | hue at 100 %, 3 px | change-style `bars` (Cline uses a 4 px stripe, note 08 §1.7) |
| `color.diff.hunk.header` | `fg.muted` on `surface.sunken` | `@@` rows, "⋮" separators |
| `color.diff.kept` / `.undone` | `fg.muted` / `fg.muted` + strike | resolved hunks |
| `color.checkpoint.marker` / `.current` | `fg.muted` / accent | timeline |
| `color.impact.irreversible` | `danger` | "can't be undone" rows |
| `color.impact.external` | `warn` | "may have external effects" rows |
| Preset `codex` | dark add #213A2B / del #4A221D; light GitHub add #dafbe1 / del #ffebe9, gutter #aceebb / #ffcecb (note 08 §1.1) | Studio token preset |
| Preset `daltonized` | additions blue, deletions orange (proposed hues) | colour-blind users |

**Motion:**

| Token | Value | Used for |
|---|---|---|
| `delay.marker.hover` | 150 ms | before marker actions reveal |
| `dur.marker.reveal` | 120 ms `ease.out` opacity + x −4 → 0 | marker actions |
| `delay.preview` / `dur.preview.in` | 200 ms / 140 ms `ease.out` | hover previews (= run-replay.md) |
| `dur.sheet.enter` / `.exit` | 220 ms `spring.snap`, y +12 → 0 / 160 ms `ease.in` | impact sheet, action menu |
| `delay.skeleton` | 150 ms | impact / diff loading skeleton shows only after this |
| `dur.rewind.fold` | 280 ms `ease.in-out`, stagger 20 ms cap 6 | rewound turns collapse (= tools.md turn fold) |
| `dur.restore.tick` | per file, count roll `dur.digit.roll` | "Restoring 4 of 7" |
| `undo.window` | 10 s, paused on hover / focus, min 3 s after pointer leaves | undo toast |
| `dur.hunk.resolve` | 240 ms tint → neutral `ease.out`; actions → chip 160 ms `spring.snap` | keep / undo |
| `stagger.bulk` | 30 ms, cap 8, rest instant | accept all / reject all cascade |
| `dur.line.enter` | 120 ms opacity only (no height animation) | streaming diff lines |
| `dur.layout.switch` | 160 ms crossfade, scroll anchored to the focused hunk | split ↔ unified ↔ stacked |
| `dur.context.expand` | 180 ms `spring.snap` | "Show 24 hidden lines" |
| **Reduced motion** | every reveal / sheet / fold → 80 ms opacity crossfade; no y offsets; heights instant; counts swap; undo hairline replaced by a static "Undo (10 s)" label that updates every 1 s; streaming lines appear without fade | — |

### 5.3 Timing table (family defaults)
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Marker actions reveal | 120 ms | 150 ms | `ease.out` | — | 60 Hz |
| Hover preview | 140 ms fade | 200 ms | `ease.out` | — | 60 Hz |
| Sheet / menu enter | 220 ms | 0 | `spring.snap` | — | 60 Hz |
| Sheet / menu exit | 160 ms | 0 | `ease.in` | — | 60 Hz |
| Snapshot banner enter / leave | 200 ms y −8 → 0 / 160 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Rewind fold | 280 ms | after confirm | `ease.in-out` | 20 ms, cap 6 | 60 Hz |
| Prompt back to composer | ≤ 260 ms | after fold | `ease.in-out` | — | 60 Hz |
| Undo hairline | `undo.window` (10 s) | 0 | linear | — | 10 Hz |
| Hunk resolve | 240 ms + 160 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Bulk resolve cascade | 240 ms each | 0 | `ease.out` | 30 ms, cap 8 | 60 Hz |
| Streaming diff line | 120 ms | 0 | `ease.out` | — | ≤ 30 Hz batches |
| **Reduced motion** | 80 ms opacity crossfades, instant heights, no cascade, no hairline (1 Hz text countdown) | — | linear | none | event-driven |

### 5.4 Busy signal and frame budget
- Capturing and restoring use the turn's busy region; family H adds no loop of its own except the
  undo hairline (10 Hz) and the streaming diff (≤ 30 Hz line batches).
- A settled diff or timeline repaints nothing. Off-screen diff rows are virtualised (diff-viewer.md §7).

## 6. Variants

### 6.1 `snapshot-banner/*` (owned by this file)
The bar shown while the user previews a checkpoint or past version (note 05 §2 calls it
SnapshotPreviewBanner: "Viewing version from 3:42 PM · Back to latest · Restore"; Lovable has Back to
latest).

**Free (3)**
| Name | Idea | Surfaces |
|---|---|---|
| `snapshot-banner/bar` | Full-width bar above the content: clock glyph, "Viewing checkpoint from 3:42 PM", Back to latest, Restore… | all GUI |
| `snapshot-banner/pill` | Floating pill centred over the content; same actions as icon buttons with tooltips | desktop, chat, mobile |
| `snapshot-banner/terminal` | Reverse-video status line `VIEWING 15:42 · b back · r restore` | terminal |

**Pro (5)**
| Name | One-line idea |
|---|---|
| `snapshot-banner/sepia` | Previewed content gets a desaturated "past" filter; the live head stays in colour in a thumbnail corner |
| `snapshot-banner/drift-counter` | Shows "Agent made 2 new changes since" with a rolling count while the live run continues |
| `snapshot-banner/compare-toggle` | A Then / Now segmented switch that flips the view between the checkpoint and the live head |
| `snapshot-banner/edge-frame` | A thin accent frame around the whole window instead of a bar, for full-screen previews |
| `snapshot-banner/step-arrows` | ◀ ▶ arrows step to the previous / next checkpoint without leaving preview |

### 6.2 Family H variant catalog
Names follow [../variants.md](../variants.md). **F** = free, **P** = Pro. Rows for merge-conflict,
git-chips, pr-card and review-card are copied from their own specs as of 2026-11-11; **those specs are
authoritative** and may rename or recount.

| Component | Spec | F | P | Free names | Pro names |
|---|---|---|---|---|---|
| `snapshot-banner` | this file | 3 | 5 | bar, pill, terminal | sepia, drift-counter, compare-toggle, edge-frame, step-arrows |
| `checkpoint-timeline` | checkpoint-timeline.md | 4 | 14 | inline-marker, hover-revert, side-list, terminal | scrubber, tree, filmstrip, diff-peek, bookmarks, milestone-cards, ghost-rewind, branch-rail, cost-delta, summarize-from-here, composer-dock, day-heat, compare-pair, tape-rewind |
| `rollback-impact` | rollback-impact.md | 3 | 9 | sheet, inline, terminal | severity-meter, effects-timeline, outcome-tree, two-column, hold-to-confirm, file-diffs, compensations, receipt, blast-radius |
| `diff-viewer` | diff-viewer.md | 4 | 16 | unified, split, stacked, terminal | review-pane, inline-editor, minimap-heat, follow-agent, redline, image-compare, table-cells, json-tree, moved-blocks, semantic-fold, commit-filter, walkthrough, bar-style, hunk-cards, focus-dim, origin-lanes |
| `merge-conflict` | [merge-conflict.md](merge-conflict.md) | 3 | 20 | three-pane, inline-stacked, card | result-center, two-way, base-overlay, radar, commit-rail, review-queue, rationale-thread, confidence-gutter, word-merge, semantic-blocks, ghost-hover, binary-compare, rename-map, rebase-stepper, test-gated, lockfile-regen, minimap-heat, multiplayer, tui, focus-mode |
| `git-chips` | [git-chips.md](git-chips.md) | 3 | 11 | pill, inline, workspace-bar | prompt, split-flap, sync-ring, mini-rail, worktree-stack, lane-color, agent-owner, local-cloud, dirty-count, copy-burst, pixel |
| `pr-card` | [pr-card.md](pr-card.md) | 3 | 13 | card, compact, checks-list | commit-rail, flow, tick-tests, metro, auto-fix-loop, merge-when-ready, stack, story, review-avatars, preview-thumb, pill-expand, merged-arrival, tui |
| `review-card` | [review-card.md](review-card.md) | 3 | 12 | by-severity, by-file, inline-thread | walkthrough, verdict-chip, triage-metro, severity-bar, heat-gutter, fix-stack, batch-comments, multi-reviewer, confidence, fix-popover, ultrareview, tui |
| **Part 1 subtotal** (this writer) | | **14** | **44** | | **58** |
| **Part 2 subtotal** (parallel writer) | | **12** | **56** | | **68** |
| **Family H total** | | **26** | **100** | | **126** (target 30+) |

Counting rule (same as loaders.md §6): a variant differs in structure or motion. Diff colours, change
style presets that only recolour, density, radius and font are tokens and are **not** counted.

## 7. Edge cases (family level)
- **Shell edits not tracked.** If the host cannot see files changed by shell commands, every capture
  card and impact sheet says so once ("Changes made by commands aren't in checkpoints"), matching what
  Claude Code documents (note 05 §2). Hosts that diff-scan after commands set `tracks_shell: true`.
- **User edited files after the checkpoint.** Detected by content hash; the impact sheet lists them as
  "Your edits will be overwritten" with a per-file Keep mine (rollback-impact.md §3).
- **Expired snapshots.** File actions disappear, the marker says "Files no longer kept"; a restore that
  races expiry fails with "No files were restored" wording (Claude's error text style, note 05 §2) and
  changes nothing.
- **Cloud / background agents.** Cursor does not offer checkpoints for cloud agents (note 05 §2); hosts
  set `supports_restore: false` and markers render `Unavailable` with that reason.
- **Very fast agents.** Turns that write within `delay.row_guard` (150 ms) still get a checkpoint; the
  marker enters with the turn, not before.
- **Very long sessions** (500+ checkpoints): inline markers stay; the side list virtualises and groups
  by day (Lovable groups history by date, note 05 §2); `Auto` checkpoints older than retention prune
  first, `Manual` last.
- **Binary and huge files.** Captured up to `capture.max_file` (8 MB, proposed); larger files are
  `Partial` and named in the capture card.
- **Concurrent agents in one workspace.** Checkpoints are per thread; a restore that would overwrite a
  file another thread changed later is flagged in the impact sheet as "Changed by another agent".
- **RTL.** Timelines and file lists mirror; paths, code, SHAs, times and diffs stay LTR in bidi isolates.
- **Tiny sizes (16–20 px).** A marker collapses to a 6 px diamond with all actions in its menu; a diff
  never renders under 240 px wide (it opens in a sheet instead).
- **Dense IDE panels.** 24 px rows, icon actions with tooltips, no hover previews (Enter opens preview).
- **Touch.** Markers get a 44 px hit area; hover previews become long-press; destructive confirms are
  never swipe-only.
- **Slow networks / remote stores.** Previews and impact computation show skeletons after
  `delay.skeleton`; a restore never starts until the impact is computed or the user accepts a generic
  warning (rollback-impact.md §3 `ImpactUnavailable`).

## 8. Accessibility (family level)
- **Words, not just glyphs.** Every marker, hunk and file has a text state ("Checkpoint, 3:42 PM, 3
  files, available"; "Hunk 2 of 5, kept"). Diff lines carry `+` / `−` signs, never colour alone.
- **Destructive actions are named by effect.** Buttons say "Restore 7 files", "Rewind 3 turns",
  never "OK"; the impact sheet is a `dialog` with its summary as the description.
- **Live regions (polite):** restore result ("Restored 7 files to 3:42 PM. Undo available for 10
  seconds."), hunk decisions ("Hunk kept, 3 of 5 remaining"), streaming diff start/finish once per file.
  Failures are assertive once.
- **Focus.** After a restore, focus moves to the undo toast's Undo button only if focus was inside the
  sheet; otherwise it stays put. Closing the sheet returns focus to the marker that opened it.
- **Undo window.** Paused while the toast or its button has focus; screen-reader users can reach undo
  later through the `RestorePoint` checkpoint in the timeline (it never depends on the toast).
- **Contrast.** Diff text over row tints ≥ 4.5:1; word highlights ≥ 3:1 against the row tint (non-text
  contrast) and also get an underline in high-contrast mode.
- **Reduced motion.** §5.2 last row.

## 9. Props sketch (shared types in `oxyn-core::history`)
```rust
pub struct Checkpoint {
    pub id: CheckpointId,
    pub kind: CheckpointKind,          // Auto | ChatOnly | Manual { name } | PreRisk { reason }
                                       // | Milestone { summary } | RestorePoint { of: RestoreId }
    pub at: SystemTime,
    pub turn: TurnId,
    pub chat: ChatPosition,            // { message: MessageId, run_seq: u64 }  (RunLog, run-replay.md §9)
    pub files: FileCapture,            // { fidelity: Fidelity, paths: Vec<FilePath>, stats: DiffStats }
    pub tool_state: ToolStateCapture,  // which items exist: plan, mode, permissions, model, terminals, memory
    pub branch: BranchId,              // live branch or a rewound one
    pub git_head: Option<GitRef>,      // { branch, sha } when the workspace is a repo
    pub status: CheckpointStatus,      // Capturing | Available | Expired | CaptureFailed { reason } | Offline
}

pub enum Fidelity { Exact, Partial { skipped: Vec<(FilePath, SkipReason)> }, NotCaptured }

pub enum RestoreAction { Files, Chat, FilesAndChat, Fork { start: ForkStart } } // ForkStart: CurrentFiles | NewWorktree

/// Host-implemented; Oxyn components call it, never touch the disk themselves.
pub trait CheckpointStore {
    fn list(&self, thread: ThreadId) -> Vec<Checkpoint>;
    fn change_set(&self, from: CheckpointId, to: Head) -> Task<Result<ChangeSet>>;   // previews + diff
    fn impact(&self, id: CheckpointId, action: RestoreAction) -> Task<Result<Impact>>; // rollback-impact.md §9
    fn restore(&self, id: CheckpointId, action: RestoreAction) -> RestoreHandle;       // progress stream
    fn undo(&self, restore: RestoreId) -> RestoreHandle;
    fn capabilities(&self) -> StoreCaps;  // supports_restore, tracks_shell, worktrees, retention, max_file
}

/// Shared diff model (§2.6), read by every family-H view.
pub struct ChangeSet { pub base: Head, pub target: Head, pub files: Vec<FileChange> }
pub struct FileChange {
    pub path: FilePath, pub op: EditOp,          // EditOp from file-edit-chip.md §10
    pub stats: DiffStats,                        // added, removed, hunks
    pub content: DiffContent,                    // Text { hunks: Vec<Hunk> } | Binary { before, after, mime }
                                                 // | TooLarge { bytes, lines } | Unavailable { reason }
    pub review: FileReview,                      // Unviewed | Viewed | AllKept | AllUndone | Mixed
}
pub struct Hunk {
    pub id: HunkId, pub old: LineRange, pub new: LineRange,
    pub lines: Vec<DiffLine>,                    // Context | Added | Removed, with word spans
    pub origin: Origin,                          // Agent { turn, agent } | User | Unknown
    pub state: HunkState,                        // diff-viewer.md §3
}

pub struct SnapshotBanner<'a> {
    pub checkpoint: &'a Checkpoint,
    pub agent: &'a AgentState,                   // live run keeps going → drift counter
    pub drift: u32,                              // changes since, for drift-counter
    pub variant: SnapshotBannerVariant,          // Bar | Pill | Terminal | Pro(..)
    pub on_back: Callback<()>,
    pub on_restore: Callback<CheckpointId>,
    pub motion: MotionLevel,
}
```

## 10. Related components and references
**Related:** [file-edit-chip.md](file-edit-chip.md) (summary chip that opens the diff) ·
[tool-call.md](tool-call.md) and [tools.md](tools.md) (edit calls, expand-while-pending rule) ·
[run-replay.md](run-replay.md) (`RunLog`, checkpoint markers on the replay track) ·
[plan-todo.md](plan-todo.md) (plan snapshot) · [background-jobs.md](background-jobs.md) (processes that
can't be rewound) · [streaming-markdown.md](streaming-markdown.md) §3.3 (highlighter reused by the diff)
· [composer.md](composer.md) (rewound prompt returns here) · family G approvals (pre-risk checkpoints) ·
[merge-conflict.md](merge-conflict.md) · [git-chips.md](git-chips.md) · [pr-card.md](pr-card.md) ·
[review-card.md](review-card.md).

| Reference | What we take | Licence note |
|---|---|---|
| Cursor checkpoints (notes 03 §1E, 05 §2) | markers only on turns that changed files; preview on click; files-only restore; prompt back to composer | Proprietary — reference only |
| Claude Code rewind (note 05 §2) | four scopes incl. summarize; code options hidden without edits; shell edits untracked; expiry error | Proprietary — reference only |
| Kiro, Copilot (note 03 §1E) | restore files + context; fork from checkpoint | Proprietary — reference only |
| Replit checkpoints (note 05 §2) | agent-created milestones with description; impact dialog with database opt-in; roll forward / branches | Proprietary — reference only |
| Lovable history (note 05 §2) | grouped by date, Back to latest, revert creates a new version, bookmarks | Proprietary — reference only |
| OpenCode `/undo` `/redo` (note 03) | git-backed checkpoint store | MIT — port with attribution |
| Pi `/tree` (note 03) | session tree navigator | Not in licensing.md — study only until checked |
| Codex CLI diff colours (note 08 §1.1) | `codex` token preset | MIT per licensing.md (note 08 says Apache-2.0); portable with attribution — re-check |
| Claude Code themes (note 08 §1.3) | separate word-level diff colours; daltonized swap | Proprietary — reference only (we define our own values) |
| Cline DiffEditRow (note 08 §1.7) | 4 px stripe change style | Apache-2.0 — port with attribution |
| AI Elements Checkpoint (note 05 §1) | inline icon + trigger row | Not in licensing.md — study only until checked |
| affectUI Agent Checkpoints, Inline Diff, Change Summary (note 11) | names / scope only | Paid / unclear — study only |
| signoff-ui DiffReview [R107] | per-hunk review, unified or split | MIT — may port with attribution |
| Thread-row diff badges [R016]; ready-to-merge card [R027]; merge-conflict gen-UI [R054] | how H surfaces in sidebars and composers | Study only |
