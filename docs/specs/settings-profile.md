**Status:** built v0.1 (2026-11-18, S6: Settings I).

# Spec: Settings · Profile (`settings-profile`)

Family: Screens. Reference: affectUI Settings / Profile (`/screens/settings-profile`), layout and behaviour only; no source or assets copied.
Components: `ProfileSettingsScreen` (Dioxus) · `ProfileSettingsEl` (GPUI), models in `oxyn_core::parity::setscr`.

## 1. Purpose
How you appear to the people and agents you work with.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| About you | Picture (initials or a picture, Upload / Change / Remove), Full name (checked, 60 max), Role (menu), Email (Change opens an inline field; a valid address is confirmed by link) |
| Time | Time zone (menu), Time format 12- / 24-hour (segmented, the hint shows the time in that format), First day of the week |
| Personal context | A text area with a 600-character counter |
| Sign-in | Google connected (official mark) and Password: Add or Change opens a form with a four-bar strength meter (red, amber, green) and a repeat field; Save needs Fair or better and a match |

Chrome is monochrome neutral; colour only means something: official marks (Google, model providers), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Edits collect in a save bar (N unsaved changes · Discard · Save changes); a field with a problem blocks Save and says why beside it. A changed row carries a dot after its label. Every other settings page (S6 and the Settings II pages) opens in place from the sidebar.
prefers-reduced-motion: no slides, fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Unsaved** (opens with three edits pending). Frame: Inset · Flat. Oxyn extras: `compact` (tighter rows without hints, plus a card that shows how others see you) and `autosave` (each valid change is kept at once, no save bar).

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every field has a label; a field with a problem is aria-invalid with its message beside it. Radio groups (segments, tiles, hues) move with the arrow keys; destructive actions ask first and focus the confirm. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-profile/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 560 px rows put their control under the label and tiles go one per row. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every settings screen takes `data: Option<Prefs>` (`oxyn_core::parity::setscr::Prefs`: profile, preferences, notifications, privacy and general fields, and `ws: Ws` for members, models and keys, security, billing and the checklist). `None` shows the sample for the state; to show your own, start from `Prefs::default()` (the sample) and replace what you have. The save bar compares edits against this data.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, places and the workspace are fictional; model versions are fictional, marks are the providers' official ones.
