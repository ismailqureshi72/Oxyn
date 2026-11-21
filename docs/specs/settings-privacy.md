**Status:** built v0.1 (2026-11-18, S6: Settings I).

# Spec: Settings · Privacy (`settings-privacy`)

Family: Screens. Reference: affectUI Settings / Privacy (`/screens/settings-privacy`), layout and behaviour only; no source or assets copied.
Components: `PrivacySettingsScreen` (Dioxus) · `PrivacySettingsEl` (GPUI), models in `oxyn_core::kit::setscr`.

## 1. Purpose
What is kept, what is shared, and where you are signed in.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| How your data is used | Switches for Improve the models, Product analytics and Ad measurement; Cookies (menu) and Product emails |
| Active sessions | Each device with app, place and last seen; This device is marked; Sign out per session and Sign out of all others |
| Shared links | Two tabs with counts (In the workspace, Public links); each link with views and when, Copy link (says Copied) and Stop sharing |
| Your data | Export your data as a stepped progress bar (collecting chats, files, memories, packing, writing the link) and Delete all chats, which needs the word delete typed |

Chrome is monochrome neutral; colour only means something: official marks (Google, model providers), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Switches go through the save bar; sessions, links, export and delete act at once and say what they did. Every other settings page (S6 and the Settings II pages) opens in place from the sidebar.
prefers-reduced-motion: no slides, fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Exporting** (an export running step by step to its link). Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every field has a label; a field with a problem is aria-invalid with its message beside it. Radio groups (segments, tiles, hues) move with the arrow keys; destructive actions ask first and focus the confirm. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-privacy/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 560 px rows put their control under the label and tiles go one per row. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every settings screen takes `data: Option<Prefs>` (`oxyn_core::kit::setscr::Prefs`: profile, preferences, notifications, privacy and general fields, and `ws: Ws` for members, models and keys, security, billing and the checklist). `None` shows the sample for the state; to show your own, start from `Prefs::default()` (the sample) and replace what you have. The save bar compares edits against this data.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, places and the workspace are fictional; model versions are fictional, marks are the providers' official ones.
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
