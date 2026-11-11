**Status:** draft v0.1 (2026-11-12).

# Spec: workspace chrome (family J overview · panel & resizer model · focus regions)

Family J of [../component-taxonomy.md](../component-taxonomy.md): "Terminal tabs, side panels, file tree,
status bar, MCP / plugin list (9 states), notification centre, ⌘K palette with AI answers, tray popover,
hotkey launcher, floating HUD, notch status" — target **40+ variants**. Same depth as the worked example
[composer-send-control.md](composer-send-control.md). This file owns three shared things every family J
component builds on, and specifies them to the 10-point standard of [README.md](README.md):

1. the **panel & resizer model** (split, dock, collapse, persist sizes) — itself a component, `panel-split`;
2. **focus regions and F6 cycling**;
3. the **family catalogue** (variant table for all family J components).

Component specs: [terminal-tabs.md](terminal-tabs.md) · [side-panels.md](side-panels.md) (side panels,
file tree, status bar) · [mcp-plugin-list.md](mcp-plugin-list.md) ·
[command-palette.md](command-palette.md) (⌘K palette, notification centre) ·
[desktop-overlays.md](desktop-overlays.md) (tray popover, hotkey launcher, floating HUD, notch / island).
All numbers are Oxyn **proposals** unless a source is cited next to them.

## 1. Purpose
Agent apps are no longer one chat column. A coding agent window holds a thread list, a transcript, a
diff, a terminal, a browser preview, a file tree and a status bar at once; the user moves between them
dozens of times an hour and expects the layout to **stay where they left it**. What we saw: drag-and-drop
pane layout with chat, diff, browser, terminal, file, plan, tasks and subagent panes, pop-out to a window
and **minimize to a floating corner tile** (Claude desktop, research note 03 §1G); a resizable 6-area
layout (Verdent, note 03); split panes right or down, drag tabs between panes, resize dividers, layout
remembered per workspace, single pane with a scrolling tab strip on phones, Cmd/Ctrl+B to swap chat and
the secondary view (Hoplite, note 10); a sidebar resizable by dragging its right edge and collapsible
from a top-left button (Hoplite, note 10); a rail that **snaps to icon-only under ≈160 px with labels
fading in ≈0.18 s** [R046]; icon-anchored collapse where labels clip and icons don't move [R016];
"sidebar and side panel open/close animate without layout jumps" (Antigravity, note 08). The user is
trying to **arrange the workspace once, move through it by keyboard, and never lose a panel**.

## 2. Panel & resizer model
```
 Window
 ┌ title / tab bar ─────────────────────────────────────────────────────────────────┐
 │┌ Dock.Left ─┐┊┌ Center (split tree) ──────────────────────────┐┊┌ Dock.Right ──┐ │
 ││ threads    │┊│ ┌ PaneGroup A (tabs) ─────┐┊┌ PaneGroup B ──┐ │┊│ file tree    │ │
 ││ (rail 48)  │┊│ │ transcript              │┊│ diff          │ │┊│ / MCP list   │ │
 ││            │┊│ └─────────────────────────┘┊└───────────────┘ │┊│              │ │
 │└────────────┘┊└────────────────────────────────────────────────┘┊└──────────────┘ │
 │              ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈  ← resizer (8 px hit, 1 px line)│
 │ ┌ Dock.Bottom: terminal tabs ───────────────────────────────────────────────────┐ │
 │ └───────────────────────────────────────────────────────────────────────────────┘ │
 └ status bar (side-panels.md) ─────────────────────────────────────────────────────┘
   Floating: popped-out windows, corner tiles (minimized panes), HUD (desktop-overlays.md)
```
**Data model (proposed).** A layout is three docks plus a center split tree:
`Node = Split { axis: H|V, children: Vec<(Node, Size)> } | Group { tabs: Vec<PaneId>, active }`.
A `Size` is a **fraction** of the parent plus a **min px** and optional **max px**; fractions survive window
resizes, min px protect content.

| Rule | Value (proposed) |
|---|---|
| Resizer | 1 px line `color.border.subtle`; hit area 8 px (centred), 12 px on touch; cursor `col-resize` / `row-resize` |
| Minimums | left dock 200 px (rail 48 px), right dock 240 px, bottom dock 120 px, center group 320 px wide / 160 px tall |
| Snap-to-rail | a left / right dock dragged under 160 px snaps to a 48 px icon rail ([R046]); dragged under 96 px (or past the rail by 48 px) it **collapses** to hidden |
| Snap points | 25 / 33 / 50 / 67 / 75 % of the parent with an 8 px magnet; holding ⌥ / Alt disables magnets |
| Default sizes | left 260 px, right 320 px, bottom 33 % of window height (clamped 120–60 %) |
| Reset | double-click a resizer → that split returns to its default |
| Collapse | dock toggles: ⌘/Ctrl+B left (proposed, matches Hoplite's Cmd+B role, note 10), ⌘/Ctrl+⌥+B right, ⌘/Ctrl+J bottom (proposed); collapsed docks remember their last size |
| Overflow | when the window cannot fit every minimum: right dock becomes an **overlay** (slides over center with scrim) under 900 px window width; left dock becomes overlay under 720 px; under 600 px the center collapses to one group with a scrolling tab strip (Hoplite phone layout, note 10) |
| Split | drag a tab to a group's edge (outer 25 % of the group) → drop indicator shows the half it will take; ⌘/Ctrl+\\ splits right (proposed); max depth 4 |
| Pop out / minimize | a pane can pop out to its own window, or minimize to a 160 × 96 px **corner tile** floating over the center (Claude desktop idea, note 03); tile keeps live status |
| Persistence | `LayoutState` (versioned JSON) saved per **workspace**, debounced 500 ms after a drag ends or a dock toggles; restored before first paint; sizes clamped to the current window; unknown pane kinds dropped with a toast "1 panel from a newer version was hidden" |
| Zero-jump rule | opening / closing a dock never reflows the transcript's scroll anchor (the anchored line stays at the same y ± 1 px) |

## 3. State machine
Per dock / split (resizer + collapse):
```
                      pointer enters hit area (hover delay 300 ms)
   Rest ──────────────────────────────────────────▶ ResizerHover (line thickens 1→2 px, accent)
    ▲  ◀──────────── pointer leaves ───────────────────┘   │ pointer down
    │                                                       ▼
    │                                   Dragging{size} ──crosses 160 px──▶ SnapPreview(Rail) ──release──▶ Rail
    │                                     │   │                                   └─ back above 160 ─▶ Dragging
    │                                     │   └─crosses collapse threshold──▶ SnapPreview(Hidden) ──release──▶ Collapsed
    │                         release ◀───┘ (Esc cancels → size before drag)
    │◀── persisted (500 ms debounce) ── Settled
 Expanded ⇄ Rail ⇄ Collapsed        (toggle shortcut / button: Expanded ⇄ Collapsed; rail via drag or menu)
 Expanded ── window narrows below breakpoint ──▶ Overlay(closed) ── toggle ──▶ Overlay(open, scrim) ── Esc / outside ──▶ Overlay(closed)
 any ── window widens above breakpoint + 48 px hysteresis ──▶ restored mode
 Restoring (first paint from LayoutState) ──clamp ok──▶ Rest | ──clamp failed / corrupt──▶ DefaultLayout + toast
 Disabled{reason} (app locks the layout, e.g. kiosk / presentation) — resizers inert, tooltip says why
```
Pane drag (tab → split):
```
 Idle ─pointer down on tab + 4 px──▶ DraggingTab ──over group edge──▶ DropPreview{group, side}
                                       │ over another tab strip ──▶ DropPreview{strip, index}
                                       │ outside window (desktop) ──▶ DropPreview{new window}
                                       └ Esc ──▶ Idle (tab flies back)          release ──▶ Splitting ──▶ Idle
```

## 4. States table
| State | Shows | Primary action | Secondary | Keyboard |
|---|---|---|---|---|
| `Rest` | 1 px line | — | — | resizer focusable via F6-region tab order |
| `ResizerHover` | line 2 px `accent` after 300 ms; resize cursor immediately | drag | double-click reset | — |
| `ResizerFocused` | 2 px line + 2 px focus ring around the hit area | arrow resizes 8 px (Shift 64 px) | Home = min, End = max, Enter = collapse/expand, Esc = cancel to start | ←/→ (or ↑/↓) |
| `Dragging{size}` | live size; pixel readout tooltip "320 px · 33 %" after 400 ms still | release | ⌥/Alt free drag (no magnets) | Esc cancels |
| `SnapPreview(Rail/Hidden)` | dock content dims to 60 % and shows the rail / hidden outline | release to commit | keep dragging | Esc |
| `Expanded` | full dock | — | toggle button | shortcut collapses |
| `Rail` | 48 px icons, labels hidden, badges kept | click icon opens that section as a flyout | drag edge to expand | shortcut expands |
| `Collapsed` | nothing; a 4 px edge hot-zone (desktop) shows a "›" handle on hover | expand | — | shortcut expands |
| `Overlay(open)` | dock over center + 20 % scrim | — | close | Esc closes, focus returns to the toggle |
| `DropPreview` | translucent accent region (12 % alpha) over the target half, 2 px outline | release | Esc cancel | — |
| `Restoring` | last saved layout painted on first frame (no animation) | — | — | — |
| `DefaultLayout` | default layout + toast "Layout reset · Undo" (6 s) | undo | — | — |
| `Disabled{reason}` | resizers 1 px, no hover | — | tooltip reason | not focusable |

## 5. Micro-interactions
- **Hover intent.** The cursor changes immediately; the line thickens only after 300 ms so mouse
  travel across a resizer never flickers it.
- **Drag.** Size follows the pointer 1:1, painted every frame; no easing while dragging. Content inside
  the resizing panel reflows live; heavy panels (diff, terminal grid) may **freeze-scale** their last frame
  during drag and reflow on release if a reflow costs > 8 ms (proposed).
- **Magnets.** Within 8 px of a snap point the size jumps to it with a 1 px tick on the line and, on
  supported trackpads, a haptic tick (macOS only, opt-in).
- **Snap to rail** ([R046], [R016]). Labels clip from the inline-end and fade over 180 ms while icons stay
  put (icon-anchored); expanding reverses with labels cascading 20 ms apart (cap 8).
- **Collapse / expand via shortcut.** Width animates with `spring.snap`; the center's content does not
  slide — its scroll anchor holds (zero-jump rule) and only its width changes.
- **Overlay open.** Dock slides in 16 px + fade with scrim fade; close reverses faster.
- **Tab drag to split.** Ghost tab follows the pointer at 0.9 scale with 8 px shadow; the drop region
  fades in (120 ms) and morphs between halves with `spring.snap`; on release the new split opens from the
  drop edge.
- **Double-click reset.** Size glides to default with `spring.snap`.
- **Restore.** Never animated: the saved layout is the first frame (Hoplite's "Paint from your saved
  session" principle, note 10).
- **Corner tile minimize.** Pane scales into the tile's position (`dur.travel` 240 ms); the tile keeps a
  live loader dot and a title; click restores along the reverse path.

## 6. Timing table
| Transition | Duration | Delay | Easing / spring | Stagger | Frame rate |
|---|---|---|---|---|---|
| Resizer hover highlight | 120 ms in / 120 ms out | 300 ms | `ease.out` | — | 60 Hz |
| Drag follow | 0 (1:1) | — | none | — | display rate |
| Pixel readout tooltip | 120 ms fade | 400 ms still | `ease.out` | — | 60 Hz |
| Magnet tick | 80 ms line flash | — | `ease.out` | — | 60 Hz |
| Snap to rail: labels out / in | 180 ms ([R046] ≈0.18 s) | 0 | `ease.out` | 20 ms, cap 8 (in only) | 60 Hz |
| Collapse / expand (shortcut) | ~220 ms settle | 0 | `spring.snap` | — | 60 Hz |
| Overlay open / close | 200 ms / 160 ms; scrim 160 / 120 ms | 0 | `ease.out` / `ease.in` | — | 60 Hz |
| Drop region in / morph | 120 ms / ~180 ms | 0 | `ease.out` / `spring.snap` | — | 60 Hz |
| Split open on drop | 200 ms | 0 | `spring.snap` | — | 60 Hz |
| Tab fly-back on cancel | 200 ms | 0 | `ease.in-out` | — | 60 Hz |
| Corner-tile minimize / restore | 240 ms | 0 | `ease.in-out` | — | 60 Hz |
| Layout persist | debounce 500 ms | after release | — | — | — |
| Breakpoint hysteresis | 48 px | — | — | — | — |
| **Reduced motion** | all size changes instant; labels and overlays use an 80 ms opacity crossfade; no ghost scaling (tab ghost is a flat outline); drag still follows 1:1 (it is direct manipulation) | — | linear | none | — |

## 7. Variants (`panel-split`)
### Free (3)
| Name | Idea | Surfaces |
|---|---|---|
| `panel-split/hairline` | Baseline: 1 px line, 8 px hit area, accent on hover after 300 ms, docks + center split tree | desktop, ide |
| `panel-split/rail-snap` | Left dock snaps to a 48 px icon rail under 160 px with icon-anchored label fade ([R046]) | desktop, ide |
| `panel-split/tui` | Box-drawing dividers `│ ─ ┼`; resize with Ctrl+arrow in 1-cell steps; collapse to a 1-col marker | terminal |

### Pro (8)
| Name | One-line idea |
|---|---|
| `panel-split/grip` | Visible 3-dot grip at the resizer centre that grows on hover; better discoverability on touch |
| `panel-split/magnetic` | Stronger snap with visible ticks at snap points and a size badge while dragging |
| `panel-split/corner-tiles` | Minimized panes float as live corner tiles that stack (Claude desktop idea, note 03) |
| `panel-split/peek` | Collapsed docks peek on edge hover (300 ms) as a temporary overlay; pin to keep |
| `panel-split/presets` | Named layouts ("Review", "Pair", "Focus") switchable from the palette; morph between them |
| `panel-split/focus-mode` | One shortcut collapses every dock and the status bar; a ghost outline shows where they went |
| `panel-split/liquid` | Resizer drawn as a soft gooey seam that bulges toward the pointer while dragging |
| `panel-split/mobile-sheets` | On mobile, docks become bottom sheets with detents (25 / 50 / 90 %) |

## 8. Focus regions and F6 cycling
Every family J surface registers as a **focus region**. A region is a landmark with a name, a remembered
last-focused element and an entry rule.

| Order (default, proposed) | Region | Landmark role | Entry rule |
|---|---|---|---|
| 1 | Left dock (threads / nav) | `navigation` | last focused row, else the active thread |
| 2 | Center group(s), left → right, top → bottom | `main` (first group) / `region` | last focused element, else the transcript's latest message |
| 3 | Composer | `form` ("Message the agent") | the input, caret at end |
| 4 | Right dock (file tree, MCP list, …) | `complementary` | last focused node |
| 5 | Bottom dock (terminal tabs) | `region` "Terminal" | active terminal's grid |
| 6 | Status bar | `status` container with focusable items (`toolbar` semantics) | first item |
| 7 | Toasts / notification centre (only when present) | `region` "Notifications" | newest toast (matches [checkpoint-timeline.md](checkpoint-timeline.md) F6 to toasts) |

Rules (proposed):
1. **F6** moves to the next region, **Shift+F6** to the previous; wraps. Collapsed and hidden regions are
   skipped; an overlay dock that is closed is skipped; a **rail** is included (its icons are focusable).
2. Laptop keyboards may need Fn for F6, so the palette lists "Focus next region" / "Focus previous
   region" and the app may bind an extra chord (proposed ⌘/Ctrl+⌥+→ / ←). We avoid Ctrl+\` because
   several apps use it to toggle the terminal (Claude desktop, note 03).
3. Entering a region restores its **last focused element**; if that element is gone, its entry rule
   applies. Focus never lands on a resizer by F6 (resizers are reached with Tab inside a region's edge).
4. A **region flash** marks arrival: a 2 px inset outline in `color.focus` on the region for 400 ms, then
   only the element's own focus ring stays. Reduced motion: outline shows for 400 ms with no fade.
5. Modal surfaces (palette, dialogs, approval takeover) trap focus; F6 inside them cycles their own
   sub-regions (e.g. palette input → results → answer pane) or does nothing.
6. Region order follows **visual order** after the user rearranges the layout (re-computed from the split
   tree), so RTL layouts and moved docks cycle in reading order.
7. Screen readers: arriving announces the region name and its live summary once ("Terminal, 3 tabs, 1
   running").

## 9. Family catalogue (variant table)
| Component | Spec | Free | Pro | Total |
|---|---|---|---|---|
| Panel & resizer `panel-split` | this file §7 | 3 | 8 | 11 |
| Terminal tabs `terminal-tabs` | [terminal-tabs.md](terminal-tabs.md) | 4 | 12 | 16 |
| Side panel / pane switcher `side-panel` | [side-panels.md](side-panels.md) | 3 | 6 | 9 |
| File tree `file-tree` | [side-panels.md](side-panels.md) | 3 | 8 | 11 |
| Status bar `status-bar` | [side-panels.md](side-panels.md) | 3 | 6 | 9 |
| MCP / plugin list `mcp-list` | [mcp-plugin-list.md](mcp-plugin-list.md) | 4 | 12 | 16 |
| Command palette `command-palette` | [command-palette.md](command-palette.md) | 3 | 10 | 13 |
| Notification centre `notification-center` | [command-palette.md](command-palette.md) | 3 | 8 | 11 |
| Tray popover `tray-popover` | [desktop-overlays.md](desktop-overlays.md) | 2 | 4 | 6 |
| Hotkey launcher `hotkey-launcher` | [desktop-overlays.md](desktop-overlays.md) | 2 | 4 | 6 |
| Floating HUD `floating-hud` | [desktop-overlays.md](desktop-overlays.md) | 2 | 4 | 6 |
| Notch / island `island` | [desktop-overlays.md](desktop-overlays.md) | 2 | 5 | 7 |
| **Family J** | | **34** | **87** | **121** |

Target in the taxonomy is 40+; we exceed it because the family has twelve distinct components. Launch
(V2.2) ships the free set; Pro rows are the backlog.

## 10. Edge cases (family level)
- **Window too small for the saved layout** (laptop after an external monitor): clamp fractions to
  minimums in dock priority center > bottom > left > right; collapse right first; never delete the saved
  layout — the clamped version is a view, the saved one is restored when space returns.
- **Multi-monitor / DPI change** mid-drag: sizes are stored in logical px; a DPI change ends the drag.
- **Corrupt or future-version `LayoutState`**: fall back to default + "Layout reset · Undo".
- **RTL**: docks mirror (left dock becomes inline-start on the right); resize arrows invert so → always
  moves the resizer visually right; F6 order follows visual reading order.
- **Tiny sizes**: rails at 48 px show 20 px icons with 16 px badges; below 600 px window width there are
  no side-by-side splits.
- **Dense IDE**: 22 px tab strips, 6 px resizer hit area (still ≥ 8 px on touch-capable devices).
- **Touch**: 12 px hit area, long-press 400 ms on a tab to start drag; overlay docks swipe closed.
- **Slow networks / remote panes**: a pane whose content is remote shows its skeleton
  ([skeletons.md](skeletons.md)) inside the restored frame; layout never waits on data.
- **Very fast agents** opening many panes (diff, terminal, browser in 1 s): panes open into their
  dock's tab strip, not as new splits; an agent never creates a split without user permission (proposed).
- **Agent-opened pane while user types**: the pane opens unfocused with a 1-pulse attention dot on its
  tab; focus stays with the user.

## 11. Accessibility (family level)
- Resizer: `role="separator"`, `aria-orientation`, `aria-valuenow/min/max` in px, `aria-controls` the
  panel, label "Resize file tree"; Enter toggles collapse and announces "File tree collapsed".
- Docks: landmarks per §8; collapse buttons have `aria-expanded`.
- Contrast: resizer line ≥ 3:1 against both neighbours when hovered or focused (1 px resting line may be
  subtle; it is not the only cue — the cursor and focus ring are).
- Every drag has a keyboard path (resizer arrows, "Move tab to new split right" in the tab menu).
- Live region: layout resets and pane pop-outs are announced politely; nothing is announced during drag.
- Reduced motion: §6 last row.

## 12. Props sketch
```rust
pub struct Workspace {
    pub layout: LayoutState,                 // docks + center split tree, versioned
    pub regions: Vec<FocusRegion>,           // computed from layout in visual order
    pub persist: LayoutPersist,              // PerWorkspace(WorkspaceId) | None ; debounce 500 ms
    pub breakpoints: Breakpoints,            // overlay_right: 900, overlay_left: 720, single: 600, hysteresis: 48
    pub locked: Option<SharedString>,        // Disabled{reason}
    pub pane_status: PaneStatusFn,           // PaneId -> PaneBadge, derived from each pane's AgentState
    pub motion: MotionLevel,
    pub on_layout_change: Callback<LayoutState>,
}

pub struct LayoutState { pub version: u16, pub left: Dock, pub right: Dock, pub bottom: Dock, pub center: Node }
pub struct Dock { pub mode: DockMode /* Expanded | Rail | Collapsed | Overlay */, pub size: Size, pub tabs: Vec<PaneId>, pub active: Option<PaneId> }
pub enum Node { Split { axis: Axis, children: Vec<(Node, Size)> }, Group { tabs: Vec<PaneId>, active: PaneId } }
pub struct Size { pub fraction: f32, pub min_px: f32, pub max_px: Option<f32> }

pub struct PanelSplit {
    pub variant: PanelSplitVariant,          // Hairline | RailSnap | Tui | Pro(..)
    pub hit_px: f32,                         // 8 (12 touch)
    pub snaps: SmallVec<[f32; 5]>,           // 0.25, 0.33, 0.5, 0.67, 0.75
    pub rail_threshold_px: f32,              // 160
    pub collapse_threshold_px: f32,          // 96
    pub label: SharedString,                 // "Resize file tree"
}

pub struct FocusRegion { pub id: RegionId, pub name: SharedString, pub landmark: Landmark, pub last_focus: Option<FocusId>, pub summary: Option<SharedString> }
```

## 13. Related components and references
**Related:** every family J spec above · [composer.md](composer.md) (motion tokens) ·
[composer-status-strip.md](composer-status-strip.md) · [background-jobs.md](background-jobs.md) (jobs
panel, corner tile) · [acting-indicator.md](acting-indicator.md) (agent-opened panes, follow) ·
[skeletons.md](skeletons.md) · [checkpoint-timeline.md](checkpoint-timeline.md) (F6 to toasts) ·
[diff-viewer.md](diff-viewer.md) and [terminal-output.md](terminal-output.md) (pane contents).

| Reference | What we take | Licence note |
|---|---|---|
| Claude desktop panes (notes 03, 05) | pane kinds, pop-out, minimize to corner tile, ⋮ overflow | Proprietary — reference only |
| Verdent 6-area layout (note 03) | resizable multi-area layout | Proprietary — reference only |
| Hoplite (note 10) | split right/down, drag tabs between panes, layout per workspace, phone single pane, Cmd+B, sidebar drag/collapse, "Paint from saved session" | Proprietary — reference only |
| @_adamboyle sidebar [R046] | snap to icon-only ≈160 px, labels ≈0.18 s cascade | Study only |
| @keviduk sidebar [R016] | icon-anchored collapse | Study only |
| Antigravity (note 08) | open/close without layout jumps | Proprietary — reference only |
| Arc UI Resizable panels (note 12) | catalogue presence only | Paid / unclear — study only |
| Astryx Resizable (note 16 [R103]) | presence of a Resizable primitive | "Open source (Meta)", licence not recorded in licensing.md — study only until checked |
| Zed panels / docks (note 03) | dock model familiarity for GPUI users | Mixed GPL / Apache by crate — study only unless the crate is Apache-2.0 (check per crate) |
| Rune `SplitView` app shell (note 07) | an agent-chat split shell exists in Rust | MIT — portable with attribution; re-check before porting |
| Warp (note 03) | — | AGPL — study only |
