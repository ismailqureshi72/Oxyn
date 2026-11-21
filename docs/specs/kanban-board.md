**Status:** built v0.1 (2026-11-19, S12: Arc screens I).

# Spec: Kanban board (`kanban-board`)

Family: Arc screens (pages at `/screens/kanban-board/` and `/full`). Reference: the kanban board screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, workspace and project is fictional; dates are absolute ISO dates.
Components: `KanbanBoardScreen` (Dioxus) · `KanbanBoardEl` (GPUI), rules in `oxyn_core::kit::arcscr`.

## 1. Purpose
Move a cycle's issues from Backlog to Done and see the cycle's progress.

## 2. Parts
The Arc frame (shared by the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Brindle: My issues 4, Current cycle (count), All issues; **Projects** (Storefront v2, Mobile 5.1, Payments API, Design system, each with a toned glyph and count). Context card: **Cycle 12 · n days left**, a progress bar of done points, "6 of 78 points · Day 4 of 14" (live) |
| Header | Brindle › Projects › Storefront v2, "18 issues · 79 points"; Share (copies), **New issue** (opens the field in Todo) |
| Second row | Filter issues (text, key or label; `/` hint; Escape clears), **Priority** and **Labels** menus with a removable chip, Status / Assignee grouping |
| Columns | Backlog (dashed circle), Todo (circle), In progress (amber half, limit 4), In review (blue, limit 3), Done (green check): name, count (`4/4`; rose with a title when over the limit), + new issue |
| Card | Key (`STF-208`), linked pull request with the GitHub mark, assignee avatar; title (two lines); priority bars (or a rose ! for Urgent), label chip with a hue dot, sub-issues ring `3/5`, comments, due date (ISO; amber within two days, rose when overdue) |
| Group by assignee | One column per person with their open issues and each issue's status glyph |

## 3. Behaviour
Drag a card with the pointer and drop it on any column (the column outlines while it is the target). With the keyboard, Space or Enter picks a card up (outlined, aria-pressed), arrows move it across columns or within one, Escape puts it down; Alt + arrows move without picking. New issue validates (at least 3 characters) and adds to the end of the column. Moving cards to Done advances the cycle card.

## 4. Variants
States: Data · Loading · Empty ("No issues in this cycle" with New issue) · **Dragging** (a card lifted over In review) · **Filtered** (Backend) · **By assignee**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Columns are labelled regions; cards are buttons with aria-roledescription Draggable issue and a label that says how to move them. Moves report in a polite live region (`STF-219 moved to In review`, `STF-198 is 2 of 2 in Done`). The sidebar is a nav landmark; the open row has aria-current=page; the account menu and every menu close on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it), columns scroll sideways. Under 640 px header actions become icon buttons, the meta line hides and grids drop to one column. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`KanbanBoardScreen` takes `data: Option<Vec<Issue>>` (`oxyn_core::kit::arcscr`): id, title, column, label, priority, points, assignee, due day, comments, pull request, sub-issues. `None` shows the sample for the state.

## 8. Known gaps
No backend; a new issue is assigned to the signed-in person. In GPUI a column shows its first five cards and "n more", labels filter from chips, and New issue and Share are said back (no inline field).

## 9. GPUI
`KanbanBoardEl` draws one state (`.state()`, `.extra()`). `ScreenView` (oxyn-gpui, `kit::screenview`) runs it live as part of the **Arc screens I** set, driven by `oxyn_core::kit::screenui::ScreenUi` and its `ArcUi`: `ArcUi` keeps the board, the picked card and the label filter: press a card (`kb-pick-<n>`), then drop it on a column header or use Alt + arrows (`kb-left|right|up|down`); drag a card onto a column (`kb-drop-<col>`); `kb-label-<n>`, `kb-clear`, `kb-group`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S12 Arc I).
