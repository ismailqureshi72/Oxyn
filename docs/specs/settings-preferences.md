**Status:** built v0.1 (2026-11-18, S6: Settings I).

# Spec: Settings · Preferences (`settings-preferences`)

Family: Screens. Reference: affectUI Settings / Preferences (`/screens/settings-preferences`), layout and behaviour only; no source or assets copied.
Components: `PreferencesSettingsScreen` (Dioxus) · `PreferencesSettingsEl` (GPUI), models in `oxyn_core::parity::setscr`.

## 1. Purpose
How the assistant answers you: models, tools, look and what it remembers.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Models | Default model menu with official model marks (Auto, Opus 5.5, GPT-6 mini, Gemini 3.8 Pro); with Auto, three rows show which model it picks for which kind of message, sliding in one after another |
| Language and look | Language menu; Appearance as three tiles (System, Light, Dark) with a small preview each |
| Chat tools | Four tiles (Web search, Image generation, Code runner, Canvas) with a toned glyph and a switch; the heading counts how many are on |
| Custom instructions | A text area with a 1500-character counter, and Answer length |
| Memory | Remember things across chats (switch), each kept fact with Forget (×), and Clear all, which asks first |

Chrome is monochrome neutral; colour only means something: official marks (Google, model providers), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Switches, tiles and menus edit a working copy; the save bar counts changes. Forget and Clear all act on the list and are counted as one change. Every other settings page (S6 and the Settings II pages) opens in place from the sidebar.
prefers-reduced-motion: no slides, fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Memory off** (memory switched off: facts are neither shown nor used). Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every field has a label; a field with a problem is aria-invalid with its message beside it. Radio groups (segments, tiles, hues) move with the arrow keys; destructive actions ask first and focus the confirm. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-preferences/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 560 px rows put their control under the label and tiles go one per row. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
Every settings screen takes `data: Option<Prefs>` (`oxyn_core::parity::setscr::Prefs`: profile, preferences, notifications, privacy and general fields, and `ws: Ws` for members, models and keys, security, billing and the checklist). `None` shows the sample for the state; to show your own, start from `Prefs::default()` (the sample) and replace what you have. The save bar compares edits against this data.

## 8. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, places and the workspace are fictional; model versions are fictional, marks are the providers' official ones.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
