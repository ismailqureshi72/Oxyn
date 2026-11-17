**Status:** built v0.1 (2026-11-18, S4: Agent ops II).

# Spec: Agent · Agents (`agent-agents`)

Family: Screens. Reference: affectUI Agent / Agents (`/screens/agents`), layout and behaviour only; no source or assets copied.
Components: `AgentsScreen` (Dioxus) · `AgentsEl` (GPUI), models in `oxyn_core::parity::opsscr`.

## 1. Purpose
The agent team at a glance: who is working, on what model, with which apps, and what each costs; and a quick way to make another.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Fieldwork persona: Thread · Inbox 5 · Agents · Tasks · Memory · Skills, then the Workspace group (Schedules, Activity, Usage, Permissions, Settings); this page is current |
| Top bar | Agents, tabs by state (each with its count), a grid or list toggle and New agent |
| Cards | A 3 × 3 pixel mark in the agent's hue, name and role, a status tag (Working blue, Idle grey, Paused amber), official app marks with Access to N apps, the model chip with its official mark, runs and spend |
| New agent | A dialog: the mark draws itself from the typed name (mirrored 3 × 3, same name same mark), what it does, the model as a radio group, Create agent |
| Work area (348) | The agent with Message and Pause; Status, Model, Reports to; runs over two weeks; Instructions with Show all; Skills; Can use (app marks); Recent tasks |

Chrome is monochrome neutral; colour only means something: official app and model marks, identity hues (purple, blue, green, orange, teal, pink) and status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Tabs filter by state with counts. Message opens a field (Enter sends, Escape closes). Pause toggles the agent between Idle and Paused. New agent adds the agent, selects it and announces it.
prefers-reduced-motion: no fills, pulses or transitions.

## 4. Variants
States: Data · Loading · Empty · **New agent**. Frame: Inset · Flat. Oxyn extras: `by_spend` (cards sorted by spend) and `budget_bars` (a spend bar against a €50 cap).

## 5. Keyboard and accessibility
Cards are buttons with aria-current; tabs are a tablist with ←/→; New agent is a dialog (Enter creates, Escape cancels) with the model as a radio group; Pause is a toggle button (aria-pressed). Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/agent-agents/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the sidebar slides in from a menu button and the work area becomes a sheet. Under 640 px the view toggle hides; under 560 px cards are one column. Every label in a row, chip or card ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or an option; typing, timers and menus belong to the host. Sample people, companies and projects are fictional; model versions are fictional, model marks are the providers' official ones.
