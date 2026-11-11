**Status:** draft v0.1 (2026-11-11).

# Spec: 3-way merge-conflict editor (`merge-conflict`)

Part of family H, "Checkpoints, diff & git" (overview in [checkpoints-git.md](checkpoints-git.md);
taxonomy in [../component-taxonomy.md](../component-taxonomy.md)). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). **Hero component: target 3 free + 20 Pro = 23
variants.** Hunk rendering, the line gutter, syntax highlighting, intra-line (word) highlights,
split / stacked layout and virtualised scrolling are **reused from [diff-viewer.md](diff-viewer.md)**;
this spec adds the three-input model, the result pane, the conflict navigator, resolution actions,
the agent's suggestions and the merge lifecycle. Sibling specs: [git-chips.md](git-chips.md) (branch
names on each pane), [pr-card.md](pr-card.md) (the card that says "3 conflicts" and opens this),
[review-card.md](review-card.md), [checkpoint-timeline.md](checkpoint-timeline.md) and
[rollback-impact.md](rollback-impact.md) (the safety net before "Complete merge").

## 1. Purpose
An agent rebases a worktree onto `main`, merges a teammate's branch, or opens a PR that "can't merge
cleanly". The user wants to **understand each clash, decide it fast (or let the agent decide and check
its work), make sure nothing is left half-resolved, and either finish or back out safely**.

What we saw:
- Merge-conflict UI is **"thinly documented everywhere — design opportunity"**: only Conductor
  ("Verify, review, and resolve conflicts") and Verdent (coordinated rebasing, "Rebase / Sync with
  main") mention it; no resolver screenshots or descriptions were found (research note 03, §1F and
  findings).
- pitus_ generative-UI #2: a PR card "Refresh sessions in the auth middleware #128" with a commit-graph
  rail and branch chips; "Checking it merges cleanly" → "⚠ 3 conflicts with main" → **Resolve
  conflicts** → "Resolving" with "Running 36 of 42 tests" → checks row → **Merge** → Merged; the
  composer pill "#128 3 conflicts · +482 −37"; conflicting rows turn amber and later resolve [R054];
  a v1 → v2 redesign of the same flow [R055] (note 15).
- Hoplite's long-running story lists "rebased" between review comments and "merged #212-215" and
  sells "Verify and merge" as a step (note 10).
- affectUI lists a **Conflict Radar** (name only, Pro, study only, note 11).
- signoff-ui ships per-hunk DiffReview (unified or split), MIT [R107] (note 16).
- Paradev draws green / red markers in the diff scrollbar (note 03) — reused as the conflict minimap.

Where it appears:
- **IDE / desktop**: a full editor tab or pane (`three-pane`), opened from the PR card, the status-strip
  PR pill ([composer-status-strip.md](composer-status-strip.md)), a file-edit chip in `Conflict`
  ([file-edit-chip.md](file-edit-chip.md)) or a git chip in `Merging` / `Rebasing`.
- **Chat / mobile**: an inline card in the transcript (`card`) that summarises and runs the agent
  resolution, then opens `inline-stacked` for review.
- **Terminal**: Pro `tui`.

## 2. Anatomy and vocabulary
```
┌ Merging feature/auth-refresh → main · 5 conflicts in 3 files ─── ✦ Resolve all with agent · Abort merge ┐
│ files ▸ ● auth.rs 2   ● session.rs 2   ◆ logo.png (binary)                 3 unresolved · 2 resolved     │  file strip
├──────────────────────────────┬──────────────────────────────┬──────────────────────────────┬──────────┤
│ CURRENT · main  a1b2c3d      │ BASE · merge-base 9f8e7d6    │ INCOMING · feature/auth…  e4f5 │ minimap  │  input panes
│ 41  let ttl = 300;           │ 41  let ttl = 600;           │ 41  let ttl = cfg.ttl();      │   ▮ amber │  (diff-viewer
│ 42  log::info!("refresh");   │ 42                           │ 42                            │   ▮ green │   rendering)
├──────────────────────────────┴──────────────────────────────┴──────────────────────────────┤   ▮       │
│ RESULT  src/auth.rs                                                    ⚑ 1 marker left       │          │  result pane
│ 41 ┌ conflict 2 of 5 ─────────────────────────────────────────────────── unresolved ┐       │          │  (editable)
│    │ ‹ ›  Accept current · Accept incoming · Accept both ▾ · ✦ Agent suggestion     │       │          │  action bar
│    │ ✦ "Use incoming cfg.ttl(); keep current's log line — both changes are needed." │       │          │  rationale
│ 42 └────────────────────────────────────────────────────────────────────────────────┘       │          │
├─────────────────────────────────────────────────────────────────────────────────────────────┴──────────┤
│ ✓ Mark file resolved                                   Complete merge (disabled · 3 unresolved) ⌘⇧↵   │  footer
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```
**Vocabulary (labels are tokens in `MergeLabels`, proposed defaults):**
| Term | Means | Shown as |
|---|---|---|
| Current | the side checked out (`HEAD`) — git's "ours" in a merge | "Current · <branch>" |
| Incoming | the side being merged in — git's "theirs" in a merge | "Incoming · <branch>" |
| Base | the merge base (common ancestor) | "Base · <short sha>" |
| Result | the working-tree file being written | "Result" |
| Conflict | one region where current and incoming both changed the base | "conflict 2 of 5" |
| Marker | a literal `<<<<<<<`, `|||||||`, `=======` or `>>>>>>>` line still in the result | "⚑ 1 marker left" |

**Rebase / cherry-pick swap.** During a rebase git's "ours" is the branch being rebased **onto** and
"theirs" is the commit being replayed. Panes are therefore always labelled with **branch or commit
names first** and "current / incoming" second; the adapter supplies which is which (`MergeKind`, §10),
and the component never infers it from the words ours / theirs.

## 3. State machine

### 3.1 Session (the whole merge)
```
                    adapter reports a conflicted operation
  (none) ─────────────────────────────▶ Detecting ──index read──▶ Loading{files} ──▶ Ready{unresolved n}
                                           │                                          │   ▲
                                           └─ no conflicts (raced) ─▶ Clean ──────────┘   │ user resolves the last
                                                                                          │ conflict / marks files
   Ready ── ✦ Resolve all ──▶ AwaitingApproval? ──allow──▶ AgentResolving{done/total} ───▶ ReviewingAgent{k to review}
     │      (permission selector says ask)   └─deny──▶ Ready                    │  stop ──▶ Ready (partial suggestions kept)
     │                                                                          └─ agent error ──▶ Ready + banner
     ├── file changed on disk / new conflicts appear ──▶ Stale ──reload──▶ Loading
     ├── all conflicts resolved + 0 markers ───────────▶ AllResolved ──Complete──▶ Verifying? (tests, Pro test-gated)
     │                                                                                 │ pass       │ fail
     │                                                                                 ▼            ▼
     │                                                                            Committing ──▶ Completed{sha}
     │                                                                                 └─ hook / write error ─▶ CommitFailed{reason} ─▶ AllResolved
     ├── Abort ──▶ ConfirmAbort ──confirm──▶ Aborting ──▶ Aborted (files restored)   └─cancel─▶ Ready
     │                                          └─ error ─▶ AbortFailed{reason}
     ├── rebase step done ──▶ NextStep{k of N} ──▶ Detecting  (Continue rebase replaces Complete)
     └── offline / backend gone ──▶ Offline (read-only; edits kept locally, actions disabled-with-reason)
  Failed{reason}: index unreadable, file too large, unsupported conflict type (read-only fallback view).
```

### 3.2 Per conflict
```
  Unresolved ──focus──▶ Focused ──hover an action──▶ Previewing{choice} (ghost in result) ──leave──▶ Focused
      │                    │
      │                    ├─ Accept current / incoming / both (order ▾) ─▶ Resolved{choice}
      │                    ├─ ✦ Agent suggestion ─▶ Suggesting (streaming) ─▶ Suggested{text, rationale, confidence}
      │                    │                              │ agent declines                │ accept        │ reject
      │                    │                              ▼                               ▼               ▼
      │                    │                        AgentDeclined{why}            Resolved{Agent}   Unresolved (suggestion kept, dimmed)
      │                    └─ user types in the result ─▶ EditedManually ─(markers gone)─▶ Resolved{Manual}
      └─ Resolve all ran ─▶ Suggested (queued for review) ─accept─▶ Resolved{Agent, reviewed}
  Resolved ── Undo (⌘Z) / Reopen ──▶ Unresolved      Resolved{Agent, unreviewed} counts as unresolved
                                                       for "Complete merge" unless policy allows (§8)
```

### 3.3 Per file
`Conflicted{n}` → `PartlyResolved{k of n}` → `AllResolved` → `MarkedResolved` (staged). Special
types: `Binary` (pick current or incoming whole file), `ModifyDelete` (keep modified / accept
delete), `RenameRename{a, b}` (pick a name, content merged separately), `RenameDelete`,
`AddAdd` (both created the same path; 2-way, no base), `Mode` (executable bit), `Submodule` (pick a
commit), `Generated` (lockfile: "Regenerate" suggestion instead of a text merge).

Presentation layers on every interactive element: Rest · Hovered · Focused · Pressed · Disabled{reason}.

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Detecting` / `Loading` | header "Checking conflicts…" with a row loader; panes as skeletons (3 columns of line bars, [skeletons.md](skeletons.md)) | — | Abort merge | Esc closes the pane (merge stays in progress) |
| `Ready` | header "5 conflicts in 3 files", file strip, navigator "conflict 1 of 5" focused on the first unresolved | resolve the focused conflict | ✦ Resolve all, Abort | Alt+↓ / Alt+↑ next / previous conflict; Alt+1/2/3/4 accept current / incoming / both / agent |
| `Focused` (conflict) | amber outline + amber gutter bar in all four panes, aligned; action bar under it in the result | Accept … | Edit in result | Alt+1–4; Enter opens "Accept both ▾" order menu |
| `Previewing{choice}` | the result region shows the chosen text as a ghost (60 % opacity, dashed outline); source lines tinted in their pane | click commits | — | releasing focus restores |
| `Suggesting` | ✦ glyph with text-shimmer "Thinking about conflict 2…" ([text-shimmer.md](text-shimmer.md)); ghost text streams in the result | Stop | — | Esc stops this suggestion |
| `Suggested` | ghost text + ✦ rationale line (≤ 2 lines, "More" expands) + confidence chip `high / medium / low` when supplied; diff of suggestion vs current and vs incoming on hover | Accept suggestion | Reject, Edit, Ask again (opens composer prefilled) | Alt+4 accept; Alt+Backspace reject |
| `AgentDeclined` | "Agent couldn't merge this: both sides rewrite `refresh()` differently" | pick manually | Ask again | — |
| `Resolved{choice}` | outline turns `color.merge.resolved` (success, low chroma), small tag "current / incoming / both / agent / edited"; navigator skips it | — | Reopen | Alt+R reopen (proposed) |
| `EditedManually` | caret in the result; marker counter live | — | Mark resolved | typing; ⌘Z undo |
| `AgentResolving{done/total}` | header progress "Resolving 3 of 5" (determinate bar), each conflict flips to `Suggested` as it arrives; panes stay interactive | Stop | — | Esc stops (keeps finished ones) |
| `ReviewingAgent{k}` | review banner "Agent resolved 5 conflicts · 5 to review"; navigator jumps between `Suggested` only; "Accept all reviewed" stays disabled until each is opened once (policy, §8) | Accept (per conflict) | Accept all (policy-gated), Reject all | Alt+↓ next unreviewed; Alt+4 accept |
| `AllResolved` | header "All conflicts resolved"; Complete merge enabled; marker counter 0 | Complete merge / Continue rebase | Review changes (opens diff-viewer of result vs current) | ⌘/Ctrl+Shift+Enter |
| Markers left | "⚑ 2 markers left" chip (danger tone); Mark resolved shows "Markers still in file · Mark anyway" | jump to marker | Mark anyway | F8 / Shift+F8 next / previous marker (proposed) |
| `Verifying` (Pro) | "Running 36 of 42 tests" progress [R054] | Stop | Skip checks (if policy) | — |
| `Committing` | Complete button label → "Completing…" in the same cell | — | — | — |
| `Completed{sha}` | ✓ "Merged into main · a1b2c3d" (or "Rebase step 3 of 7 done"); pane read-only; auto-closes after 4 s in chat `card`, stays in IDE | Close | Open commit, Undo merge (via checkpoint, [checkpoint-timeline.md](checkpoint-timeline.md)) | Enter close |
| `CommitFailed{reason}` | danger banner "Pre-commit hook failed: lint" + Copy output | Retry | Open terminal output | R retry |
| `ConfirmAbort` | inline confirm (not a modal in chat): "Abort merge? Your 3 resolutions will be lost." + impact from [rollback-impact.md](rollback-impact.md) | Abort merge (danger) | Keep resolving | Enter confirms, Esc cancels |
| `Aborting` / `Aborted` | "Aborting…" → "Merge aborted · files restored to main" | Close | — | — |
| `Stale` | amber banner "auth.rs changed on disk · Reload"; resolutions kept where text still matches, others reopened | Reload | Keep mine (overwrite) | — |
| `Offline` | banner "Offline · changes saved locally"; Complete / Abort / Agent disabled-with-reason | — | — | — |
| `Failed{reason}` | read-only raw view of the file with markers, reason, "Open in editor" | Open in editor | Copy path | — |
| Binary file | two thumbnails / size rows "current 24 KB · incoming 31 KB"; Accept current / Accept incoming only | Accept … | Open both | Alt+1 / Alt+2 |
| Rename conflict | "Renamed differently: current → `auth/mod.rs`, incoming → `auth/session.rs`" with radio choice + "Keep both" | Choose name | — | ↑/↓ choose, Enter |
| Modify / delete | "Deleted in incoming, modified in current" + diff of the modification | Keep file | Delete file | Alt+1 / Alt+2 |
| Generated (lockfile) | "Lockfile conflict · Regenerate with `cargo update -w`" (command from the adapter, never invented) | Regenerate (runs a tool call, family F) | Pick a side | — |
| Disabled-with-reason | any action: tooltip + inline reason ("Agent unavailable offline", "Complete merge: 3 unresolved") | — | — | focusable, reason read out |

## 5. Micro-interactions
- **Conflict focus.** Navigator move scrolls all four panes together so the conflict's first line sits
  at 30 % of viewport height (proposed); the amber outline draws in (opacity 0 → 1, 160 ms); the
  previous conflict's outline fades. Panes **scroll-sync by aligned regions**, not by line number, so
  a 2-line current vs 12-line incoming region stays aligned (spacer rows as in [diff-viewer.md](diff-viewer.md)).
- **Navigator count.** "conflict 2 of 5" rolls digits with `dur.digit.roll` (280 ms); the total rolls
  **down** as conflicts resolve; the "unresolved" counter in the file strip and in the status-strip PR
  pill roll together (one `PrStatus.conflicts` value, [composer-status-strip.md](composer-status-strip.md) §3).
- **Hover preview.** Hovering Accept current / incoming / both shows the ghost result after 120 ms
  (hover-intent), tints the source region in its pane (`color.merge.current` / `.incoming` at 18 %
  alpha); no layout shift: the ghost replaces the region's height with a 160 ms `spring.snap` tween.
- **Accept.** The chosen lines **travel** from the source pane into the result (a translucent copy,
  `dur.travel` 240 ms `ease.in-out`, capped at 12 lines; larger regions crossfade instead); the outline
  morphs amber → resolved (300 ms colour ease in OKLab) and the tag fades in. Navigator auto-advances to
  the next unresolved after 400 ms (proposed; off with `auto_advance = false`).
- **Accept both ▾.** Default order current-then-incoming; the menu offers "Incoming first" and
  "Combine line by line" (only when the regions don't overlap at line level — the adapter says so).
- **Agent suggestion.** Ghost text streams line by line using [text-reveal.md](text-reveal.md)
  `line` reveal; the ✦ glyph shimmers while streaming; the rationale appears after the text, one line,
  with a 160 ms fade. Confidence chip colour is neutral (never green) — it is the agent's claim, not a
  verification.
- **Resolve all.** A determinate bar fills per finished conflict (no fake progress); finished conflicts
  flip to `Suggested` with a 120 ms stagger cap 6 if several land in one frame; the minimap marks turn
  amber → violet-neutral (`color.merge.suggested`). The header text rolls "Resolving 3 of 5".
- **Review.** Each `Suggested` conflict shows an "unreviewed" dot; opening it for ≥ 600 ms (proposed)
  or pressing Accept clears the dot. "Accept all" enables only when no dots remain (default policy).
- **Reject suggestion.** Ghost dims to 30 % and collapses to a one-line "Suggestion rejected · Show"
  stub (200 ms); the conflict returns to amber.
- **Markers counter.** Typing that removes the last marker of a region flips the region to
  `EditedManually → Resolved{Manual}` after a 600 ms idle debounce; the counter rolls; adding a
  marker back reopens it. Markers in the result are highlighted with `color.merge.marker` background
  and a gutter ⚑.
- **Mark file resolved.** ✓ stroke draws (`dur.check.draw`), the file's chip in the strip morphs ● → ✓
  and slides to the end of the strip (200 ms `spring.snap`, after a 300 ms delay so it never moves
  under the pointer).
- **Complete merge.** Button label swaps to "Completing…" in one fixed-width cell; on success the
  header morphs to ✓ "Merged into main" (`dur.row.morph` + `dur.check.draw`); the pane goes read-only
  with a 200 ms desaturate. No confetti in free variants.
- **Abort.** The confirm expands inline under the button (`dur.expand`); on confirm, conflict outlines
  fade out and panes crossfade to the restored file (240 ms). Error → 2 × 3 px nudge of the banner.
- **Stale.** Banner slides from the pane top (200 ms `ease.out`); regions whose text no longer matches
  pulse amber once (600 ms) and reopen.
- **Minimap.** One mark per conflict on the right rail (amber unresolved, violet suggested, success
  resolved, danger markers-left); click scrolls; current conflict mark is 2 px wider.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Focus scroll (4 panes synced) | 260 ms | 0 | `ease.in-out` | — | 60 Hz |
| Focus outline in / out | 160 ms / 120 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Navigator / counter digit roll | 280 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz during roll |
| Hover preview ghost | 160 ms height tween | 120 ms hover-intent | `spring.snap` | — | 60 Hz |
| Accept travel (≤ 12 lines) | 240 ms | 0 | `ease.in-out` | — | 60 Hz |
| Accept crossfade (> 12 lines) | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Outline amber → resolved | 300 ms | 0 | linear in OKLab | — | 30 Hz |
| Auto-advance to next conflict | as focus scroll | 400 ms (proposed) | `ease.in-out` | — | 60 Hz |
| Suggestion stream | line reveal 120 ms per line | first line after first token | `ease.out` | 40 ms per line | 30 Hz |
| ✦ shimmer while suggesting | 1400 ms sweep / 800 ms rest | 600 ms | `ease.coast` | — | 30 Hz |
| Rationale fade-in | 160 ms | after text completes | `ease.out` | — | 60 Hz |
| Resolve-all bar step | 300 ms per change | 0 | `ease.out` | 120 ms cap 6 | 30 Hz |
| Unreviewed dot clear | 120 ms | 600 ms dwell (proposed) | `ease.out` | — | 60 Hz |
| Reject collapse | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Manual-resolve debounce | — | 600 ms idle (proposed) | — | — | — |
| Mark-resolved ✓ + chip move | 160 + 240 ms; move 200 ms | move after 300 ms | `spring.snap` / `ease.out` | — | 60 Hz |
| Complete → ✓ header | 160 ms morph + 240 ms stroke | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Read-only desaturate | 200 ms | after ✓ | `ease.out` | — | 30 Hz |
| Abort confirm expand / panes restore | 220 ms / 240 ms | 0 | `spring.snap` / `ease.in-out` | — | 60 Hz |
| Stale banner / region pulse | 200 ms / 600 ms ×1 | 0 | `ease.out` / `ease.in-out` | 60 ms cap 6 | 60 Hz |
| Error nudge | 300 ms (2 × 3 px) | after morph | linear | — | 60 Hz |
| `card` auto-close after Completed | 240 ms collapse | 4 s hold | `ease.in` | — | 60 Hz |
| **Reduced motion** | no travel, no ghost height tween (ghost appears in place), scrolls jump (`scroll-behavior: auto`), outlines and tints swap instantly with an 80 ms opacity crossfade, counters swap without rolling, no shimmer (static ✦ + "Suggesting…"), suggestion text appears in one block when complete (a "streaming" label shows meanwhile), no pulses or nudges (a static icon + text carry the meaning), chip does not slide (re-sorts on next open) | — | linear | none | event-driven only |

Motion tokens are the family-F set ([tools.md](tools.md) §5.2) plus `dur.travel` from
[composer.md](composer.md). Proposed colour tokens (to live in [checkpoints-git.md](checkpoints-git.md)
if the family overview defines them): `color.merge.current` (blue-ish), `color.merge.incoming`
(teal-ish), `color.merge.base` (neutral), `color.merge.conflict` (= `color.loader.waiting`, amber),
`color.merge.suggested` (low-chroma violet), `color.merge.resolved` (low-chroma success),
`color.merge.marker` (danger at 16 % alpha). Current / incoming hues avoid the add / remove greens and
reds so a diff and a merge never read the same.

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `merge-conflict/three-pane` | Classic layout: Current · Base · Incoming above an editable Result, synced scroll, navigator, minimap, footer with Mark resolved / Complete merge. Collapses to two input panes (base hidden behind a "Show base" toggle) under 1100 px (proposed) | desktop, ide |
| `merge-conflict/inline-stacked` | One column: the result file with each conflict expanded inline as a stacked block (Current, then Incoming, Base on demand) and the action bar under it; best for review on narrow panes and mobile | chat, mobile, ide (narrow) |
| `merge-conflict/card` | Transcript card in the pitus flow [R054]: "⚠ 5 conflicts with main" → file list with counts → ✦ Resolve with agent → "Resolving 3 of 5" → "5 to review · Review" (opens `inline-stacked`) → Complete merge → Merged | chat, desktop, mobile |

### Pro (20)
| Name | One-line idea |
|---|---|
| `merge-conflict/result-center` | Current · Result · Incoming side by side, result in the middle, arrows on each region push a side into the result |
| `merge-conflict/two-way` | Base never shown; differences from each side to base painted as intra-line highlights in the input panes |
| `merge-conflict/base-overlay` | Base appears as a ghost underlay inside each input pane on hold of ⌥ (no third pane) |
| `merge-conflict/radar` | A radial "conflict radar" overview: files as sectors, conflicts as blips by size, coloured by state (affectUI Conflict Radar name only, note 11) |
| `merge-conflict/commit-rail` | Left rail draws the two branch lines meeting at the merge base with branch chips, conflicting commits marked amber (pitus rail [R054]) |
| `merge-conflict/review-queue` | After Resolve all, suggestions are a queue of cards: one at a time, Accept / Reject / Edit, a "4 of 5 reviewed" ring |
| `merge-conflict/rationale-thread` | Each suggestion's rationale is a small thread; the user replies to steer ("keep ours' timeout") and the agent re-suggests in place |
| `merge-conflict/confidence-gutter` | Gutter bar shaded by the agent's stated confidence per conflict; low-confidence ones sort first in review |
| `merge-conflict/word-merge` | Token-level merge: when both sides edit different words on the same line, offer a combined line with each word's origin tinted |
| `merge-conflict/semantic-blocks` | Syntax-aware grouping: conflicts shown per function / item (from the host's parser), "3 conflicts in `fn refresh`" |
| `merge-conflict/ghost-hover` | Hovering any line of an input pane previews taking just that line into the result (line-level picking) |
| `merge-conflict/binary-compare` | Binary/image conflicts as swipe, onion-skin or side-by-side with size and dimension deltas |
| `merge-conflict/rename-map` | Rename/rename and rename/delete as a small node graph (old path → two new paths), pick by clicking a node |
| `merge-conflict/rebase-stepper` | For rebases and cherry-picks: a stepper "commit 3 of 7 · <subject>" across the top with per-step conflict counts and Continue / Skip commit |
| `merge-conflict/test-gated` | Complete merge first runs the app's checks with a live "Running 36 of 42 tests" bar; failures link to the failing file [R054][R037] |
| `merge-conflict/lockfile-regen` | Generated files grouped under "2 generated files · Regenerate", runs the adapter-supplied command as a tool call and shows its terminal block ([terminal-output.md](terminal-output.md)) |
| `merge-conflict/minimap-heat` | Full-height scrollbar heat strip for very large files (Paradev-style markers, note 03), conflicts as amber ticks, markers as red |
| `merge-conflict/multiplayer` | Presence avatars on conflicts a teammate or another agent is resolving; regions lock with "Ines is on this" |
| `merge-conflict/tui` | Terminal look: box-drawn panes, `<<<<<<< ======= >>>>>>>` regions, single-key actions (`o` current, `t` incoming, `b` both, `a` agent, `n`/`p` navigate, `m` mark) |
| `merge-conflict/focus-mode` | One conflict fills the view, everything else hidden; a dot row "● ● ○ ○ ○" at the bottom; swipe left/right on touch |

## 8. Edge cases
- **Agent-resolved but unreviewed.** Default policy (proposed): `Resolved{Agent, reviewed: false}`
  counts as unresolved for Complete merge; the app can relax it with `ReviewPolicy::TrustAgent` (e.g.
  autonomy set to full in [composer-permission-selector.md](composer-permission-selector.md)), and then
  the Complete button reads "Complete merge · 5 unreviewed".
- **Agent writes outside the conflict.** A suggestion may only replace its own region; any change
  elsewhere is shown as a separate `EditedManually` region tagged "agent" so it is reviewable.
- **Conflict counts disagree.** The adapter's count wins in headers; the marker scan is a second,
  local signal ("⚑ markers left") and never silently overrides the adapter.
- **Markers in content on purpose** (docs about git, test fixtures): "Mark anyway" exists for every
  file; the counter then shows "⚑ 2 (kept)".
- **Huge files** (≥ 20k lines, proposed threshold): panes virtualise (diff-viewer); minimap becomes
  the heat strip; travel animation is disabled; syntax highlighting falls back to plain text above
  the host's highlighter limit with a "Highlighting off for large file" note.
- **Huge merges** (100+ files): file strip becomes a filterable list (unresolved first, then by
  conflict count); Resolve all processes in batches of 10 (proposed) with Stop between batches.
- **Very fast agent.** A suggestion that completes inside `delay.row_guard` (150 ms) renders as
  `Suggested` without the streaming phase.
- **Very slow agent.** Per-conflict suggestion inherits the tool stall thresholds (soft 5 s, hard 45 s,
  [composer-status-strip.md](composer-status-strip.md)); stalled suggestions show "Still thinking ·
  Stop" and never block manual resolution.
- **Rebase ours/theirs swap.** Covered in §2; with no branch name available the labels fall back to
  "Current (being rebased onto)" / "Incoming (your commit)".
- **Add/add and no base.** Base pane shows "No common ancestor" and the editor runs two-way.
- **Whitespace-only and EOL conflicts.** Header chip "whitespace only" with one-click "Accept
  incoming whitespace"; CRLF vs LF shown as a visible `␍` glyph toggle.
- **Long branch names.** Pane headers use [git-chips.md](git-chips.md) truncation (keep prefix and
  last segment); full name in tooltip.
- **RTL.** Chrome mirrors (navigator, actions); code panes stay LTR; pane order **does not mirror**
  (Current stays at inline-start in LTR and inline-end in RTL by logical order, so "current" is always
  the reading-start pane).
- **Tiny sizes (16–20 px).** The merge appears only as a chip: "⚠ 5" amber, tooltip "5 merge
  conflicts"; the status-strip PR pill carries the same number.
- **Dense IDE.** 20 px line height, action bar becomes icon buttons (C, I, B, ✦) with tooltips, rationale
  collapses to a ✦ icon with hover card.
- **Touch.** Actions become a bottom sheet per conflict; swipe left / right on a conflict block =
  Accept current / incoming (with undo toast 5 s); targets ≥ 44 px.
- **Slow network / remote sandbox.** File load shows skeleton panes; first conflict becomes interactive
  as soon as its file arrives (others stay skeleton); writes are optimistic with a per-region "saving"
  dot and rollback on failure.
- **Offline.** Read and manual resolution work on the local copy; Agent, Complete and Abort are
  disabled-with-reason; on reconnect, if the file changed remotely → `Stale`.
- **Concurrent agent run.** If the agent is still running a turn that touches a conflicted file, the
  file shows "Agent is editing" and actions are disabled until the edit settles (file-edit chip
  `Writing` state).
- **Undo after Complete.** Only via checkpoint restore ([checkpoint-timeline.md](checkpoint-timeline.md));
  the component never runs a reset itself.

## 9. Accessibility
- **Landmarks.** The editor is a `region` "Merge conflicts: feature/auth-refresh into main"; each pane
  is a labelled `group` ("Current, main", "Base", "Incoming, feature/auth-refresh", "Result, editable").
- **Conflict accessible name.** "Conflict 2 of 5 in auth.rs, lines 41 to 42, unresolved. Current: 2
  lines. Incoming: 1 line. Agent suggestion available, medium confidence." Changes per state
  ("resolved, took incoming", "agent suggestion, not reviewed").
- **Text alternatives for colour.** Every region carries a text tag (current / incoming / both / agent /
  edited / unresolved) and a gutter glyph; colour is never the only cue; contrast of tags and outlines
  ≥ 3:1 against the pane, text ≥ 4.5:1; amber and violet pass in both themes and the daltonized preset.
- **Announcements (polite):** "Conflict 3 of 5" on navigation; "Resolved, 2 conflicts left" on accept;
  "Agent suggested a resolution for conflict 2" when it finishes; "Agent resolved 5 conflicts, 5 to
  review" once for Resolve all (never per step); "All conflicts resolved". **Assertive:** commit
  failure, abort failure, file changed on disk.
- **Keyboard (proposed; all remappable via `MergeKeymap`).** Alt+↓ / Alt+↑ next / previous unresolved
  conflict; Alt+Shift+↓ next any; Alt+1 accept current; Alt+2 accept incoming; Alt+3 accept both;
  Alt+4 accept agent suggestion; Alt+Backspace reject suggestion; Alt+R reopen; F8 / Shift+F8 next /
  previous marker; ⌘/Ctrl+Alt+R resolve all with agent; ⌘/Ctrl+Alt+M mark file resolved;
  ⌘/Ctrl+Shift+Enter complete merge; Abort has **no** shortcut. Esc leaves the result editor to the
  navigator. Shortcuts avoid ⌘/Ctrl+Shift+A (status-strip approval jump).
- **Focus order.** Header actions → file strip → navigator → focused conflict action bar → result
  editor → footer. Focus stays on the action bar after Accept and moves with auto-advance; the move is
  announced.
- **Screen-reader reading of panes.** Each conflict offers "Read current / base / incoming" buttons that
  read the region's lines without leaving the result.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct MergeSession {
    pub id: MergeId,
    pub kind: MergeKind,                  // Merge | Rebase { step: u16, total: u16 } | CherryPick | Revert | StashPop
    pub current: SideRef,                 // { label: BranchName | CommitRef, sha: ShortSha }
    pub incoming: SideRef,
    pub base: Option<ShortSha>,           // None for add/add or unknown
    pub files: Vec<ConflictFile>,
    pub state: MergeState,                // Detecting | Loading | Ready | AwaitingApproval { request } | AgentResolving { done, total }
                                          // | ReviewingAgent { unreviewed } | AllResolved | Verifying { passed, total }
                                          // | Committing | Completed { sha } | CommitFailed { reason } | ConfirmAbort
                                          // | Aborting | Aborted | AbortFailed { reason } | Stale { paths } | Offline | Failed { reason }
}

pub struct ConflictFile {
    pub path: FilePath,
    pub kind: FileConflictKind,           // Text | Binary { current_bytes, incoming_bytes } | ModifyDelete { deleted_in: Side }
                                          // | RenameRename { current_to, incoming_to } | RenameDelete | AddAdd | Mode | Submodule | Generated { regen: Option<Command> }
    pub conflicts: Vec<Conflict>,
    pub markers_left: u32,                // local scan of the result buffer
    pub marked_resolved: bool,
    pub language: Option<LanguageId>,     // syntax highlighting via diff-viewer
}

pub struct Conflict {
    pub id: ConflictId,
    pub result_range: LineRange,
    pub current: TextRegion,  pub base: Option<TextRegion>,  pub incoming: TextRegion,
    pub state: ConflictState,             // Unresolved | Suggesting | Suggested | AgentDeclined { why } | EditedManually | Resolved
    pub resolution: Option<Resolution>,   // Current | Incoming | Both { order: BothOrder } | Agent { reviewed: bool } | Manual
    pub suggestion: Option<AgentSuggestion>,
}

pub struct AgentSuggestion {
    pub text: String,
    pub rationale: String,                // shown verbatim, ≤ 2 lines collapsed
    pub confidence: Option<Confidence>,   // High | Medium | Low — only if the agent supplies it
    pub based_on: Vec<Side>,              // which inputs it draws from
    pub streaming: bool,
}

pub struct MergeConflictEditor<'a> {
    pub session: &'a MergeSession,
    pub agent: &'a AgentState,            // ToolRunning while suggesting; AwaitingApproval for gated writes
    pub variant: MergeVariant,            // ThreePane | InlineStacked | Card | Pro(..)
    pub focus: Option<ConflictId>,
    pub review_policy: ReviewPolicy,      // RequireOpenEach (default) | TrustAgent
    pub auto_advance: bool,               // true
    pub show_base: BaseDisplay,           // Pane | Toggle | Hidden
    pub keymap: MergeKeymap,
    pub labels: MergeLabels,              // "Current", "Incoming", "Base", "Result", …
    pub on_resolve: Callback<(ConflictId, Resolution)>,
    pub on_request_suggestion: Callback<Option<ConflictId>>,   // None = resolve all
    pub on_mark_file: Callback<FilePath>,
    pub on_complete: Callback<MergeId>,
    pub on_abort: Callback<MergeId>,      // the host performs the git operation
    pub motion: MotionLevel,
}
```
The component never runs git itself: every lifecycle action is a callback; the host (or an adapter) maps
git / provider state onto `MergeSession`.

## 11. Related components and references
**Related:** [checkpoints-git.md](checkpoints-git.md) (family overview) · [diff-viewer.md](diff-viewer.md)
(hunk rendering, highlighting, split / stacked, virtualisation) · [checkpoint-timeline.md](checkpoint-timeline.md)
(checkpoint before Complete, undo after) · [rollback-impact.md](rollback-impact.md) (abort impact) ·
[pr-card.md](pr-card.md) (conflicts row → this) · [git-chips.md](git-chips.md) (`Merging` / `Rebasing`
chip state, pane labels) · [review-card.md](review-card.md) · [file-edit-chip.md](file-edit-chip.md)
(`Conflict` state) · [composer-status-strip.md](composer-status-strip.md) (PR pill conflict count) ·
[terminal-output.md](terminal-output.md) (hook output, lockfile regen) · [text-shimmer.md](text-shimmer.md),
[text-reveal.md](text-reveal.md) · family G approvals (agent write permission).

| Reference | What we take | Licence note |
|---|---|---|
| pitus_ merge-conflict gen-UI [R054], v1 → v2 [R055] (note 15) | card flow check → conflicts → resolve → tests → merge; amber rows resolving; composer pill | Study only |
| Conductor, Verdent (note 03) | "verify, review, resolve"; rebase / sync to main — confirms the gap | Proprietary — reference only |
| Hoplite (note 10) | rebase and "Verify and merge" as steps in a long run | Proprietary — reference only |
| affectUI Conflict Radar (note 11) | name only → our `radar` idea | Paid — study only |
| signoff-ui DiffReview [R107] (note 16) | per-hunk accept model, unified / split | MIT — port with attribution |
| Paradev diff scrollbar markers (note 03) | minimap marks | Proprietary — reference only |
| Running-tests tick rows [R037] | `test-gated` progress | Study only |
| Codex CLI diff render (note 08) | diff line rendering conventions reused via diff-viewer | Apache-2.0 per note 08 (MIT per licensing.md) — port with attribution; re-check |
| Zed multibuffer review (note 03) | per-hunk keep / reject interaction | Mixed licence by crate — study only |
| git conflict-marker format | marker lines `<<<<<<<` `|||||||` `=======` `>>>>>>>` are a file format we parse; no git code used | format only, no code ported |
| "Current / Incoming" wording | common editor vocabulary; ours is written in-house in `MergeLabels` | not from our research notes; no code or assets used |
