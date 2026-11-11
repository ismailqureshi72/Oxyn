**Status:** draft v0.1 (2026-11-11).

# Spec: rollback impact sheet (`rollback-impact`)

Part of family H ([checkpoints-git.md](checkpoints-git.md) — capture layers §2.2, the four actions
§2.4 and family tokens §5 live there). Same depth as the worked example
[composer-send-control.md](composer-send-control.md). Target **3 free + 9 Pro = 12 variants**. Opened
by [checkpoint-timeline.md](checkpoint-timeline.md) (every restore, rewind and "Undo after further
edits"); also usable by hosts for any bulk revert (e.g. "Undo all" in [diff-viewer.md](diff-viewer.md)
when hunks are already applied).

## 1. Purpose
Before the user rolls back, they need to know **exactly what will change, what of theirs they could
lose, and what the agent did that no rollback can reach** — a published package, a database insert, a
pushed branch, an email. The sheet turns that into a short, honest list with a confirm button that names
the effect. Its job is to make "going back" safe enough to do often, without pretending a file restore
undoes the world.

What we saw: Replit's rollback confirmation lists impacted files, Agent memory and tasks, with an opt-in
**Database** checkbox (note 05 §2); Lovable notes that database data is not restored on revert (note
05 §2); Claude Code documents that bash and manual edits are not tracked by checkpoints (note 05 §2);
research note 05 proposes a RestoreDialog with an impact list and "a warning that shell/DB side effects
are not reverted"; affectUI's Agent Ledger lists every side effect with undo and a dependency preview
(note 01 §6.B item 11, Pro — study only). None of the coding agents in note 03 classifies the agent's
commands by whether they can be undone in the restore dialog itself; that is the gap this component fills.

Anatomy (`rollback-impact/sheet`, opened for "Restore files and chat"):
```
┌ Restore files and chat to 3:42 PM ─────────────────────────────────────────── ✕ ┐
│ Before: "now add rate limiting" · 2 turns ago · on main                           │ header
│ [ Files ][ Chat ][ Both ]                                                         │ scope switch
│                                                                                   │
│ ↶ Files restored                                                        7 files  │ section 1
│   auth.rs            back to before (+2 −4 vs now)                                │
│   rate.rs            will be removed (created since)                              │
│   legacy.rs          will come back (deleted since)                               │
│   and 4 more ›                                                                    │
│ ✎ Your edits will be overwritten                                         1 file  │ section 2 (only if any)
│   session.rs   you edited at 3:51 PM                         [ Overwrite ▾ ]      │
│ 💬 Chat: 2 turns rewound — you can bring them back                               │ section 3
│ ⚙ Plan, mode and session approvals go back to how they were                     │ section 4
│ ⛔ Can't be undone by this restore                                    3 actions  │ section 5
│   ⬆  npm publish                 published to a package registry     danger       │
│   ⛁  psql -c "insert …"          wrote to a database                 danger       │
│   ?  ./scripts/sync.sh           unknown script — may have effects   warn         │
│   ↯  dev server (pid 4120)        still running · Stop                 info        │
│ ☐ Also restore database (offered by your host)                                   │ host options
│ ─────────────────────────────────────────────────────────────────────────────    │
│ ☐ I understand 2 actions can't be undone                                          │ ack (danger only)
│                                   [ Cancel ]  [ Restore 7 files and rewind 2 turns ] │ footer
└───────────────────────────────────────────────────────────────────────────────────┘
```
Rules (proposed):
1. **Fixed section order**: files restored → your edits at risk → chat → tool state → can't be undone →
   host options. Empty sections are omitted, never shown as "0".
2. **The confirm button names the effect with counts**; it never says OK / Confirm.
3. **Can't-be-undone items are listed, not summarised away.** Up to 5 shown, then "and N more ›".
4. **Classification is honest about certainty**: an item is `Known` (the adapter or tool declared it),
   `Inferred` (matched a heuristic) or `Unknown` (shell / script we can't read). Inferred items say
   "likely"; unknown items say "may have effects".
5. **Acknowledge only for danger.** The checkbox appears only when at least one `danger` item exists;
   otherwise the warning is enough (no confirm fatigue).

## 2. Effect classification (proposed, `oxyn-core::history::effects`)
Each tool call after the checkpoint is classified once, when it finishes, and stored on the `RunLog`
(so the sheet opens fast). The host can override every rule.

| Class | Examples (heuristic matches are host-overridable) | Severity | Shown as |
|---|---|---|---|
| `FileInWorkspace` (tracked) | Edit / Write / Create / Delete tool calls | — | section 1 (restored) |
| `FileByCommand` (untracked) | shell commands that changed files when `tracks_shell = false` | warn | "Files changed by commands won't be restored" + list of commands |
| `FileOutsideWorkspace` | writes to paths outside the root, `~/.config`, global caches | danger | ⛔ |
| `NetworkRead` | Web fetch / search tools, `curl` without a write method | info | collapsed "4 web reads (no effect)" |
| `NetworkWrite` | MCP / API tools declared as writing, `curl -X POST/PUT/PATCH/DELETE`, webhook calls | danger | ⛔ "sent data to <host>" |
| `Database` | DB tools, `psql` / `mysql` / `sqlite3` with write statements, ORM migrate commands | danger (warn if the host offers a DB restore option) | ⛔ |
| `PackageLocal` | `npm install`, `cargo add`, `pip install` into a workspace venv | warn | "Installed packages may not match the restored lockfile — reinstall after" |
| `PackageGlobal` | `npm i -g`, `brew install`, `cargo install` | danger | ⛔ |
| `PackagePublish` | `npm publish`, `cargo publish` | danger | ⛔ |
| `GitLocal` | `git commit`, `git branch`, `git tag` (not pushed) | info | "2 commits stay in git history" |
| `GitRemote` | `git push`, PR created / merged ([pr-card.md](pr-card.md)) | danger | ⛔ "pushed to origin/feature" |
| `Deploy` | deploy / release tools, `kubectl apply`, `terraform apply` | danger | ⛔ |
| `Message` | email / chat / issue-comment tools | danger | ⛔ "sent a message to …" (recipient shown, content not) |
| `Payment` | payment / purchase tools | danger | ⛔ |
| `ProcessRunning` | dev servers, watchers, background jobs still alive ([background-jobs.md](background-jobs.md)) | info | "Still running" + Stop |
| `Unknown` | any shell command or script not matched above | warn | "? may have effects" |

Severity drives the sheet tone: `Clean` (no warn / danger), `Warn` (≥ 1 warn), `Danger` (≥ 1 danger).
Heuristics are a starting list only; the sheet shows "Based on the commands that ran; not a guarantee"
under section 5 whenever an `Inferred` or `Unknown` item is present.

## 3. State machine
```
                   action chosen in checkpoint-timeline
  Closed ─────────────────────────────────────────────▶ Computing ──150 ms──▶ Computing(skeleton visible)
    ▲                                                       │ impact ready
    │                                                       ▼
    │                                         Ready{severity: Clean | Warn | Danger}
    │          ┌────────────── scope switch (Files / Chat / Both) ─────────────┐
    │          │                                                                 ▼
    │          └──────────────────────────────────────────────────────── Recomputing (rows keep, counts shimmer)
    │                                                       │
    │   workspace changed while open ──▶ Stale ("Files changed since you opened this · Refresh") ──refresh──▶ Computing
    │                                                       │
    │                    Danger & ack unchecked ──▶ ConfirmDisabled{"Tick the box to continue"}
    │                                                       │ confirm (Enter / click; hold in hold-to-confirm)
    │                                                       ▼
    │                                                   Applying{n of m}  (sheet stays; footer becomes progress)
    │                                     ┌─────────────────┼──────────────────────┐
    │                                     ▼                 ▼                      ▼
    │                                   Done            PartialFailure         Failed{reason} (nothing changed)
    │       (closes; timeline shows      │             (list of failed files;   (Retry / Cancel)
    │        Restored + undo toast)      │              Retry failed / Undo all)
    │◀──────── close ────────────────────┘
    │◀── Cancel / Esc / ✕ (any state before Applying) ── Cancelled
  ImpactUnavailable (compute failed or > 5 s): generic warning "Couldn't work out the full impact" + Retry + "Restore anyway"
  Blocked (agent started running while open): confirm disabled with reason "Agent is running · Stop and restore"
  Offline (remote store): confirm disabled, "Checkpoint store offline · Retry"
```

## 4. States table
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Computing` | header + scope switch at once; body skeleton rows after 150 ms | — | Cancel | Esc |
| `Ready{Clean}` | sections 1–4, neutral tone | confirm ("Restore 7 files") | Cancel, scope switch | Enter (on button), Esc, ←/→ scope, Tab through sections |
| `Ready{Warn}` | as Clean + warn rows with `color.impact.external` glyphs | confirm | Cancel | — |
| `Ready{Danger}` | danger rows, ack checkbox; confirm button styled destructive | confirm (after ack) | Cancel | Space toggles ack |
| `ConfirmDisabled` | confirm dimmed, reason as text under it | — | — | button stays focusable with `aria-disabled` and the reason as description |
| `Recomputing` | rows stay; counts shimmer (`text-shimmer/calm`) until new numbers roll in | — | Cancel | — |
| `Stale` | top notice "Files changed since you opened this" | Refresh | Cancel | R refresh |
| `Applying` | footer swaps to progress "Restoring 4 of 7 files" with a 2 px bar; body read-only | — | — (no cancel after the first write) | — |
| `Done` | sheet closes; result handed to the timeline toast | — | — | focus per checkpoint-timeline.md §8 |
| `PartialFailure` | failed files listed with reasons ("locked", "permission denied") | Retry failed | Undo all | R |
| `Failed` | danger notice "Couldn't restore · nothing changed" + reason | Retry | Cancel | R, Esc |
| `ImpactUnavailable` | generic warning list ("Files will be restored; commands that ran may have effects we couldn't check") | Retry | Restore anyway | R |
| `Blocked` | confirm disabled, "Agent is running" | Stop and restore | Cancel | — |
| `Offline` | confirm disabled, "Checkpoint store offline" | Retry | Cancel | R |
| Row hover / focus | file row: full path + `+N −N` vs now; effect row: full command, tool, time, classification certainty | open diff (file) / jump to tool call (effect) | — | Enter |
| "Your edits" row | per-file menu: Overwrite (default) · Keep mine · View both | change choice | — | Space opens menu |

## 5. Micro-interactions
- **Enter.** Sheet rises with `dur.sheet.enter` (220 ms `spring.snap`, y +12 → 0); on mobile it is a
  bottom sheet with the same spring. Header and scope switch render immediately; sections appear in
  order with `stagger.row` (40 ms, cap 6) as soon as the impact is ready — never before.
- **Counts.** Section counts and the confirm label roll (`dur.digit.roll`); the confirm button keeps the
  width of its longest possible label in this sheet (one-cell rule, Arc idea, note 12) so it never jumps.
- **Scope switch.** Segmented control thumb slides 160 ms `spring.snap`; sections that drop out collapse
  (`dur.collapse`), new ones expand (`dur.expand`); the button label rolls to the new effect.
- **Severity tone.** The sheet's top edge carries a 2 px tone line (neutral / warn / danger) that
  crossfades 200 ms when severity changes; danger never pulses or loops.
- **Ack.** Ticking the box draws the check stroke (`dur.check.draw`, 240 ms) and enables the confirm
  button with a 120 ms fade from 40 % → 100 % opacity.
- **Hold to confirm** (Pro `hold-to-confirm`): the button fills start → end over 800 ms while held;
  releasing early drains it back in 200 ms; completion snaps with `spring.snap`.
- **Applying.** The footer crossfades into progress (160 ms); file rows tick to ✓ one by one in list
  order (`dur.row.morph`); failed rows morph to ✕ and stay.
- **Stale.** A notice slides down under the header (160 ms) — counts are **not** silently changed under
  the pointer; the user presses Refresh.
- **Exit.** `dur.sheet.exit` 160 ms `ease.in`; on Done the exit starts after the last row's ✓ (≤ 200 ms
  beat) so the user sees completion.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Sheet enter | 220 ms, y +12 → 0 | 0 | `spring.snap` | — | 60 Hz |
| Skeleton | — | 150 ms | — | — | 30 Hz shimmer |
| Sections appear | 200 ms each (`dur.row.enter`) | on impact ready | `spring.snap` / `ease.out` | 40 ms, cap 6 | 60 Hz |
| Count / label roll | 280 ms | 0 | `ease.out` | 30 ms digits | 30 Hz |
| Scope thumb | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Section collapse / expand | 280 ms / 220 ms | 0 | `ease.in` / `spring.snap` | — | 60 Hz |
| Severity tone crossfade | 200 ms | 0 | `ease.in-out` | — | 60 Hz |
| Ack check draw + enable | 240 ms + 120 ms | 0 | `ease.out` | — | 60 Hz |
| Hold-to-confirm fill / drain | 800 ms / 200 ms | 0 | linear / `ease.out` | — | 60 Hz |
| Footer → progress | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Row ✓ / ✕ during apply | 160 ms | as each file finishes | `spring.snap` | — | 60 Hz |
| Stale notice | 160 ms | 0 | `ease.out` | — | 60 Hz |
| Sheet exit | 160 ms | ≤ 200 ms after last ✓ | `ease.in` | — | 60 Hz |
| **Reduced motion** | sheet and sections appear with an 80 ms opacity fade, no offsets, all sections at once; counts and labels swap without rolling; scope thumb jumps; tone line swaps; hold-to-confirm becomes a two-step press ("Press again to restore", 3 s window) instead of a fill; row ✓ / ✕ appear without morph | — | linear | none | event-driven |

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `rollback-impact/sheet` | Modal dialog (desktop) / bottom sheet (mobile) with the six sections (anatomy above) | desktop, chat, mobile, ide |
| `rollback-impact/inline` | Expands in place under the checkpoint marker inside the transcript; same sections, compact rows | chat, ide |
| `rollback-impact/terminal` | TUI block: `↶ 7 files`, `💬 2 turns`, `⛔ 3 actions can't be undone` with `!`/`?` prefixes, then `y restore · n cancel` | terminal |

### Pro (9)
| Name | One-line idea |
|---|---|
| `rollback-impact/severity-meter` | A three-step meter (clean / warn / danger) at the top that fills to the sheet's severity |
| `rollback-impact/effects-timeline` | Can't-be-undone items plotted on a mini timeline from the checkpoint to now, aligned with the turns that caused them |
| `rollback-impact/outcome-tree` | Files shown as a folder tree with per-file outcome glyphs (restored / removed / brought back / overwrite yours) |
| `rollback-impact/two-column` | "Goes back" vs "Stays as it is" columns so what is not reverted is as visible as what is |
| `rollback-impact/hold-to-confirm` | Danger sheets confirm with an 800 ms press-and-hold instead of a checkbox |
| `rollback-impact/file-diffs` | Each file row expands into a mini diff (checkpoint vs now) using the diff viewer's `unified` layout |
| `rollback-impact/compensations` | Danger rows carry host-supplied follow-up actions ("Unpublish version", "Revert PR") the user can queue after the restore |
| `rollback-impact/receipt` | After apply, the sheet turns into a receipt card in the transcript listing what was restored and what was left as is |
| `rollback-impact/blast-radius` | Concentric rings — workspace, machine, outside world — with each effect placed in its ring (side-effect ledger idea, note 01) |

## 8. Edge cases
- **Nothing to restore** (chat-only rewind, no tool-state change, no effects): the sheet is skipped
  (checkpoint-timeline.md rule 2) and the undo toast is the safety net.
- **Hundreds of files.** Section 1 shows 5 rows + "and 195 more ›" which expands into a virtualised list;
  counts stay exact. Lockfiles and generated files sort last (file-edit-chip.md `low_signal`).
- **Hundreds of effects** (an agent that ran 400 commands): group by class with counts ("212 web reads",
  "3 database writes"); danger classes always expanded.
- **Secrets in commands.** Effect rows show the command head (first token + 1 arg, tools.md §2.4) and
  redact arguments the adapter marks secret; the full command is in the tooltip only if not redacted.
- **User chose Keep mine** for a file: section 1's count drops by one and the row moves under "Kept as
  is"; if every file is kept, the scope collapses to Chat and the button says "Rewind 2 turns".
- **Effects inside the rewound turns but outside files** (e.g. memory entries written): listed under
  section 4 if the host captures them, otherwise under "Can't be undone" as `Unknown`.
- **Impact computation slow** (remote store, many files): skeleton after 150 ms; after 5 s
  `ImpactUnavailable` offers Restore anyway with the generic warning; the computed sheet replaces it if
  it arrives before the user confirms (with the Stale-style notice, not a silent swap).
- **Agent starts running while the sheet is open** (e.g. a queued message fires): `Blocked`.
- **Long commands / paths.** Middle-truncate paths, end-truncate commands at 48 ch; full text in tooltip.
- **RTL.** Sheet mirrors; commands, paths and counts stay LTR in bidi isolates.
- **Tiny / dense.** In IDE panels under 320 px the sheet becomes a full-panel overlay with 24 px rows;
  icons stay with words (never icon-only for danger rows).
- **Touch.** Bottom sheet with a 44 px confirm; hold-to-confirm uses a platform haptic at completion;
  swipe-down dismiss only before `Applying`.
- **Offline.** Confirm disabled with reason; the computed impact stays visible.
- **Very fast restore** (< 150 ms): progress footer is skipped; the sheet closes straight to Done.

## 9. Accessibility
- **Role:** `dialog` (`alertdialog` when severity is Danger) labelled by the header ("Restore files and
  chat to 3:42 PM") and described by a one-sentence summary generated from the sections: "7 files will be
  restored, 1 of your edits will be overwritten, 2 turns will be rewound, 3 actions can't be undone."
- **Sections** are `group`s with headings; lists are real lists; each effect row's accessible name
  includes its class and certainty ("npm publish, published to a package registry, can't be undone,
  known").
- **Initial focus:** the Cancel button for Danger, the confirm button otherwise (proposed — destructive
  sheets never default-focus the destructive action).
- **Keyboard:** Tab order header ✕ → scope switch → sections (rows focusable) → host options → ack →
  Cancel → confirm; ←/→ on the scope switch; Space on ack; Enter activates the focused button only (Enter
  elsewhere does not confirm); Esc cancels before `Applying`. Hold-to-confirm also accepts holding Space /
  Enter on the button for 800 ms; reduced motion turns it into press-twice.
- **Live region:** polite "Impact ready: 7 files, 3 actions can't be undone" when computing finishes
  after the skeleton showed; polite progress every 25 % during Applying (not per file); assertive on
  failure.
- **Contrast:** warn and danger text ≥ 4.5:1; glyphs (⛔ ? ↯) always paired with words; the 2 px tone
  line is decoration (severity is also in the dialog description).
- **Reduced motion:** §6 last row.

## 10. Props sketch
```rust
pub struct Impact {
    pub checkpoint: CheckpointId,
    pub action: RestoreAction,                  // checkpoints-git.md §2.4
    pub files: Vec<FileOutcome>,                // { path, outcome: Restore | Remove | BringBack, stats_vs_now: DiffStats }
    pub at_risk: Vec<UserEdit>,                 // { path, edited_at, by: User | OtherAgent(AgentId), choice: Overwrite | KeepMine }
    pub chat: Option<ChatImpact>,               // { turns: u32, messages: u32, branch: BranchId }
    pub tool_state: Vec<ToolStateItem>,         // Plan | Mode | Permissions | Model | Terminals | Memory
    pub effects: Vec<Effect>,                   // §2
    pub host_options: Vec<HostOption>,         // e.g. "Also restore database" { id, label, default: false }
    pub severity: Severity,                     // Clean | Warn | Danger (derived)
    pub computed_at: Instant,                   // Stale detection
}

pub struct Effect {
    pub call: CallId,                           // jump target in the transcript (tools.md §9)
    pub class: EffectClass,                     // §2 table
    pub certainty: Certainty,                   // Known | Inferred | Unknown
    pub summary: SharedString,                  // "published to a package registry"
    pub target: Option<SharedString>,           // host / db / remote, never message content
    pub still_running: Option<ProcessRef>,      // ProcessRunning → Stop action
}

pub struct RollbackImpact<'a> {
    pub agent: &'a AgentState,                  // Blocked when not idle
    pub state: ImpactState,                     // Computing | Ready | Recomputing | Stale | Applying { done, total }
                                                // | PartialFailure { failed } | Failed { reason } | ImpactUnavailable | Offline
    pub impact: Option<&'a Impact>,
    pub variant: ImpactVariant,                 // Sheet | Inline | Terminal | Pro(..)
    pub scopes: SmallVec<[RestoreAction; 3]>,   // which scope tabs to show
    pub classifier: &'a dyn EffectClassifier,   // default heuristics + host overrides
    pub ack: AckPolicy,                         // DangerOnly (default) | Always | Never | HoldToConfirm { ms: 800 }
    pub show_limit: usize,                      // 5 rows per section before "and N more"
    pub compute_timeout: Duration,              // 5 s → ImpactUnavailable
    pub labels: ImpactLabels,
    pub on_scope: Callback<RestoreAction>,
    pub on_confirm: Callback<ConfirmedRestore>, // carries per-file KeepMine choices + host options
    pub on_cancel: Callback<()>,
    pub on_jump_to_call: Option<Callback<CallId>>,
    pub motion: MotionLevel,
}

pub trait EffectClassifier {
    fn classify(&self, call: &ToolCall) -> (EffectClass, Certainty);
}
```

## 11. Related components and references
**Related:** [checkpoint-timeline.md](checkpoint-timeline.md) (opens the sheet; undo toast after) ·
[checkpoints-git.md](checkpoints-git.md) (capture layers, actions) · [diff-viewer.md](diff-viewer.md)
(file rows open diffs; file-diffs variant) · [tool-call.md](tool-call.md) (effect rows jump to their call)
· [terminal-output.md](terminal-output.md) (commands) · [background-jobs.md](background-jobs.md) (still
running processes) · family G approvals (the same risk classes can drive pre-risk checkpoints and approval
risk tiers) · [pr-card.md](pr-card.md) and [git-chips.md](git-chips.md) (pushed / merged effects).

| Reference | What we take | Licence note |
|---|---|---|
| Replit rollback dialog (note 05 §2) | impacted files, memory and tasks listed; opt-in database checkbox → host options | Proprietary — reference only |
| Lovable revert (note 05 §2) | database not restored → stated plainly | Proprietary — reference only |
| Claude Code checkpointing (note 05 §2) | shell and manual edits not tracked → `FileByCommand` warning | Proprietary — reference only |
| Note 05 §2 RestoreDialog proposal | scope selector, impact list, side-effect warning, 4 + 1 scope presets | Our own research |
| affectUI Agent Ledger (note 01 §6.B item 11; note 11) | every side effect with undo and dependency preview → blast-radius / compensations ideas | Paid — study only |
| signoff-ui risk levels [R107] | risk level wording shared with approvals | MIT — may port with attribution |
| Arc one-cell label (note 12) | confirm button keeps its width | Paid — study only |
