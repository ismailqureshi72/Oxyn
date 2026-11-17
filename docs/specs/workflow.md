**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Workflow builder (`workflow`)

Family: Screens. Reference: affectUI Workflow Builder, behaviour only.
Components: `WorkflowScreen` (Dioxus) · `WorkflowEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
Build an automation as nodes on a canvas, set each step up and test it before publishing.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Toolbar | Palette toggle, Add node menu, fit the flow, zoom − and + around the percentage (click it for 100 %), Run workflow (Stop while running, Run again after) |
| Palette | Triggers, Tools, Logic and Actions with counts; click adds after the selected step, drag drops a node where you let go |
| Canvas | Dot grid; nodes with kind, title, setting summary and Ready / Not configured or their run state; connections as focusable curves (select, Delete removes); grows to the right as nodes are added |
| Inspector | Selected node: name, its one setting (marked when empty), ready tag, run state, position, Comes from / Goes to with Disconnect, Connect to… (no loops, nothing into a trigger), Delete step; a selected connection: Remove; nothing selected: steps, ready count, last test and the run order |
| Header | Publish (only when every step is set up), Published tag |

## 3. Behaviour
Drag a node with the pointer (it snaps to a 20 px grid on release) or move the focused node with the arrow keys (Shift: five steps); each move is announced. A node added after the selected one lands one column to its right (lower when that spot is taken) and joins on. Run walks the run order from the triggers, one step every 0.9 s: the running node is outlined, finished ones turn done and the curve into the next step flows; a step without its setting fails, and the rest are skipped, with the outcome in a status line. Empty starts with a blank canvas and an Add New member button.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (nodes placed from the model, no curves, drag, zoom or run); the host owns state. Connections are made from the inspector, not by dragging between ports. Dropping from the palette needs HTML drag and drop; click-to-add covers keyboards and touch.
