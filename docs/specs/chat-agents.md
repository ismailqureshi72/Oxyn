**Status:** built v0.2 (2026-11-17, S1: AI Chat parity; v0.1 screens 1).

# Spec: AI Chat · Agents (`chat-agents`)

Family: Screens. Reference: affectUI AI Chat / Agents (`/screens/agent-chat`), layout and behaviour only.
Components: `ChatAgentsScreen` (Dioxus) · `ChatAgentsEl` (GPUI), models in `oxyn_core::parity::screensfid`.

## 1. Purpose
A chat with the orchestrator and its crew at work, beside the brief they are filling in.

## 2. Parts
| Part | Look |
|---|---|
| Top bar | Crumb Home / Ads for the overshirt, a ⋯ menu |
| Chat | Your message on the right, the reply as plain paragraphs, “Worked for 3m 38s” with a blue agent glyph, the Agents working chips (Launch lead, Brand scout, Studio, Audience desk, Art direction; neutral glyphs, the working ones pulse), the Orchestrator line (Routes each request) over the composer |
| Work area (368) | A tab strip (Overshirt launch ×, +), Brief / Assets / Activity, the product card with a green Top seller tag, Selling angle radios with green signal bars, Audience chips, Channels with switches, Palette swatches with names, Formats, and a sticky footer: 5 ads · ~2 min · Generate ads |

Chrome is monochrome neutral; colour only means something: official brand and model marks, semantic object glyphs (folder #00A1FF, file types), status (running blue, needs-you amber, done green, failed red).

## 3. Behaviour
Chips toggle which agent's status line shows; angles are a radio group; audience and format chips and channel switches change the ad estimate; Generate ads makes the ads one at a time (Assets shows them filling in) and the footer counts up.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
States (the full view's state bar): Data · Loading · Empty · **Generating** (the ads already being made). Frame: Inset · Flat. Oxyn extras: the agent status line, the live ad estimate, Generating.

## 5. Keyboard and accessibility
Sidebar is a labelled nav with aria-current on the open chat; the composer's Add menu is a menu with aria-activedescendant (↑/↓/Home/End move, Enter or Space picks, → opens Attach app and ← or Escape goes back, Escape closes); the approval chip and Auto ⌄ open menus of menuitemradio rows (Escape shuts them, a click outside closes them); Enter sends and Shift+Enter breaks a line; Send is disabled until there is text and the mic reports aria-pressed; hairlines are focusable separators with a value (←/→ resize, double-click resets); skeletons are aria-hidden; live notes use role=status.

## 6. Responsive
Every screen is authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/<slug>/full`) renders at the real viewport with the state bar under it (R2). Under 860 px of real width the work area becomes a sheet opened from the top bar; under 640 px the sidebar slides in from a menu button and the composer's approval and model chips drop their labels (icons stay). Every label in a row, chip or card title ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot (`.extra("making")` for Generating).
