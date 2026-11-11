**Status:** draft v0.1 (2026-11-11).

# Spec: pull-request card with CI (`pr-card`)

Part of family H, "Checkpoints, diff & git" (overview in [checkpoints-git.md](checkpoints-git.md)).
Same depth as [composer-send-control.md](composer-send-control.md). Target **3 free + 13 Pro = 16
variants**. The card is the **expanded form of the status-strip PR pill** ("#128 · 3 conflicts · +482
−37", [composer-status-strip.md](composer-status-strip.md)): both read one `PrStatus` value, so the
numbers, tints and states always agree. Conflicts open [merge-conflict.md](merge-conflict.md); agent
review findings are [review-card.md](review-card.md); branch display is [git-chips.md](git-chips.md).

## 1. Purpose
After the agent pushes work, the user wants to know **is the PR up, is CI green, has anyone reviewed it,
can it merge — and if not, what is in the way and can the agent fix it**. Table-stakes across coding
agents (research note 03 §1F): Claude desktop's Changes card with CI checks and ⋮ **Auto-fix** /
**Merge when ready** (auto-archive on merge); Conductor (checks, PR, merge, archive, **Stacks**);
Paradev (PR#, "Waiting for review / All checks passed"); Otty ("Merged 2h ago … ✓ 4 checks passed");
Codex app (review PRs in the sidebar); Jules (Open PR, stacked diff). pitus_ gen-UI #2 is the most
complete flow we saw: title + `#128`, commit rail with branch chips, "Checking it merges cleanly" →
"⚠ 3 conflicts with main" → Resolve → "Running 36 of 42 tests" → checks row (Tests 42 passed / Lint
passed / Types passed / Preview ready) → **Merge** → Merged [R054]; pitus also lists a "#6 CI checks"
gen-UI [R058]. A tick-row test runner shows each test as a tick that fills, failed rows stay lit
[R037]. Hoplite tells a PR story: "opened #212, CI failed overnight & fixed, 6 review comments,
rebased, merged" and sells "PR Auto-fix" (note 10).

Where it appears: transcript (after a "Create PR" tool call), thread side rail, the PR pill popover,
review queue rows (compact), fan-out lane footers, dashboards.

## 2. Anatomy
```
┌ ⑂ feature/auth-refresh → main ─────────────────────────────────────────────── Open ● ┐  header: git-chips + state badge
│ Refresh sessions in the auth middleware                                     #128 ↗  │  title (2 lines max) + number link
│ +482 −37 · 12 files · 6 commits · updated 4m ago                                    │  meta (file-edit counts style)
├─────────────────────────────────────────────────────────────────────────────────────┤
│ Checks  ▮▮▮▯ 3 of 4 · 1 running                                            ▾        │  checks summary + segmented meter
│   ✓ Tests        42 passed                                       1m 12s             │  check rows: glyph · name · summary · duration
│   ✓ Lint                                                           18s               │
│   ◌ Types        running                                           0:41 ⏱           │
│   ⊘ Preview      skipped                                            —                │
│ Review  ◐ 1 of 2 approvals · changes requested by Ines                               │
│ Merge   ⚠ 3 conflicts with main                         Resolve conflicts ›          │  blocker row (one at a time, worst first)
├─────────────────────────────────────────────────────────────────────────────────────┤
│ [ Merge ▾ ] disabled · "3 conflicts"         ✦ Fix with agent     ⋯                  │  footer: merge button + agent action
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## 3. State machine
```
  PR lifecycle:   Creating (push + open) ─▶ Draft ⇄ Open ─▶ Merged
                       │ error                 │  ▲    └─▶ Closed ─reopen─▶ Open
                       ▼                       │  └ ready for review
                  CreateFailed{reason}         └─ marked draft
  Merge layer (only when Open):
     Unknown/Checking ("Checking it merges cleanly") ─▶ Clean | Conflicts{n} | Behind{n} | Blocked{reason}
     Clean + checks ok + review ok ─▶ Mergeable
     Mergeable ─ Merge ─▶ ConfirmMerge? ─▶ Merging ─▶ Merged
                                          └─ error ─▶ MergeFailed{reason} ─▶ Mergeable
     any ─ "Merge when ready" ─▶ AutoMergeArmed ─(all gates pass)─▶ Merging    ─disarm─▶ previous
     Conflicts ─ Resolve ─▶ Resolving{done/total} (merge-conflict) ─▶ Checking
  Checks layer (per check):  Queued ─▶ Running{elapsed} ─▶ Passed | Failed | Skipped | Cancelled | Neutral | TimedOut | ActionRequired
     Failed ─ ✦ Fix with agent ─▶ AutoFixing (agent run) ─push─▶ checks re-queue
  Review layer: NoReviewers | ReviewRequired{n} | Commented | ChangesRequested{by} | Approved{k of n} | Dismissed
  Sync layer:   Live | Refreshing | Stale{since} | Offline | NotFound | NoAccess
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (focused) |
|---|---|---|---|---|
| `Creating` | skeleton card with title from the agent's draft, "Opening pull request…" shimmer | — | Cancel | Esc cancel |
| `CreateFailed` | danger row "Couldn't open PR: branch not pushed" | Retry | Copy error | R |
| `Draft` | grey "Draft" badge (dashed outline); checks may run; merge button replaced by "Ready for review" | Ready for review | Close PR | Enter |
| `Open` | green-neutral "Open" badge with a dot | depends on merge layer | open on host ↗ | O open ↗ |
| `Checking` | merge row "Checking it merges cleanly…" with row loader [R054] | — | — | — |
| `Conflicts{n}` | amber row "⚠ 3 conflicts with main" | Resolve conflicts ([merge-conflict.md](merge-conflict.md)) | ✦ Fix with agent | Enter |
| `Behind{n}` | "4 commits behind main" | Update branch (app's method) | — | U |
| `Blocked{reason}` | the first unmet gate: "Review required", "Checks failing", "Branch protected" | jump to that section | — | Enter |
| `Mergeable` | merge row "Ready to merge" success-neutral; merge button enabled | Merge (method ▾ from the host: e.g. merge / squash / rebase) | Merge when ready (if host supports) | ⌘/Ctrl+Enter merge (proposed) |
| `ConfirmMerge` | inline confirm "Squash and merge into main?" (only if `confirm_merge = true`, default true for agent-initiated merges) | Confirm | Cancel | Enter / Esc |
| `AutoMergeArmed` | button reads "Will merge when ready" with a slow ring; list of remaining gates | Disarm | — | Enter disarm |
| `Merging` | button label "Merging…" in the same cell | — | — | — |
| `Merged` | purple-neutral "Merged" badge, "Merged into main · a1b2c3d · just now"; checks collapse to one line "✓ 4 checks passed" (Otty style, note 03) | View commit | Delete branch (host) | — |
| `MergeFailed` | danger row with host reason | Retry | Open on host ↗ | R |
| `Closed` | grey "Closed" badge, card dims to 70 % | Reopen (host) | — | — |
| Checks summary | segmented meter, one segment per check, filled by result; text "3 of 4 · 1 running" | expand / collapse | — | ←/→ collapse / expand |
| Check `Queued` | hollow ○, "queued" | — | — | — |
| Check `Running` | row loader, live elapsed `0:41` ([elapsed-timer.md](elapsed-timer.md)) | open logs ↗ | — | Enter logs |
| Check `Passed` | ✓ + summary from the provider ("42 passed") + final duration `1m 12s` | — | logs | — |
| Check `Failed` | ✕ danger, summary ("2 failed: auth::refresh_expired…"), duration; row stays expanded | ✦ Fix with agent | logs, Re-run (host) | F fix, L logs |
| Check `Skipped` / `Neutral` / `Cancelled` | ⊘ / – / ■ dim, duration "—" | — | logs | — |
| Check `TimedOut` / `ActionRequired` | ⏱ warning / ! amber with host text | logs | Re-run | — |
| `AutoFixing` | failed row shows "Agent fixing · 0:32" with loader; on push the check re-queues | Stop | View agent run | Esc stop |
| Review `ReviewRequired{n}` | "Review required · 0 of 2" ◯ | Request review (host) | — | — |
| Review `ChangesRequested` | ✕ amber "Changes requested by Ines" + comment count | View comments / open [review-card.md](review-card.md) | ✦ Address with agent | Enter |
| Review `Approved{k of n}` | ◐ / ● with avatars, "2 of 2 approvals" | — | — | — |
| `Stale{since}` | "updated 12m ago" warning-dim; refresh icon | Refresh | — | R |
| `Offline` / `NoAccess` / `NotFound` | banner with reason; last-known data dimmed or hidden | Retry / Sign in (host) | — | — |
| Disabled-with-reason (merge) | button disabled, reason inline beside it ("3 conflicts", "2 checks failing", "Draft") — never only in a tooltip | — | — | focusable; reason read |

**Gate priority** (one blocker row, proposed): Conflicts > Checks failing > Changes requested > Review
required > Behind > Checks running > Draft.

## 5. Micro-interactions
- **Checks meter.** Segments fill one by one as checks finish (`segment fill 120 ms + 60 ms stagger`,
  the status-strip pill's numbers); a running segment breathes (opacity 0.6 ↔ 1, `loop.calm`); the
  meter is the same widget as the pill's `▮▮▮` so expanding the pill into the card looks continuous.
- **Check finish.** Loader → ✓ / ✕ (`dur.row.morph` + `dur.check.draw`); live elapsed freezes into the
  final duration with a 200 ms tense crossfade ("0:41" → "1m 12s" format switch); failures nudge once.
- **Order.** Rows keep host order; a failed check never jumps under the pointer — the summary line
  reads "1 failing" and a "Show failing first" toggle exists (Amp no-reshuffle rule, note 08 via
  [file-edit-chip.md](file-edit-chip.md)).
- **Conflicts.** Amber row pops in (`spring.snap`); while [merge-conflict.md](merge-conflict.md) runs,
  it shows "Resolving · 2 of 5" and its count rolls down with the pill's; done → row crossfades to
  "Checking it merges cleanly…" then to the next gate (pitus flow [R054]).
- **Merge button.** One fixed-width cell holds every label (Merge · Merging… · Will merge when ready ·
  Merged ✓), crossfading 120 ms so the footer never reflows (Arc one-cell rule, note 12 via
  [file-edit-chip.md](file-edit-chip.md)). Disabled → enabled: 200 ms tint ease + a single 1 px
  outline flash (no bounce). Auto-merge armed: a thin ring circles the button glyph every 2.4 s.
- **Merged.** Badge morphs Open → Merged (colour 300 ms OKLab, glyph ⑂ → merge glyph 160 ms
  `spring.snap`); checks collapse into one line (`dur.collapse`); Pro `merged-arrival` adds one
  arrival ring. In the status strip the pill turns merged and hides after 10 s
  ([composer-status-strip.md](composer-status-strip.md) §2).
- **Review avatars.** New approval: avatar slides into the stack (160 ms) and the `k of n` rolls.
- **Stale.** "updated 4m ago" ticks once a minute; after `stale_after` (5 min, proposed) it tints
  warning and a refresh icon fades in.
- **Hover / press.** Rows tint 80 ms; links underline; card itself is not one big button (it has
  several actions), only the title / number link opens the host.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Card enter (transcript) | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | sections 40 ms cap 6 | 60 Hz |
| Checks segment fill | 120 ms | 0 | `ease.out` | 60 ms | 60 Hz |
| Running segment breathe | 1600 ms loop | 0 | sine | — | 30 Hz |
| Check loader → glyph | 160 ms + 240 ms stroke | ≤ 200 ms beat | `spring.snap` / `ease.out` | — | 60 Hz |
| Elapsed → final duration | 200 ms crossfade | on finish | `ease.out` | — | 60 Hz |
| Elapsed repaint | 1 Hz | — | — | — | 1 Hz |
| Blocker row swap | 200 ms crossfade | 0 | `ease.in-out` | — | 60 Hz |
| Conflict count roll | 300 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz |
| Merge label swap | 120 ms | 0 | `ease.out` | — | 60 Hz |
| Merge enable tint + flash | 200 ms + 240 ms | 0 | `ease.out` | — | 60 Hz |
| Auto-merge ring | 2400 ms loop | 0 | `ease.in-out` | — | 30 Hz |
| Open → Merged morph | 300 ms colour, 160 ms glyph | 0 | linear OKLab / `spring.snap` | — | 60 Hz |
| Checks collapse on merge | 280 ms | 400 ms after Merged | `ease.in` | 20 ms per row cap 6 | 60 Hz |
| Avatar enter + count roll | 160 ms + 280 ms | 0 | `spring.snap` / `ease.out` | — | 60 Hz |
| Failure nudge | 300 ms (2 × 3 px) | after morph | linear | — | 60 Hz |
| "Updated n ago" refresh | every 60 s | — | — | — | — |
| **Reduced motion** | no breathing (running segment is striped static), no ring (text "Will merge when ready" carries it), glyphs and counts swap with 80 ms crossfade, no nudge, sections appear without height spring, merged collapse is instant | — | linear | none | 1 Hz for elapsed only |

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `pr-card/card` | Full card per §2: header, title, meta, checks list, review, blocker row, merge footer | chat, desktop, ide |
| `pr-card/compact` | Two lines: `#128 Refresh sessions…` + state badge, then `▮▮▮▯ checks · ◐ review · ⚠ 3 conflicts`; click expands to `card` | sidebar rows, lane footers, mobile |
| `pr-card/checks-list` | Checks-first layout for CI-heavy views: summary meter, all checks with durations and logs, the merge button pinned at the bottom | ide, desktop |

### Pro (13)
| Name | One-line idea |
|---|---|
| `pr-card/commit-rail` | Left rail draws commits on the branch line joining main, with branch chips at each end (pitus [R054]) |
| `pr-card/flow` | The gen-UI sequence as one morphing card: checking → conflicts → resolving → tests → merge → merged [R054] |
| `pr-card/tick-tests` | Test check expands into a tick row, one tick per test filling its colour, failures stay lit, tests/sec counter [R037] |
| `pr-card/metro` | Checks as a metro line of stations (build → test → lint → preview), a dot travels as stages pass (triage-line idea [R038]) |
| `pr-card/auto-fix-loop` | Failed check shows the agent's fix attempts as a mini timeline "attempt 2 · pushed · re-running" (Auto-fix idea, notes 03, 10) |
| `pr-card/merge-when-ready` | Gate checklist with each unmet gate ticking off live and a countdown-free "merges when all ticks are green" footer |
| `pr-card/stack` | Stacked PRs as overlapping cards ordered base → top, each with its own state; merge order shown (Conductor Stacks idea, note 03) |
| `pr-card/story` | Long-running PR history as a compact timeline: opened, CI failed & fixed, 6 review comments, rebased, merged (Hoplite story, note 10) |
| `pr-card/review-avatars` | Reviewer avatars with state rings (approved / changes / pending), agent reviewers marked ✦ |
| `pr-card/preview-thumb` | Preview-deploy check shows a live thumbnail of the preview URL from the host |
| `pr-card/pill-expand` | The status-strip pill grows into the card in place (shared element morph, 260 ms) |
| `pr-card/merged-arrival` | One arrival ring and a soft purple wash on merge; holds 1.2 s |
| `pr-card/tui` | Terminal box: `#128 open  checks ✓✓◌⊘  review 1/2  ⚠ 3 conflicts`, keys `m` merge, `f` fix, `l` logs |

## 8. Edge cases
- **Many checks** (40+ matrix jobs): grouped by workflow / prefix with per-group meter; list virtualised;
  summary "38 of 42 · 2 failing · 2 running".
- **Re-runs.** A re-run check resets to `Queued` but keeps "previous: failed in 1m 02s" as a dim
  sub-line so the history isn't lost.
- **Flaky checks.** If the host marks a check as retried, show "passed on retry 2" with a warning dot.
- **Checks that never report durations.** Show "—"; never estimate.
- **Very long titles.** Two lines then ellipsis; full title in tooltip; number never truncates.
- **Merge methods.** Only methods the host reports as allowed appear in the ▾ menu; none reported →
  plain "Merge".
- **Permissions.** User can't merge (no rights / protected branch): button hidden, blocker row "You
  can't merge into main" — no disabled button with an unexplained reason.
- **Agent-initiated merge.** Merging is a binding, outbound action: when the agent proposes it, the card
  shows a family G approval ("Agent wants to squash and merge #128") instead of merging itself.
- **Merge conflicts appear after approval.** Gate order re-evaluates; approvals stay but the row shows
  conflicts first.
- **Closed and merged elsewhere.** Host truth wins; transitions animate only if the card is visible,
  otherwise it renders the final state on next show.
- **Very fast CI** (all checks < 1 s): checks inside `delay.row_guard` render final; segments fill with
  the stagger only.
- **Very slow CI** (hours): elapsed switches to `1h 12m` format; stall rules don't apply to CI (no
  colour ramp) — only `TimedOut` from the host.
- **RTL.** Card mirrors; branch names, shas, numbers and durations stay LTR in bidi isolates; the
  `source → target` arrow mirrors with the layout.
- **Tiny sizes (16–20 px).** Becomes the status-strip pill or just `#128 ●` coloured by worst state.
- **Dense IDE.** `checks-list` at 22 px rows, durations right-aligned tabular.
- **Touch.** 44 px rows; logs and re-run in a ⋯ sheet; swipe never merges.
- **Offline / host rate-limited.** Last-known state dimmed with "as of 3m"; merge disabled-with-reason
  "Offline".

## 9. Accessibility
- **Card name:** "Pull request 128, Refresh sessions in the auth middleware, open, 3 of 4 checks passed,
  1 running, 1 of 2 approvals, 3 merge conflicts".
- **Structure.** `article` with heading (title); sections are labelled groups (Checks, Review, Merge);
  checks list is a `list` with each row "Tests, passed, 42 passed, 1 minute 12 seconds".
- **Announcements (polite):** a check fails ("Types check failed"), all checks passed, review state
  changes, "Ready to merge", "Merged into main". Running elapsed timers are never announced.
  **Assertive:** merge failed.
- **Keyboard:** Tab through header link → checks toggle → check rows (Enter logs, F fix) → review →
  blocker action → merge button → agent action → ⋯. ⌘/Ctrl+Enter merges only when the merge button is
  focused-within the card and enabled (proposed).
- **Disabled reasons** are visible text beside the button and part of its accessible description.
- **Contrast.** Badges ≥ 4.5:1 text; segment meter ≥ 3:1 per segment against the track; glyphs
  (✓ ✕ ◌ ⊘) always accompany colour.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct PrStatus {                      // shared with composer-status-strip's PR pill
    pub number: u64,
    pub title: String,
    pub url: Option<Url>,                  // host link; the component never builds one
    pub source: BranchName,  pub target: BranchName,
    pub state: PrState,                    // Creating | CreateFailed { reason } | Draft | Open | Merged { sha, at } | Closed
    pub additions: u32,  pub deletions: u32,  pub files: u32,  pub commits: u32,
    pub merge: MergeGate,                  // Unknown | Checking | Clean | Conflicts { n } | Resolving { done, total }
                                           // | Behind { n } | Blocked { reason } | Mergeable | AutoMergeArmed | Merging | MergeFailed { reason }
    pub methods: Vec<MergeMethod>,         // from the host, may be empty
    pub checks: Vec<Check>,
    pub review: ReviewSummary,             // { required: u8, approvals: Vec<Reviewer>, changes_requested: Vec<Reviewer>, comments: u32 }
    pub updated_at: Option<Instant>,
    pub sync: SyncState,                   // Live | Refreshing | Stale | Offline | NotFound | NoAccess
}

pub struct Check {
    pub name: String,
    pub status: CheckStatus,               // Queued | Running { started } | Passed | Failed | Skipped | Cancelled | Neutral | TimedOut | ActionRequired
    pub summary: Option<String>,           // host text, e.g. "42 passed"
    pub duration: Option<Duration>,
    pub logs: Option<Url>,
    pub previous: Option<(CheckStatus, Duration)>,
    pub fixing: Option<RunRef>,            // agent auto-fix run
}

pub struct PrCard<'a> {
    pub pr: &'a PrStatus,
    pub agent: &'a AgentState,
    pub variant: PrCardVariant,            // Card | Compact | ChecksList | Pro(..)
    pub confirm_merge: bool,               // true (proposed)
    pub stale_after: Duration,             // 5 min (proposed)
    pub on_merge: Option<Callback<MergeMethod>>,
    pub on_auto_merge: Option<Callback<bool>>,
    pub on_resolve_conflicts: Option<Callback<()>>,   // → merge-conflict
    pub on_fix_with_agent: Option<Callback<FixTarget>>, // Check(name) | Conflicts | Review
    pub on_ready_for_review: Option<Callback<()>>,
    pub on_refresh: Option<Callback<()>>,
    pub labels: PrCardLabels,
    pub motion: MotionLevel,
}
```
Adapters map provider PR / check / review payloads onto `PrStatus`; state names above are ours.

## 11. Related components and references
**Related:** [composer-status-strip.md](composer-status-strip.md) (PR pill, same `PrStatus`) ·
[merge-conflict.md](merge-conflict.md) · [review-card.md](review-card.md) · [git-chips.md](git-chips.md) ·
[checkpoints-git.md](checkpoints-git.md) · [diff-viewer.md](diff-viewer.md) (Files changed) ·
[terminal-output.md](terminal-output.md) (check logs inline) · [elapsed-timer.md](elapsed-timer.md) ·
[file-edit-chip.md](file-edit-chip.md) (`+482 −37` format) · family G approvals (agent merge).

| Reference | What we take | Licence note |
|---|---|---|
| pitus_ gen-UI #2 [R054], CI checks gen-UI listed [R058] | full PR flow, checks row, pill | Study only |
| Running-tests tick rows [R037] | `tick-tests` | Study only |
| HacktronAI metro pipeline [R038] | `metro` | Study only |
| Claude desktop, Conductor, Paradev, Otty, Codex app, Jules (note 03) | Auto-fix, Merge when ready, Stacks, "✓ 4 checks passed", waiting-for-review wording | Proprietary (Codex CLI Apache-2.0) — reference only |
| Hoplite (note 10) | PR story timeline, PR Auto-fix | Proprietary — reference only |
| affectUI Review Center (note 11) | name only | Paid — study only |
| Arc one-cell label rule (note 12) | merge button label cell | Paid — study only |
