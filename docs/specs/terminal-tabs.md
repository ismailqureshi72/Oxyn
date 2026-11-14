**Status:** draft v0.1 (2026-11-12).

# Spec: terminal tabs (`terminal-tabs`)

Part of family J ([workspace-chrome.md](workspace-chrome.md) — panel model, docks and F6 regions live
there). Same depth as [composer-send-control.md](composer-send-control.md). Target **4 free + 12 Pro =
16 variants**. This spec owns the **tab strip** of the in-app terminal panel: one tab per shell session,
its owner, run status, exit code, bell and activity, rename, drag reorder, overflow and close. The
**inline command block** in the transcript is [terminal-output.md](terminal-output.md); the terminal grid
inside a tab reuses its rendering rules (§2.1 there). "Who is driving" a shared terminal is
[acting-indicator.md](acting-indicator.md) (`acting-indicator/terminal-owner` is the marker this strip
hosts).

## 1. Purpose
In coding-agent apps the in-app terminal is table-stakes: Claude desktop (Ctrl+\`), Verdent, Conductor,
Paradev, Orca (splits), Codex app, Zed (**terminal threads** as agent-panel tabs), Warp, pi-gui, Devin
(Shell tab with command history and jump-in-time) (research note 03 §1G). The new problem is that some
tabs are **the agent's** and some are **the user's**. The user wants to know, from the strip alone:
**which tabs are running, which finished and how (exit code), which need input, which rang the bell or
printed something unseen, and which ones the agent owns** — then rename, reorder, split and close them
without killing a build by accident.

Research note 05 §6 found **no dedicated reference spec for agent terminal tabs** and proposed the states
`running`, `idle`, `exited-0`, `exited-nonzero`, `awaiting-input`, `agent-owned` vs `user-owned`,
`background`, `killed`. We adopt that proposal and complete it. OpenCode's TUI `tab-pulse` gives concrete
motion: running pulse 2.8 s with a 450 ms attack, completion pulse 1.2 s, edge flash 800 ms, unread glow
(note 08 §1.5, MIT).

## 2. Anatomy
```
 tab strip (32 px desktop / 26 px ide / 1 row terminal)
 ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
 │ [◆ ⠋ cargo test   1:02 ✕] [● zsh ·  ✕] [◆ ✓ migrate ✕] [● ✕101 dev-server 🔔 ✕] ‹ › [⌄ 3] [+ ⌄] │
 └──────────────────────────────────────────────────────────────────────────────────────────────┘
   │ │   │            │                                                    │     │      │
   │ │   title        elapsed (running ≥ 10 s, active tab or hover)         │     │      new tab ▸ menu
   │ status slot (16 px): loader | ✓ | ✕N | ⌨ | ■ | 🔔 | · unseen          │     overflow list
   owner glyph: ◆ agent (agent identity colour) · ● you (neutral) · avatar (teammate)   scroll arrows
```
- **Owner.** `Agent(id)` tabs show the agent's glyph in its identity tone (same 8 agent tokens as
  [acting-indicator.md](acting-indicator.md)); `User` tabs show a neutral dot (or nothing, in the `minimal`
  variant); `Teammate` shows a 12 px avatar. Agent tabs are **read-only for typing by default**; typing
  raises "Take over to type" (acting-indicator's `TakeoverPolicy::Explicit`).
- **Status slot** (one glyph, priority order): `AwaitingInput` ⌨ > `Bell` 🔔 > `Running` loader >
  `Failed` ✕N > `Killed` ■ > `Succeeded` ✓ (fades after 4 s) > `Unseen` dot > nothing.
- **Title.** Precedence: user rename > agent-given title > OSC 0/2 window title > current process name >
  shell name. Max 24 ch, end-truncated (process names) or middle-truncated (paths).
- **Close ✕.** On hover / active tab only (always on touch); replaced by the status glyph otherwise so the
  strip stays calm. Middle-click closes.
- **New tab** `+` opens a user shell; `⌄` menu: profile / cwd / "New agent terminal" only if the app
  allows the user to create agent-owned sessions.

## 3. State machine
Per tab (process state × attention flags × ownership):
```
                      spawn                     first byte                        exit 0
  (none) ──────────▶ Starting ───────────────▶ Running ⇄ Idle(prompt shown) ─────────────▶ Succeeded ──4 s──▶ Idle*
                        │ spawn error            │  │  ▲                           exit n   ▼
                        ▼                        │  │  └── output ──┐           ──────────▶ Failed{n}
                     SpawnFailed                 │  ▼               │           signal   ▼
                                                 │ AwaitingInput{prompt}        ──────────▶ Killed{sig}
                                                 │  (read on stdin / y/N detected)
                                                 ├─ move to background ──▶ Backgrounded (tab dims; jobs tray owns it)
                                                 └─ connection lost ──────▶ Lost (sandbox / remote PTY gone) ── reconnect ──▶ Running
  * a shell tab returns to Idle after its foreground command; a single-command tab (agent `exec`) ends in its exit state.

  Attention flags (independent, cleared on view):  Unseen(output while not active) · Bell(BEL received) · Pinned
  Ownership:  Agent{id} ⇄ User   (take over / hand back — acting-indicator.md) ; Teammate{id} (shared sessions)
  Strip:      Normal ─ tabs exceed width ─▶ Overflow{hidden: k}
  Edit:       Viewing ─ double-click / F2 ─▶ Renaming ─ Enter ─▶ Viewing(renamed) | Esc ─▶ Viewing
  Drag:       Idle ─ 4 px ─▶ Dragging{index} ─ release ─▶ Reordered | over pane edge ─▶ SplitPreview | Esc ─▶ Idle
  Close:      Closing ─ process running? ─yes─▶ ConfirmClose{cmd, elapsed} ─ Close ─▶ Stopping ─▶ removed
                                         └no─▶ removed (Undo 6 s, user tabs only, scrollback kept)
```

## 4. States table
| State | Shows | Primary | Secondary | Keyboard (strip focused) |
|---|---|---|---|---|
| `Starting` | loader after 300 ms, title = shell | — | close | — |
| `Idle` | owner + title; no status glyph | focus terminal | rename, split, close | Enter focus grid |
| `Running` | loader in slot (pulse on the tab's top edge for the active tab, `tab-pulse` idea), elapsed after 10 s | — | Stop (menu), move to background | ⌘/Ctrl+C in grid; Delete → close with confirm |
| `AwaitingInput{prompt}` | ⌨ in `color.loader.waiting` (amber), tab tinted 8 %; tooltip shows the prompt line | focus grid (user tabs) / "Take over" (agent tabs) | — | Enter |
| `Succeeded` | ✓ success tone for 4 s, then fades to nothing (shell) or stays (single-command tab) | — | rerun (single-command) | R rerun |
| `Failed{n}` | `✕101` danger, sticky until viewed | focus grid at the failing output | rerun, copy last 20 lines | Enter, R |
| `Killed{sig}` | ■ neutral + "SIGINT" tooltip | — | rerun | R |
| `Backgrounded` | tab at 60 % opacity, ⟳ glyph, "In background" tooltip | bring back | stop | Enter |
| `Lost` | ! warning, "Connection lost" | reconnect (adapter) | close | Enter |
| `SpawnFailed` | ✕ with "Couldn't start zsh" | retry | choose profile | Enter |
| `Unseen` | 6 px dot (accent) after the title | view | — | — |
| `Bell` | 🔔 glyph (or ring-tinted dot) until viewed; never a sound by default | view | mute bells for this tab | — |
| `Renaming` | inline text field, pre-selected title, 24 ch hint | Enter commit | Esc cancel; empty → revert to auto title | Enter / Esc |
| `Dragging` | ghost tab follows pointer; gap opens at the drop index | release | Esc cancel | ⌘/Ctrl+Shift+←/→ move tab (keyboard reorder) |
| `Overflow{k}` | ‹ › scroll arrows and a `⌄ k` button listing hidden tabs **with their status glyphs**; hidden tabs that need attention tint the `⌄` button | open list | — | ⌘/Ctrl+Shift+\\ opens list (proposed) |
| `ConfirmClose` | inline popover under the tab: "Close terminal? `cargo test` is still running (1m 02s)." Agent tabs add: "The agent will see it as stopped." | **Close and stop** (danger) | Keep running · Move to background | Enter = Keep (safe default), ⌘/Ctrl+Enter = close |
| `Disabled{reason}` | new-tab `+` disabled with reason ("Sandbox offline", "Session limit 8 reached") | — | — | — |

Global keys (proposed; apps may remap): ⌘/Ctrl+T new tab (when the terminal region is focused),
⌘/Ctrl+W close, ⌃Tab / ⌃⇧Tab next / previous, ⌘/Ctrl+1…9 jump, F2 rename. Arrow keys move between tabs
when the strip has focus (`tablist` roving focus).

## 5. Micro-interactions
- **New tab.** Inserts after the active tab; width grows from 0 with `spring.snap` while neighbours slide;
  title fades in 120 ms.
- **Running pulse** (from OpenCode `tab-pulse`, note 08, MIT): the active running tab draws a 2 px
  highlight that travels along its top edge — 2.8 s loop, 450 ms attack, rest opacity 0.18. Inactive
  running tabs show only the slot loader (one busy signal per region, [loaders.md](loaders.md) §2.3).
- **Completion.** Loader morphs to ✓ / ✕N (`spring.snap` 160 ms); a single 1.2 s completion pulse
  (attack 12 %) runs on the tab edge; failure adds an 800 ms edge flash in `danger` and a 2 × 3 px nudge of
  the glyph only (the tab itself never shakes).
- **Unseen output.** Dot fades in 120 ms on the first byte that arrives while inactive; viewing the tab
  clears it with a 240 ms "diffuse" (dot scales 1 → 1.6 and fades; OpenCode's unread glow idea).
- **Bell.** BEL triggers a 400 ms ring wobble (± 12°, 2 cycles) on the 🔔 glyph, rate-limited to once per
  2 s per tab; system attention (dock bounce / taskbar flash) only if the app opts in and the window is
  unfocused.
- **Rename.** Title morphs into a field with the same baseline (no jump); commit crossfades 120 ms.
- **Drag reorder.** 4 px threshold; ghost at 0.95 scale + shadow; neighbours slide to open a gap with
  `spring.snap`; drop settles the ghost into the gap 160 ms. Dragging onto a pane edge hands off to the
  split preview of [workspace-chrome.md](workspace-chrome.md) §5. Pinned tabs only reorder among pinned.
- **Overflow.** Scrolling the strip glides 200 ms `ease.in-out`; the active tab is always scrolled into
  view with 24 px margin; the `⌄ k` count rolls (odometer, tabular digits).
- **Close.** Tab width collapses to 0 (160 ms `ease.in`), neighbours fill; Undo toast for user tabs.
  `ConfirmClose` popover scales from the tab (0.96 → 1, 160 ms).
- **Ownership change.** Owner glyph crossfades (200 ms) and the tab edge flashes once in the new owner's
  tone (acting-indicator rule 2: control changes are visible).

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Tab insert / remove | ~200 ms / 160 ms | 0 | `spring.snap` / `ease.in` | — | 60 Hz |
| Loader show | 120 ms fade | 300 ms (`Starting`) | `ease.out` | — | glyph frame rate ([loader-glyph.md](loader-glyph.md)) |
| Running edge pulse | 2.8 s loop, 450 ms attack | 0 | sine | max 1 tab (active) | 30 Hz |
| Elapsed appear | 120 ms | at 10 s | `ease.out` | — | 1 Hz |
| Exit morph | 160 ms | 0 | `spring.snap` | — | 60 Hz |
| Completion pulse | 1.2 s, attack 12 % | after morph | `ease.out` | — | 30 Hz |
| Failure edge flash + glyph nudge | 800 ms + 300 ms (2 × 3 px) | after morph | `ease.out` / linear | — | 60 Hz |
| ✓ fade (shell tabs) | 240 ms | 4 s | `ease.in` | — | 60 Hz |
| Unseen dot in / diffuse out | 120 ms / 240 ms | 0 | `ease.out` | — | 60 Hz |
| Bell wobble | 400 ms (2 × ±12°) | rate limit 2 s | `spring.soft` | — | 60 Hz |
| Rename morph / commit | 120 ms / 120 ms | 0 | `ease.out` | — | 60 Hz |
| Drag gap / drop settle | ~180 ms / 160 ms | 4 px threshold | `spring.snap` / `ease.out` | — | 60 Hz |
| Overflow scroll | 200 ms | 0 | `ease.in-out` | — | 60 Hz |
| Confirm popover in / out | 160 ms / 120 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Owner change flash | 200 ms crossfade + 400 ms edge | 0 | `ease.out` | — | 60 Hz |
| **Reduced motion** | no edge pulses, wobbles, nudges or slides; status glyphs swap with an 80 ms crossfade; running = static `◌` still frame ([loaders.md](loaders.md) §2.6); insert/remove/reorder instant; unseen dot and bell glyph still appear (information) | — | linear | none | 1 Hz (elapsed only) |

## 7. Variants
### Free (4)
| Name | Idea | Surfaces |
|---|---|---|
| `terminal-tabs/strip` | Baseline (anatomy above): owner glyph, status slot, title, hover close, overflow list | desktop, ide |
| `terminal-tabs/minimal` | Title + status slot only; agent tabs marked by a 2 px top border in the agent tone instead of a glyph | ide (dense) |
| `terminal-tabs/sidebar-list` | Vertical list in a dock: one row per session with status, elapsed, cwd; good for 10+ sessions (Paradev-style rows, note 03) | desktop, ide |
| `terminal-tabs/tui` | One-row `1:zsh* 2:◆cargo⠋ 3:✕101` tmux-like strip; Alt+N jumps | terminal |

### Pro (12)
| Name | One-line idea |
|---|---|
| `terminal-tabs/edge-pulse` | Full OpenCode-style edge choreography: running travel, completion pulse, failure flash, unread glow (note 08, MIT) |
| `terminal-tabs/grouped` | Tabs grouped under owner headers "Agent · 3" / "You · 2" with collapsible groups |
| `terminal-tabs/thread-linked` | Each agent tab links to the thread and tool call that opened it; click the chip to jump (Zed terminal threads idea, note 03) |
| `terminal-tabs/history-scrubber` | Per-tab command history rail; click a command to jump the scrollback there (Devin jump-in-time idea, note 03) |
| `terminal-tabs/split-grid` | Tabs can tile into a 2 × 2 grid with per-cell status headers (Orca splits idea, note 03) |
| `terminal-tabs/exit-ribbon` | Thin bottom ribbon per tab coloured by the last N exit codes (green/red ticks) |
| `terminal-tabs/sparkline` | Tiny output-rate sparkline under each running tab's title |
| `terminal-tabs/agent-picker` | `+ ⌄` menu with provider logos to start an external agent CLI in a new tab (multi-harness launcher, note 03) |
| `terminal-tabs/pinned` | Pinned tabs shrink to icon-width at the strip start and never close by ⌘W |
| `terminal-tabs/peek` | Hovering an inactive tab for 500 ms shows a live 8-line preview card |
| `terminal-tabs/send-to-agent` | Select output in a user tab → "Send to agent" chip attaches it to the composer |
| `terminal-tabs/ascii-frame` | Box-drawn tab frames `╭─ cargo test ⠋ ─╮` for terminal-look apps |

## 8. Edge cases
- **Many tabs (30+)**: overflow list is searchable (type to filter); tabs needing attention sort first
  in the list; the strip keeps min width 96 px per tab (max 200 px).
- **Long titles / paths**: middle-truncate paths (`~/…/api/src`), end-truncate commands; full title in
  tooltip and accessible name.
- **Rapid exit/restart** (watchers, `cargo watch`): status changes faster than 400 ms are coalesced; the
  completion pulse runs at most once per 2 s per tab.
- **Bell storms**: rate limit 1 wobble / 2 s; > 10 bells / 10 s auto-mutes that tab's bell with a tooltip
  "Bells muted (too many)".
- **Agent closes its own tab**: removed without confirm; the transcript keeps the command block.
- **User closes an agent tab while the agent waits on it**: confirm copy says so; on close the agent's tool
  call receives `Killed` (the app reports it) and the tool row shows "Stopped by you".
- **Agent wants to type into a user tab**: never silently; the app must request control (family G); the
  tab shows the pending request as `AwaitingInput`-style amber with "Agent requests this terminal".
- **Restored workspace**: tabs restore titles, owners and scrollback; processes do not restart; restored
  tabs show `Idle` with a dim "Restored" tooltip, never fake `Running`.
- **Remote / sandbox loss**: all tabs go `Lost` together; one banner in the panel header rather than
  N toasts.
- **RTL**: strip order mirrors; owner glyph stays at inline-start; terminal content stays LTR.
- **Tiny sizes**: at 20 px strips, status slot + 8 ch titles; owner moves into the tooltip.
- **Touch**: tabs ≥ 44 px tall in a horizontal scroller; long-press opens the tab menu (rename, close,
  move); close ✕ always visible.
- **Slow networks**: `Unseen` and status come from the PTY host's events; stale status (> 5 s since last
  heartbeat) shows the glyph at 60 % with "last update 12s ago".
- **Very fast commands** (< 150 ms): no loader, no completion pulse; only an exit badge if non-zero.

## 9. Accessibility
- Strip: `role="tablist"` labelled "Terminals"; tabs `role="tab"` with `aria-selected`, `aria-controls` the
  grid panel. Accessible name composes owner, title and status: "Agent terminal, cargo test, running,
  1 minute 2 seconds" / "Your terminal, zsh, failed with exit code 101, unread output".
- Close button: "Close cargo test"; confirm popover is an `alertdialog` with initial focus on **Keep
  running**.
- Live region (polite): agent tab opened ("Agent opened terminal: cargo test"), finished, failed (with
  code); **assertive** only for `AwaitingInput` on a tab the user owns. Bells are announced as "Bell in
  dev-server" at most once per 10 s.
- Contrast: status glyphs ≥ 3:1, titles ≥ 4.5:1; failure carries the code as text; owner carries a glyph
  plus the name in the accessible label — never colour alone.
- Keyboard: §4 global keys; reorder via ⌘/Ctrl+Shift+←/→ announces "Moved to position 3 of 5".
- F6 lands on the active terminal grid; Shift+Tab from the grid reaches the strip.
- Reduced motion: §6 last row.

## 10. Props sketch
```rust
pub struct TerminalTabs<'a> {
    pub tabs: &'a [TerminalTab],
    pub active: Option<TabId>,
    pub variant: TerminalTabsVariant,        // Strip | Minimal | SidebarList | Tui | Pro(..)
    pub min_tab_px: f32,                     // 96
    pub max_tab_px: f32,                     // 200
    pub elapsed_after: Duration,             // 10 s
    pub success_hold: Duration,              // 4 s
    pub bell: BellPolicy,                    // Glyph (default) | GlyphAndSystemAttention | Muted ; rate 2 s
    pub close_confirm: CloseConfirm,         // WhenRunning (default) | Always | Never
    pub undo_close: Option<Duration>,        // 6 s, user tabs only
    pub allow_user_agent_tabs: bool,
    pub labels: TerminalTabLabels,
    pub on_new: Callback<NewTabRequest>,
    pub on_select: Callback<TabId>,
    pub on_rename: Callback<(TabId, Option<SharedString>)>, // None = revert to auto title
    pub on_reorder: Callback<(TabId, usize)>,
    pub on_split: Callback<(TabId, SplitSide)>,
    pub on_close: Callback<(TabId, CloseMode)>,             // Stop | Background
    pub on_take_over: Callback<TabId>,                      // routes to acting-indicator / family G
    pub motion: MotionLevel,
}

pub struct TerminalTab {
    pub id: TabId,
    pub owner: TabOwner,                     // Agent(AgentId) | User | Teammate(PersonId)
    pub title: TabTitle,                     // { user: Option<_>, agent: Option<_>, osc: Option<_>, process: _ }
    pub process: TabProcess,                 // Starting | Idle | Running{since} | AwaitingInput{prompt}
                                             // | Exited{code} | Killed{signal} | Backgrounded | Lost | SpawnFailed{err}
    pub flags: TabFlags,                     // unseen, bell, pinned, restored
    pub call: Option<CallId>,                // tool call that opened it (agent tabs); its AgentState drives status
}
```
Mapping from the agent: an agent tab's `Running` mirrors its `Shell` tool call
([terminal-output.md](terminal-output.md) `TermState`); `AgentState::AwaitingApproval` for a command
in that tab shows `AwaitingInput`-style amber with "Needs approval" text.

## 11. Related components and references
**Related:** [terminal-output.md](terminal-output.md) (grid rendering, exit badge rules, ok exit codes) ·
[acting-indicator.md](acting-indicator.md) (owner marker, take over) · [background-jobs.md](background-jobs.md)
(backgrounded sessions) · [workspace-chrome.md](workspace-chrome.md) (bottom dock, split, F6) ·
[loader-glyph.md](loader-glyph.md) · [elapsed-timer.md](elapsed-timer.md) · [side-panels.md](side-panels.md)
(status bar terminal count).

| Reference | What we take | Licence note |
|---|---|---|
| Research note 05 §6 TerminalPanel proposal | state list incl. agent-owned vs user-owned | Our own proposal |
| OpenCode `tab-pulse.tsx` (note 08 §1.5) | 2.8 s running pulse, 450 ms attack, 1.2 s completion pulse, 800 ms edge flash, unread glow | MIT — portable with attribution (re-check before porting) |
| Zed terminal threads (note 03) | terminal sessions as agent-panel tabs | Mixed GPL / Apache by crate — study only by default |
| Devin Shell tab (note 03) | command history, jump in time | Proprietary — reference only |
| Orca splits, Paradev session rows, Claude desktop Ctrl+\`, Conductor per-workspace terminal (note 03) | splits, row list, toggle key, per-workspace sessions | Proprietary / not checked — reference only |
| Multi-harness launcher logos (note 03, Paradev / Emdash) | agent picker in the new-tab menu | Reference only |
| Codex CLI "You ran" wording (note 08 via terminal-output.md) | owner wording | MIT / Apache-2.0 — portable with attribution |
| Warp tabs / blocks (note 03) | — | AGPL — study only |

## Plan #27 implementation — terminal tabs

Implemented in `oxyn-core::shell::TermTabs`, `oxyn-dioxus::TerminalTabs` and `oxyn-gpui::TermTabsEl`.

### 27.1 Purpose
Several shells in one pane, with a hint when a hidden one prints.

### 27.2 State machine
```
add ─▶ new tab active   close(id) ─▶ neighbour active   select(id) ─▶ active, activity cleared
output(id) on a background tab ─▶ activity   dblclick ─▶ renaming ─Enter / blur─▶ renamed │ Escape
```

### 27.3 States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| active | joined to the output | ← → , Ctrl+Tab | `tab` selected |
| activity | blue pulsing dot | — | “new output” in the name |
| exited | exit code chip (green 0 / red) | — | text |
| renaming | inline field | Enter / Escape | labelled |

### 27.4 Micro-interactions
Activity dots pulse; the prompt caret blinks.

### 27.5 Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Activity pulse | 1.4 s loop | ease-in-out | static |
| Caret | 1 s steps | — | static |

### 27.6 Variants
Free: tabs, add, close.

**Oxyn plus:** rename, reorder, activity markers, exit codes, Ctrl+Tab cycling.

### 27.7 Edge cases
Closing the last tab shows “No terminals”. Renaming to blank keeps the old title.

### 27.8 Accessibility
`tablist` / `tab` / `tabpanel`, roving tabindex, labelled close buttons.

### 27.9 Props sketch
```rust
TerminalTabs { now_ms, titles: Vec<String>, activity_ms: u64, height }
```

### 27.10 Related components and references
`terminal-output.md`, `workspace-chrome.md`.
