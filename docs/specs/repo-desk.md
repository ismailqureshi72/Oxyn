**Status:** built v0.1 (2026-11-20, S16a: Agent apps IV).

# Spec: Repo Desk (`repo-desk`)

Family: Agent apps (pages at `/screens/repo-desk/` and `/full`). Reference: the chat-first code-host agent app pattern (the GitHub Copilot app's Home, Inbox, sessions and canvases, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model, agent or code host is named (Lobe Icons / Simple Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo.
Components: `RepoDeskScreen` (Dioxus) · `RepoDeskEl` (GPUI), rules in `oxyn_core::parity::repodesk`.

## 1. Purpose
Hand issues and reviews to an agent from one inbox, follow each session's plan and pull request in shared canvases, and track a project's goal.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name; **New session** ⌘N; **Home**, **Inbox** (with its count), **Projects**; **Sessions** with a spinner or chat glyph, the title and the repository. Account row |
| Top bar (48) | Session title, Working / Idle and the session's own branch; Inbox; or the project's name |
| Session | Your prompt, the agent's reply, cards for the Plan and Pull request canvases; while working a shimmer, then the summary. Composer: attach, the repository chip with the code host's official mark, **Model** with coloured marks, Stop, Send |
| Inbox | Filters **All · Issues · Pull requests · Reviews**; items with a kind glyph (issue green, pull request blue, review amber), the repository and number, the ISO date, the title, the kind and why it needs you, **Done** and **Start session** |
| Project | Goal, due date (ISO), "n of m milestones · p%" with a progress bar, milestones to tick, and the sessions on the goal |
| Canvases pane (440) | Tabs **Plan** (steps to tick, "n of 5 done"), **Pull request** (#418 with the code host mark, Draft / Open, branch, files and lines, checks Passed / Running, View on the code host, **Mark ready**), **Terminal** (test output), **Browser** (the exported table with its totals row, Export CSV) |

## 3. Behaviour
The open session goes idle after about 3 s. Start session turns an Inbox item into a session on a branch named from its title and removes it from the Inbox; Done removes it. Mark ready turns the draft into an open pull request. Plan steps and milestones tick; the goal's percentage follows. Sending starts a session. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Inbox** · **Project**. Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; Home, Inbox, Projects and the open session carry aria-current=page. Inbox filters are a radiogroup. The canvases are a tablist (← →). Plan steps and milestones are checkboxes; the goal is a progressbar with its value. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and rows stack. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`RepoDeskScreen` takes `data: Option<RepoDeskData>` (`oxyn_core::parity::repodesk`): `sessions: Vec<RdSession>` (id, title, repo, branch, running), `inbox: Vec<RdItem>` (id, kind, refs, title, why, at). Start from `RepoDeskData::default()`.

## 8. Known gaps
No code host is called: the plan, pull request, checks, terminal and browser are fixed fictional data. The GPUI view draws the canvases on Plan; its canvas tabs fire `canvas-*` presses for the host.

## 9. GPUI
`AgentAppView::new(AgentApp::RepoDesk, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Repo Desk app, drawn from the same screen code as the snapshot element `RepoDeskEl` and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the session, Inbox, Project) and Escape closes the menu, then the rail. App presses: Home, Inbox and Projects (`view:`, `view:inbox`, `view:project`); Start session (`start-N`) adds a sent line and returns to the session. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
