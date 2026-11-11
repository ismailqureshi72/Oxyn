**Status:** draft v0.1 (2026-11-11).

# Spec: agent code-review card (`review-card`)

Part of family H, "Checkpoints, diff & git" (overview in [checkpoints-git.md](checkpoints-git.md)).
Same depth as [composer-send-control.md](composer-send-control.md). Target **3 free + 12 Pro = 15
variants**. Owns the **result of an agent reviewing code**: a summary verdict, findings grouped by
severity or file, `file:line` links into [diff-viewer.md](diff-viewer.md), suggested fixes with Apply,
Dismiss and Resolve thread. Human PR review state lives on [pr-card.md](pr-card.md); this card can feed
it.

## 1. Purpose
The user asks the agent to review a diff, a branch or a PR (or a reviewer agent runs automatically
after edits). They want to know **is it safe to ship, what are the real problems and where, and can I
fix them in one click** — then clear the list. Research note 03 lists the "code-review findings card"
as common: Claude desktop `/code-review` → card grouped by file with **Walk through in diff**, **Fix
this one**, **Apply fixes** (★ the walk-through stepper); Codex `/review` with prioritized findings;
Verdent Review Changes → Code Review tab; Jules critic; Cursor Bugbot / Agent Review. The Codex app
shows an automatic review item with review status and **risk level** (approved / denied / stopped /
timed out) — ★ "a visible reviewer verdict chip" (note 03). Batched line comments sent back to the
agent are "the defining 2026 review loop" (note 03 §1F). Claude Code marks an "ultrareview" with ◇
running / ◆ done glyphs (note 08). HacktronAI routes each security finding as a dot along metro lines
to Patched / False positive outcomes [R038]. A running-tests card pops a fix card with a diff over the
failing file [R037]. affectUI lists Agent Findings and Review Center (names only, Pro, note 11).

Where it appears: transcript (result of a review tool call), the diff viewer's side rail and gutter,
PR card ("Agent review: 2 issues"), review queues, fan-out lanes (reviewer lane).

## 2. Anatomy
```
┌ ✦ Agent review · feature/auth-refresh · 12 files ─────────────────── ◆ done · 48s ┐
│ Verdict  ⚠ Changes suggested — 1 critical, 2 warnings, 3 nits                      │  verdict chip + counts
│ "Token refresh can race with logout; the rest is solid."                            │  summary (≤ 3 lines)
│ Group by: Severity ▾        Unresolved 6 · Resolved 0              Walk through ›   │
├─────────────────────────────────────────────────────────────────────────────────────┤
│ ● CRITICAL  Race between refresh and logout              src/auth.rs:142            │  finding row
│   Refresh writes the session after logout cleared it…                  More         │
│   ┌ suggested fix ───────────────────────────────── +3 −1 ┐                         │  fix preview (diff-viewer
│   │ 142 − session.store(tok);                               │                        │   mini hunk)
│   │ 142 + if !session.is_revoked() { session.store(tok); }  │                        │
│   └──────────────────────────────────────────────────────────┘                       │
│   [Apply fix]  Dismiss ▾  Reply  · Resolve                                          │
│ ● WARNING   Missing test for expired token            tests/auth.rs  (no line)      │
│ ○ NIT       Rename `tok` → `token`                    src/auth.rs:139  [Apply fix]  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│ [Apply 4 fixes]   Send comments to agent (2)   Post to PR ▾                          │  batch footer
└─────────────────────────────────────────────────────────────────────────────────────┘
```
**Severity scale (proposed, labels in `ReviewLabels`):**
| Level | Glyph | Meaning | Token |
|---|---|---|---|
| Critical | ● filled, danger | likely bug, security issue or data loss; blocks the verdict | `color.review.critical` |
| Warning | ● filled, amber | correctness risk or missing test | `color.review.warning` |
| Suggestion | ◐ half, accent-neutral | better approach, readability | `color.review.suggestion` |
| Nit | ○ hollow, neutral | style, naming | `color.review.nit` |
| Info | ⓘ, neutral | note, praise or context; never counted as an issue | `color.review.info` |
Adapters map their own levels onto these (e.g. P0–P3 priorities); unknown levels map to Suggestion.

**Verdicts (proposed):** `Looks good` (✓) · `Changes suggested` (⚠, no criticals) · `Blocking issues`
(✕, ≥ 1 critical) · `Couldn't review` (!) — derived from findings unless the agent supplies one; if
the agent's stated verdict disagrees with the counts, both show ("Agent says: looks good · 1 critical
found") and the stricter one colours the chip.

## 3. State machine
```
  Review run:  Requested ─▶ Reviewing{files_done/total, findings so far} ─▶ Done{verdict}
                  │                 │ stop                 │ error          ▲
                  │                 ▼                      ▼                │ re-run
                  └─ queued ─▶ Queued{pos}   Cancelled{partial}   Failed{reason}
               Done ── new commits / files changed since review ──▶ Stale{commits} ──re-run──▶ Reviewing
               Done ── all findings resolved or dismissed ──▶ Cleared (✓ "All findings addressed")
  Per finding: Open ─▶ Focused (link → diff-viewer at file:line)
               Open ─Apply fix─▶ (AwaitingApproval if permission = ask) ─▶ Applying ─▶ Applied ─▶ Resolved
                                                                           ├─ patch no longer applies ─▶ FixConflict ─▶ Open (offer "Ask agent to redo")
                                                                           └─ error ─▶ FixFailed{reason} ─▶ Open
               Open ─Dismiss{reason}─▶ Dismissed  (Not an issue · Won't fix · False positive · Duplicate)
               Open ─Reply─▶ Discussing (thread) ─agent answers─▶ Open | Resolved
               Open ─Resolve─▶ Resolved          Resolved / Dismissed ─Reopen─▶ Open
               Open ─ code at its line changed by someone ─▶ Outdated (line moved / removed)
  Batch:       Apply N fixes ─▶ ApplyingBatch{done/N} ─▶ results per finding
               Send comments ─▶ Sending ─▶ Sent (agent turn starts, family A queue/steer)
               Post to PR ─▶ ConfirmPost (outbound, family G) ─▶ Posting ─▶ Posted{n} | PostFailed
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (focused) |
|---|---|---|---|---|
| `Queued` | "Review queued · 2nd" with static ◇ | — | Cancel | Esc |
| `Reviewing` | ◇ glyph with [text-shimmer.md](text-shimmer.md) "Reviewing session.rs · 5 of 12 files"; findings stream in as they are reported; verdict chip shows "Reviewing…" | Stop | — | Esc stop |
| `Done{verdict}` | ◆ + verdict chip + counts + summary; elapsed "48s" | Walk through | Apply all fixes, group by | W walk through |
| `Cancelled{partial}` | "Stopped after 5 of 12 files · 3 findings" (findings kept, verdict "Partial") | Resume / Re-run | — | — |
| `Failed{reason}` | `!` "Couldn't review: diff too large for context" | Re-run | Review fewer files (if host supports) | R |
| `Stale{commits}` | amber strip "2 new commits since this review" ; findings keep their state, line links may be `Outdated` | Re-run review | Keep | R |
| `Cleared` | ✓ "All findings addressed" (resolved 4 · dismissed 2), card collapses to one line after 4 s | — | Show findings | Enter expand |
| Finding `Open` | severity glyph + label, title, `file:line` link, 2-line body, fix preview if any | Apply fix (if fix) / Go to line | Dismiss ▾, Reply, Resolve | Enter go to line; A apply; D dismiss; Shift+R resolve |
| Finding `Focused` | row highlight; linked line pulses in the diff viewer | — | — | ↑/↓ prev / next finding |
| `AwaitingApproval` (fix) | amber bar; family G approval inline "Apply fix to src/auth.rs?" | Allow | Deny | per family G |
| `Applying` | Apply button label "Applying…" in the same cell; row loader | — | — | — |
| `Applied` → `Resolved` | ✓ "Fix applied" + file-edit chip `+3 −1` ([file-edit-chip.md](file-edit-chip.md)), row dims to 60 % and moves to "Resolved" group on next regroup | Undo (via checkpoint) | View diff | U undo |
| `FixConflict` | "Fix no longer applies — the code changed" | Ask agent to redo | Go to line | — |
| `FixFailed` | danger line with reason | Retry | Copy fix | R |
| `Dismissed{reason}` | struck title, reason tag ("False positive"), dim | Reopen | — | O reopen |
| `Discussing` | thread under the finding: user reply + agent answer streaming | Send | Resolve | ⌘/Ctrl+Enter send |
| `Outdated` | "Line changed since review" tag; link goes to nearest line | Go to nearest | Re-check this finding | — |
| Batch `ApplyingBatch` | "Applying 2 of 4 fixes" bar | Stop | — | Esc |
| `ConfirmPost` | inline "Post 6 comments to #128 as a review?" with preview list (outbound — never automatic) | Post | Cancel | Enter / Esc |
| `Posted{n}` | "Posted 6 comments to #128" with link | Open PR ↗ | — | — |
| Disabled-with-reason | e.g. Apply fix while the agent is editing that file: "File is being edited"; no fix: Apply hidden, "Go to line" primary | — | — | focusable |

## 5. Micro-interactions
- **Streaming findings.** Each finding enters with `dur.row.enter` and slots into its severity group;
  groups keep a stable order (Critical → Info) and a finding never moves after it has rendered — new
  ones append to their group (no reshuffle under the pointer). Group counts and verdict counts roll
  (`dur.count.roll`).
- **Verdict reveal.** On `Done`, ◇ → ◆ fills (160 ms) and the verdict chip crossfades from
  "Reviewing…" to its verdict, tinting over 300 ms; no celebration on "Looks good" beyond a ✓ stroke
  draw.
- **Link to line.** Hover `file:line` shows the 3 surrounding lines in a hover card after 400 ms;
  click scrolls the diff viewer to the line and pulses its gutter once in the severity colour (600 ms).
- **Walk through.** A stepper "1 of 6" pins to the card header; Next moves focus to the next open
  finding **and** scrolls the diff viewer to it (Claude's walk-through-in-diff idea, note 03); the
  card scrolls so the finding sits at the top of its viewport.
- **Apply fix.** Fix preview's `+` lines briefly brighten, Apply label → "Applying…" (one cell), then
  ✓ and a file-edit chip slides out from the button (160 ms). The finding dims and, after a 1.2 s
  hold, collapses to one line (`dur.collapse`), unless it is focused.
- **Dismiss.** Menu of reasons; choosing one strikes the title left → right (200 ms) and dims; the
  row collapses after 1.2 s; a 5 s "Undo" toast appears.
- **Resolve.** Check draws in the row's glyph (`dur.check.draw`), row dims; resolved count rolls.
- **Cleared.** When the last open finding closes, the verdict chip morphs to ✓ "All findings
  addressed" and the card folds after 4 s (like the status strip's Done hold).
- **Stale.** Amber strip slides in from the header (200 ms); outdated line links get a dashed underline.
- **Hover / press.** Row tint 80 ms; buttons press scale 0.97 for 80 ms.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Finding enter | 200 ms height + 160 ms fade | 0 | `spring.snap` / `ease.out` | 40 ms cap 6 | 60 Hz |
| Count roll (groups, verdict) | 280 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz |
| "Reviewing…" shimmer | 1400 ms sweep / 800 ms rest | 600 ms | `ease.coast` | — | 30 Hz |
| ◇ → ◆ + verdict crossfade | 160 ms + 300 ms tint | 0 | `spring.snap` / linear OKLab | — | 60 Hz |
| Hover card | 160 ms | 400 ms | `ease.out` | — | 60 Hz |
| Gutter pulse at line | 600 ms ×1 | after scroll | `ease.in-out` | — | 60 Hz |
| Walk-through scroll | 260 ms | 0 | `ease.in-out` | — | 60 Hz |
| Apply label swap / chip out | 120 ms / 160 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Resolved / applied collapse | 280 ms | 1200 ms hold | `ease.in` | — | 60 Hz |
| Dismiss strike | 200 ms | 0 | `ease.out` | — | 60 Hz |
| Undo toast | 5000 ms hold, 160 ms in / out | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Batch bar step | 300 ms per change | 0 | `ease.out` | 120 ms cap 6 | 30 Hz |
| Cleared fold | 280 ms | 4000 ms hold | `ease.in-out` | 20 ms per row cap 6 | 60 Hz |
| Stale strip in | 200 ms | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | findings appear with 80 ms fade and instant height; counts swap; no shimmer (static ◇ + "Reviewing 5 of 12 files"); no strike draw (strike appears), no pulses (line gets a static 2 px bar for 4 s), no auto-collapse animation (instant), walk-through jumps | — | linear | none | event-driven |

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `review-card/by-severity` | Verdict + summary, findings grouped Critical → Info with counts, per-finding fix preview and actions, batch footer | chat, desktop, ide |
| `review-card/by-file` | Same data grouped by file with per-file counts and `line` links; matches the Claude "grouped by file" layout idea (note 03) | ide, desktop |
| `review-card/inline-thread` | No card: each finding is a thread anchored in the diff viewer gutter with severity glyph; a small header chip "6 findings · Walk through" sits above the diff | ide, desktop |

### Pro (12)
| Name | One-line idea |
|---|---|
| `review-card/walkthrough` | Full-screen stepper: one finding + its diff at a time, Apply / Dismiss / Next, progress dots |
| `review-card/verdict-chip` | Minimal reviewer-verdict chip for rows and PR cards: ✓ / ⚠ / ✕ + counts, expands to the card (Codex auto-review idea, note 03) |
| `review-card/triage-metro` | Findings as dots travelling a metro line to outcomes Fixed / Dismissed / Open [R038] |
| `review-card/severity-bar` | A stacked bar of findings by severity across the header that shrinks as they resolve |
| `review-card/heat-gutter` | Diff scrollbar heat marks per finding, coloured by severity (Paradev marker idea, note 03) |
| `review-card/fix-stack` | Suggested fixes as a stack of mini diffs; Apply all applies in order with per-fix results |
| `review-card/batch-comments` | User adds comments on findings or lines, then "Send 4 comments to agent" as one steer (the batched review loop, note 03) |
| `review-card/multi-reviewer` | Several reviewer agents (security, tests, style) as tabs or columns, findings de-duplicated with "found by 2" |
| `review-card/confidence` | Each finding shows the agent's stated confidence; low-confidence findings fold under "Maybe" |
| `review-card/fix-popover` | Fix card pops over the failing file in the diff with its patch, like the test-runner fix card [R037] |
| `review-card/ultrareview` | ◇ / ◆ glyph header with depth level and elapsed, for long deep reviews (Claude Code glyph idea, note 08) |
| `review-card/tui` | Terminal list: `[C] src/auth.rs:142 Race between refresh and logout`, keys `a` apply, `d` dismiss, `n`/`p` |

## 8. Edge cases
- **No findings.** Verdict "Looks good" with the summary and "0 issues in 12 files"; never an empty
  card.
- **Hundreds of findings.** Groups show the first 5 per group then "and 41 more"; Nit and Info groups
  collapsed by default when > 20 findings total (proposed); list virtualised.
- **Finding without a line** (file- or PR-level): link shows the file only; PR-level shows "General".
- **Line ranges.** `src/auth.rs:142–150`; ranges > 30 lines link to the start with a "30+ lines" tag.
- **Files not in the diff.** Link opens the file at the line (host), tagged "outside this change".
- **Fix touches several files.** Preview shows per-file mini hunks and a combined `+a −r`; Apply is
  atomic — all or none.
- **Agent proposes a fix in prose only.** No Apply button; "Ask agent to apply" sends a steer.
- **Duplicate findings** across re-runs: matched by file + line + title hash (proposed); state carries
  over ("previously dismissed" tag) instead of reopening.
- **Very fast reviews.** Findings arriving inside `delay.row_guard` render without enter animation;
  verdict shows directly.
- **Very slow reviews.** Streaming progress by files; stall thresholds from the status strip; partial
  findings are actionable before Done.
- **Posting to a PR** is outbound and visible to others: always `ConfirmPost` with the exact comments
  previewed; never auto-post, whatever the autonomy setting (family G).
- **Long titles / bodies.** Title 1 line in compact, 2 lines in card; body 2 lines then "More"; code
  in bodies uses inline code styling and never wraps mid-token.
- **RTL.** Card mirrors; paths, `file:line` and code stay LTR in bidi isolates; severity glyph stays at
  inline-start.
- **Tiny sizes (16–20 px).** Only the verdict glyph with a count (`⚠ 3`); tooltip has the breakdown.
- **Dense IDE.** Rows at 22 px, fix previews collapsed by default (expand with →).
- **Touch.** 44 px targets; swipe right = resolve, swipe left = dismiss (with undo toast); fix preview
  scrolls horizontally inside its frame.
- **Offline.** Apply / Post disabled-with-reason; dismiss / resolve queue locally and sync later with a
  "pending" dot.
- **Colour-blind.** Severity always has glyph shape + text label; daltonized preset swaps danger/amber
  hues per [file-edit-chip.md](file-edit-chip.md) §8.

## 9. Accessibility
- **Card name:** "Agent review of feature/auth-refresh, changes suggested, 1 critical, 2 warnings, 3
  nits, 6 unresolved".
- **Structure.** `article` with heading; each severity / file group is a labelled `list`; each finding
  is a `listitem` with heading "Critical: Race between refresh and logout, src/auth.rs line 142".
- **Fix preview** is a labelled region "Suggested fix, 3 lines added, 1 removed" with the diff
  readable line by line ("removed: …", "added: …").
- **Announcements (polite):** "Review finished: changes suggested, 6 findings" once; "Fix applied" /
  "Finding dismissed as false positive" / "Resolved, 5 left". Streaming findings are **not**
  announced individually (only a count every 10 s while reviewing, proposed). **Assertive:** fix
  failed, post failed.
- **Keyboard:** ↑/↓ between findings; Enter go to line; A apply fix; D dismiss (opens reason menu);
  Shift+R resolve; O reopen; W walk through; Esc stop review / close menus. Keys act only within the
  focused card.
- **Contrast.** Severity text labels ≥ 4.5:1; glyphs ≥ 3:1; struck dismissed titles stay ≥ 4.5:1.
- **Focus.** After Apply / Dismiss, focus moves to the next open finding (announced); never lost to
  the document when a row collapses.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct Review {
    pub id: ReviewId,
    pub target: ReviewTarget,              // Diff(DiffRef) | Branch { name, base } | Pr { number }
    pub reviewer: ReviewerRef,             // agent / model label
    pub state: ReviewState,                // Queued { pos } | Reviewing { files_done, files_total } | Done
                                           // | Cancelled | Failed { reason } | Stale { commits } | Cleared
    pub verdict: Option<Verdict>,          // LooksGood | ChangesSuggested | Blocking | CouldNotReview
    pub agent_verdict: Option<Verdict>,    // as stated by the agent, shown if it disagrees
    pub summary: Option<String>,
    pub findings: Vec<Finding>,            // arrival order within each group, never re-sorted live
    pub elapsed: Option<Duration>,
}

pub struct Finding {
    pub id: FindingId,
    pub severity: Severity,                // Critical | Warning | Suggestion | Nit | Info
    pub title: String,
    pub body: Option<String>,
    pub location: Option<CodeLocation>,    // { path, lines: Option<LineRange>, outdated: bool, in_change: bool }
    pub fix: Option<SuggestedFix>,         // { diff: DiffRef, added, removed, files: u16 }
    pub confidence: Option<Confidence>,
    pub state: FindingState,               // Open | AwaitingApproval | Applying | Applied | Resolved | FixConflict
                                           // | FixFailed { reason } | Dismissed { reason } | Discussing | Outdated
    pub thread: Vec<ThreadMessage>,
}

pub struct ReviewCard<'a> {
    pub review: &'a Review,
    pub agent: &'a AgentState,
    pub variant: ReviewVariant,            // BySeverity | ByFile | InlineThread | Pro(..)
    pub collapse_low: CollapseLow,         // NitsAndInfoOver(20) (proposed)
    pub on_go_to: Callback<CodeLocation>,  // → diff-viewer
    pub on_apply: Option<Callback<Vec<FindingId>>>,
    pub on_dismiss: Callback<(FindingId, DismissReason)>,
    pub on_resolve: Callback<FindingId>,
    pub on_reply: Option<Callback<(FindingId, String)>>,
    pub on_send_to_agent: Option<Callback<Vec<FindingId>>>,
    pub on_post_to_pr: Option<Callback<Vec<FindingId>>>,   // host shows family G confirm first
    pub on_rerun: Option<Callback<()>>,
    pub labels: ReviewLabels,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [checkpoints-git.md](checkpoints-git.md) · [diff-viewer.md](diff-viewer.md) (line links,
fix previews, gutter threads) · [pr-card.md](pr-card.md) (review section, "Address with agent") ·
[merge-conflict.md](merge-conflict.md) · [file-edit-chip.md](file-edit-chip.md) (applied fix chip) ·
[checkpoint-timeline.md](checkpoint-timeline.md) (undo of applied fixes) · [text-shimmer.md](text-shimmer.md) ·
[composer-queue-stack.md](composer-queue-stack.md) (sending comments while the agent runs) · family G
approvals (apply / post).

| Reference | What we take | Licence note |
|---|---|---|
| Claude desktop `/code-review` (note 03) | group by file, Walk through in diff, Fix this one, Apply fixes | Proprietary — reference only |
| Codex `/review`, app auto-review with risk level (note 03) | prioritized findings, verdict chip | Codex CLI Apache-2.0 per note 08 (MIT per licensing.md) — re-check; app proprietary |
| Verdent, Jules, Cursor Bugbot (note 03) | review tab, critic | Proprietary — reference only |
| Claude Code ◇ / ◆ ultrareview glyphs (note 08) | running / done glyph pair | Proprietary — reference only (glyph idea, our own drawing) |
| HacktronAI triage metro [R038] | findings routed to outcomes | Study only |
| Test-runner fix card [R037] | fix popover over the failing file | Study only |
| affectUI Agent Findings, Review Center (note 11) | names only | Paid — study only |
| signoff-ui DiffReview [R107] | per-hunk apply model for fix previews | MIT — port with attribution |
