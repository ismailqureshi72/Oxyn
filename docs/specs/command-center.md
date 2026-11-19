**Status:** built v0.1 (2026-11-18, S10: Agent apps II).

# Spec: Command Center (`command-center`)

Family: Agent apps (pages at `/screens/command-center/` and `/full`). Reference: the threads-and-review pattern of desktop coding-agent apps (the Codex app and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `CommandCenterScreen` (Dioxus) · `CommandCenterEl` (GPUI), rules in `oxyn_core::parity::cmdstudio`.

## 1. Purpose
Hand a coding agent a change, watch the thread work, and review what it changed hunk by hunk before keeping it.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; New thread ⌘N; **Automations**; Search threads; projects as disclosure groups (folder glyph in the semantic folder blue, count) with their threads: status mark (spinner running, amber dot to review, green check done, ring queued), title, and +added −removed in diff colours. Account row at the bottom |
| Top bar (48) | Thread title, status pill, the **Work in** picker (Local · Worktree · Cloud, each with its line) and the Review toggle with the diff total |
| Thread | Your prompt as a bubble, then the work log (searched, read, edited, added, running tests) on a hairline rail; while running a shimmer "Working · n of 5 steps", then the agent's summary and a changes card. In a worktree a banner names the branch with **Hand off to Local** |
| Queue | "Queued next" disclosure above the composer: an ordered list with Move up and Remove; Enter while the thread runs queues the text instead of sending |
| Composer | Text area, + (add files), **model** picker with the official coloured marks, **reasoning** picker (Low, Medium, High, Max), Stop while running, Send |
| Automations | A page in the main area: "Automations" and New automation (name, When: Every night at 02:00 · When an issue is labelled bug · When CI fails on main · Every Monday at 09:00, What it does; checked before Create), then rows with a bolt glyph, name, trigger · project, the prompt, the last run pill (Ran 6h ago green, Failed red, Running blue, Not run yet), Run now and an on/off switch. Run now runs it (Running, then Ran just now) |
| Pane tabs | Review · Terminal · Browser (a tablist; ← → move) |
| Terminal | Tabs for each terminal (the thread's `cargo test` run and a `zsh`), close × on each while more than one is open, + for a new tab; the output in mono on the inset grey (commands bold) and a `$` prompt: Enter runs the command (simulated output; `clear` empties the tab) |
| Browser | Back, Reload (a thin loading bar), an address bar (Enter opens `localhost:3000/uploads`, `/settings` or the retry docs; anything else shows "Can't reach that page"); the page on white with links to the other pages |
| Review pane (440; 380 under 1180) | Header with "n files changed · +a −r · d of n reviewed", Revert all and Accept all; each file a card (path in mono, New tag, stat, verdict, Revert file, Accept file, fold) with its hunks: header in mono, Revert / Accept per hunk, Undo after a decision; lines with green + and red − backgrounds; reverted hunks dim and strike through |

## 3. Behaviour
Each sample thread has its own changes in Review (the sidebar's +/− comes from them); a thread you start has none until it edits. The open thread finishes on its own after about 3 s (CSS-timed; reduced motion makes it near instant) and moves to To review. Accept or Revert per hunk, per file or for all updates the totals (reverted hunks leave the stat) and the summary; Undo returns a hunk to review. Switching Work in moves the thread and announces its branch (`cc/thread-n` for a worktree). Sending with no thread open starts a new one in the chosen project; a blank or too-short prompt shows an inline error. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty (the start screen: "What should we build in …?", project picker, composer) · **Worktree** (finished in a worktree, ready to review) · **Automations** (the Automations page) · **Terminals** (the pane on Terminal) · **Browser** (the pane on Browser). Frame: Inset · Flat. Oxyn extras: `rail` (56 px icon rail) and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open thread has aria-current=page. Pickers are menus of menuitemradio closing on Escape. The review pane is a labelled complementary region; file and hunk buttons name their target ("Accept hunk 1 of src/upload/client.rs"). The queue is an ordered list. Enter sends or queues; Shift + Enter adds a line.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the top bar (with a scrim) and the review pane opens over the thread from its top-bar button or the changes card. Under 620 px chips shorten; under 440 px the status pill hides. Labels ellipsise in flex children with min-width 0; scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`CommandCenterScreen` takes `data: Option<CommandCenterData>` (`oxyn_core::parity::cmdstudio`). `None` shows the sample for the state; to show your own, start from `CommandCenterData::default()` (the sample) and replace fields: `threads: Vec<Thread>` (id, title, project, state, place, +/− stat, age), `diffs: Vec<(u32, Vec<DiffFile>)>` (each thread's files and hunks; `diff(id)` looks one up), `queue: Vec<(u32, String)>`, `automations: Vec<Automation>` and `terminals: Vec<Term>`. The screen opens the first thread. `DiffFile` paths and hunk lines are `&'static str` for now (leak or intern runtime strings); owned strings are a later change.

## 8. Known gaps
The GPUI snapshot element draws one state (`.extra("worktree" | "automations" | "terminal" | "browser")`, `rail`, `hidden`); the interactive `AgentAppView` (section 9) runs the whole app. In GPUI, the terminal tabs and the browser pane take no input and the work log does not tick. No backend: the work log, diffs, terminal output, pages and tests are fixed fictional data, and a new thread has no diff. Model marks are the official coloured marks, used only to name the model.

## 9. GPUI
`AgentAppView::new(AgentApp::CommandCenter, window, cx)` (oxyn-gpui, `parity::apps`) is a real GPUI view (`Entity` + `Render`) of the whole Command Center app, drawn from the same screen code as the snapshot element and driven by `oxyn_core::parity::appui::AppUi`, the rules the Dioxus app follows. Sidebar rows open on press; the logo toggles the icon rail; the account row opens a menu (Switch to light / dark, Sign out). The composer is a real `ImeInput`: Enter or the send button sends and the sent prompts show above it. With focus on the app, Up / Down move the open row, Left / Right step through the views and Escape closes the menu, then the rail; inside the composer the keys edit text and Escape returns focus to the app. App presses: the Local / Worktree chip switches to worktree review and Hand off to Local back; Left / Right step through the thread, worktree, automations, terminal and browser views. Call `bind_ime_keys(cx)` once at start; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` (from the repo root) opens all six apps in one window.
