**Status:** built v0.1 (2026-11-19, S12: Arc screens I).

# Spec: Gantt timeline (`gantt-timeline`)

Family: Arc screens (pages at `/screens/gantt-timeline/` and `/full`). Reference: the gantt timeline screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, workspace and project is fictional; dates are absolute ISO dates.
Components: `GanttTimelineScreen` (Dioxus) · `GanttTimelineEl` (GPUI), rules in `oxyn_core::parity::arcscr`.

## 1. Purpose
Plan a release across phases and change dates by dragging, with dependencies kept honest.

## 2. Parts
The Arc frame (shared by the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Northgate: Timeline, List, Updates 3; **Milestones** (diamond in the phase hue, name, ISO date; a press focuses it); **Saved views** (My tasks, At risk, Next two weeks) |
| Header | Northgate › Harbor 2.0 › Timeline, "18 tasks · 45% done"; Share, **New task** (5 days from today in Build) |
| Second row | Filter tasks, **Owner** menu, Day / Week / Month, **Today** (scrolls to today) |
| Task list (262) | "18 tasks", phases (chevron, name, count; collapse), rows (avatar, name, end date ISO; a rose ! when it starts too early) |
| Timeline | Sticky header with months (`2026-11`) and day numbers (every day, Mondays or the 1st and 15th by zoom), weekend shading, a **Today** pill and line; phase summary bars with progress; bars tinted by phase (Discovery teal, Design magenta, Build blue, Launch amber) with a progress fill, name and percent (names outside short bars); milestones as diamonds with name and date; dependency arrows (grey; rose when the successor starts before its predecessor ends) |
| Footer | The drag hint, **Undo** (disabled with nothing to undo) and **Reset** |

## 3. Behaviour
Drag a bar to move it, its left edge to change the start or its right edge to change the end; a tooltip shows the new ISO dates while dragging and the change lands on release (at least one day long; milestones only move). Focused bars move with ← / → and change their end with Shift + ← / →. Every change is undoable (40 steps); Reset returns to the sample plan and can itself be undone. Zoom sets 36, 15 or 6 px a day.

## 4. Variants
States: Data · Loading · Empty ("No tasks on the timeline") · **Dragging** (Payments with saved cards four days later, with its tooltip) · **Month view**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Bars are buttons named with their dates and progress and the keys that move them; milestones name their day; phases are buttons with aria-expanded; changes report in a polite live region (`Offline sync · 2026-11-09 → 2026-11-22 · 14 days`). The sidebar is a nav landmark; the open row has aria-current=page; the account menu and every menu close on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it), the task list narrows to 150 px. Under 640 px header actions become icon buttons, the meta line hides and grids drop to one column. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`GanttTimelineScreen` takes `data: Option<Vec<Bar>>`: id, name, phase, start day, length (0 for a milestone), percent done, owner and the ids it waits on. `None` shows the sample plan.

## 8. Known gaps
No backend. In GPUI the timeline draws no dependency arrows (a rose ! in the task list marks a task that starts too early), a dragged bar drops on whole days without a live tooltip, phases do not collapse and Today does not scroll.

## 9. GPUI
`GanttTimelineEl` draws one state (`.state()`, `.extra()`). `ScreenView` (oxyn-gpui, `parity::screenview`) runs it live as part of the **Arc screens I** set, driven by `oxyn_core::parity::screenui::ScreenUi` and its `ArcUi`: `ArcUi` keeps the plan with undo, the picked bar and the zoom: press a bar (`gt-pick-<id>`), then `gt-earlier|later|shorter|longer`, or drag it onto a day column (`gt-at-<day>`); `gt-zoom-<n>`, `gt-undo`, `gt-reset`, `gt-today`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S12 Arc I).
