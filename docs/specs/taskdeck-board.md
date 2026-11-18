**Status:** built v0.1 (2026-11-18, S9: Agent apps I, Taskdeck).

# Spec: Taskdeck · Board (`taskdeck-board`)

Family: Agent apps (a new category after Screens; pages at `/screens/taskdeck-board/`). Reference: the task-runner pattern shared by desktop agent apps (MiniMax Code and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `TaskdeckBoardScreen` (Dioxus) · `TaskdeckBoardEl` (GPUI), rules in `oxyn_core::parity::taskdeck`.

## 1. Purpose
See every task on this side (Local or Cloud) by where it stands, and move them along.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (264) | App mark and name, Search (filters tasks) and Hide sidebar; New task ⌘N, Tasks (running + needs-you count), Plugins & skills, Scheduled (count of active schedules), Sites (each opens its page); a Local / Cloud radio group; Projects (folders in the semantic folder blue, expandable, with their tasks) and Recents. Rows show a spinner (running), an amber dot (needs you), a red dot (failed) or a ring, then the title and age |
| Sidebar bottom | A "2 running · 1 need you" card (opens the board), then the account row (avatar, name, plan, ⌄) with the Inbox bell. The account menu (S0 rules) holds the header with plan badge, Settings, Usage ▸ (plan, credits meter, quotas), Upgrade plan, **Daily check-in** (+50, then "Checked in today"), Language, Appearance, Keyboard shortcuts, Integrations, Download apps, Changelog, Feedback & help ▸ and Log out. The bell lists tasks that need you |
| Header (64) | Tasks, "Local · n tasks", Board / List and New task |
| Board | Four columns on the inset grey: Queued, Running, Needs you, Done (failed tasks sit in Done with a red pill), each with its count. Cards: title (opens the task), ⋯ menu, project folder and age, a progress bar with n/N while running, waiting or failed |
| List | One row per task: title, project, status pill, steps and age |

## 3. Behaviour
A card's menu offers Open, then the moves that make sense for it (Run now, Back to queue, Mark done) and Delete; moving updates the column, the sidebar and the counts at once. The Local / Cloud switch swaps the tasks. An empty side shows "No local tasks yet" with New task.

## 4. Variants
States: Data · Loading · Empty · **Account menu (open)**. Frame: Inset (the main area a rounded card on the sidebar grey) · Flat (edge to edge with a hairline). Oxyn extras: `rail` (the sidebar as a 56 px icon rail; the top bar expands it) and `sidebar_hidden` (the top bar brings it back).

## 5. Keyboard and accessibility
The sidebar is a nav landmark; the current view and open task carry aria-current=page. Local / Cloud and Board / List are radiogroups; run mode and model are menus of menuitemradio; every menu and popover closes on Escape and the account menu's submenus open with → and close with ←. Steps are an ordered list; the approval is a labelled group. The side panel is a tablist (← → move). Every action reports in a polite live region; skeletons are aria-busy.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/taskdeck-board/full`) renders at the real viewport with the state bar under it (R2). Under 960 px the sidebar becomes a drawer from a top-bar button (with a scrim) and the task's side panel opens over the task. Under 620 px the composer chips shorten, the board scrolls sideways and the list keeps title and status. Labels ellipsise in flex children with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every Taskdeck screen takes `data: Option<TaskdeckData>` (`oxyn_core::parity::taskdeck`). `None` shows the sample for the state; to show your own, start from `TaskdeckData::default()` (the sample) and replace fields: `tasks: Vec<Task>` (newest first), `plugins: Vec<(bool, bool)>` (installed, on, per `CATALOG` entry), `skills: Vec<Skill>`, `schedules: Vec<Schedule>` and `sites: Vec<Site>`. `TaskdeckData::for_state(ScreenState::Empty)` is the empty set. The screen keeps edits in its own state; read them back through your host if you need them.

## 8. Known gaps
The GPUI snapshot element draws one state (`.extra(...)`, `rail`, `hidden`); the interactive `AgentAppView` (section 9) runs the whole app. In GPUI, plugin switches, the Skills tab, schedule editing and the site preview are still states set with `.extra(...)`, and runs do not tick. There is no backend: runs advance on a timer through a fixed plan, tool calls and outputs are simulated, and the open task's files are fictional. Model marks are the official coloured marks (Lobe Icons, MIT), used only to name the model. Everything else (people, companies, stores, desks) is fictional.

## 9. GPUI
`AgentAppView::new(AgentApp::Taskdeck, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Taskdeck app, drawn from the same screen code as the snapshot element and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views and Escape closes the menu, then the rail; inside the composer the keys edit text and Escape returns focus to the app. App presses: the nav (New task, Tasks, Plugins & skills, Scheduled, Sites) opens its page, a task row or board card opens the Task page, Local / Cloud switches the list, Allow once / Deny settles the pending approval, and Start sends the composer and opens the task. Call `bind_ime_keys(cx)` once at start; `cargo run -p oxyn-gpui --features app --example agent_apps` opens all six apps in one window.

The view opens on the Task page; this spec's page is Board in the same view.
