**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Kanban (`kanban`)

Family: Screens. Reference: affectUI Kanban board, behaviour only.
Components: `KanbanScreen` (Dioxus) · `KanbanEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A team board: cards move left to right through To do, In progress, In review and Done.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Header | Title, card and done counts |
| Bar | Find (title, notes or key; Escape clears), Only mine toggle |
| Columns | Name, count (n/limit where a column has a work limit, marked when over, with a line saying so), + to add a card in place |
| Cards | Key, priority bars, title, label, sub-tasks done, due (late marked), assignee initials; the open card outlined |
| Card area | Key and status, editable title, Status / Priority / Assignee menus, due, sub-task meter, notes, Delete card |

## 3. Behaviour
Drag a card onto a column to put it at the bottom, or onto a card to put it before that card; the column under the pointer is marked. Alt+←/→ moves the focused card to the next column (focus follows), Alt+↑/↓ moves it within its column; each move is announced. Enter or Space opens the card beside the board; the Status menu moves it too. + opens a focused field: Enter adds (and stays open for the next), Escape or Cancel closes. Empty shows four empty columns and how to start.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no drag, menus or editing); the host owns state. Dragging is pointer only in browsers without HTML drag and drop; the Alt+arrow keys cover keyboards.
