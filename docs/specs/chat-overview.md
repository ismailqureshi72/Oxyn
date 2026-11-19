**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Overview (`chat-overview`)

Family: Screens. Reference: affectUI AI Chat / Overview (`/screens/assistant-overview`), layout and behaviour only.
Components: `ChatOverviewScreen` (Dioxus) · `ChatOverviewEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
The start composer with the numbers under it, in one centred 880 px column.

## 2. Parts
| Part | Look |
|---|---|
| Column | Centred 880 px; the mark (monogram + name) over the 672 px start composer (with its strip) |
| Activity | Day-by-day heat map, 38 weeks × 7 days in blue steps; a status line (872 chats · 214 files · 38 automations) and a range menu (Feb – Nov, Last 12 weeks) |
| Recent chats | Three rows with a chat glyph and the time, n more / Show less |
| Usage | 1,284 / 2,000 credits, a blue meter, Resets 1 Dec; amber with Upgrade near the limit |
| Files made for you | Rows with file glyphs in their tones (md blue, csv green, png pink), 2 more |
| This month | Chats 36 +12 (green) and Files made 14 −3 (red) |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Arrow keys step through the heat map (↑/↓ a day, ←/→ a week) and the status line reads the day; the range menu narrows the map; the more buttons expand the lists. The composer works as on Start.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading · Empty (zeros and one line per card) · **Near limit** (1,870 of 2,000: amber meter, 93% used, Upgrade). Frame: Inset · Flat. Oxyn extras: the range menu, the keyboard heat map, the Near limit state.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot (`.extra("limit")` for the near-limit meter).
In a window, `ScreenView` (oxyn-gpui, `parity::screenview`) runs this screen live from `oxyn_core::parity::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
