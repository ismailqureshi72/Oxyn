**Status:** built v0.2 (2026-11-18, S5: brought to affectUI parity; v0.1 2026-11-17, screens 2).

# Spec: Agent · Orchestration (`agent-orchestration`)

Family: Screens. Reference: affectUI Agent / Orchestration, behaviour only.
Components: `OrchestrationScreen` (Dioxus) · `OrchestrationEl` (GPUI), models in `oxyn_core::parity::screens2fid`.

## 1. Purpose
Watch one multi-agent run: who is working on what, what it costs, and every call it made.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | The agent sidebar: workspace switcher, search, Thread, Inbox (count), Tasks, Agents, Runs, Memory, Skills, then a Workspace group (Schedules, Activity, Usage, Permissions, Settings); the current page is marked |
| Top bar | Project and Run menus, a Live / Paused / Stopped tag, Pause (Resume) and Stop |
| Canvas | Dot grid; a toolbar to jump to the next agent at work, fit the tree, step the zoom down or up and reset it from the percentage; the agents as a tree of cards (name, status, task, role and model, branch cost and time) joined by connectors, each branch folding to a count |
| Run panel | A resizable panel (drag or ↑/↓ on its edge, 120–380 px) with Trace / Tokens & cost / Events (with the error count) tabs, a summary (agents finished of all, time, tokens, spend) and Fold |
| Trace | Calls nested under the agent turn that made them, folding per turn, with duration and a bar on one time axis; a running call grows, a failed one is marked |
| Agent area | The picked agent: role and model, status, task, who it reports to, its own and its branch cost, tokens, the agents it handed work to (open them) and its context |

## 3. Behaviour
The clock runs while the run is live: elapsed time, cost, tokens and running trace bars grow. Pause freezes them and Resume carries on; Stop freezes them for good and removes the controls. Next agent at work cycles through running agents, unfolds the branches above the next one, opens it beside the canvas and focuses its card. Zoom steps 50–200 %; Fit the tree sets 80 %.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset · Flat. Light and dark follow the page theme.

## 5. Keyboard and accessibility
The agent sidebar is a labelled nav whose current page has aria-current; tab rows are tablists (←/→ move, the selected tab is the only one in the Tab order); menus open from a trigger reporting aria-expanded and close on Escape; hairlines are focusable separators with a value and arrow keys; skeletons are aria-hidden; changes are announced in a role=status line.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button.

## S5 additions (v0.2)
- Each card carries a role tag (Orchestrator blue, Research green, Analysis grey, Writing amber, Review red tints) and its model with the official mark (Opus / Sonnet: Claude, GPT: OpenAI, Gemini: Google Gemini).
- States: Data · Loading · Empty · **Paused** (the clock and cost hold; Resume continues).
- Oxyn extras: `cost_bars` (each card shows its share of the run's cost) and `dim_done` (finished agents fade).
- 390: the project and run menus and the status tag leave the top bar and Pause / Stop go icon-only, so the Runs title no longer truncates.

## 7. Known gaps
The tree is laid out as an indented outline of cards, not affectUI's free canvas with curved connectors and travelling light; the GPUI element does not show the S5 role tags or extras yet.
GPUI is a static snapshot (no zoom, folding, panel or clock); the host owns state. Fit uses a fixed 80 % rather than measuring the tree. The tree is a role=tree of buttons; arrow-key movement between cards is not wired (Tab reaches every card).
