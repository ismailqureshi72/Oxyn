**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Double sidebar (`chat-double-sidebar`)

Family: Screens. Reference: affectUI AI Chat / Double Sidebar, behaviour only.
Components: `ChatRailScreen` (Dioxus) · `ChatRailScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A plain chat with web search inside the three-pane frame, artifacts on the right.

## 2. Parts
| Part | Look |
|---|---|
| Chat | Title, user ask, answer with a "Ran 3 searches" disclosure, artifact chips, composer |
| Artifacts | Closable tabs; a small sheet: undo, bold, sort up or down, add a row below, the cell name, an edit field and the grid; a sources list |

## 3. Behaviour
Artifact chips open or focus their tab. Grid: click or arrow to select; typing in Contents edits the cell; Enter moves down; sort keeps the header row and compares numbers as numbers; Undo restores the last snapshot.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
