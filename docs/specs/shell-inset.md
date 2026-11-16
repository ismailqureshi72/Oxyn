**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: App shell · Inset (`shell-inset`)

Family: Screens. Reference: affectUI App Shell (Inset), behaviour only.
Components: `AppShellScreen` (Dioxus) · `AppShellScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
The frame every Oxyn screen sits in, with the page as an inset card and a workspace on the right.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Work area | Optional right region, resizable 260–480 px, closable; a sheet when narrow |
| Workspace | Browser-style tabs (new, close), address field (Enter renames the tab), a Tools list (each with its key chord and an overflow menu), Suggested pages that open as tabs |

## 3. Behaviour
Drag or arrow the hairlines to resize sidebar (200–340 px) and work area. Fold the sidebar to a 56 px rail. Pin moves a chat to Pinned; opening a chat clears its unread dot. Ticking setup steps moves the progress bar.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
