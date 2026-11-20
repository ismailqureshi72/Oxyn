**Status:** built v0.1 (2026-11-20, S16b: Agent apps IV, part b).

# Spec: Pilot Panel (`pilot-panel`)

Family: Agent apps (pages at `/screens/pilot-panel/` and `/full`). Reference: the editor agent-panel pattern (Cline in VS Code: Plan / Act, tool approvals, auto-approve, checkpoints, context and cost, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken. Brand marks appear only where a real model is named (Lobe Icons, credited in `THIRD_PARTY_NOTICES.md`), never as this app's logo, which is our own drawn mark.
Components: `PilotPanelScreen` (Dioxus) · `PilotPanelEl` (GPUI), rules in `oxyn_core::parity::pilotpanel`.

## 1. Purpose
Give an agent a coding task, let it plan first or act, approve the tool calls auto-approve does not cover, and step back to any checkpoint if a change goes wrong, while watching context and cost.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New task** ⌘N; **History**: tasks with title, ISO date and cost. Account row with the account menu |
| Top bar (48) | Task title, a pill (Act, Plan, Needs approval), a **context meter** (38.2k / 200k tokens) and the task's **cost** |
| Task | Your prompt; steps: **Read** a file, the agent's note, **Edited** files with +/− and a Diff button, **Ran** a command with its result; after every edit and command a dashed **Checkpoint** line with **Compare** and **Restore** |
| Plan mode | The steps it read, then a **Plan** card (four numbered steps, "Nothing changed yet") with **Switch to Act** |
| Needs approval | An alertdialog: the agent wants to run `npm run migrate -- --env staging`, why it asks, **Reject** and **Approve**; then a line with the outcome |
| Composer | **Auto-approve** panel (Read files, Edit files, Run commands as checkboxes) above the box; text area, Add context, **Plan / Act** segmented toggle, the Auto-approve chip, **Model** with coloured marks (Claude, Gemini, DeepSeek), Send. Start screen: "What can I do for you?" with three suggestions |
| Editor pane (440) | Tabs **Editor** (the edited route's diff, + and − tinted) and **Checkpoints** (each with the step it follows and **Restore**) |

## 3. Behaviour
Restore drops the steps after that checkpoint (core `restore`). Whether a tool call waits is `needs_approval(kind, auto, plan)`: Plan mode never asks; turning on Run commands in auto-approve makes the pending command run without asking. Switch to Act leaves Plan mode. A new task runs and finishes on its own after about 3 s. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty · **Plan mode** · **Needs approval**. Frame: Inset · Flat. Oxyn extras: `rail` and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open task has aria-current=page. Plan / Act are aria-pressed buttons in a labelled group; auto-approve is a labelled group of checkboxes. The pending command is an alertdialog. Checkpoints are labelled groups. Editor / Checkpoints are a tablist with roving tabindex (← →). The context meter has a text label with its percent. Enter sends; Shift + Enter breaks the line.

## 6. Responsive
1440 × 900 stage; `/full` at the real viewport. Under 960 px the sidebar is a drawer and the pane opens over the conversation from the top bar. Under 620 px chip labels shorten and dates hide in rows. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`PilotPanelScreen` takes `data: Option<PilotPanelData>` (`oxyn_core::parity::pilotpanel`): `tasks: Vec<PkTask>` (id, title, cents, running, at) and `steps: Vec<PkStep>` (kind, title, detail). Start from `PilotPanelData::default()`.

## 8. Known gaps
No backend: steps, diff, costs and tokens are fixed fictional data; Compare opens the Checkpoints tab rather than a side-by-side diff. The GPUI view draws Restore, Approve and Reject as presses (`restore`, `approve`, `reject`) for the host.

## 9. GPUI
`AgentAppView::new(AgentApp::PilotPanel, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Pilot Panel app, drawn from the same screen code as the snapshot element `PilotPanelEl` and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views (Act, Plan mode, the pending approval) and Escape closes the menu, then the rail. App presses: Switch to Act fires `act` and leaves Plan mode; Approve, Reject and Restore fire `approve`, `reject`, `restore`. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens every agent app in one window.
