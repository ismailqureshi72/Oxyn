**Status:** built v0.1 (2026-11-19, PR #59: Agent apps gap pass).

# Spec: Taskdeck · Plugins & skills (`taskdeck-plugins`)

Family: Agent apps (pages at `/screens/taskdeck-plugins/`). Reference: the task-runner pattern shared by desktop agent apps (see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `TaskdeckPluginsScreen` (Dioxus) · `TaskdeckPluginsEl` (GPUI), rules in `oxyn_core::parity::taskdeck`.

## 1. Purpose
Choose which plugins a task may use and which saved skills it loads, in one place.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (264) | App mark and name, Search (filters tasks) and Hide sidebar; New task ⌘N, Tasks (running + needs-you count), Plugins & skills, Scheduled (count of active schedules), Sites; a Local / Cloud radio group; Projects (folders in the semantic folder blue, expandable, with their tasks) and Recents. Rows show a spinner (running), an amber dot (needs you), a red dot (failed) or a ring, then the title and age |
| Sidebar bottom | A "2 running · 1 need you" card (opens the board), then the account row (avatar, name, plan, ⌄) with the Inbox bell. The account menu (S0 rules) holds the header with plan badge, Settings, Usage ▸ (plan, credits meter, quotas), Upgrade plan, **Daily check-in** (+50, then "Checked in today"), Language, Appearance, Keyboard shortcuts, Integrations, Download apps, Changelog, Feedback & help ▸ and Log out. The bell lists tasks that need you |
| Header (64) | Plugins & skills, "n plugins installed · n skills on", a Plugins / Skills radio group |
| Plugins | Search and category chips (All, Browse, Work, Code, Create); a grid of cards: the official mark (coloured, Simple Icons) or a glyph, name, category, an on/off switch once installed, a one-sentence summary, Install (primary) or Details and Remove. Details lists what the plugin may do |
| Skills | A line on what skills are and New skill; rows with a glyph, name, when it applies, a Bundled / Yours pill, Delete (yours only) and a switch |
| New skill form | Name and When it applies, Cancel and Save skill; errors under the fields |


## 3. Behaviour
Install puts a plugin on at once; the switch turns it off and on; Remove uninstalls it. Search and category filter together; nothing matching shows "No plugins match". New skill checks the name (required, under 40 characters, not already used) and when it applies (required) and adds the skill at the top, on. Every change reports in the live region.

## 4. Variants
States: Data · Loading · Empty (nothing installed, no skills) · **Skills tab**. Frame: Inset (the main area a rounded card on the sidebar grey) · Flat (edge to edge with a hairline). Oxyn extras: `rail` (the sidebar as a 56 px icon rail; the top bar expands it) and `sidebar_hidden` (the top bar brings it back).

## 5. Keyboard and accessibility
The sidebar is a nav landmark; the current view and open task carry aria-current=page. Local / Cloud and Board / List are radiogroups; run mode and model are menus of menuitemradio; every menu and popover closes on Escape and the account menu's submenus open with → and close with ←. Steps are an ordered list; the approval is a labelled group. The side panel is a tablist (← → move). Every action reports in a polite live region; skeletons are aria-busy.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/taskdeck-plugins/full`) renders at the real viewport with the state bar under it (R2). Under 960 px the sidebar becomes a drawer from a top-bar button (with a scrim). Under 620 px the cards stack one per row and the skill pills hide. Labels ellipsise in flex children with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every Taskdeck screen takes `data: Option<TaskdeckData>` (`oxyn_core::parity::taskdeck`). `None` shows the sample for the state; to show your own, start from `TaskdeckData::default()` (the sample) and replace fields: `tasks: Vec<Task>` (newest first), `plugins: Vec<(bool, bool)>` (installed, on, per `CATALOG` entry), `skills: Vec<Skill>`, `schedules: Vec<Schedule>` and `sites: Vec<Site>`. `TaskdeckData::for_state(ScreenState::Empty)` is the empty set. The screen keeps edits in its own state; read them back through your host if you need them.

## 8. Known gaps
The GPUI snapshot element draws one state (`.extra(...)`, `rail`, `hidden`); the interactive `AgentAppView` (section 9) runs the whole app. In GPUI, plugin switches, the Skills tab, schedule editing and the site preview are still states set with `.extra(...)`, and runs do not tick. There is no backend. Plugin marks are the official coloured marks (Simple Icons, CC0), used only to name the integration. Everything else (people, companies, sites, addresses) is fictional.

## 9. GPUI
`AgentAppView::new(AgentApp::Taskdeck, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Taskdeck app, drawn from the same screen code as the snapshot element and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views and Escape closes the menu, then the rail; inside the composer the keys edit text and Escape returns focus to the app. App presses: the nav (New task, Tasks, Plugins & skills, Scheduled, Sites) opens its page, a task row or board card opens the Task page, Local / Cloud switches the list, Allow once / Deny settles the pending approval, and Start sends the composer and opens the task. Call `bind_ime_keys(cx)` once at start; `cargo run -p oxyn-gpui --features app --example agent_apps` opens all six apps in one window.

The view opens on the Task page; this spec's page is Plugins & skills in the same view.
