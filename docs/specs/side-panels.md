**Status:** draft v0.1 (2026-11-12).

# Spec: side panels, file tree & status bar (`side-panel`, `file-tree`, `status-bar`)

Part of family J ([workspace-chrome.md](workspace-chrome.md) — docks, resizers, rails, F6 regions).
Same depth as [composer-send-control.md](composer-send-control.md). Three components that share one
region model: the **side panel** (a dock with a pane switcher), the **file tree** that most side panels
host, and the **status bar** at the window foot. Target **3 + 3 + 3 free, 6 + 8 + 6 Pro = 29 variants**.
Numbers are Oxyn **proposals** unless a source is cited.

## 1. Purpose
- **Side panel.** The user wants a second view next to the conversation — files, changes, plan, tasks,
  sub-agents, browser, trace, MCP servers — with a badge telling them which one has news. Claude desktop
  has pane kinds chat, diff, browser, terminal, file, plan, tasks, subagent with a ⋮ overflow for panes
  without buttons (notes 03, 05); Claude desktop also has a session-details rail with cards (where it runs,
  Changes, running tasks, Context %) (note 03); Devin uses workspace tabs Progress / Shell / IDE / Browser
  (note 03); Hoplite thread tabs Chat, Diff, Pull requests, Previews, Terminal, Subagents from a "+ tab
  menu" (note 10); OpenCode desktop plugins render **their own sidebar UI** (note 03).
- **File tree.** "Which files did the agent touch, which are changed vs the base, which are in context,
  and where is the one I need?" Research note 05 §6 lists node states `unchanged`, `added`, `modified`,
  `deleted`, `renamed`, `agent-touched` (sparkle dot), `in-context` (@-mentioned), `ignored`,
  `conflicted`, `reviewed`; AI Elements File Tree shows "files the agent changed, as a folder tree" (note
  05); Hoplite's changed-files tree carries per-file + / − counts, filter and jump, mark as viewed, [ / ]
  to step files, drag-to-resize and a hide button (note 10); a tree that **grows** as the agent touches
  files [R004] [R038].
- **Status bar.** One glance for workspace health: branch, sync, agent activity, context, model, jobs,
  MCP health, problems, notifications. Codex CLI's footer collapses by width with priority rules; Claude
  CLI's `/statusline` (model, dir, context, cost); Zed's ✨ agent icon in the status bar; Verdent's
  workspace bar (all note 03); ObsidianUI Status Bars with a gliding tooltip and keyboard grid [R094].

## 2. Anatomy
```
 side panel (right dock)                         file tree node row (24 px desktop / 22 px ide / 44 px touch)
 ┌ [Files][Changes 12][Plan][Agents ●][⋮] ┐      ┌─────────────────────────────────────────────────────┐
 │ ⌕ Filter files…            [◆][±][⋯]  │      │ ▸ 📁 src                       ● 4           M      │
 │ ▾ src                       ● 4    M   │      │   ▾ 📁 auth                    ● 2                  │
 │   ▾ auth                    ● 2        │      │     📄 refresh.rs  ✦ +42 −7  ◎  M    ← agent-touched ✦,
 │     refresh.rs  ✦ +42 −7 ◎  M          │      │     📄 tokens.rs   ✦⠋ editing…   M      in-context ◎,
 │     tokens.rs   ✦⠋           M          │      │     📄 old.rs        −88        D      git status letter
 │     old.rs      −88          D          │      │   indent 12 px/level · guides 1 px · rows virtualised
 │   ▸ api  (loading ◌)                    │      └─────────────────────────────────────────────────────┘
 │ ⋯ 2 ignored hidden                      │
 └─────────────────────────────────────────┘
 status bar (24 px desktop / 22 px ide / 1 row terminal)
 ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
 │ ⑂ feat/auth ↑2 ●  ⟳  ⚠ 3  ✕ 1 │        ◆ Editing tokens.rs 0:42  │ ◔ 56%  Opus  ⟳ 2  ⚡ 5/6 MCP  🔔 2 │
 └ left: workspace items ────────┴──── centre: agent activity ──────┴──── right: run items ──────────┘
```
**File tree marks (proposed):**
| Mark | Meaning | Rule |
|---|---|---|
| `M` `A` `D` `R` `U` `?` `!` | git modified, added, deleted, renamed, unmerged (conflict), untracked, ignored | letter at row end in the git tone; deleted rows strike through and stay until committed / reviewed |
| `+42 −7` | line counts vs base | tabular digits; only when the app supplies diffs; ≥ 10 000 → `+9.9k` |
| `✦` agent-touched | edited or created by an agent in this thread / session | agent identity tone; tooltip "Edited by Researcher · 2m ago"; multiple agents → stacked 2 glyphs + `+n` |
| `✦⠋` live | agent is writing this file now | glyph loader in the agent tone (one busy signal per region) |
| `◎` in-context | file is in the agent's context (attached, @-mentioned or read this turn) | hollow ring; click opens [composer-context-meter.md](composer-context-meter.md) breakdown |
| `✓` reviewed | user marked viewed (Hoplite "mark as viewed", note 10) | dims row to 70 % |
| `⚠` conflicted | merge conflict | opens [merge-conflict.md](merge-conflict.md) |
| folder roll-up | any descendant changed / touched | folder shows `● n` (count of changed descendants) and the strongest mark (conflict > agent-live > modified) |

**Status bar items and priority** (lowest number survives longest; width-adaptive like Codex's footer,
note 03):
| Pri | Item | Source | Click |
|---|---|---|---|
| 1 | Agent activity: glyph + verb + elapsed, or "Needs you" | `AgentState` (same clock as [composer-status-strip.md](composer-status-strip.md)) | jump to live step / card |
| 2 | Branch chip | [git-chips.md](git-chips.md) `terminal` / `pill` | branch menu |
| 3 | Problems ⚠ n / ✕ n | app diagnostics | problems panel |
| 4 | Context % | [composer-context-meter.md](composer-context-meter.md) `bar` | breakdown |
| 5 | Notifications 🔔 n | [command-palette.md](command-palette.md) notification centre | open centre |
| 6 | Jobs ⟳ n | [background-jobs.md](background-jobs.md) | jobs panel |
| 7 | MCP health ⚡ 5/6 | [mcp-plugin-list.md](mcp-plugin-list.md) | MCP list |
| 8 | Model | model picker | picker |
| 9 | Sandbox badge | [permission-rules.md](permission-rules.md) | sandbox detail |
| 10 | Sync / remote | app | — |
Items hide from the highest number down as width shrinks; hidden items move into a `⋯` overflow menu that
keeps their badges. An item that **needs attention** (needs-you, error) gets temporary priority 0.

## 3. State machines
Side panel:
```
 Closed ─open / shortcut / agent-requested badge click─▶ Open{pane}
 Open{pane} ─switch tab─▶ Switching ─▶ Open{pane'} ; Open ─dock collapsed─▶ Rail{badges} ─▶ Closed
 Pane content: Loading(skeleton after 150 ms) ─▶ Ready ─▶ Empty{reason} | Error{retry} | Stale(last update Ns)
 Badge per pane: None ⇄ Count{n} ⇄ Attention(needs-you) ⇄ Live(agent working in this pane)
```
File tree:
```
 Node:  Collapsed ─expand─▶ Loading(children) ─ok─▶ Expanded ; ─err─▶ LoadError{retry} ; ─timeout 10 s─▶ LoadError
        Expanded ─collapse─▶ Collapsed ;  Hover · Focused · Selected{multi} · Renaming · DragSource · DropTarget{into|before|after}
 Marks (independent): git{status} · diff{+a −r} · agent{touched | live(agent)} · in_context · reviewed · conflicted
 Tree:  Idle ─type in filter─▶ Filtering{query} (debounce 80 ms; ancestors of matches auto-expand) ─clear / Esc─▶ Idle
        Scope: All ⇄ Changed ⇄ AgentTouched ⇄ InContext   (toggle chips)
        Follow-agent: Off ⇄ On (the live file is revealed and scrolled into view, not focused)
        FS watcher: Synced ⇄ Rescanning ⇄ Desynced{reason} (show "Refresh")
```
Status bar:
```
 Item: Hidden ⇄ Visible{value} ─value changes─▶ Updating(roll/crossfade) ─▶ Visible
       Visible ─attention─▶ Attention(tone + priority 0) ─resolved─▶ Visible
       Visible ─width shrinks─▶ Overflowed(in ⋯) ─width grows─▶ Visible
 Bar:  Normal ⇄ Offline banner tint ⇄ Focused(roving toolbar) ⇄ HoverTooltip(gliding)
```

## 4. States tables
### Side panel
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Open{pane}` | tab strip + pane; active tab underline | — | ⋮ more panes, close | ⌃Tab / ⌃⇧Tab next/prev pane (in region), ⌘/Ctrl+⌥+1…9 jump |
| `Rail` | 48 px icon column with badges | click → flyout | expand | ↑/↓ icons, Enter |
| `Loading` | skeleton (after 150 ms) | — | — | — |
| `Empty{reason}` | one-line reason + action ("No changes yet") | action | — | — |
| `Error` | inline error + Retry | retry | — | Enter |
| `Stale` | content + "last update 40s ago" dim | refresh | — | — |
| Badge `Count{n}` | number pill (max "99+") | — | — | — |
| Badge `Attention` | amber dot + "needs you" in tooltip | open pane | — | — |
| Badge `Live` | tiny loader in the tab | open pane | — | — |

### File tree
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Collapsed` / `Expanded` | ▸ / ▾ | toggle | — | → expand / ← collapse or go to parent |
| `Loading(children)` | ◌ in place of ▸ after 150 ms; a skeleton child row after 400 ms | — | cancel (Esc) | — |
| `LoadError` | "Couldn't read folder · Retry" child row | retry | — | Enter |
| `Focused` / `Selected` | row highlight (one gliding highlight), focus ring | open file | multi-select ⇧ / ⌘ | Enter open, Space preview, type-ahead |
| `Renaming` | inline field (user files only; agent-live files are locked) | Enter | Esc | — |
| `DragSource` / `DropTarget` | ghost row; target folder highlights after 600 ms hover auto-expand | drop (move) | ⌥ copy; drag to composer = attach | — |
| `Filtering{query}` | matches with highlighted characters; non-matching ancestors dimmed; count "12 of 4,210" | Enter opens first | Esc clears | ↓ into results |
| Scope `Changed` / `AgentTouched` / `InContext` | flat or tree list of only those files | — | — | ⌥1–4 switch scope (proposed) |
| Mark `agent live` | ✦⠋ + "editing…" | open (read-only while live) | follow agent | — |
| `Desynced` | banner "File list may be out of date · Refresh" | refresh | — | — |
| `Empty` (repo has no files / filter no match) | "No files match 'tok'" + clear filter | clear | — | — |

### Status bar
| State | Shows | Primary | Secondary | Keyboard |
|---|---|---|---|---|
| `Visible{value}` | glyph + short value | item action | context menu (hide item) | ←/→ roving focus, Enter, ⇧F10 menu |
| `Attention` | tone (amber / danger), priority 0 | resolve (jump) | — | Enter |
| `Overflowed` | in `⋯` menu with badge | open menu | — | Enter on ⋯ |
| `HoverTooltip` | one tooltip that glides between items ([R094] idea) | — | — | focus shows the same tooltip |
| `Offline` | bar left edge tinted warning + "Offline · retrying in 8s" | retry now | — | — |

## 5. Micro-interactions
- **Pane switch.** Underline glides between tabs (`spring.snap`); content crossfades 120 ms with a 4 px
  shift in the direction of travel; no height animation (panes fill the dock).
- **Badge.** Count rolls (odometer, tabular digits, ≤ 2 Hz); a new Attention badge pops 0.8 → 1 with
  `spring.soft` once.
- **Agent touches a file** (tree visible). The row's ✦ fades in and the row gets a 600 ms tint in the agent
  tone that decays; if the file's folder is collapsed, the folder's `● n` rolls instead (no auto-expand
  unless Follow-agent is on). New files created by the agent **grow in**: row height 0 → 24 px with
  `spring.snap`, siblings slide; several new files stagger 40 ms (cap 6) — a calm version of [R004]'s
  growth-as-reveal (≈40–60 ms stagger).
- **Live edit.** ✦⠋ loader while the agent writes; on finish it morphs to ✦ and `+a −r` rolls in.
- **Expand / collapse.** Children height-open with `spring.snap`, rows fade in staggered 15 ms (cap 12);
  chevron rotates 90° in 120 ms. Collapse is faster and unstaggered.
- **Lazy load.** No spinner for loads under 150 ms; after 150 ms the chevron becomes ◌; after 400 ms one
  skeleton row; children then replace it without layout jump.
- **Filter.** Results update per keystroke after 80 ms debounce; highlight characters in medium weight;
  the list reflows with small glides (≤ 8 rows) and snaps on long jumps (same rule as the slash menu,
  [composer-slash-mentions.md](composer-slash-mentions.md)).
- **Follow agent.** The live file scrolls into view with a 200 ms `ease.in-out` glide (instant if > 2,000 px)
  and a brief outline; user focus never moves.
- **Status bar value change.** Numbers roll; text crossfades 160 ms; attention tone ramps in 400 ms; the
  gliding tooltip moves between items with `spring.snap` and changes content with a 100 ms crossfade.
- **Width collapse.** Items leaving for `⋯` fade 120 ms; the `⋯` badge pulses once if a hidden item gains
  attention.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Pane underline glide | ~180 ms | 0 | `spring.snap` | — | 60 Hz |
| Pane content crossfade | 120 ms, 4 px shift | 0 | `ease.out` | — | 60 Hz |
| Badge roll / pop | 280 ms / ~240 ms | max 2 Hz | `ease.out` / `spring.soft` | 30 ms per digit | 30 Hz |
| Agent-touch tint decay | 600 ms | 0 | `ease.out` | — | 30 Hz |
| New file grow-in | ~200 ms | 0 | `spring.snap` | 40 ms, cap 6 | 60 Hz |
| Expand children / chevron | ~200 ms / 120 ms | 0 | `spring.snap` / `ease.out` | 15 ms, cap 12 | 60 Hz |
| Collapse | 140 ms | 0 | `ease.in` | none | 60 Hz |
| Lazy-load indicator / skeleton | 120 ms fade | 150 ms / 400 ms | `ease.out` | — | — |
| Filter debounce / reflow | 80 ms / ≤ 160 ms glide | 80 ms | `spring.snap` (≤ 8 rows) else instant | — | 60 Hz |
| Follow-agent reveal | 200 ms (instant > 2,000 px) | 0 | `ease.in-out` | — | 60 Hz |
| Status value crossfade / roll | 160 ms / 280 ms | 0 | `ease.out` | — | 30 Hz |
| Attention tone ramp | 400 ms | 0 | OKLab linear | — | 30 Hz |
| Gliding tooltip move / swap | ~160 ms / 100 ms | 400 ms first show, 0 after | `spring.snap` | — | 60 Hz |
| **Reduced motion** | no glides, grows, tint decays or pops: state appears with an 80 ms crossfade; marks and counts still update (information); live-edit loader shows its still frame; tooltip jumps | — | linear | none | ≤ 2 Hz |

## 7. Variants
### Side panel — free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `side-panel/tabs` | Baseline: tab strip with badges + ⋮ overflow for extra panes | desktop, ide |
| `side-panel/rail` | Icon rail with badges; click opens a flyout pane (pairs with `panel-split/rail-snap`) | desktop, ide |
| `side-panel/segmented` | 2–4 panes as a segmented control above content (Files / Changes / Plan) | desktop, chat |
### Side panel — Pro (6)
| Name | One-line idea |
|---|---|
| `side-panel/details-cards` | Session-details rail of stacked cards: where it runs, changes, running tasks, context % (Claude desktop idea, note 03) |
| `side-panel/plugin-host` | A slot where a plugin renders its own pane with a sandbox frame and its own badge (OpenCode desktop idea, note 03) |
| `side-panel/progress-tab` | One "Progress" pane unifying shell, edits and browser steps with "Follow agent" (Devin idea, note 03) |
| `side-panel/stacked` | Two panes stacked vertically with a resizer (Files over Changes) |
| `side-panel/peek-card` | Hovering a rail icon shows a mini card of that pane's summary before opening |
| `side-panel/liquid-tabs` | Active tab indicator as a soft blob that stretches between tabs |

### File tree — free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `file-tree/explorer` | Baseline full tree with git letters, ✦ agent marks, ◎ in-context, filter, lazy load | desktop, ide |
| `file-tree/changed` | Only changed files as a compact tree with `+a −r`, mark-as-viewed, [ / ] stepping (Hoplite idea, note 10) | desktop, ide, chat (review) |
| `file-tree/tui` | `├──` / `└──` box-drawing tree, git letters, `*` for agent-touched | terminal |
### File tree — Pro (8)
| Name | One-line idea |
|---|---|
| `file-tree/grow` | Folders sprout children along a curve with staggered leaf pops when the agent touches them ([R004]) |
| `file-tree/heat` | Row background heat by how often the agent read / edited the file this session |
| `file-tree/agent-lanes` | Multiple agents: one coloured tick column per agent next to each file |
| `file-tree/scan-sweep` | During repo indexing, a sweep line passes down the tree and rows tick as scanned ("974 files, 20 agents checking", [R038] idea) |
| `file-tree/context-picker` | Checkbox mode to add / remove files from context with live token cost per file |
| `file-tree/minimap` | Collapsed tree drawn as a dot-matrix minimap of changes for huge repos |
| `file-tree/timeline` | Each file shows a tiny strip of edit times this session; click jumps to the checkpoint |
| `file-tree/ownership` | CODEOWNERS / team badge per folder with a warning when the agent edits outside its scope |

### Status bar — free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `status-bar/ide` | Baseline 24 px bar, three zones, priority collapse into `⋯` | desktop, ide |
| `status-bar/minimal` | Only agent activity + branch + attention items; everything else in `⋯` | desktop, chat |
| `status-bar/tui` | One-line Codex-style footer that collapses by width ("? for shortcuts" → context %) (note 03) | terminal |
### Status bar — Pro (6)
| Name | One-line idea |
|---|---|
| `status-bar/gliding-tooltip` | One tooltip glides between items, dimming the others ([R094] idea) |
| `status-bar/run-strip` | Tiny bars for the last 20 runs (ok / failed / cancelled) with a keyboard grid ([R094] Status Bars idea) |
| `status-bar/statusline-builder` | User-composable items and order, Claude `/statusline`-style (note 03), saved per workspace |
| `status-bar/cost-ticker` | Live token / cost ticker with digit roll (cost only when prices are supplied) |
| `status-bar/agent-roster` | One dot per running agent with its tone; hover lists them |
| `status-bar/island-mirror` | When the window is unfocused, the top items mirror into the desktop island ([desktop-overlays.md](desktop-overlays.md)) |

## 8. Edge cases
- **Huge repos** (100k+ files): rows virtualised (render window + 20 rows overscan); folders with > 2,000
  children show the first 2,000 then "Show 2,000 more"; filter runs over the app's index, not the DOM,
  and shows "Searching…" after 300 ms.
- **Symlink loops**: detect by canonical path; show the link with ↪ and don't descend.
- **Permission denied / unreadable folders**: `LoadError` with the OS reason; never infinite spinner
  (10 s timeout).
- **Rename / delete while the agent is editing that file**: blocked with tooltip "Researcher is editing
  this file"; the user can take over (acting-indicator).
- **Agent edits 200 files at once**: tint and grow animations are skipped above 20 changes per second;
  folder counts roll once at the end; a summary "Agent changed 200 files" appears in the pane header.
- **Deleted files**: stay as struck-through rows until committed, so the user can review deletions.
- **Long names**: middle-truncate with extension kept (`refresh_tok…_test.rs`); full path tooltip.
- **Case-only renames / Unicode normalisation**: display the new case; compare NFC-normalised paths.
- **Ignored files**: hidden by default with "⋯ 2 ignored hidden"; in-context ignored files are shown with
  a warning (the agent can see something the project ignores).
- **Status bar overflow**: below 480 px only priority 1–2 plus `⋯`; attention items always win.
- **RTL**: tree indents from the inline-start (right); chevrons mirror; paths stay LTR inside bidi
  isolates; status bar zones mirror.
- **Tiny sizes**: tree rows at 20 px drop diff counts first, then git letters (kept as colour + tooltip,
  plus the letter in the accessible name).
- **Dense IDE**: 22 px rows, 10 px indent, 11–12 px text.
- **Touch**: 44 px rows; swipe-left on a row reveals Mark viewed / Attach; long-press = context menu.
- **Slow networks / remote FS**: lazy-load timeout 10 s → `LoadError`; watcher desync banner; status
  values older than 30 s show a dim "stale" dot.
- **Very fast agents**: live-edit loaders shorter than 300 ms are not shown (only the final ✦ mark).

## 9. Accessibility
- **Side panel:** tab strip = `tablist`; panes = `tabpanel`; badges are part of the tab name ("Changes,
  12 files", "Agents, needs you").
- **File tree:** `role="tree"` / `treeitem` with `aria-level`, `aria-setsize`, `aria-posinset`,
  `aria-expanded`, `aria-busy` while loading; name composes file, git status in words, agent marks and
  in-context ("refresh.rs, modified, 42 added 7 removed lines, edited by Researcher, in context").
  Type-ahead; `*` expands siblings; Home / End. Filter input `aria-controls` the tree and announces
  "12 matches" politely (debounced 500 ms).
- **Agent changes** are not announced per file; one polite summary per agent step ("Researcher changed 3
  files").
- **Status bar:** container `role="status"` is wrong for an interactive bar, so the bar is a `toolbar`
  labelled "Status bar" with roving focus, plus a separate visually hidden polite live region for attention
  changes ("Agent needs approval", "3 problems"). Values never announced per tick.
- **Contrast:** git letters and diff counts ≥ 4.5:1; marks ≥ 3:1 and always backed by text in the name;
  conflict and deletion use glyph + strike, not colour alone; colour-blind palette option swaps green/red
  for blue/orange (Claude daltonized idea, note 08).
- **Focus:** F6 lands on the last focused tree row; agent activity never moves focus.
- **Reduced motion:** §6 last row.

## 10. Props sketch
```rust
pub struct SidePanel<'a> {
    pub panes: &'a [PaneDesc],               // { id, kind: Files|Changes|Plan|Tasks|Agents|Browser|Trace|Mcp|Plugin(id), badge: PaneBadge }
    pub active: PaneId,
    pub variant: SidePanelVariant,           // Tabs | Rail | Segmented | Pro(..)
    pub max_visible_tabs: u8,                // 5, rest in ⋮
    pub on_select: Callback<PaneId>,
    pub motion: MotionLevel,
}
pub enum PaneBadge { None, Count(u32), Attention, Live(AgentId) }

pub struct FileTree<'a> {
    pub source: &'a dyn FileTreeSource,      // lazy: children(path) -> Future<Result<Vec<Node>>>
    pub marks: &'a dyn MarkSource,           // git, diff, agent touches, in-context, reviewed
    pub agents: &'a [AgentIdentity],         // tone + name per AgentId
    pub variant: FileTreeVariant,            // Explorer | Changed | Tui | Pro(..)
    pub scope: TreeScope,                    // All | Changed | AgentTouched | InContext
    pub filter: Option<SharedString>,
    pub follow_agent: bool,
    pub show_ignored: bool,
    pub page_size: u32,                      // 2,000
    pub load_timeout: Duration,              // 10 s
    pub on_open: Callback<PathBuf>,
    pub on_attach: Option<Callback<Vec<PathBuf>>>,   // drag to composer / context picker
    pub on_mark_viewed: Option<Callback<PathBuf>>,
    pub labels: FileTreeLabels,
    pub motion: MotionLevel,
}
pub struct NodeMarks { pub git: Option<GitStatus>, pub diff: Option<(u32, u32)>, pub agent: AgentMark /* None | Touched(Vec<AgentId>) | Live(AgentId) */,
                       pub in_context: bool, pub reviewed: bool, pub conflicted: bool }

pub struct StatusBar<'a> {
    pub items: &'a [StatusItem],             // { id, priority, zone: Left|Centre|Right, content, attention: bool, on_activate }
    pub agent: &'a AgentState,               // drives the activity item
    pub variant: StatusBarVariant,           // Ide | Minimal | Tui | Pro(..)
    pub offline: Option<OfflineInfo>,
    pub motion: MotionLevel,
}
```

## 11. Related components and references
**Related:** [workspace-chrome.md](workspace-chrome.md) · [git-chips.md](git-chips.md) ·
[composer-context-meter.md](composer-context-meter.md) · [composer-status-strip.md](composer-status-strip.md)
(shared activity clock) · [background-jobs.md](background-jobs.md) · [mcp-plugin-list.md](mcp-plugin-list.md) ·
[command-palette.md](command-palette.md) (notification count, "go to file") ·
[file-edit-chip.md](file-edit-chip.md) · [diff-viewer.md](diff-viewer.md) · [merge-conflict.md](merge-conflict.md) ·
[permission-rules.md](permission-rules.md) (sandbox badge) · [acting-indicator.md](acting-indicator.md) ·
[subagent-tree.md](subagent-tree.md) (Agents pane).

| Reference | What we take | Licence note |
|---|---|---|
| Research note 05 §6 FileTree / SidePanel proposal | node states, pane kinds | Our own proposal |
| AI Elements File Tree (note 05) | "files the agent changed" tree | Licence not recorded in licensing.md — study only until checked |
| @pierre/trees (note 05) | tree paired with a diff view | Licence not checked — study only |
| Hoplite changed-files tree (note 10) | + / − counts, filter, mark viewed, [ / ], resize, hide | Proprietary — reference only |
| Claude desktop panes & details rail (notes 03, 05) | pane kinds, ⋮ overflow, details cards | Proprietary — reference only |
| Devin Progress tab, OpenCode desktop plugin sidebars (note 03) | unified progress pane; plugin-rendered panes | Devin proprietary; OpenCode MIT (idea only here) |
| Codex CLI footer, Claude `/statusline`, Zed ✨ status icon, Verdent workspace bar (note 03) | width-adaptive priority; composable items | Codex MIT/Apache — portable; Claude proprietary; Zed study only; Verdent reference only |
| ObsidianUI Status Bars [R094] | gliding tooltip, keyboard grid, run bars | "Free (source in public registry)", licence not stated — study only |
| @FinnoTaylor pixel-tree [R004], HacktronAI growing tree [R038] | growth-as-reveal, ≈40–60 ms stagger | Study only |
| Claude daltonized diff palette (note 08) | colour-blind swap | Proprietary — idea only |
| Rune-kit Tree (note 07) | Rust tree widget exists | MIT — portable with attribution; re-check |
