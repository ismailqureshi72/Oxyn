**Status:** draft v0.1 (2026-11-12).

# Category kit: coding agents

Kit for section B row **Coding agents** of [../component-taxonomy.md](../component-taxonomy.md):
"Desktop IDE agent window, CLI/TUI look, review queue · Diff/merge, checkpoints, terminal tabs, PR
card, sub-agent grid". Built only from families A–L; registry ids are `shell/coding-<screen>`
(family M). Tokens from [../tokens.md](../tokens.md), motion from [../motion.md](../motion.md).
Everything marked *proposed* is Oxyn's own starting point, to be tuned in the Studio.

## 1. Who it's for

| | |
|---|---|
| Job | "Take this issue or failing check, change the code, prove it works, and hand me a diff I can trust." Then: run several of those at once, review them in one place, merge. |
| User | A developer who runs 1–10 agent threads a day against one or more repos, locally (worktrees) or in a cloud sandbox, and reviews every diff before it merges. |
| Moments | start a thread from an issue · watch it work · answer approvals · steer mid-run · review hunks · send line comments back · restore a checkpoint · fan out a refactor · resolve a conflict · open and merge a PR. |
| Not for | full editors (we embed a diff and a file peek, not an IDE), app builders with live preview ([app-builder.md](app-builder.md)). |

**Demo company (invented).** **Larch Labs** — a small dev-tools company. Product **Tallow**, an
open-source Rust job queue. Repo `larch-labs/tallow`: crates `tallow-core`, `tallow-worker`,
`tallow-http`, a web `console/`, CI with four checks (Tests, Lint, Types, Preview). Deploy target
`tallow-cloud · production`. People: **Noor Halvorsen** (staff engineer, the demo user), **Tomas
Ekwueme** (maintainer, reviewer), **Wren Castellanos** (teammate). Agent: "Larch agent"; models
`ridge-1` and `ridge-1-mini`. No real people, companies, logos or model names.

## 2. Shell

### 2.1 Desktop (GPUI) — default `shell/coding-workspace`

```text
┌ ● ● ●  tallow ▾  ⑂ fix/retry-flake ↑2 ●  ▣ wt-retry   Local ▾  Hand off  Open ▸   ⚡5/6  ◔ 41%    ┐
├──────────────┬──────────────────────────────────────────────┬────────────────────────────────────┤
│⌕ Search  ⌘K  │ Fix flaky retry_backoff test            ⋯    │ [Changes 3] [Files] [Plan] [Checks]│
│+ New thread  │ ──────────────────────────────────────────   │ file-tree/changed                  │
│NEEDS YOU     │ ◆ Thinking · 3s                        ▸     │  M worker/retry.rs   +18 −4        │
│⚠ Deploy fix  │ ▸ Ran cargo test -p tallow-worker  ✕ 1       │  M tests/retry.rs    +9 −1         │
│RUNNING       │ ▸ Read worker/retry.rs                       │  A tests/clock.rs    +31           │
│⠋ Retry flake │ ▸ Edited worker/retry.rs        +18 −4       │ ─────────────────────────────────  │
│⠋ Split http  │ The test used wall time; I injected a…       │ diff-viewer/unified                │
│DONE          │ ◇ checkpoint 3 · Restore ▾                   │ 41 −  let now = Instant::now();    │
│✓ Docs typo   │ ──────────────────────────────────────────   │ 41 +  let now = self.clock.now();  │
│✕ Bench run   │ ┌────────────────────────────────────────┐   │      [Keep] [Undo]   hunk 1 of 4   │
│              │ │ Ask, @file, /command…                  │   │                                    │
│⚙  ⚡ ▤        │ │ Agent ▾ ridge-1 ▾ ◑ Med  Ask ▾     [■] │   │ [Keep all] [Send 2 comments]       │
│              │ └────────────────────────────────────────┘   │                                    │
├──────────────┴─ [◆ ⠋ cargo test 0:41] [● zsh] [+] ──────────┴────────────────────────────────────┤
│ $ cargo test -p tallow-worker retry_backoff  …  test retry::backoff_caps ... ok                  │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ ● Working · Editing retry.rs 0:42   ⑂ fix/retry-flake ↑2   ▣ sandbox   12.4k tok   ? keys        │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

| Region | Component (spec) | Default size |
|---|---|---|
| Title / thread header | **new** `thread-header` (§6) + [git-chips](../specs/git-chips.md) `git-chips/pill` + [mcp-plugin-list](../specs/mcp-plugin-list.md) `mcp-list/chip` + [composer-context-meter](../specs/composer-context-meter.md) `context-meter/ring` | 40 px |
| Left dock | **new** `thread-list` (§6) | 260 px, rail 48 px |
| Center | [messages](../specs/messages.md) `transcript/ide-panel` + [composer](../specs/composer.md) `composer/ide-panel` | ≥ 320 px |
| Right dock | [side-panels](../specs/side-panels.md) `side-panel/tabs` hosting [diff-viewer](../specs/diff-viewer.md), `file-tree/changed`, [plan-todo](../specs/plan-todo.md), [pr-card](../specs/pr-card.md) `pr-card/checks-list` | 320 px (grows to 50 % in review) |
| Bottom dock | [terminal-tabs](../specs/terminal-tabs.md) `terminal-tabs/strip` + [terminal-output](../specs/terminal-output.md) `terminal-output/block` | 33 %, collapsed by default |
| Status bar | [side-panels](../specs/side-panels.md) `status-bar/ide` + [composer-status-strip](../specs/composer-status-strip.md) | 24 px |
| Layout | [workspace-chrome](../specs/workspace-chrome.md) `panel-split/hairline`, F6 region cycling | sizes saved per project |

### 2.2 Web (Dioxus) `shell/coding-web`
Same regions. Differences: no native title bar (thread header is the top row); terminal dock shows
`terminal-output/tail` (read-only stream from the sandbox, no PTY); pop-out windows become
`panel-split/peek` overlays; notifications through [command-palette](../specs/command-palette.md)
`notification-center/toasts` instead of OS notifications.

### 2.3 Breakpoints (from [workspace-chrome](../specs/workspace-chrome.md) §2, 48 px hysteresis)

| Window width | Layout |
|---|---|
| ≥ 1280 px | three docks visible; right dock 320–50 % |
| 900–1279 px | left dock snaps to 48 px rail; right dock visible |
| 720–899 px | right dock becomes an overlay (scrim) opened from the Changes badge |
| 600–719 px | left dock also an overlay; terminal dock opens as a bottom sheet |
| < 600 px | narrow / mobile (§2.4) |

### 2.4 Narrow / mobile `shell/coding-mobile`
Phone is for **watching, approving, reviewing and steering**, not editing. One pane with a segmented
switch; approvals and diffs are full-width sheets.

```text
┌──────────────────────────────┐
│ ‹ Threads  Retry flake  ⋯    │
│ ⑂ fix/retry-flake ↑2  ⠋ 0:42 │
├──────────────────────────────┤
│ [Chat] [Changes 3] [Checks]  │  side-panel/segmented
│ ▸ Ran cargo test  ✕ 1 fail   │
│ ▸ Edited retry.rs  +18 −4    │
│ ◇ checkpoint 3               │
│ ┌ ▲ High risk ─────────────┐ │  approval-card/modal (sheet)
│ │ $ git push --force-with… │ │
│ │ [Deny]  [Allow once]     │ │
│ └──────────────────────────┘ │
├──────────────────────────────┤
│ Steer or queue…        [↑]   │  composer/mobile
└──────────────────────────────┘
```
Thread list is its own screen (back arrow); terminal is read-only `terminal-output/tail`.
Critical approvals on touch use the hold (1200 ms, haptic ticks) per
[approvals.md](../specs/approvals.md) §4.

## 3. Screens

| # | Screen | Registry id | Surface |
|---|---|---|---|
| S1 | Project home (dashboard) | `shell/coding-dashboard` | desktop, web |
| S2 | Thread workspace | `shell/coding-workspace` | desktop, web, mobile |
| S3 | Terminal look | `shell/coding-terminal` | terminal |
| S4 | Review queue | `shell/coding-review-queue` | desktop, web, mobile |
| S5 | Change review (detached) | `shell/coding-review` | desktop, web |
| S6 | Parallel agents / worktrees | `shell/coding-parallel` | desktop, web |
| S7 | Pull request | `shell/coding-pr` | desktop, web, mobile |
| S8 | Merge conflict | `shell/coding-merge` | desktop, web, terminal |
| S9 | History & checkpoints | `shell/coding-history` | desktop, web, terminal |
| S10 | Settings · permissions | `shell/coding-settings` | desktop, web |
| S11 | MCP & plugins | `shell/coding-mcp` | desktop, web, terminal |
| S12 | Onboarding / empty project | `shell/coding-onboarding` | desktop, web, terminal |

### S1 Project home (dashboard)
Purpose: what needs me, what is running, what shipped. Opens on launch and on "Home" in the rail.

```text
┌ tallow ▾ · larch-labs/tallow ──────────────────────────────────── [+ New thread ⌘N] ┐
│ ┌ composer/desktop (hero) ──────────────────────────────────────────────────────┐ │
│ │ Describe a task, paste an issue link, or @ a file…                             │ │
│ │ ⑂ main ▾   Worktree ▾   ridge-1 ▾   Ask ▾                         [Start]     │ │
│ └────────────────────────────────────────────────────────────────────────────────┘ │
│ Needs you (2)                         │ Running (3)                                │
│ ⚠ Deploy fix · ⬣ Critical approval    │ ⠋ Retry flake    Editing retry.rs   0:42   │
│ ✎ Split http · 2 hunks to review      │ ⠋ Split http     4 of 6 sub-agents  6:10   │
│                                       │ ⠋ Bench refresh  Running cargo bench 1:55  │
│ Recent PRs                            │ Usage today                                │
│ #212 Fix retry flake   ◌ 3/4 checks   │ run-meter/panel  412k tok · 18 runs        │
│ #209 Docs typo         ● merged 2h    │ sparkline of runs (Pro: run-meter/sparkl…) │
└───────────────────────────────────────┴────────────────────────────────────────────┘
```
Uses: [composer](../specs/composer.md) `composer/desktop` + [composer-input-shell](../specs/composer-input-shell.md) `composer-input/hero-dock` ·
**new** `review-queue` compact list (Needs you) · [background-jobs](../specs/background-jobs.md) `background-jobs/panel` (Running) ·
[pr-card](../specs/pr-card.md) `pr-card/compact` · [run-meter](../specs/run-meter.md) `run-meter/panel` · [risk-badge](../specs/approvals.md) `risk-badge/pill`.

| State | Shows |
|---|---|
| Empty | S12 onboarding instead |
| Loading | [skeletons](../specs/skeletons.md) `skeleton/shimmer` per card after `delay.container.show` |
| Error | per-card inline error ("Couldn't reach GitHub · Retry"); other cards keep working |

### S2 Thread workspace
Purpose: one agent thread end to end. Wireframe is §2.1. Transcript rows:

| Row | Component |
|---|---|
| user turn | [message-bubble](../specs/message-bubble.md) `message/flat` + `message-actions/hover-bar` (edit, fork) |
| reasoning | [reasoning-block](../specs/reasoning-block.md) `reasoning-block/inline-summary` ("Thought for 3s") |
| tool calls | [tools](../specs/tools.md) `tool-stack/worked-for` grouping [tool-call](../specs/tool-call.md) `tool-call/row` |
| edits | [file-edit-chip](../specs/file-edit-chip.md) `file-edit-chip/row` (+N −N, opens the diff) |
| shell | [terminal-output](../specs/terminal-output.md) `terminal-output/block`, auto-collapse after `delay.auto-collapse` |
| plan | [plan-todo](../specs/plan-todo.md) `plan-card/document` + [plan-approval](../specs/plan-approval.md) `plan-approval/footer` |
| approval | [approval-card](../specs/approval-card.md) `approval-card/footer` inside the tool-call card |
| checkpoint | [checkpoint-timeline](../specs/checkpoint-timeline.md) `checkpoint-timeline/inline-marker` per turn |
| answer | [streaming-markdown](../specs/streaming-markdown.md) `markdown/compact` + `code-block/file` |
| working line | [status-text](../specs/status-text.md) + [text-shimmer](../specs/text-shimmer.md) `text-shimmer/sweep` + [verb-cycler](../specs/verb-cycler.md) `verb-cycler/fade` + [elapsed-timer](../specs/elapsed-timer.md) `elapsed-timer/units` |
| jump to live | [messages](../specs/messages.md) `jump-pill/labelled` |

Composer row: [composer-mode-switch](../specs/composer-mode-switch.md) `mode-switch/chip-menu` (Plan / Agent / Ask) ·
[composer-model-picker](../specs/composer-model-picker.md) `model-picker/pill-popover` ·
[composer-effort-control](../specs/composer-effort-control.md) `effort-control/signal-bars` ·
[composer-permission-selector](../specs/composer-permission-selector.md) `permission-selector/chip-list` ·
[composer-send-control](../specs/composer-send-control.md) `send-control/classic` (send / stop / queue / steer) ·
[composer-queue-stack](../specs/composer-queue-stack.md) `queue-stack/chips` ·
[composer-slash-mentions](../specs/composer-slash-mentions.md) `trigger-menu/popover` (`@file`, `@issue`, `/review`, `/undo`).

| State | Shows |
|---|---|
| Empty thread | [citations-sources](../specs/citations-sources.md) `empty-state/starters-grid` with repo-aware starters ("Fix a failing check", "Explain crate layout") |
| Loading history | `skeleton/static` rows; scroll anchor kept (zero-jump rule) |
| Agent error | tool row `Failed` + inline "Retry step" / "Ask agent to recover"; status strip `Error` |
| Offline | composer disabled with reason; queued messages kept; approvals disabled (fail safe) |
| Stalled | status line tone shifts (`color.state.stalled`, `text-shimmer/stall-ember` Pro) after the stall threshold |

### S3 Terminal look `shell/coding-terminal`
Same thread as S2 for a TUI or a "terminal theme" desktop window. Monospace grid, `radius.none`,
`color.ansi.*` only.

```text
 tallow  ⑂ fix/retry-flake ↑2*  ▣ wt-retry  ridge-1 · med · ask                    ◔ 41%
 ─────────────────────────────────────────────────────────────────────────────────────────
 › fix the flaky retry_backoff test
 • Ran cargo test -p tallow-worker retry_backoff
   └ test retry::backoff_caps ... FAILED (expected 400ms, got 412ms)
 • Edited worker/retry.rs (+18 −4)
   41 -    let now = Instant::now();
   41 +    let now = self.clock.now();
 ◇ checkpoint 3
 ▲ high  Run `git push --force-with-lease origin fix/retry-flake`?
   Reason: rebased onto main
   1 Yes (y)   2 Yes, for this session (s)   3 No, and tell the agent why (esc)
 ⠋ Working (12s · esc to interrupt)
 ↳ queued: also update CHANGELOG
 › _
 ─────────────────────────────────────────────────────────────────────────────────────────
 ? for shortcuts                                              12.4k tok · 41% context left
```
Uses `*/terminal` and `*/tui` variants: `transcript/terminal`, `composer/terminal`
(`composer-input/terminal-prompt`), `tool-call/terminal`, `diff-viewer/terminal`,
`approval-card/terminal`, `checkpoint-timeline/terminal`, [loader-glyph](../specs/loader-glyph.md)
`loader/glyph-braille`, `text-shimmer/ascii`, `queue-stack/minimal`, `status-bar/tui`,
`risk-badge/glyph`. Empty: `empty-state/terminal`. Error: red `✕` row plus a plain-text hint line.

### S4 Review queue `shell/coding-review-queue`
Purpose: one inbox for everything waiting on the user across threads: approvals, diffs, review
findings, failing PR checks, conflicts. Ordered by risk, then age.

```text
┌ Review queue · 7 waiting ──────── Filter: All ▾  Repo ▾  Agent ▾ ───── [j/k] move [↵] open ┐
│ ⬣ Critical  Deploy fix        $ tallow-cli deploy --env production     2m  [Open]        │
│ ▲ High      Retry flake       $ git push --force-with-lease            4m  [Open]        │
│ ✎ Diff      Split http        14 files  +482 −37  · 3 of 14 viewed     9m  [Review]      │
│ ● Finding   Split http        1 critical, 2 warnings (agent review)    9m  [Walk through]│
│ ✕ Check     #212 Retry flake  Types failed · ✦ Auto-fix available     12m  [Fix]         │
│ ⚠ Conflict  Split http        3 conflicts with main                   15m  [Resolve]     │
│ ? Question  Bench refresh     "Keep the old baseline?" (recommended)  20m  [Answer]      │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ preview pane: the selected item's own component (approval-card / diff-viewer / review-card)│
└───────────────────────────────────────────────────────────────────────────────────────────┘
```
Uses: **new** `review-queue` (rows + preview) · [approval-card](../specs/approval-card.md)
`approval-card/paged` · [diff-viewer](../specs/diff-viewer.md) `diff-viewer/unified` ·
[review-card](../specs/review-card.md) `review-card/by-severity` · [pr-card](../specs/pr-card.md)
`pr-card/checks-list` · [merge-conflict](../specs/merge-conflict.md) `merge-conflict/card` ·
[genui-forms](../specs/genui-forms.md) `genui-question/numbered` · `risk-badge/shape`.
Batch: only rows the approval batch rule allows (≤ Medium, one kind, one agent; approvals.md §2.6).

| State | Shows |
|---|---|
| Empty | "Nothing waiting on you" + running count; `empty-state/minimal` |
| Loading | 4 `skeleton/static` rows |
| Error | row-level "Couldn't load · Retry"; an item answered elsewhere resolves in place with its origin |

### S5 Change review (detached) `shell/coding-review`
Full-window review of one thread's changes before commit / PR (the right dock expanded).

```text
┌ Review · Split http · 14 files +482 −37 ── by file ▾  unified|split  ✦ Agent review ─ Done ┐
│ file-tree/changed      │ tallow-http/src/router.rs                         +120 −12       │
│ ✓ Cargo.toml   +3 −1   │ 88  −  pub fn route(&self, req: Request) -> Response {          │
│ ● router.rs  +120 −12  │ 88  +  pub fn route(&self, req: Request) -> Result<Response> {  │
│ ○ handler.rs  +40 −8   │       ┌ 💬 Tomas: keep the infallible variant for 0.x ─────────┐ │
│ ○ lib.rs       +6 −2   │       └ [Reply] [Resolve]                                       ┘ │
│ … 10 more              │       [Keep] [Undo] [Comment]                 hunk 2 of 9        │
├────────────────────────┴────────────────────────────────────────────────────────────────────┤
│ file-edit-chip/review-bar  3 of 14 viewed · 2 comments  [Send comments to agent] [Commit ▾]│
└────────────────────────────────────────────────────────────────────────────────────────────┘
```
Uses `diff-viewer/split` / `diff-viewer/unified` (Pro: `diff-viewer/review-pane`,
`diff-viewer/commit-filter`, `diff-viewer/minimap-heat`), `file-tree/changed`,
`file-edit-chip/review-bar` (Pro), [review-card](../specs/review-card.md) `review-card/inline-thread`.
Empty: "No changes yet". Large / binary files follow diff-viewer §2.5. Error: per-file "Couldn't
load diff · Retry".

### S6 Parallel agents / worktrees `shell/coding-parallel`
Purpose: one task fanned out to N sub-agents, each in its own worktree; or N attempts at one task to
pick a winner.

```text
┌ Split tallow-http into router + handlers · 4 agents · 6:10 ─────── [Stop all] [Merge done ▾] ┐
│ subagent-tree/rail            │ fanout-grid/columns                                           │
│ ◆ Larch agent  waiting on 4   │ ┌ api-split ──────┐┌ worker-split ──┐┌ docs ───────┐┌ tests ──┐ │
│ ├ ◆ api-split   ⠋ 3:02        │ │ ▣ wt-api       ││ ▣ wt-worker    ││ ▣ wt-docs   ││ ▣ wt-tst│ │
│ ├ ◆ worker-split ⚠ needs you  │ │ ⠋ Editing      ││ ⚠ Approval     ││ ✓ Done 2:10 ││ ✕ Failed│ │
│ ├ ◆ docs        ✓ 2:10        │ │ router.rs      ││ ▲ cargo add …  ││ +40 −6      ││ 2 tests │ │
│ └ ◆ tests       ✕ failed      │ │ +210 −30       ││ [Open]         ││ [Review]    ││ [Retry] │ │
│                               │ └────────────────┘└────────────────┘└─────────────┘└─────────┘ │
│ Worktrees: 4 of 15 · 1.8 GB   │ trace-view/waterfall (Pro) · run-meter/footer-line 212k tok   │
└───────────────────────────────┴───────────────────────────────────────────────────────────────┘
```
Uses [subagent-tree](../specs/subagent-tree.md) `subagent-tree/rail`, [fanout-grid](../specs/fanout-grid.md)
`fanout-grid/columns` (attempt mode: `fanout-grid/inbox-board`; Pro `fanout-grid/judge-bracket`,
`fanout-grid/diff-tabs`), `git-chips/pill` + worktree badge (Pro `git-chips/worktree-stack`,
`git-chips/agent-owner`), [acting-indicator](../specs/acting-indicator.md) `acting-indicator/chip`,
[trace-view](../specs/trace-view.md) `trace-view/waterfall`, `run-meter/footer-line`.
Agent identity tones: `color.agent.*` (8 tones; labels always carry the name too).

| State | Shows |
|---|---|
| Empty | fanout-grid `Empty` ("No attempts") + "Split this task" starter |
| Dispatching | lanes stagger in (`stagger.base`), each `Pending` hollow dot |
| Error | lane `Failed` with Retry; grid keeps running; `BudgetReached` banner pauses all lanes |

### S7 Pull request `shell/coding-pr`
```text
┌ ⑂ fix/retry-flake → main ──────────────────────────────────────────────── Open ● ┐
│ Fix flaky retry_backoff test                                           #212 ↗    │
│ +58 −5 · 3 files · 2 commits · updated 1m ago                                    │
│ Checks ▮▮▮▯ 3 of 4                                                                │
│   ✓ Tests 214 passed 1m 12s   ✓ Lint 18s   ✕ Types 1 error 41s   ✓ Preview       │
│ Review ◐ 0 of 1 approvals · Tomas requested                                       │
│ [Merge ▾] disabled · "Types failing"        ✦ Fix with agent   ⋯                 │
└──────────────────────────────────────────────────────────────────────────────────┘
```
Uses [pr-card](../specs/pr-card.md) `pr-card/card` (Pro `pr-card/auto-fix-loop`,
`pr-card/merge-when-ready`, `pr-card/stack`), `git-chips/pill`, [review-card](../specs/review-card.md)
`review-card/verdict-chip` (Pro). Opening a PR is a `Network` request (Medium) through approvals.
Empty: "No PR yet · Create PR" in the thread header. Error: "GitHub didn't respond · Retry"; a merge
that fails keeps the card at its last state with the reason.

### S8 Merge conflict `shell/coding-merge`
[merge-conflict](../specs/merge-conflict.md) `merge-conflict/three-pane` (desktop),
`merge-conflict/inline-stacked` (narrow), `merge-conflict/tui` (Pro, terminal) — wireframe in that
spec §2, branch names from fixtures. Kit additions: header "✦ Resolve all with agent" runs a sub-turn
whose suggestions land as `color.merge.suggested` regions with a rationale; the user accepts per
conflict; "Complete merge" stays disabled until zero markers. Pro: `merge-conflict/test-gated` (run
tests before completing), `merge-conflict/lockfile-regen` (`Cargo.lock` regenerated, not merged).
Empty: "No conflicts". Error: "Merge aborted · working tree restored".

### S9 History & checkpoints `shell/coding-history`
```text
┌ History · Retry flake ──────────────────────────────── [Restore ▾] [Fork from here] ┐
│ ● now      checkpoint 5  "Inject clock into retry"   3 files  +58 −5                │
│ ◇ 4:12 pm  checkpoint 4  "Add tests/clock.rs"        1 file   +31                   │
│ ◇ 4:09 pm  checkpoint 3  "First fix attempt"         1 file   +18 −4   ← selected   │
│ ◇ 4:02 pm  checkpoint 1  thread start (main a1b2c3d)                                 │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ rollback-impact/sheet: restoring 3 undoes 2 files; tests/clock.rs will be deleted;  │
│ chat after turn 3 kept (Restore files) · removed (Rewind chat)    [Cancel] [Restore]│
└──────────────────────────────────────────────────────────────────────────────────────┘
```
Uses [checkpoint-timeline](../specs/checkpoint-timeline.md) `checkpoint-timeline/side-list`,
[rollback-impact](../specs/rollback-impact.md) `rollback-impact/sheet`,
[checkpoints-git](../specs/checkpoints-git.md) `snapshot-banner/bar` (read-only preview), Pro
[run-replay](../specs/run-replay.md) `run-replay/scrubber`. Actions: **Restore files** / **Rewind
chat** / **Restore files and chat** / **Fork from here** (names from checkpoint-timeline). Empty:
"No checkpoints yet · one is saved before each turn that writes". Error: `RestoreFailed (nothing
changed)` with Retry.

### S10 Settings · permissions `shell/coding-settings`
Opens over the work (overlay), sections Account / Agents / Project / Organisation.

```text
┌ Settings ────────────────────────────────────────────────────────────────────────── ✕ ┐
│ Account          │ Project · tallow › Permissions                                      │
│ Agents           │ Default level  [ReadOnly | Ask | AutoEdit | FullAccess | Bypass]   │
│ ▸ Models         │ Sandbox        ▣ workspace-write · network off       [Change]     │
│ Project          │ Rules                                          [+ Rule] [Test…]    │
│ ▸ Permissions    │  Allow  Command  cargo test *          project   used 41×         │
│ ▸ Worktrees      │  Allow  Edit     tallow-*/src/**       project   used 120×        │
│ ▸ Environments   │  Ask    Network  *.crates.io           user      used 3×          │
│ ▸ MCP & plugins  │  Deny   Command  git push --force *    org 🔒   policy             │
│ Organisation     │ Critical actions always ask (cannot be changed)          ⬣ ⓘ      │
│                  │ Audit log ▸  last 30 days · 212 decisions                          │
└──────────────────┴─────────────────────────────────────────────────────────────────────┘
```
Uses **new** `settings-panel` (nav + sections) · [permission-rules](../specs/permission-rules.md)
`permission-rules/table`, `permission-rules/editor`, `sandbox-badge/chip`, `audit-log/timeline` ·
[composer-permission-selector](../specs/composer-permission-selector.md) `permission-selector/segmented`
(tier ≥ 3 runs its danger confirm) · `model-picker/compact`. Worktrees section: root path, keep-last-N
(default 15), auto-delete on archive, snapshot before delete (Codex app behaviour, §10).
Empty rules: suggested rules from recent approvals (Pro `permission-rules/suggestions`). Error: save
failed keeps the old value and shows the field error. Policy-managed rows show 🔒 with a reason.

### S11 MCP & plugins `shell/coding-mcp`
[mcp-plugin-list](../specs/mcp-plugin-list.md) `mcp-list/rows` inside `settings-panel`; health chip
`mcp-list/chip` in the title bar; `mcp-list/tui` for S3; Pro `mcp-list/directory`,
`mcp-list/budget-bar`, `mcp-list/per-session`. Icons: [plugin-icons](../specs/plugin-icons.md)
`plugin-icon/monogram` (fixture servers are invented: `larch-issues`, `crate-docs`, `ci-logs`,
`staging-db`). Repo-declared servers (`.mcp.json`) arrive as "Added by repo — approve?" (pending
approval state). Skills / plugins tab lists the same rows with "renders its own pane" marked by
`side-panel/plugin-host` (Pro). Empty: "No servers · Add from directory". Error: row `Failed to
connect · 401` + Logs.

### S12 Onboarding / empty project `shell/coding-onboarding`
```text
┌ Welcome to Larch agent ──────────────────────────────────────────────────────────────┐
│ Open a project                                                                        │
│  [Open folder…]   [Clone repository…]   Recent: ~/src/tallow  ~/src/larch-site       │
│ Setup check                                                    [Re-check]             │
│  ✓ Git repository found · main                                                        │
│  ✓ Agent runtime ready                                                                │
│  ⚠ Sandbox not available on this OS version · run `larch sandbox install`  [Copy]     │
│  ○ Connect GitHub for PRs and checks (optional)                       [Connect]       │
│ Start with                                                                            │
│  [Fix a failing check]  [Explain this repo]  [Write tests for a file]  [Review a PR]  │
└───────────────────────────────────────────────────────────────────────────────────────┘
```
Uses **new** `setup-checklist` (Hoplite-style status + fix command + Re-check, §10) ·
`empty-state/starters-grid` · Pro `empty-state/capability-tour` · S3 uses `empty-state/terminal`.
Loading: each check shows `loader/glyph-dots` after `delay.loader.show`. Error: a failed check never
blocks opening the project; it only disables the features it gates, with the reason.

## 4. Agent flows

Each step names the `AgentState` ([../architecture.md](../architecture.md)), the component that shows
it, and what the user can steer. ⏸ = approval stop.

### F1 Fix a failing test, end to end (thread "Retry flake")
| # | AgentState | Shown by | User can |
|---|---|---|---|
| 1 | `Composing` | `composer/ide-panel`; `@tests/retry.rs` token chip | pick Worktree, model, effort, level `Ask` |
| 2 | `Submitted` | send-control "submitted" pulse (< 500 ms, no spinner) | stop |
| 3 | `Thinking{verb}` | `reasoning-block/inline-summary` + shimmer working line | steer, queue |
| 4 | `ToolRunning{cargo test}` | `tool-call/row` + `terminal-output/block`, tab `◆ ⠋ cargo test` | open terminal tab (read-only) |
| 5 | `ToolRunning{read}` ×3 | `tool-stack/worked-for` folds reads | — |
| 6 | `ToolRunning{edit}` | `file-edit-chip/row` +18 −4; right dock diff streams (`diff-viewer` follow) | Keep / Undo hunk while it works |
| 7 | `ToolRunning{cargo test}` | `terminal-output/block` → ✓ 214 passed | — |
| 8 | `Streaming` | `markdown/compact` summary; checkpoint marker "checkpoint 3" | — |
| 9 | `Done{elapsed}` | "Worked for 1m 12s"; review bar 3 files | send line comments → back to step 3 |
| 10 | `AwaitingApproval` ⏸ | `git push` = Command, Medium → `approval-card/footer` | Allow once / session / Always `git push origin fix/*` |
| 11 | `ToolRunning{open PR}` | `pr-card/card` appears, checks start | Fix with agent when a check fails |

### F2 Multi-agent refactor in worktrees ("Split http")
| # | AgentState | Shown by | User can |
|---|---|---|---|
| 1 | `Thinking` | `plan-card/document`: 4 steps, one per sub-agent | edit plan |
| 2 | `AwaitingApproval` ⏸ (Plan, Low) | `plan-approval/footer` with chosen run level | approve, approve with AutoEdit, comment |
| 3 | parent `Thinking` / children `Pending → Working` | `subagent-tree/rail` + `fanout-grid/columns`, one `▣` worktree each | focus a lane, stop a lane |
| 4 | child `AwaitingApproval` ⏸ (`cargo add`, Medium) | lane flips to NeedsYou; S4 row; status strip `AwaitingYou` | answer from lane or queue |
| 5 | child `Error` (tests lane) | lane `Failed` ✕, others continue | Retry lane, ask parent to recover |
| 6 | parent `Thinking` (`WaitingOnChildren` → Working) | tree rolls up 4/4 | — |
| 7 | parent `ToolRunning{merge worktrees}` | S5 review across lanes, `diff-viewer/origin-lanes` (Pro) | keep/undo per hunk |
| 8 | conflict → F3 | `git-chips` "⚠ merging · 3" | — |
| 9 | `Done` | per-agent cost in `run-meter/per-agent-stack` (Pro) | archive worktrees (snapshot kept) |

### F3 Merge conflict resolution
| # | AgentState | Shown by | User can |
|---|---|---|---|
| 1 | `ToolRunning{git merge main}` → `Error{Conflict}` | `git-chips` merging state; S4 Conflict row | open S8 |
| 2 | `Idle` (user drives) | `merge-conflict/three-pane`, file strip "5 conflicts in 3 files" | accept current / incoming / both per conflict |
| 3 | `Thinking` → `Streaming` | "Resolve all with agent": suggestions + rationale per conflict | accept / reject each suggestion |
| 4 | `ToolRunning{cargo test}` | Pro `merge-conflict/test-gated` | — |
| 5 | `AwaitingApproval` ⏸ | "Complete merge" is a user action; the commit is Low inside the worktree | Abort merge (restores tree) |
| 6 | `Done` | conflict count rolls to 0; PR card blocker row clears | — |

### F4 Risky command approval (Critical)
Thread "Deploy fix", level **Bypass**. Agent wants `tallow-cli deploy --env production`, then
`rm -rf ./target /var/lib/tallow/queue`.

| # | AgentState | Shown by | User can |
|---|---|---|---|
| 1 | `ToolRunning{deploy}` requested | classifier: deploy to production → **Critical**; Bypass does not auto-run Critical | — |
| 2 | `AwaitingApproval` ⏸ | `approval-card/footer` (Pro `approval-card/hold-ring`): ⬣ octagon, 2 px border, static hatch (`color.risk.critical-hatch`), reasons "Deploys to production", "Not reversible from here" | **hold 1200 ms** (`dur.approval.hold`) to Allow once; Deny; Deny with reason |
| 3 | — | `Enter` maps to Deny; shortcuts ignored for `delay.approval.input-guard`; no Always / Session options | type "use staging first" → `DenyWithReason` |
| 4 | `Steering` → `Thinking` | agent reads the reason; proposes `--env staging` (Medium) | allow once |
| 5 | `AwaitingApproval` ⏸ | `rm -rf` path outside workspace → Critical; long command shows "contains `rm -rf` on line 2" | deny; Edit and allow (`./target` only → re-classified Medium) |
| 6 | `Done` | audit entries (S10 audit log) record both decisions and latency | — |

### F5 Checkpoint restore
| # | AgentState | Shown by | User can |
|---|---|---|---|
| 1 | `Done` | user dislikes turn 4; hovers marker → `checkpoint-timeline/hover-revert` | Restore ▾ |
| 2 | `Idle` | `rollback-impact/sheet`: 2 files undone, 1 deleted, terminal effects not undoable (listed) | choose Restore files / Rewind chat / both / Fork |
| 3 | — | Restoring{n of m} progress; files in tree flash `dur.highlight.fade` | cancel before start |
| 4 | `Idle` | "You are here" marker; undo toast (UndoWindow) | Undo restore |
| 5 | `Composing` | composer prefilled with the rewound prompt (Rewind chat only) | edit and resend → new branch of history |
If the agent is running when Restore is pressed: the menu offers "Stop and restore" (run → `Cancelled`
first); restore never races a live write.

## 5. Approvals and risk

Tiers per [../specs/approvals.md](../specs/approvals.md) §2.3. Floors are this kit's *proposed*
defaults; apps can raise, never lower. **Critical always stops, even in Bypass**
(`bypass_includes_critical = false`).

| Action in a coding agent | Kind | Tier | Notes |
|---|---|---|---|
| Read / search files, list dir, `git status`, `git diff` | — | none (auto) | never asks, even in `ReadOnly` |
| Edit a file inside the workspace / worktree | `Edit` | Low | auto in `AutoEdit` and up |
| Run tests / build / lint inside the sandbox (`cargo test`, `cargo clippy`) | `Command` | Low | good "Always" candidate (`cargo test *`) |
| Other shell commands inside the sandbox | `Command` | Medium | |
| `git commit`, create branch, `git push` to a non-protected branch | `Command` | Medium | |
| Add / upgrade a dependency (`cargo add`, lockfile change) | `Command` | Medium | network to registry is its own request when off |
| Network GET to a new host; MCP read tool | `Network` / `McpTool` | Medium | |
| MCP write tool (create issue, comment on PR), open a PR | `McpTool` / `Network` | Medium | |
| Command outside the sandbox; edit outside workspace or in `.git/` | `Command` / `Edit` | High | arm 600 ms; Always limited to session |
| `git push --force` / `--force-with-lease`, rewrite published history | `Command` | High | |
| Use a credential (`CRATES_TOKEN`, deploy key) | `Credential` | High | name and scope only, value never shown |
| Non-GET to a new host; write to a staging database | `Network` / `McpTool` | High | |
| `rm -rf` outside the workspace, or on `~`, `/`, `.git` | `Command` | **Critical** | hold to allow; no Always |
| Deploy to production; publish a release (`cargo publish`, tag push to protected) | `Command` | **Critical** | |
| Push / force-push to `main` or any protected branch; delete a remote branch | `Command` | **Critical** | |
| Drop / truncate tables, run migrations on production | `McpTool` | **Critical** | |
| Spend over the app threshold (paid sandbox size, paid API) | `Spend` | High → Critical | |

| Permission level | What auto-runs in this kit |
|---|---|
| `ReadOnly` | reads only; every write is refused with reason, not asked |
| `Ask` (default for new projects) | reads; everything else asks |
| `AutoEdit` | + Low `Edit` |
| `FullAccess` | + Low / Medium / High inside the sandbox |
| `Bypass` | + up to High anywhere; **Critical still stops** |

Kit rules: sub-agent requests carry the sub-agent's identity tone and name; a fan-out never batches
High or Critical; approvals answered on the phone resolve the desktop card with "Answered on your
phone"; worktree archive deletes nothing without a snapshot.

## 6. New components

Five gaps found. Each needs its full 10-point spec ([../specs/README.md](../specs/README.md)) before it
is built.

| Proposed name | Family | Tier | Variants (proposed) |
|---|---|---|---|
| `thread-list` | J Workspace chrome | free core, Pro extras | free: `thread-list/grouped`, `thread-list/compact`, `thread-list/tui` · Pro: `thread-list/kanban`, `thread-list/by-machine`, `thread-list/activity-spark` |
| `thread-header` | J Workspace chrome | free | free: `thread-header/ide`, `thread-header/minimal`, `thread-header/tui` · Pro: `thread-header/handoff-flow` |
| `review-queue` | G/H (approvals + review) — proposed home H | free core, Pro extras | free: `review-queue/list`, `review-queue/compact`, `review-queue/tui` · Pro: `review-queue/triage-board`, `review-queue/swipe` (mobile, never for High/Critical) |
| `settings-panel` | J Workspace chrome | free | free: `settings-panel/overlay`, `settings-panel/page`, `settings-panel/tui` |
| `setup-checklist` | E (empty states) | free | free: `setup-checklist/list`, `setup-checklist/terminal` · Pro: `setup-checklist/guided` |

**`thread-list`.** The left dock of every coding shell: threads grouped by project (or by machine /
local vs cloud), each row with a status glyph, title, `+N −M`, worktree badge and age. Row states:
`Running` (braille loader, `loader/glyph-braille`), **`NeedsYou`** (amber, distinct from running —
`loop.waiting` tip, never a spinner), `Done` ✓, `Failed` ✕, `Stalled`, `Archived`, `Unread` dot.
Group headers NEEDS YOU / RUNNING / DONE are derived, not user-ordered. Hover: archive, pin, rename.
Keys: ↑/↓, Enter, ⇧⌫ archive, ⌘1–9. Snaps to the 48 px rail (status dots only) per
workspace-chrome. Reads `AgentState` per thread; reuses [subagent-tree](../specs/subagent-tree.md)
node glyphs and `color.state.*`. References: Zed threads sidebar, pi-gui / Paradev row states,
Codex projects → threads (§10).

**`thread-header`.** The thread's top bar: editable title, `git-chips/pill` with worktree badge,
environment switch (Local / Worktree / Cloud), **Hand off** (moves the thread between Local and
Worktree, with a confirm listing the git steps), Create branch here, Open in editor ▸, PR chip,
MCP health chip, context ring, overflow (export as Markdown, share). States include
`HandingOff{step}`, `HandoffFailed{reason}`, `Detached` (worktree deleted, snapshot available →
"Restore worktree"). Reference: Codex app Hand off / Create branch here (§10).

**`review-queue`.** Cross-thread inbox of items waiting on the user, each item a typed reference
(`Approval`, `Diff`, `Finding`, `Check`, `Conflict`, `Question`) rendered by its owning component in a
preview pane. Sorted by risk tier then age; rows show the tier glyph, thread, one-line summary, age,
primary action. j/k move, Enter opens, `Y`/`N` only when the preview card has focus (approvals input
guard applies). Count feeds the tray / island badges ([desktop-overlays](../specs/desktop-overlays.md)
`tray-popover/approvals-only`). Differs from `approval-card/inbox` (Pro, approvals only) and
`fanout-grid/inbox-board` (one run's lanes): it spans threads and item kinds.

**`settings-panel`.** A settings frame: section nav (Account / Agents / Project / Organisation), a
breadcrumb, sections of labelled rows, per-row scope tag (user / project / org 🔒), dirty state with
Save / Revert or instant-apply per row, search over setting names, "Changed from default" dot.
Opens as an overlay over the work (Hoplite idea) or as a page (web). Hosts permission-rules,
mcp-list and model settings unchanged.

**`setup-checklist`.** Environment readiness list: each check has `Pending` / `Checking` / `Ok` /
`Warning` / `Failed` / `Skipped`, a one-line reason, the fix command with Copy, and a per-check or
global Re-check. Never blocks the project; failing checks disable only what they gate. Reference:
Hoplite "Settings › Local agents" status + Terminal command + Re-check (note 10).

Not new (covered): worktree list → `git-chips` + `thread-list` rows; destructive-command detection →
`risk-badge` + the app's classifier; test summary → `terminal-output/test-ticks` (Pro); file peek →
`code-block/file`.

## 7. Data model and fixtures

```rust
pub struct Project { pub id: ProjectId, pub name: SharedString, pub repo: Repo, pub default_level: LevelId,
                     pub sandbox: SandboxStatus, pub mcp: Vec<McpServerRef>, pub worktree_root: PathBuf,
                     pub keep_worktrees: u8 /* default 15 */ }
pub struct Repo { pub slug: SharedString /* "larch-labs/tallow" */, pub default_branch: SharedString,
                  pub remote: Option<RemoteRef>, pub protected: Vec<BranchPattern> }
pub struct Thread {
    pub id: ThreadId, pub project: ProjectId, pub title: SharedString,
    pub env: ThreadEnv,                 // Local | Worktree(WorktreeId) | Cloud { machine: SharedString }
    pub branch: GitRef,                 // shared with git-chips
    pub state: AgentState,              // drives thread-list row, status strip, title glyph
    pub approvals: ApprovalQueue,       // approvals.md §9
    pub run_log: RunLog,                // tools.md
    pub checkpoints: Vec<CheckpointRef>,// checkpoint-timeline
    pub changes: ChangeSet,             // files + hunks, diff-viewer
    pub pr: Option<PullRequestRef>,     // pr-card
    pub subagents: Option<RunGraph>,    // subagent-tree / fanout-grid
    pub terminals: Vec<TerminalSessionRef>, // terminal-tabs
    pub pinned: bool, pub archived: bool, pub unread: bool, pub updated_at: SystemTime,
}
pub struct Worktree { pub id: WorktreeId, pub name: SharedString /* "wt-retry" */, pub path: PathBuf,
                      pub base: GitRef, pub detached_head: bool, pub disk_bytes: u64,
                      pub snapshot_on_delete: bool }
pub enum ReviewItem {                   // review-queue
    Approval(ApprovalId), Diff { thread: ThreadId, files: u32, viewed: u32 },
    Finding { thread: ThreadId, review: ReviewId }, Check { pr: PullRequestRef, check: CheckId },
    Conflict { thread: ThreadId, count: u32 }, Question(ApprovalId),
}
pub struct SetupCheck { pub id: SharedString, pub label: SharedString, pub status: CheckStatus,
                        pub reason: Option<SharedString>, pub fix_command: Option<SharedString>,
                        pub gates: Vec<FeatureId> }
```
Record types already owned elsewhere (reuse, don't redefine): `ApprovalRequest` (approvals),
`PermissionRule` (permission-rules), `McpServer` (mcp-plugin-list), `Checkpoint` (checkpoints-git),
`PullRequest` / `CheckRun` (pr-card), `ReviewFinding` (review-card), `Conflict` (merge-conflict).

**Fixture plan** (`fixtures/coding/*.json`, all invented):
| Fixture | Contents |
|---|---|
| `project-tallow` | repo `larch-labs/tallow`, 4 crates, protected `main` and `release/*`, sandbox `workspace-write`, network off |
| `threads` | 7 threads: Retry flake (running), Split http (fan-out, needs you), Deploy fix (Critical pending), Docs typo (done, merged), Bench refresh (question), Bench run (failed), Onboard console (archived) |
| `run-retry-flake` | full event log for F1 (≈ 40 events, timestamps for replay at 1×, 4×, 0.25×) |
| `run-split-http` | RunGraph with 4 sub-agents, 1 failure, 1 approval, 3 conflicts (F2, F3) |
| `run-deploy-fix` | two Critical requests + one edited-and-allowed (F4) |
| `checkpoints-retry` | 5 checkpoints, impact for restoring 3 (F5) |
| `pr-212` | 4 checks (1 failing Types), 1 requested reviewer, 2 commits |
| `review-split-http` | 6 findings (1 critical, 2 warnings, 1 suggestion, 2 nits), 2 line comments |
| `mcp-servers` | `larch-issues` (connected), `crate-docs` (needs auth), `ci-logs` (reconnecting), `staging-db` (pending repo approval), 1 org-blocked |
| `rules` | 6 rules incl. one org-locked deny, audit log of 212 entries |
| `stress` | 200 threads, 1,200-file diff, 40 sub-agents, 120-char branch names, RTL thread titles |

## 8. Variants

Shell = layout × surface × motion. Motion levels per [../motion.md](../motion.md) (`none` = reduced
motion forms; `subtle` default; `expressive` adds Pro loaders and border beams).

| Shell variant | Layout | Surfaces | Tier |
|---|---|---|---|
| `shell/coding-workspace` | three docks (§2.1) | desktop, web | free |
| `shell/coding-workspace-focus` | transcript only; docks as peeks (`panel-split/focus-mode`) | desktop, web | Pro |
| `shell/coding-terminal` | single column TUI (S3) | terminal, desktop (terminal theme) | free |
| `shell/coding-mobile` | one pane + sheets (§2.4) | mobile | free |
| `shell/coding-review-queue` | list + preview (S4) | desktop, web, mobile | free |
| `shell/coding-mission-control` | dashboard of all threads + fan-outs (`fanout-grid/mission-control`) | desktop, web | Pro |
| `shell/coding-split-agents` | two threads side by side (`terminal-tabs/split-grid` idea) | desktop | Pro |

| Screen | Free variant set | Pro additions |
|---|---|---|
| S1 dashboard | cards above | `run-meter/sparkline-session`, `status-bar/agent-roster`, `background-jobs/scheduled` |
| S2 workspace | baseline rows above | `tool-call/border-beam`, `border-beam/composer-light`, `diff-viewer/follow-agent`, `checkpoint-timeline/diff-peek` |
| S4 review queue | `review-queue/list` | `review-queue/triage-board`, `approval-card/reviewer` |
| S5 review | `diff-viewer/unified`, `diff-viewer/split` | `diff-viewer/review-pane`, `diff-viewer/walkthrough`, `review-card/heat-gutter` |
| S6 parallel | `fanout-grid/columns`, `subagent-tree/rail` | `fanout-grid/judge-bracket`, `subagent-tree/canvas`, `trace-view/swimlanes-by-agent` |
| S7 PR | `pr-card/card`, `pr-card/compact` | `pr-card/auto-fix-loop`, `pr-card/merge-when-ready`, `pr-card/stack` |
| S8 merge | `merge-conflict/three-pane`, `merge-conflict/inline-stacked` | `merge-conflict/test-gated`, `merge-conflict/rebase-stepper`, `merge-conflict/tui` |
| S9 history | `checkpoint-timeline/side-list`, `rollback-impact/sheet` | `checkpoint-timeline/branch-rail`, `run-replay/scrubber`, `rollback-impact/blast-radius` |
| S10–S11 | `permission-rules/table`, `mcp-list/rows` | `permission-rules/simulator`, `mcp-list/directory`, `mcp-list/budget-bar` |
| S12 | `setup-checklist/list`, `empty-state/starters-grid` | `setup-checklist/guided`, `empty-state/capability-tour` |

Themes: dark default; light; high-contrast dark; "terminal" (ANSI palette only). Density: `compact`
for desktop / IDE, `comfortable` for web and mobile ([../tokens.md](../tokens.md) §7.3).

## 9. Accessibility and edge cases

| Case | Behaviour |
|---|---|
| Keyboard only | F6 cycles thread list → transcript → composer → right dock → terminal → status bar; ⌘K palette reaches every screen; ⌘/Ctrl+Shift+A jumps to the oldest open approval; diff hunks `[` / `]`, `y` keep, `n` undo when the diff has focus |
| Screen readers | thread rows read "Retry flake, running, 42 seconds, 3 files changed"; NeedsYou rows lead with "Needs you"; approvals announce assertively once (approvals.md §8), later arrivals politely; terminal output is not a live region (only exit summaries are announced) |
| Colour | state, risk and diff never rely on colour: glyph + label + border weight; diff uses `+`/`−` gutters and `color.diff.*` pairs that pass [../tokens.md](../tokens.md) §8 |
| Dense data | 1,200-file diff virtualises and shows file list first; thread list virtualises past 100 rows; transcript folds tool runs into `tool-stack/worked-for` |
| Long names | branch names truncate per git-chips (first + last segment); paths middle-truncate; full value in tooltip and accessible name |
| RTL | chrome mirrors; code, diffs, commands, paths and branch names stay LTR in isolated spans; docks swap sides |
| Very fast agent | rows faster than `delay.row-guard` never flash; loaders respect `delay.loader.show` and `dur.loader.min-visible`; a run finishing < 1 s shows only "Worked for 0.8s" |
| Very slow / long runs | elapsed switches to `h m`; `Stalled` after the stall threshold; OS notification on done and on needs-approval; window unfocused → shared clock slows ([../motion.md](../motion.md) §9) |
| Many agents | 40 sub-agents: `fanout-grid/cells` density, approvals paged "1 of 12 · 4 agents", no batch for High/Critical |
| Offline / remote dropped | approvals disabled (fail safe), composer queues locally, terminal shows last output with "Disconnected" banner |
| Worktree deleted | thread stays in history, header shows "Worktree removed · Restore from snapshot" |
| Touch | 44 px targets (`size.touch-target`); no swipe-to-approve for High/Critical; long-press replaces hover actions |
| Reduced motion | all shimmers, beams and loaders use their static forms; approval arming and hold durations are kept (safety, not decoration) |

## 10. References

Licence rules: [../licensing.md](../licensing.md). Opened this session (2026-11-12) are marked †; the
rest come from the research notes named.

| Product / source | What we take | Licence note |
|---|---|---|
| Codex app worktrees docs † https://developers.openai.com/codex/app/worktrees | Local vs Worktree under the composer, **Hand off** between them, "Create branch here", detached HEAD worktrees, keep last 15 managed worktrees, snapshot before delete + restore | Docs of a proprietary app — study only |
| Codex CLI (note 03, 08) | numbered approval options with keys, width-adaptive footer, `• Working (12s · esc to interrupt)`, `↳` queue | MIT / Apache-2.0 (licensing.md vs note 08) — portable with attribution; re-check |
| Zed agent panel docs † https://zed.dev/docs/ai/agent-panel | threads sidebar grouped by project, archive on hover / ⇧⌫, worktree picker in the title bar, queue + Steer, follow-agent crosshair, Review Changes multibuffer with keep/reject per hunk, terminal threads, notify when waiting / done | Zed mixed GPL / Apache by crate — study only |
| Conductor diff viewer docs † https://www.conductor.build/docs/reference/diff-viewer | changed-file list, unified view, filter by commit, line comments sent to the agent, suggested PR actions ("Create PR") | Proprietary — study only |
| Claude Code desktop (note 03) | session rail with Context card, diff line comments, PR card Auto-fix / Merge when ready, minimise pane to corner tile | Proprietary — reference only |
| Cursor (note 03) | checkpoints in the timeline (files only, messages kept), queue drag-reorder, Agents Window | Proprietary — study only |
| OpenCode (notes 03, 08) | `/undo` `/redo` git-backed, sidebar with context %, cost, MCP status, plugins with their own sidebar | MIT — portable with attribution (not opened this session) |
| Superset → Paradev (note 03) | rows with ⠋ + `+N −M` grouped by machine, Files / Changes / Review tabs, agent launcher | licence not recorded in our notes — study only |
| pi-gui (note 03) | running / finished / **needs you** row states | licence not recorded — study only |
| Hoplite (note 10) | thread tabs, Settings opened over the work, Local agents status + fix command + Re-check, keyboard-first ⌘K | Proprietary — reference only |
| affectUI Fan-out Grid, Review Center, Agent Inbox (note 11) | fan-out cells, review centre, inbox grouping | Paid — study only |
| Arc UI MCP servers / Agent run blocks (note 12) | MCP token budget + per-tool switches; run block | Paid — study only |
| signoff-ui DiffReview + ApprovalCard (note 16) | per-hunk review, once / session / always with risk | MIT — portable with attribution after checking the repo licence file |
| Beautiful UI (notes 16, 17) | paged approvals "1 / 3"; Thinking "Coding" variant | MIT — portable with attribution |

**Open questions**
1. `review-queue`: family H (review) or G (approvals)? It renders both; proposed H with a G hand-off.
2. Should `thread-list` and `subagent-tree` share one row component (status glyph + title + meta)?
   Proposed yes, as a shared `agent-row` primitive inside family F.
3. Classifier for Critical commands (`rm -rf`, deploy, protected push) is the app's; do we ship a
   default pattern list in `oxyn-core` for the demo only?
4. Hand off between Local and Worktree: is it an approval (`Command`, Medium) or a plain confirm? It
   runs git operations on the user's checkout; proposed Medium.
5. Cloud sandboxes (env `Cloud`) need a machine chip and cost; reuse `run-meter/credits` or a new
   `sandbox-badge` variant?
6. Mobile editing: kit says review / approve / steer only. Confirm no inline editing on phones.
7. Fixture names (Larch Labs, Tallow, `ridge-1`) need a quick trademark sanity check before the
   public gallery.
