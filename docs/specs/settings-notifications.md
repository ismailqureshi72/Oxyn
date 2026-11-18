**Status:** built v0.1 (2026-11-18, S6: Settings I).

# Spec: Settings · Notifications (`settings-notifications`)

Family: Screens. Reference: affectUI Settings / Notifications (`/screens/settings-notifications`), layout and behaviour only; no source or assets copied.
Components: `NotificationsSettingsScreen` (Dioxus) · `NotificationsSettingsEl` (GPUI), models in `oxyn_core::parity::setscr`.

## 1. Purpose
What reaches you, where, and when it stays quiet.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Daily digest | Switch, Send at (menu), days as seven toggle pills; the heading says it in words (Weekdays at 08:00, Mon, Wed at 09:00, Off) |
| What to tell you | A table of six kinds (Approvals amber, Finished runs green, Failed runs red, Mentions blue, Weekly usage, Product news) by Email / Mobile / Inbox checkboxes; each row says where it reaches; a column header turns the whole column on, or off when it is all on |
| Quiet hours | Switch with the window in words, From – to menus, and Let approvals through |
| Test | Send a test notification |

Chrome is monochrome neutral; colour only means something: official marks (Google, model providers), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
The grid, days and windows edit a working copy; the save bar counts each cell. Quiet hours spanning midnight are counted correctly (22:00 – 07:00 is 9 hours). Every other settings page (S6 and the Settings II pages) opens in place from the sidebar.
prefers-reduced-motion: no slides, fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Quiet now** (a banner: quiet hours are on until 07:00, End now). Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every field has a label; a field with a problem is aria-invalid with its message beside it. Radio groups (segments, tiles, hues) move with the arrow keys; destructive actions ask first and focus the confirm. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-notifications/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 560 px rows put their control under the label and tiles go one per row. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, places and the workspace are fictional; model versions are fictional, marks are the providers' official ones.
