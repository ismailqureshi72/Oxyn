**Status:** built v0.1 (2026-11-17, screens 2: Screens).

# Spec: Agent · Thread (`agent-thread`)

Family: Screens. Reference: affectUI Agent / Thread, behaviour only.
Components: `AgentThreadScreen` (Dioxus) · `AgentThreadEl` (GPUI), models in `oxyn_core::parity::screens2fid`.

## 1. Purpose
One running conversation with the orchestrator, which picks the models and starts the tasks.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | The agent sidebar: workspace switcher, search, Thread, Inbox (count), Tasks, Agents, Runs, Memory, Skills, then a Workspace group (Schedules, Activity, Usage, Permissions, Settings); the current page is marked |
| Header | Today, then "n running" and "n need you" chips that open the matching tasks in the work area |
| Log | Your messages, the orchestrator's plain answers with times, tool calls that ran side by side folded into one "Ran n tools · time" line (opens into a row per call), task cards (step, n of m steps, Open in Tasks), file chips, a "While you were away" marker and a slow upload with its progress |
| Stream | A reply streams word by word after a short Thinking; once, the connection drops: the words so far stay, a note says it carries on, Retry now picks it up at once |
| Composer | The shared composer without a model menu (the orchestrator picks); Send turns into Stop while a reply streams |
| Work area | Artifacts (file chips and a preview) and Tasks (All / Running / Needs you, with steps) |

## 3. Behaviour
Enter sends. The reply lands in the log when the stream ends; Stop keeps the words so far and marks the answer stopped. The sidebar Tasks entry and the header chips switch the work area to Tasks with the matching filter; Open in Tasks outlines that task; a file chip opens its preview.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset · Flat. Light and dark follow the page theme.

## 5. Keyboard and accessibility
The agent sidebar is a labelled nav whose current page has aria-current; tab rows are tablists (←/→ move, the selected tab is the only one in the Tab order); menus open from a trigger reporting aria-expanded and close on Escape; hairlines are focusable separators with a value and arrow keys; skeletons are aria-hidden; changes are announced in a role=status line.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button.

## 7. Known gaps
GPUI is a static snapshot (no streaming, folding or work area); the host owns state. The drop is simulated on the first reply only.
