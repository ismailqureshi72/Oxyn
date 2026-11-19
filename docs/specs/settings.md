**Status:** built v0.1 (2026-11-17, screens 1: Screens).

# Spec: Settings shell (`settings`)

Family: Screens. Reference: affectUI settings shell (the fourth frame), behaviour only.
Components: `SettingsScreen` (Dioxus) · `SettingsEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
The frame every settings page sits in, with nine pages to show what it holds.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar | Back to app, Settings title, Search settings (Enter opens the first page with a match, Escape clears), Personal (Account, Alerts, Chat defaults, Privacy) and Workspace (Workspace, People, Models, Sign-in, Plan) pages with match counts while searching, account menu |
| Page | Heading and one line, a match count with a jump to the first page that has one, sections of rows in cards |
| Rows | Label and hint, then a switch, a menu, a text field or area (checked once changed; a counter where there is a cap), an action, or a destructive action; a row that depends on a switch dims while it is off; a changed row is marked |
| Destructive rows | Confirm inline before acting; Delete workspace wants the workspace name typed |
| Save bar | n unsaved changes, Discard, Save changes (disabled while a field is invalid) |

## 3. Behaviour
Every control edits a working copy; the save bar counts rows that differ from the saved copy, Discard brings the saved one back and Save keeps it. Searching filters nothing away: pages without a match dim and matching rows are tinted. Under 640 px the settings sidebar slides in from the menu button and choosing a page closes it. Empty is a new workspace with blank fields and defaults.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
Data: Sample · Loading (skeletons) · Empty (each region says what lands there). Frame: Inset (page is a rounded card on the grey canvas) · Flat (edge to edge, hairlines). Light and dark follow the page theme.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; menus open from a trigger reporting aria-expanded (items are menuitemradio; Escape shuts them); hairlines are focusable separators with a value (←/→ resize); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Root fills its container (definite width, overflow clip, container queries). Under 860 px the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and composer menus drop their labels.

## 7. Known gaps
GPUI is a static snapshot of the Alerts page (no editing, search or save bar); the host owns state. Danger actions only report what they would do.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
