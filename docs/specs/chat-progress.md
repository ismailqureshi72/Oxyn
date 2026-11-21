**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Progress (`chat-progress`)

Family: Screens. Reference: affectUI AI Chat / Progress (`/screens/agent-chat-progress`), layout and behaviour only.
Components: `ChatProgressScreen` (Dioxus) · `ChatProgressEl` (GPUI), models in `oxyn_core::kit::screensfid`.

## 1. Purpose
A chat whose tool calls open into cards, with its working notes in a column inside the page.

## 2. Parts
| Part | Look |
|---|---|
| Chat (858) | Your messages, tool-call cards (official mark, mono tool name, what it did, a blue tick and seconds, a chevron that opens the call), the answers; the composer says Reply… |
| Progress column (300) | In the page, full height, hairline on its left, × to close: Progress (n earlier steps folded, then the steps with ticks on a line), Outputs (a document preview, then rows with tinted file glyphs and their kind), Used in this session (Uploads, Connectors with official marks, Web search pages, Skills); each heading folds |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Earlier steps unfold; headings fold; tool cards open; × closes the column and a Progress button brings it back (below 860 px the column is a sheet from that button).
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading · Empty · **Working** (steps tick off live, the third call spins, Send turns into Stop). Frame: Inset · Flat. Oxyn extras: foldable headings, the Working state, the narrow sheet.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot (`.extra("working")`).
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
