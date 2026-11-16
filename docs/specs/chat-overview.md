**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: AI Chat · Overview (`chat-overview`)

Family: Screens. Reference: affectUI AI Chat / Overview, behaviour only.
Components: `ChatOverviewScreen` (Dioxus) · `ChatOverviewScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
The start page with the numbers under the composer.

## 2. Parts
| Part | Look |
|---|---|
| Activity | 20-week heat map (monochrome levels), arrow keys / click move a focus cell, status line says week, day and chats |
| Recent chats | Three rows and an "N more" toggle |
| Credits | Used of cap with a meter and reset date |
| Made for you | Files with their project as a tooltip, "1 more" toggle |
| This month | Chats and files with signed deltas |

## 3. Behaviour
Sending from the composer starts a conversation above the cards. Loading shows skeletons per card; Empty zeroes the numbers with a line on what will land.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no menus, resize or live clock); the host owns state.
