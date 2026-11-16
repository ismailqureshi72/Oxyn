**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Folders · Overview (`folders-overview`)

Family: Screens. Reference: affectUI Folders / Overview, behaviour only.
Components: `FoldersScreen` (Dioxus) · `FoldersScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
The folders a workspace shares with its people and agents, and the way into each.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Header | Title, count line, New folder |
| Bar | Find field (Escape clears), sort menu (Recently changed / Name), Grid / List toggle |
| Folders | Cards (or rows): icon, name, one-line note, member initials (+n), an Agents badge when agents may read it, when it last changed, a star toggle |

## 3. Behaviour
Starred folders sort first, then by the chosen order. Find filters by name or note and says when nothing matches. New folder opens a card with a focused name field: Enter or Create makes it (Escape or Cancel drops it). Opening a folder shows the Detail page in the same frame; Files in the sidebar comes back.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
