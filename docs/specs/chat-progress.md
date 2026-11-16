**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Progress (`chat-progress`)

Family: Screens. Reference: affectUI AI Chat / Progress, behaviour only.
Components: `ChatProgressScreen` (Dioxus) · `ChatProgressScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A chat with the assistant's progress beside it as part of the page.

## 2. Parts
| Part | Look |
|---|---|
| Chat | Tool-call cards that open into input / output, prose with a list, composer |
| Progress pane | Collapsible Progress (steps tick off on the clock, live status), Outputs (brand marks), Used in this chat (uploads, apps, web, skills) |

## 3. Behaviour
Steps advance every 1.5 s until done; each section folds independently.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
