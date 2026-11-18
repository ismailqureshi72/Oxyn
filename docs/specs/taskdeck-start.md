**Status:** built v0.1 (2026-11-18, S9: Agent apps I, Taskdeck).

# Spec: Taskdeck · Start (`taskdeck-start`)

Family: Agent apps (a new category after Screens; pages at `/screens/taskdeck-start/`). Reference: the task-runner pattern shared by desktop agent apps (MiniMax Code and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `TaskdeckStartScreen` (Dioxus) · `TaskdeckStartEl` (GPUI), rules in `oxyn_core::parity::taskdeck`.

## 1. Purpose
Hand an agent a task: say what you want, where it works and how much it may do on its own, then let it run.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (264) | App mark and name, Search (filters tasks) and Hide sidebar; New task ⌘N, Tasks (running + needs-you count), Plugins & skills, Scheduled (count of active schedules), Sites (each opens its page); a Local / Cloud radio group; Projects (folders in the semantic folder blue, expandable, with their tasks) and Recents. Rows show a spinner (running), an amber dot (needs you), a red dot (failed) or a ring, then the title and age |
| Sidebar bottom | A "2 running · 1 need you" card (opens the board), then the account row (avatar, name, plan, ⌄) with the Inbox bell. The account menu (S0 rules) holds the header with plan badge, Settings, Usage ▸ (plan, credits meter, quotas), Upgrade plan, **Daily check-in** (+50, then "Checked in today"), Language, Appearance, Keyboard shortcuts, Integrations, Download apps, Changelog, Feedback & help ▸ and Log out. The bell lists tasks that need you |
| Hero | The app mark in a soft tile and a one-line headline ("What should we get done today?"; in Cloud "Hand it off. It keeps going in the cloud.") |
| Composer (680) | Attached chips (file-type coloured), a 3-row text area "Describe a task · @ for files, plugins and skills", then + (Attach files, Add a folder, Plugins, Skills), Run mode (Ask first, Auto for safe steps, Full auto), Model (Sonnet 5.5, GPT-6, Gemini 3 Pro, DeepSeek V4 with their marks), Dictate and a round Start |
| Under it | A hint or the error line, the Folder picker (the project folders) and where it runs (Local / Cloud) |
| Ideas | Four chips (Build a site, Analyse data, Write a report, Automate a chore) that fill the composer |

Chrome is monochrome neutral; colour only for status, file types, the folder glyph and model marks.

## 3. Behaviour
Enter starts the task (Shift + Enter for a new line). The prompt is checked: empty ("Describe the task first"), too short or over 600 characters shows the reason under the composer and marks it invalid; otherwise a task titled from the first sentence (≤ 44 characters, cut at a word) is added to the top of the sidebar in the chosen folder and Local or Cloud, starts running and opens in the Task view. Dictate shows a listening bar with a waveform; Done drops the words in. + adds chips you can remove. The Local / Cloud switch changes the headline, the sidebar's tasks and the board.

## 4. Variants
States: Data · Loading · Empty · **Cloud (Cloud mode)**. Frame: Inset (the main area a rounded card on the sidebar grey) · Flat (edge to edge with a hairline). Oxyn extras: `rail` (the sidebar as a 56 px icon rail; the top bar expands it) and `sidebar_hidden` (the top bar brings it back).

## 5. Keyboard and accessibility
The sidebar is a nav landmark; the current view and open task carry aria-current=page. Local / Cloud and Board / List are radiogroups; run mode and model are menus of menuitemradio; every menu and popover closes on Escape and the account menu's submenus open with → and close with ←. Steps are an ordered list; the approval is a labelled group. The side panel is a tablist (← → move). Every action reports in a polite live region; skeletons are aria-busy.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/taskdeck-start/full`) renders at the real viewport with the state bar under it (R2). Under 960 px the sidebar becomes a drawer from a top-bar button (with a scrim) and the task's side panel opens over the task. Under 620 px the composer chips shorten, the board scrolls sideways and the list keeps title and status. Labels ellipsise in flex children with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every Taskdeck screen takes `data: Option<TaskdeckData>` (`oxyn_core::parity::taskdeck`). `None` shows the sample for the state; to show your own, start from `TaskdeckData::default()` (the sample) and replace fields: `tasks: Vec<Task>` (newest first), `plugins: Vec<(bool, bool)>` (installed, on, per `CATALOG` entry), `skills: Vec<Skill>`, `schedules: Vec<Schedule>` and `sites: Vec<Site>`. `TaskdeckData::for_state(ScreenState::Empty)` is the empty set. The screen keeps edits in its own state; read them back through your host if you need them.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the own state, `rail` or `hidden`; menus, ticks and typing belong to the host. There is no backend: runs advance on a timer through a fixed plan, tool calls and outputs are simulated, and the open task's files are fictional. Model marks are the official coloured marks (Lobe Icons, MIT), used only to name the model. Everything else (people, companies, stores, desks) is fictional.
