**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Integrations (`integrations`)

Family: Screens. Reference: affectUI Integrations, behaviour only.
Components: `IntegrationsScreen` (Dioxus) · `IntegrationsScreenEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
Where a workspace connects apps and decides what each may do.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Header | Title and a connected count |
| Find | Search by name or description (Escape clears) |
| Tabs | All, Connected (with count), Files, Messaging, Work, Calendar, Developer; ←/→ move |
| Cards | Brand mark, name, category, one line on what it enables, Connected tag + Manage, or Connect |
| Connect dialog | Modal over the page: the permissions as checkboxes (Allow needs one), Cancel / Allow, Escape or the scrim cancel |
| App area | Mark, name, last sync; a switch per permission; Sync now; Disconnect with an inline Yes / Keep |

## 3. Behaviour
Connected apps sort first. Allow connects with the chosen permissions and opens the app beside the list; Disconnect asks once, then returns the card to Connect and closes the area. Empty shows the full catalogue with nothing connected; the Connected tab explains how to start.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no dialog or switches); the host owns state. The dialog moves focus to Allow and closes on Escape but does not trap Tab.
