**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Agents (`chat-agents`)

Family: Screens. Reference: affectUI AI Chat / Agents, behaviour only.
Components: `ChatAgentsScreen` (Dioxus) · `ChatAgentsScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A chat with an orchestrator, the crew working on it, and the brief it is filling in.

## 2. Parts
| Part | Look |
|---|---|
| Chat | User ask, orchestrator reply, worked-for line, composer |
| Crew | Five agents as chips (working ones pulse); pressing one says what it is doing; orchestrator row |
| Brief | three section tabs (Brief, Assets, Activity); a product summary; angle radio cards with signal bars; audience tags; channel switches; palette; format toggles; details; footer estimate and Make ads |

## 3. Behaviour
The footer estimate follows channels and formats (ad_estimate). Make ads counts up "Making n of N" on the clock, then "N ads ready". Chat options menu in the top bar.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
