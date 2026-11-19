**Status:** built v0.1 (2026-11-19, S12: Arc screens I).

# Spec: Project board (`project-board`)

Family: Arc screens (pages at `/screens/project-board/` and `/full`). Reference: the project board screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every person, workspace and project is fictional; dates are absolute ISO dates.
Components: `ProjectBoardScreen` (Dioxus) · `ProjectBoardEl` (GPUI), rules in `oxyn_core::parity::arcscr`.

## 1. Purpose
Run one project: status, numbers, milestones and the tasks under them, with the team and links beside.

## 2. Parts
The Arc frame (shared by the S12 screens): a 248 px sidebar (workspace mark and name, Hide sidebar, Search ⌘K, nav rows with counts, labelled groups, a context card, the account row with name and **email** and an up/down chevron that opens the screen's account menu upward), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional 328 px right card in the gutter.

| Part | Look |
|---|---|
| Sidebar | Marlow: Home, My tasks 5, Inbox 3; **Projects**, each with the next milestone's ISO date and name and a progress ring (the open project's ring is live) |
| Header | Projects › Returns portal; Share, **New task** (adds to the open milestone), side-panel toggle |
| Tabs | Overview · Tasks (open count) · Activity |
| Head | Teal project glyph and name, the summary, a **status** menu (On track green, At risk amber, Off track rose), lead, the ISO date span and an avatar stack |
| Stats | Complete % with a bar, Open tasks with "n in progress or review", Due this week with "n overdue" (rose), Until launch with the ISO launch day |
| Milestones | Five steps with a progress line (green when done), name and "2026-11-20 · 25%"; the selected one picks the task list |
| Tasks | Status glyph as a checkbox (complete or reopen; done tasks strike through), title, comments, priority, due date (rose when overdue), assignee |
| Right card | Team: avatar, name (· lead), role, an open-load bar and count. Links: Figma, GitHub and Stripe marks, title, line, Copy (turns to a check) |

## 3. Behaviour
Completing or reopening a task moves Complete, Open, Due this week, the milestone progress, the sidebar ring and the team's load. The Tasks tab groups every task by milestone; Activity lists who did what with ISO dates and times.

## 4. Variants
States: Data · Loading · Empty (no tasks; zeros) · **At risk** (status At risk with a note on what is late) · **Activity**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
Tabs and milestones are tablists; tasks are checkboxes named "Complete …" or "Reopen …"; the status is a menu of menuitemradio closed by Escape; the panel toggle is a pressed button. The sidebar is a nav landmark; the open row has aria-current=page; the account menu and every menu close on Escape.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button (Escape or the scrim closes it) and the right card hides. Under 640 px header actions become icon buttons, the meta line hides and grids drop to one column. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`ProjectBoardScreen` takes `data: Option<Vec<Task>>`: title, milestone, state (0 todo … 3 done), priority, due day, comments, assignee. `None` shows the sample.

## 8. Known gaps
No backend; Share and Copy only report. In GPUI Share, New task and Copy are said back and the Tasks tab shows the open milestone's list.

## 9. GPUI
`ProjectBoardEl` draws one state (`.state()`, `.extra()`). `ScreenView` (oxyn-gpui, `parity::screenview`) runs it live as part of the **Arc screens I** set, driven by `oxyn_core::parity::screenui::ScreenUi` and its `ArcUi`: `ArcUi` keeps the tasks, milestone, tab, status and panel: `pb-task-<n>` completes or reopens, `pb-ms-<n>`, `pb-tab-<n>`, `pb-health-<n>`, `pb-panel`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S12 Arc I).
