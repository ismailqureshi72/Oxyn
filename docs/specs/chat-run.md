**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Run (`chat-run`)

Family: Screens. Reference: affectUI AI Chat / Run (`/screens/agent-run`), layout and behaviour only.
Components: `ChatRunScreen` (Dioxus) · `ChatRunEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
One agent run: its plan ticking through, the apps it uses, live time and cost, and what needs your input.

## 2. Parts
| Part | Look |
|---|---|
| Top bar | Crumb Runs / Write the November member update, a status pill (Running blue, Needs you amber, Done green), Stop run, a ⋯ menu |
| Run | The agent and who started it, the title, a blue progress bar, Uses (the apps' official marks), Steps / Tool calls / Time / Cost |
| Plan card | Each step: a blue tick (done), a spinner (working), a hand (needs you) or its number, the title, why and the tool; seconds; a footer line Running · 2 of 5 |
| Activity card | Done steps with “Used 1 tool” (official mark, seconds) that opens the call |
| Work area (348) | Tab Needs your input ×, +: the bell and “Nothing needs you yet” until a step needs a yes, then an amber card (Send the member update? · mail.send to … · Hold / Send it); the Draft with tracked changes (count, ✕ / ✓ for the current one, Reject all, Accept all); Files |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Steps rename (Enter), move (Alt+↑/↓) and delete (Delete); Stop run freezes the clock; the run waits at the send step until Send it or Hold; Y / N accept or reject the current suggestion in the draft.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading · Empty · **Needs input** (waiting at the send step) · **Done** (every step done, the draft accepted). Frame: Inset · Flat. Oxyn extras: plan editing, Hold, the Done state.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot (`.extra("input" | "done")`).
