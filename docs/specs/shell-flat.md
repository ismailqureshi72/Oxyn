**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: App shell · Flat (`shell-flat`)

Family: Screens. Reference: affectUI App Shell (Flat), behaviour only.
Components: `AppShellScreen (layout="flat")` (Dioxus) · `ScreenShellEl` (GPUI), models in `oxyn_core::kit::screensfid`.

## 1. Purpose
The same frame laid flat: grey sidebar and edge-to-edge regions split by single hairlines.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Work area | Optional right region, resizable 260–480 px, closable; a sheet when narrow |

## 3. Behaviour
As the Inset shell; only the frame changes (no card, no canvas padding).
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
