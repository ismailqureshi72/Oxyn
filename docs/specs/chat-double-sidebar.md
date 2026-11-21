**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Double Sidebar (`chat-double-sidebar`)

Family: Screens. Reference: affectUI AI Chat / Double Sidebar (`/screens/agent-chat-rail`), layout and behaviour only.
Components: `ChatRailScreen` (Dioxus) · `ChatRailEl` (GPUI), models in `oxyn_core::kit::screensfid`.

## 1. Purpose
A chat with the web beside its artifacts, under a two-column sidebar.

## 2. Parts
| Part | Look |
|---|---|
| Rail (52) | Workspace monogram, Chats, Tasks, Runs, Files, Apps (neutral glyphs, the current one filled); at the bottom notifications (blue dot), settings and the avatar |
| Chats column (272) | All projects ⌄ switcher and a search toggle, New chat, Pinned, Projects (blue folders; the open one lists its chats, the current one filled), Chats with unread dots; it shares one card with the page |
| Thread | Your messages on the right; each answer has the Kestrel mark and time, a green-ticked “Ran 3 searches · 4.1s” line that opens the calls, the answer, and chips for the files it made |
| Work area (408) | Tabs (fabric-compare.csv, sources.md, +), the file header, a toolbar (undo, bold, sort up / down, add a row), the cell field (B2 · fx · value) and a grid with column letters and row numbers; the selected column and row light blue |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
The rail's Chats icon or the page's sidebar button folds the column; the project switcher filters projects; search filters chats (Escape clears); a file chip opens its tab; the grid moves with the arrow keys, typing in the value field edits the cell and Enter moves down; tabs close.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading · Empty (“No projects or chats yet”) · **Chats folded** (the rail alone). Frame: Inset · Flat. Oxyn extras: the search toggle, the editable sheet with undo and sort, Chats folded.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot (`.extra("folded")` folds the column).
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
