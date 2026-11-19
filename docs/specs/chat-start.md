**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Start (`chat-start`)

Family: Screens. Reference: affectUI AI Chat / Start (`/screens/assistant-home`), layout and behaviour only.
Components: `ChatStartScreen` (Dioxus) · `ChatStartEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
Where a new chat begins: a greeting over the exact composer, a folder and apps to work with, and suggestions that fill the prompt.

## 2. Parts
| Part | Look |
|---|---|
| Column | Centred 672 px on the 1440 stage, a little above the middle; the top bar is bare (no hairline, no title) |
| Greeting | By hour of day, 30 px / 600 |
| Composer | Field, then left: + (Add menu) and the approval chip (✋ Request approval); right: Auto ⌄ (model picker with coloured official marks), mic, round send (grey until there is text) |
| Add menu | Group 1: Files and folders, Attach app ›, Work in a folder, Link apps (n connected), Goal, Record a skill, Draw. Group 2, Plugins: Documents, PDF, Spreadsheets. Folder glyph #00A1FF, plugin glyphs in their file tones, the rest neutral |
| Strip under the box | Work in a folder (folder menu) on the left; Link apps and the connected apps' official marks on the right |
| Suggestions | Seven quiet buttons in two rows (neutral glyph + label); each fills the composer |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Enter sends (Shift+Enter breaks a line). The first message adds a chat to the sidebar (title from the first words), opens it, and the conversation takes over the page; the reply appears after a short Thinking state. New chat returns to the start. Add menu picks attach chips (files in their tones, apps with their marks), set the folder, add a Goal chip, start a Recording-a-skill chip, or connect the next app.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading (greeting and suggestions shimmer) · Empty (a hint that the first chat lands in the sidebar) · **Attached** (a PDF, a sheet and a Notion page attached) · **Dictating** (a live waveform replaces the field, the mic is pressed). Frame: Inset · Flat. Oxyn extras beyond affectUI: the approval chip has two more steps (Approve safe steps, Full access); Attach app opens a second list of apps; Goal and Record a skill chips; the Attached and Dictating states.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra("attached" | "dictating" | "plus" | "model" | "approval")` draws a state or a menu open; menus, typing and the live clock belong to the host.
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
