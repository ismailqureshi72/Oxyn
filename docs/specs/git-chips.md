**Status:** draft v0.1 (2026-11-11).

# Spec: branch and worktree chips (`git-chips`)

Part of family H, "Checkpoints, diff & git" (overview in [checkpoints-git.md](checkpoints-git.md)).
Same depth as [composer-send-control.md](composer-send-control.md). Target **3 free + 11 Pro = 14
variants**. Owns the compact **"where is this agent working"** label: branch name, ahead / behind,
dirty dot, worktree badge, switching and detached states. Merge/rebase-in-progress states hand off to
[merge-conflict.md](merge-conflict.md); the PR attached to a branch is [pr-card.md](pr-card.md).

## 1. Purpose
In coding-agent apps every task runs somewhere: the main checkout, a branch, or **a worktree per task**
— "universal in parallel-agent ADEs" (Codex app Local / Worktree, Claude desktop Git isolation,
Conductor, Verdent workspaces, Paradev, Emdash, Zed worktree picker, Cursor Agents Window, Copilot
worktree sessions; research note 03 §1F). agentbox: "each agent gets its own git worktree" [R027].
Users need to answer at a glance: **which branch, is it in sync, are there uncommitted changes, is this
isolated from my checkout, and is a switch still happening**. Verdent's workspace bar shows workspace
name, branch and Rebase / Sync with main; Conductor copies the branch name in one click (note 03);
Hoplite puts a "main" branch chip above the composer next to project, sandbox size and model (note 10).

Where it appears: the composer's top metadata row ([composer.md](composer.md)), fan-out lane headers
([fanout-grid.md](fanout-grid.md), "A · Opus ── worktree a1"), thread / sidebar rows, the PR card
header, merge-editor pane headers, the status bar (family J).

## 2. Anatomy and formats
```
 pill            [⑂ feature/auth-refresh  ↑2 ↓1 ●]          ⑂ branch glyph · name · ahead/behind · dirty dot
 + worktree      [▣ wt-a1 │ ⑂ feat/…/auth-refresh ↑2 ●]      worktree badge joined to the branch pill
 detached        [◇ HEAD a1b2c3d  detached]
 switching       [⑂ main → feature/auth…  ◌]                 old name crossfades out, loader in the dirty slot
 merging         [⑂ main  ⚠ merging · 3]                      hands off to merge-conflict
 terminal        main ↑2↓1 *                                 shell-prompt style
```
**Formats (proposed):**
| Part | Rule |
|---|---|
| Branch name | Max 24 ch in `pill` (≈ 180 px at 12 px mono); truncation keeps the **first path segment and the last segment**: `feature/team/auth-refresh-tokens` → `feature/…/auth-refresh-tokens`; if the last segment alone is too long, middle-truncate it keeping a leading ticket id (`ENG-482-…-tokens`). Full name in tooltip and accessible name |
| Ahead / behind | `↑2 ↓1` relative to the upstream (or to the base branch when the app sets `compare_to = Base`); a zero side is omitted (`↑2`); both zero → nothing (no "in sync" noise); ≥ 1000 → `↑999+` |
| No upstream | `☁︎ not published` dim tag instead of arrows |
| Upstream gone | `upstream gone` warning tag |
| Dirty dot | 6 px dot, `color.git.dirty` (amber-neutral); tooltip "3 files changed, 1 untracked"; Pro `count` shows `●3` |
| Worktree badge | `▣` + short worktree name (≤ 12 ch, end-truncated) joined to the branch pill with a 1 px divider; "main checkout" shows no badge |
| Detached | `◇ HEAD a1b2c3d` (7-char sha) + `detached` tag; tag name if HEAD is at one (`◇ v1.4.2`) |
| Sha | monospace, 7 chars (proposed), full sha in tooltip |

## 3. State machine
```
  Unknown (no repo info yet) ──status loaded──▶ OnBranch{name, ahead, behind, dirty}
        │                                        │  ▲
        └─ not a git repo ─▶ NoRepo (hidden)     │  └── fetch done / status refresh (counts roll)
                                                 ├── Fetching (arrows slot shows a loader) ──▶ OnBranch | FetchFailed{reason} ─▶ OnBranch (stale tag)
                                                 ├── switch requested ─▶ Switching{from, to} ──ok──▶ OnBranch{to}
                                                 │                                └─ blocked by dirty tree ─▶ SwitchBlocked{files} ─ stash/commit/cancel
                                                 │                                └─ error ─▶ SwitchFailed{reason} ─▶ OnBranch{from}
                                                 ├── checkout of a commit ─▶ Detached{sha}
                                                 ├── merge / rebase / cherry-pick starts ─▶ InOperation{kind, conflicts} ─done─▶ OnBranch
                                                 ├── worktree being created ─▶ CreatingWorktree ─▶ OnBranch + badge
                                                 ├── worktree removed / locked / missing ─▶ WorktreeGone | WorktreeLocked
                                                 └── offline / sandbox asleep ─▶ Offline (last known, dimmed, "as of 2m")
  Presentation: Rest · Hovered · Focused · Pressed · MenuOpen (picker) · Copied (1.2 s)
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard (focused) |
|---|---|---|---|---|
| `Unknown` | skeleton pill 96 × 20 px (static shimmer per [skeletons.md](skeletons.md)) | — | — | not focusable |
| `OnBranch` clean | `⑂ main` (+ arrows if any) | open branch picker (if `on_switch` set) else copy name | copy name, open PR | Enter picker, C copy |
| `OnBranch` dirty | + dirty dot | as above | "Review changes" (diff-viewer) | D review changes |
| Ahead only `↑2` | arrows in `color.git.ahead` (neutral fg) | Push (if `on_sync` supplied) | — | P push |
| Behind `↓1` | `↓` in `color.git.behind` (amber when ≥ `behind_warn` = 10, proposed) | Sync (pull / rebase per app) | — | S sync |
| `Fetching` | loader glyph ([loader-glyph.md](loader-glyph.md) braille, 12 px) in the arrows slot | — | — | — |
| `FetchFailed` | arrows dimmed + `⚠` small, tooltip reason | Retry fetch | — | R retry |
| `Switching{from,to}` | "main → feature/auth…" with loader in the dot slot; pill width tweens | — | Cancel (if supported) | Esc cancel |
| `SwitchBlocked{files}` | popover: "3 uncommitted files would be overwritten" + Stash and switch / Commit first / Cancel | Stash and switch | Cancel | Enter / Esc |
| `SwitchFailed` | danger tint 2 s, toast with reason; returns to `from` | Retry | Copy error | — |
| `Detached{sha}` | `◇ HEAD a1b2c3d detached` in warning-neutral | "Create branch here" | Switch to branch | Enter |
| `InOperation{kind,n}` | `⚠ merging · 3` / `rebasing 3/7` amber | Open merge editor ([merge-conflict.md](merge-conflict.md)) | Abort (goes to merge editor confirm) | Enter opens |
| `CreatingWorktree` | `▣ creating…` badge with loader | — | — | — |
| `WorktreeLocked` | `▣ 🔒 wt-a1`, tooltip "Locked: in use by another session" | — | — | — |
| `WorktreeGone` | badge struck, "worktree removed" | Recreate (if supported) | Dismiss | — |
| `Offline` | last-known pill dimmed 60 %, "as of 2m" in tooltip; arrows hidden | — | — | — |
| `Copied` | name crossfades to "Copied" (same width cell) for 1.2 s | — | — | — |
| Disabled-with-reason | e.g. picker unavailable during a run: "Can't switch while the agent is editing" | — | — | focusable, reason read |

## 5. Micro-interactions
- **Counts.** `↑2 → ↑3` rolls (`dur.digit.roll` 280 ms); an arrow appearing slides in from its
  direction (↑ from below, ↓ from above, 4 px, 160 ms) and widens the pill with `spring.snap`.
- **Dirty dot.** Appears with scale 0 → 1 (`spring.snap`); a new change while already dirty pulses the
  dot once (opacity 1 → 0.5 → 1, 400 ms). Goes away with a 160 ms fade on commit / stash.
- **Switching morph.** Old name slides up and fades (160 ms), arrow `→` and new name slide in from
  below; on success the `from` part collapses and the pill settles to the new width (220 ms
  `spring.snap`). Pro `split-flap` rolls letters instead.
- **Copy.** Press → "Copied" crossfade in the same fixed-width cell (Conductor one-click copy, note 03),
  holds 1.2 s, crossfades back.
- **Worktree badge join.** Created worktree: badge grows out of the pill's inline-start edge (width 0 →
  auto, 220 ms) with the divider drawn last.
- **Detached.** Branch glyph morphs ⑂ → ◇ (160 ms `spring.snap`) and the tint shifts over 300 ms.
- **In operation.** `⚠` pops in; the conflict count rolls down as [merge-conflict.md](merge-conflict.md)
  resolves; on completion the operation tag collapses (200 ms) and arrows return.
- **Hover / press.** Background tint 80 ms; press scale 0.97 for 80 ms; tooltip after 400 ms.
- **Picker.** Popover from the chip (`dur.base` 180 ms, y −4 → 0); list of branches with their own
  mini chips; type-to-filter.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Digit roll (ahead / behind / conflicts) | 280 ms | 0 | `ease.out` | 30 ms right → left | 30 Hz |
| Arrow enter + pill widen | 160 ms + 200 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Dirty dot in / pulse / out | 160 ms / 400 ms / 160 ms | 0 | `spring.snap` / `ease.in-out` / `ease.in` | — | 60 Hz |
| Switching slide out / in | 160 ms / 180 ms | in after 40 ms | `ease.in` / `ease.out` | — | 60 Hz |
| Switch settle (width) | 220 ms | after success | `spring.snap` | — | 60 Hz |
| Fetch loader | 80 ms per braille frame | 150 ms guard | — | — | 12.5 Hz |
| Copy label swap / hold | 120 ms / 1200 ms | 0 | `ease.out` | — | 60 Hz |
| Worktree badge grow | 220 ms | 0 | `spring.snap` | — | 60 Hz |
| Detached glyph morph / tint | 160 ms / 300 ms | 0 | `spring.snap` / linear in OKLab | — | 60 Hz |
| Failure tint hold | 2000 ms then 240 ms fade | 0 | `ease.in` | — | 30 Hz |
| Picker open / close | 180 ms / 120 ms | 0 | `ease.out` / `ease.in` | rows 20 ms cap 8 | 60 Hz |
| Hover / press | 80 ms | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | numbers swap without rolling; arrows and dots appear with an 80 ms fade; width changes instantly; switching shows "main → feature/auth…" statically then swaps; fetch loader replaced by static "…"; no pulses | — | linear | none | event-driven |

Proposed tokens: `color.git.branch` (fg), `color.git.ahead` (fg), `color.git.behind` (fg → amber at
`behind_warn`), `color.git.dirty`, `color.git.detached` (warning at low chroma), `color.git.worktree`
(accent at low chroma; per-lane colours in `lane-color`).

## 7. Variants

### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `git-chips/pill` | Rounded pill: glyph, truncated name, ahead/behind, dirty dot; optional joined worktree badge; picker on click | all GUI |
| `git-chips/inline` | Borderless text for status bars and lane headers: `⑂ main ↑2 ↓1 ●` at body size | ide, desktop, chat headers |
| `git-chips/workspace-bar` | Full-width bar: worktree name, branch, ahead/behind vs base, dirty count, Sync / Push / Copy actions (Verdent-style workspace bar, note 03) | desktop, ide |

### Pro (11)
| Name | One-line idea |
|---|---|
| `git-chips/prompt` | Shell-prompt look for terminals: `main ↑2↓1 *`, colours from the ANSI palette |
| `git-chips/split-flap` | Branch name changes letter by letter on a split-flap when switching |
| `git-chips/sync-ring` | A tiny ring around the glyph: filled clockwise by ahead, counter-clockwise by behind |
| `git-chips/mini-rail` | Two short branch lines diverging from a base dot, lengths scaled by ahead / behind |
| `git-chips/worktree-stack` | Many worktrees as stacked badges that fan out on hover, each with its branch and state |
| `git-chips/lane-color` | Each worktree gets a stable hue that also tints its fan-out lane and sidebar row ([fanout-grid.md](fanout-grid.md)) |
| `git-chips/agent-owner` | Avatar of the agent / model working in the worktree pinned to the badge |
| `git-chips/local-cloud` | Segmented `Local · Worktree · Cloud` selector joined to the chip (Codex Local / Worktree idea, note 03) |
| `git-chips/dirty-count` | Dirty dot replaced by `●3` with a hover list of changed files and file-edit counts |
| `git-chips/copy-burst` | Copy confirmation as a short tick burst around the chip |
| `git-chips/pixel` | Pixel-glyph branch mark that "grows" a pixel per ahead commit (agentbox pixel-grid idea [R027]) |

## 8. Edge cases
- **Very long names** (80+ ch): §2 truncation; `workspace-bar` may wrap to two lines only in `mobile`.
- **Names with slashes only** (`a/b/c/d`): keep `a/…/d`. **Unicode / emoji names:** truncation counts
  grapheme clusters, never splits one.
- **RTL.** Chip mirrors; branch names and shas render in LTR bidi isolates; arrows keep their meaning
  (↑ ahead, ↓ behind) and do not mirror.
- **Tiny sizes (16–20 px).** Glyph + dirty dot only, name in tooltip; 12–16 px: glyph only, coloured by
  worst state (amber operation > detached > behind > dirty > clean).
- **Dense IDE.** `inline` at 11 px; arrows use 10 px glyphs; actions in a context menu.
- **Touch.** 44 px target; long-press = copy name; picker as a bottom sheet.
- **Many worktrees** (20+ lanes): `lane-color` hues cycle a 12-hue set (proposed) and add a 2-char
  suffix to tell repeats apart.
- **Fast agents** switching branches several times per second: switches inside 150 ms
  (`delay.row_guard`) show the final name only; the morph never queues more than one transition.
- **Slow fetches** (> 10 s): loader keeps running; tooltip shows "Fetching for 12s"; after 60 s
  (proposed) → `FetchFailed{timeout}`.
- **Sandbox asleep / offline.** `Offline` with last-known values; never shows stale arrows as live.
- **Shallow clone / no merge base.** Arrows show `?` with tooltip "History too shallow to compare".
- **Bare repo or no repo.** Chip hidden; host can show "No git repository" in its place.
- **Protected / default branch while agent edits** (app-supplied flag): pill gets a small shield and
  tooltip "Agent is editing main directly" — policy stays in family G.

## 9. Accessibility
- **Accessible name:** "Branch feature/auth-refresh, 2 commits ahead, 1 behind, uncommitted changes,
  worktree wt-a1". Detached: "Detached HEAD at commit a1b2c3d". Switching: "Switching from main to
  feature/auth-refresh". Operation: "Merge in progress, 3 conflicts".
- **Role.** `button` with `aria-haspopup="listbox"` when it opens the picker; otherwise a `button` that
  copies ("Copy branch name").
- **Announcements (polite):** switch finished ("Now on feature/auth-refresh"), switch failed (assertive),
  "Copied branch name", entering detached HEAD. Count changes are **not** announced.
- **Keyboard:** Enter picker; C copy; D review changes; P push; S sync; R retry fetch; Esc closes
  picker. Shortcuts only when the chip is focused.
- **Contrast.** Name ≥ 4.5:1; arrows and dot ≥ 3:1 (dot also described in text); amber behind/detached
  pass in both themes.
- **Focus.** Ring follows the pill; joined badge + pill are one focus stop with the badge read first.
- **Reduced motion.** §6 last row.

## 10. Props sketch
```rust
pub struct GitLocation {
    pub branch: BranchState,              // OnBranch { name } | Detached { sha, tag: Option<String> } | Unknown | NoRepo
    pub upstream: Upstream,               // Tracking { ahead: u32, behind: u32 } | NotPublished | Gone | Unknown
    pub compare_to: CompareTo,            // Upstream (default) | Base { name }
    pub dirty: Option<DirtySummary>,      // { changed: u32, untracked: u32 }
    pub worktree: Option<Worktree>,       // { name, path, locked: bool, gone: bool }
    pub operation: Option<GitOperation>,  // Merging { conflicts } | Rebasing { step, total, conflicts } | CherryPicking | Reverting
    pub activity: GitActivity,            // Idle | Fetching | Switching { from, to } | SwitchBlocked { files } | CreatingWorktree
                                          // | SwitchFailed { reason } | FetchFailed { reason }
    pub as_of: Option<Instant>,           // for Offline display
}

pub struct GitChip<'a> {
    pub loc: &'a GitLocation,
    pub agent: &'a AgentState,            // disables switching while ToolRunning edits files
    pub variant: GitChipVariant,          // Pill | Inline | WorkspaceBar | Pro(..)
    pub max_chars: u8,                    // 24 (proposed)
    pub behind_warn: u32,                 // 10 (proposed)
    pub on_switch: Option<Callback<BranchName>>,
    pub on_sync: Option<Callback<()>>,
    pub on_push: Option<Callback<()>>,
    pub on_open_operation: Option<Callback<()>>,   // → merge-conflict
    pub on_copy: Option<Callback<BranchName>>,
    pub labels: GitChipLabels,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [checkpoints-git.md](checkpoints-git.md) · [merge-conflict.md](merge-conflict.md) ·
[pr-card.md](pr-card.md) (branch → PR) · [diff-viewer.md](diff-viewer.md) (Review changes) ·
[fanout-grid.md](fanout-grid.md) (worktree per lane) · [composer.md](composer.md) (metadata row) ·
[composer-status-strip.md](composer-status-strip.md) (PR pill "copy branch") · [loader-glyph.md](loader-glyph.md).

| Reference | What we take | Licence note |
|---|---|---|
| Worktree-per-task apps: Codex app, Claude desktop, Conductor, Verdent, Paradev, Emdash, Zed, Cursor, Copilot (note 03) | worktree badge, Local / Worktree choice, workspace bar, copy branch | Codex CLI Apache-2.0 (note 08) — port with attribution; others proprietary / mixed — reference only |
| agentbox [R027] | "each agent gets its own git worktree"; pixel-grid mark | Study only |
| Hoplite (note 10) | branch chip in the composer metadata row | Proprietary — reference only |
| git status ahead / behind / detached concepts | data model only | concept, no code |
