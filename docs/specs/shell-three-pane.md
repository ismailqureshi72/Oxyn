**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Three-pane shell (`shell-three-pane`) – Oxyn extra

Family: Screens. **Oxyn-only extra:** affectUI has no three-pane screen (its closest are AI Chat / Double Sidebar and Agent / Inbox). Kept as our own frame and labelled as such on the site (S5).
Components: `RailShellScreen / RailFrame` (Dioxus) · `ScreenRailEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
An icon rail, a chats column with a project switcher, the page and a work area.

## 2. Parts
| Part | Look |
|---|---|
| Rail | Workspace mark, Chats / Tasks / Runs / Files / Apps, alerts, settings, avatar |
| Chats column | Project switcher menu, search, New chat, Pinned and Chats with pin |
| Page + work area | As the app frame |

## 3. Behaviour
Pressing Chats again folds the column; other rail sections reopen it. Search filters live; pin and unread behave as in the frame.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
