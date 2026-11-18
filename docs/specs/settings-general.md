**Status:** built v0.1 (2026-11-18, S6: Settings I).

# Spec: Settings · General (`settings-general`)

Family: Screens. Reference: affectUI Settings / General (`/screens/settings-general`), layout and behaviour only; no source or assets copied.
Components: `GeneralSettingsScreen` (Dioxus) · `GeneralSettingsEl` (GPUI), models in `oxyn_core::parity::setscr`.

## 1. Purpose
The workspace everyone here shares: its name, address and context.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Workspace | Icon: initials on one of six identity hues (radio group); Workspace name (checked); Workspace address `<slug>.oxyn.app` that follows the name until edited, with live checks (taken, too short, characters, hyphens), Available, and a Use … suggestion |
| Company context | A text area with a 1000-character counter; New chats are visible to (Only me, Members, Anyone with the link) |
| Danger zone | Delete workspace asks for the exact name before it schedules deletion (14 days) |

Chrome is monochrome neutral; colour only means something: official marks (Google, model providers), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Fields edit a working copy; a blocked field stops Save and says why. Delete is confirmed inline and never removes anything in the sample. The other S6 pages open in place from the sidebar; Members, Models, Security and Billing show a short summary until Settings II.
prefers-reduced-motion: no slides, fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Deleting** (the delete confirmation open with a partly typed name). Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every field has a label; a field with a problem is aria-invalid with its message beside it. Radio groups (segments, tiles, hues) move with the arrow keys; destructive actions ask first and focus the confirm. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-general/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 560 px rows put their control under the label and tiles go one per row. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, places and the workspace are fictional; model versions are fictional, marks are the providers' official ones. Members, Models, Security and Billing are summaries here; their full pages come with Settings II.
