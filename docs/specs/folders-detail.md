**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Folders · Detail (`folders-detail`)

Family: Screens. Reference: affectUI Folders / Detail, behaviour only.
Components: `FoldersScreen (detail=true)` (Dioxus) · `FoldersScreen (detail=true)El` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
One folder open: its files, and who and what can read them.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Work area | Optional right region, resizable 260–480 px, closable; a sheet when narrow |
| Page | All folders link, name, note, members, Add file |
| Files | Table with select-all and per-row checkboxes; Name, Kind, Edited and Size headers sort (aria-sort; again reverses); owner initials; a total line |
| Selection bar | n selected, Clear, Delete; after deleting, a status line with Undo |
| Details | About, Shared with (owner, then each person with a Can edit / Can view / Remove menu), Invite by email (checked), an agents switch, recent activity |

## 3. Behaviour
Add file appends the next free untitled name. Delete (button or key) removes the selected files and Undo restores them. The invite field adds a member from a valid address and says so when it is not one; removing a member updates the page faces and the Overview card; the agents switch updates the Agents badge.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
