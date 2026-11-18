**Status:** built v0.1 (2026-11-19, S11: Agent apps III).

# Spec: Fleet Window (`fleet-window`)

Family: Agent apps (pages at `/screens/fleet-window/` and `/full`). Reference: the agents-window pattern of desktop coding-agent apps (Cursor's agents view and others, see the agent-apps research), behaviour and layout only, under our own name and design; no source, artwork or copy taken.
Components: `FleetWindowScreen` (Dioxus) · `FleetWindowEl` (GPUI), rules in `oxyn_core::parity::fleetwin`.

## 1. Purpose
Run several coding agents at once, each on its own worktree, and pick the best result when the same prompt ran on more than one model.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | App mark and name, Hide sidebar; **New agent** ⌘I; **Compare**; Search agents; the agents: status mark (spinner running, amber dot to review, green check done), title, the model's official coloured mark and +added −removed in diff colours. A parallel run is a group row ("Paginate the invoices API ×3", opens Compare) over one row per model. The account row at the bottom opens the account menu (Settings, Usage, Keyboard shortcuts, Help, Log out) |
| Top bar (48) | Agent title, status pill, the agent's worktree branch in mono (`fw/agent-n`) and the Changes toggle with the diff total |
| Steps | Your prompt as a bubble, then the agent's steps (searched, read, edited, added, running tests) on a hairline rail; while running a shimmer "Running tests · 4 of 5 steps", then the summary and a changes card |
| Compare | The prompt, then one card per model: mark and name, **Suggested** on the best (most tests passing, then fewest lines), tests passed in green or red, +/−, time (m:ss), what it did, **View changes** (opens that attempt's diff in the pane) and **Apply**. Applying keeps one (Applied, green edge) and discards the rest (dimmed), with a banner naming the worktree and Undo |
| Plan | Plan mode answers with a dashed card: numbered steps as checkboxes, "n of m checked", **Build** (starts an agent on it) and Dismiss |
| Composer | Text area, + (add files), **Mode** (Agent · Plan · Ask, each with its line), **Models** (menuitemcheckbox: tick more than one to run them side by side; the chip stacks their marks and shows ×n), Stop while running, Send |
| Changes pane (440; 380 under 1180) | The S10 review pane: files, hunks, Accept / Revert per hunk and file, Accept all, Undo |

## 3. Behaviour
The open agent finishes on its own after about 3 s (CSS-timed) and moves to review with its diff. Sending with one model starts one agent; with several, one agent per model in a new group. Plan mode turns the prompt into a plan instead of starting an agent. Each sample agent has its own changes. A blank or too-short prompt shows an inline error. Every action reports in a polite live region.

## 4. Variants
States: Data · Loading · Empty (start screen: "What should the agents work on?", repo, worktree and model chips, composer) · **Compare** · **Plan**. Frame: Inset · Flat. Oxyn extras: `rail` (56 px icon rail) and `sidebar_hidden`.

## 5. Keyboard and accessibility
Sidebar is a nav landmark; the open agent has aria-current=page. Mode is a menu of menuitemradio, Models of menuitemcheckbox; menus close on Escape. Compare is a list; Apply and View changes name the model. Plan steps are real checkboxes. The composer sends on Enter and breaks the line on Shift + Enter.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer and the Changes pane opens over the conversation. Under 620 px the worktree chip hides and Compare stacks its cards. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`FleetWindowScreen` takes `data: Option<FleetWindowData>` (`oxyn_core::parity::fleetwin`). `None` shows the sample for the state; start from `FleetWindowData::default()` and replace fields: `agents: Vec<Agent>` (id, title, model index, state, +/− stat, age, parallel group), `attempts: Vec<Attempt>` (agent, model, seconds, stat, tests, summary), `diffs: Vec<(u32, Vec<DiffFile>)>` and `plan: Vec<(String, bool)>`. The first agent opens.

## 8. Known gaps
GPUI is a static snapshot (`.extra("compare" | "plan" | "rail" | "hidden")`). No backend: steps, diffs and attempts are fixed fictional data; an agent you start has no diff. Model marks are the official coloured marks, used only to name the model.
