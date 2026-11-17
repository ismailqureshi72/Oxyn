**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: CRM (`crm`)

Family: Screens. Reference: affectUI CRM, behaviour only.
Components: `CrmScreen` (Dioxus) · `CrmEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A sales team's deals in one table, cut into saved views.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Workspace switcher (menu), fold-to-rail button, live search (Escape clears), nav, Pinned / Projects (expand) / Chats with pin and unread dot, dismissible notice, setup checklist with progress, account menu |
| Page | Top bar (menu button when narrow, title, actions, work-area toggle) over a scrolling body |
| Header | Title, deal count and open pipeline, New deal |
| Views | All deals, My deals, Closing soon, Won as tabs with counts; ←/→ move |
| Find | Company, contact or note (Escape clears) |
| Table | Company with contact, Stage (menu in the row), Value, Owner, Close (overdue marked); Company, Stage, Value and Close headers sort (aria-sort; again reverses); a shown count and total |
| Deal area | Stage and value, company, contact, Stage and Owner menus, close, notes, value by stage as bars with the open stage marked, Mark won, Delete deal |

## 3. Behaviour
Clicking a company opens the deal beside the table; double-click or F2 renames it in place (Enter or leaving the field keeps it, Escape cancels). Clicking a value turns it into a field that reads 12,400, $12,400 or 12k; an amount it cannot read is marked and announced and the old value stays. The row's Stage menu moves the deal and views recount at once (Won picks up a deal marked won). New deal adds a lead you own and opens its name for typing. Empty explains how to start; a view with nothing in it says so.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot (no editing, menus or sorting); the host owns state. The table is a plain table of buttons and fields, not a full ARIA grid with arrow-key cell navigation.
