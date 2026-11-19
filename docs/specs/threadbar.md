**Status:** built v0.1 (2026-11-20, S16a: Agent apps IV).

# Spec: Threadbar (`threadbar`)

Family: Agent apps (pages at `/screens/threadbar/` and `/full`). Reference: the editor-plus-agent-panel pattern (Zed's agent panel and threads sidebar, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model, agent or code host is named (Lobe Icons / Simple Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo.
Components: `ThreadbarScreen` (Dioxus) · `ThreadbarEl` (GPUI), rules in `oxyn_core::parity::threadbar`.

## 1. Purpose
Talk to an agent inside a code editor, keep or reject what it changes, switch between agent threads and terminal threads, and commit the result.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New thread** ⌘N; Search threads; threads grouped under their project (ledger-api, field-notes): agent threads with a chat glyph or a spinner while working, terminal threads with a terminal glyph, each with its ISO date. Account row with the account menu |
| Top bar (48) | Thread title and a pill (Working, Terminal, or the profile) and the Project toggle |
| Agent thread | Your prompt; tool calls (Searched, Read, Edited, Ran with the object in mono and a detail); the **edit card** for `src/webhooks/deliver.rs` (+18 −4, the diff with + and − lines tinted green and red, **Reject** and **Keep**); while working a shimmer, then the summary |
| Terminal thread | The test run's transcript (commands bold) and a command line that runs what you type (canned answers for `ls`, `git`, `cargo`, `echo`…) |
| Composer | Text area, Add context, **Profile** (Write · Ask · Minimal), **Agent** (Threadbar agent, or Claude Code, Codex and GitHub Copilot as outside agents with their official coloured marks), **Model** with coloured marks (hidden when an outside agent is picked, since it brings its own), Stop while working, Send. Start screen: "Start a thread" with three suggestions |
| Project pane (440) | Header "Project" with the branch; tabs **Files** (a tree with git letters: M amber, A green) and **Git** (changed files as checkboxes with +/− counts, "n staged · +a −d", the suggested commit message, **Commit n files**) |

## 3. Behaviour
The open thread finishes on its own after about 3 s. Keep and Reject resolve the edit card (Rejected says the file is back as it was). Unchecking a file unstages it; Commit with nothing staged or an empty message shows an inline error; a commit removes the staged files and shows the message. Opening a terminal thread swaps the conversation for the terminal. Sending starts a thread at the top of the first project. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Terminal thread** (the terminal thread open) · **Git** (the pane on Git). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open thread has aria-current=page. Files / Git are a tablist with roving tabindex (← →). The edit is a labelled group; the diff marks lines with + and −, not colour alone. Each changed file is a labelled checkbox. Profile, Agent and Model are menus of menuitemradio. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and rows stack. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`ThreadbarScreen` takes `data: Option<ThreadbarData>` (`oxyn_core::parity::threadbar`): `threads: Vec<TbThread>` (id, title, project, terminal, running, at), `changes: Vec<TbChange>` (path, add, del, staged), `branch: String`. Start from `ThreadbarData::default()`.

## 8. Known gaps
No backend: tool calls, the diff, the tree and the terminal answers are fixed fictional data; outside agents are listed by name and mark only. The GPUI view draws Keep / Reject and Commit as presses (`keep`, `reject`, `commit`) for the host; the staging checkboxes are drawn, not toggled.

## 9. GPUI
`AgentAppView::new(AgentApp::Threadbar, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Threadbar app, drawn from the same screen code as the snapshot element `ThreadbarEl` and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the agent thread, the terminal thread, Git) and Escape closes the menu, then the rail. App presses: a terminal row opens the terminal view; Files and Git switch the pane (`view:`, `view:git`). Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
