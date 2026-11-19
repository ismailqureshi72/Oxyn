**Status:** built v0.1 (2026-11-19, S11: Agent apps III).

# Spec: Command Board (`command-board`)

Family: Agent apps (pages at `/screens/command-board/` and `/full`). Reference: the session-and-workspace pattern of autonomous software-agent apps (Devin and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `CommandBoardScreen` (Dioxus) · `CommandBoardEl` (GPUI), rules in `oxyn_core::parity::cmdboard`.

## 1. Purpose
Hand an autonomous agent a ticket, answer its questions, see exactly what it did at any moment, and get a pull request back.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New session** ⌘N; Knowledge; Search sessions; sessions as two-line rows: status mark (spinner working, amber needs you, hollow sleeping, PR glyph in green, check done), title, where it started (the coloured Linear or Slack mark), repo and status, age. Account row with the account menu |
| Top bar (48) | Session title, status pill, credits used, **Sleep** / **Wake** while it can run, and the Workspace toggle |
| Chat | Your message as a bubble, the agent's updates beside its mark; its question in an amber-edged card with two quick answers (they send the reply); while working a shimmer; then the **pull request** card: GitHub mark, number and title, n/4 checks tag, branch in mono, +/−, each check with its state in words, View diff and Open PR |
| Composer | "Reply to the agent" (or "Describe a task" on a new session), attach, the repo (a picker with the GitHub mark on a new session), Send |
| Workspace pane (440) | Tabs **Shell** · **Browser** · **Editor** · **Planner** (tablist, ← →); the tool's view at the chosen moment (commands and output; the page; the file with line numbers; the plan with done / running / to do); a **timeline** at the foot: the moment's time and label, a range slider over the events with dots, and **Live**. Scrubbing back shows a "Viewing 2:40 · …" note with Back to live |

## 3. Behaviour
Answering the question (a quick answer or a typed reply) sets the session Working; after about 3 s it opens the PR with three checks passed and the preview deploy running. Sleep pauses a session (hollow dot, "Sleeping"), Wake resumes it. Scrubbing replays the workspace: the shell holds only the commands run so far, the editor the last file opened, the planner each step's state at that moment. Sending on New session starts a session in the chosen repo. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty (start: "What should the agent take on?" with Link a ticket / tag it in a channel) · **Pull request** (the PR card with all checks passed) · **Planner** (pane on Planner). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open session has aria-current=page. The agent's question and the PR are labelled groups. The timeline is a range input with aria-valuetext naming the moment; Live is an aria-pressed toggle. The composer sends on Enter and breaks the line on Shift + Enter.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the Workspace opens over the chat. Under 620 px the credits chip and the Sleep label hide. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`CommandBoardScreen` takes `data: Option<CommandBoardData>` (`oxyn_core::parity::cmdboard`): `sessions: Vec<BSession>` (id, title, repo index, status, age, source mark), `events: Vec<Event>` (time, tool, label, body), `msgs: Vec<(bool, String)>` (agent or you, text) and `pr: Option<Pr>` (shown when the first session is PR open or Done). Start from `CommandBoardData::default()`.

## 8. Known gaps
The GPUI snapshot element draws one state (`.extra("pr" | "planner")`, `rail`, `hidden`); the interactive `AgentAppView` (section 9) runs the whole app. In `AgentAppView` the timeline scrubs (press a stop, ◀ / ▶ or Alt + Left / Right, Live to jump back), Sleep / Wake changes the session and its row, and the working clock ticks. No backend: events, files, pages, replies and checks are fixed fictional data. Brand marks (GitHub, Linear, Slack) are the official coloured marks, used only to name the service.

## 9. GPUI
`AgentAppView::new(AgentApp::CommandBoard, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Command Board app, drawn from the same screen code as the snapshot element and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views and Escape closes the menu, then the rail; inside the composer the keys edit text and Escape returns focus to the app. App presses: Open PR shows the pull request, the Shell and Planner tabs switch the Workspace pane, and the answer chips reply to the agent; Left / Right step through updates, the pull request and the planner. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens all six apps in one window.
