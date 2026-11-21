**Status:** built v0.1 (2026-11-20, S16a: Agent apps IV).

# Spec: Tabstack (`tabstack`)

Family: Agent apps (pages at `/screens/tabstack/` and `/full`). Reference: the agent-first terminal pattern (Warp's vertical tabs, blocks and agent management, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model, agent or code host is named (Lobe Icons / Simple Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo.
Components: `TabstackScreen` (Dioxus) · `TabstackEl` (GPUI), rules in `oxyn_core::kit::tabstack`.

## 1. Purpose
Run commands and agents side by side in one terminal, approve what the agent wants to run, and review its change.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name; **New tab** ⌘T; **Tabs**: one per session with the agent's official coloured mark (or the terminal glyph for a plain shell), the title, a status dot, the branch and the pull request number. Account row |
| Top bar (48) | Tab title, a pill (Working, Needs you, Done, Shell), the worktree folder, the **bell** with the count of tabs that need you (it opens the first) and the Agents toggle |
| Blocks | Each command a block: the folder and branch, `❯ command`, its output, a footer with the exit status in words (exit 0 green check, exit 101 red cross), the time, and **Ask agent** on a failed block. Your prompts and the agent's turns sit between blocks |
| Approval card | "The agent wants to run" with the profile, the command in full, **Deny**, **Edit** (puts the command in the input in Terminal mode) and **Run** |
| Input | **Terminal / Agent** toggle; in Agent mode **Profile** (Default · Auto · Prod) and **Model** with coloured marks; Run or Send. Start screen with three suggestions |
| Pane (440) | Tabs **Agents** (runs under Local and Cloud with a status dot, start time and state; a local run opens its tab) and **Review** (the diff of `tests/upload.rs`, **Request changes** and **Approve**) |

## 3. Behaviour
The working tab finishes its step after about 3 s and asks to run a command (the tab turns to Needs you and the bell counts it). Run adds a passing block and the agent's note; Deny adds the agent's reply; Edit hands the command to you. In Terminal mode the input runs commands and adds blocks (canned answers); in Agent mode it adds your prompt and the agent's reply, or opens a new agent tab from the start screen. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Needs approval** (the approval card) · **Review** (the pane on Review). Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Tabs are a nav landmark; the open tab has aria-current=page. Blocks are a list and focusable; exit status is text. The approval card is an alertdialog. Terminal / Agent are aria-pressed toggles. Agents / Review are a tablist (← →). Enter runs or sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and rows stack. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`TabstackScreen` takes `data: Option<TabstackData>` (`oxyn_core::kit::tabstack`): `tabs: Vec<TsTab>` (id, title, agent mark id, branch, worktree, pr, state), `blocks: Vec<Block>` (`Cmd(command, output, exit code, ms)`, `Ask`, `Say`). Start from `TabstackData::default()`.

## 8. Known gaps
No real shell: typed commands get canned answers. Only the first tab has blocks; the others say where their worktree is. The GPUI view shows the approval card on the Needs approval view and the run's block on Review.

## 9. GPUI
`AgentAppView::new(AgentApp::Tabstack, window, cx)` (oxyn-gpui, `kit::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Tabstack app, drawn from the same screen code as the snapshot element `TabstackEl` and driven by `oxyn_core::kit::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (the blocks, Needs approval, Review) and Escape closes the menu, then the rail. App presses: Run opens Review with the passing block, Deny returns to the blocks; Agents and Review switch the pane (`view:`, `view:review`). Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
