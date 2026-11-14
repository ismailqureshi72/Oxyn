**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Slot text (`slot-text`)

Family: Loaders & text ([status-text.md](status-text.md)). Closes the parity row(s): Arc UI · Slot text.

## 1. Purpose
Text and numbers that spin into their new value on reels: digits roll through 0–9 plus `spins` extra turns, other changed characters flip, reels stop left to right `stagger_ms` apart after `duration_ms` with a fast start and a soft landing that never passes the target; blur follows each reel's speed; new characters grow in and removed ones collapse; reels match from the end for numbers so $999 → $1,000 only spins digits; changing the value mid-spin restarts from what is on screen. `group` formats en-US.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| settled | Value |  |
| spinning | Reels moving | → settled |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| number | Grouped number |
| money | Currency |
| status | Words |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- The value is the accessible name; with announce a polite status.

## 7. Responsive
- Inherits font size and colour.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | SlotText | — | from, to, at, duration_ms (900), stagger_ms (70), spins (1), align_end |
| now | ClockMs | — |  |
| announce | bool | false |  |

## 9. Edge cases
- Unchanged reels never move.
- Reel widths do not spring (characters are tabular).

## 10. Implementation
- Core: `SlotText`, `Reel`, `group` (`oxyn_core::parity::text`), with tests.
- Dioxus: `SlotReels` (parity/text.rs, text.css).
- GPUI: `SlotTextEl` (reels at the clock, grow and collapse widths; `reduced(true)` shows the landed value). No speed blur.

## 11. References
- Arc UI Slot text (https://uiarc.dev/components/slot-text) — behaviour-only rebuild, no source copied.
