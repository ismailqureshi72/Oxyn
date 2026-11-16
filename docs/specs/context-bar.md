**Status:** built v0.1 (2026-11-16, variants 2: Context Visualizer family).

# Spec: Context bar (`context-bar`)

Family: Context Visualizer. Reference: a user screenshot of a context card (stacked segment bar, kind chips, Compact) and the behaviour of a public context-meter block (bar or field of dots, colour or mono, legend cross-highlight, Compact / Undo). Behaviour only; layout, names, wording and colours are our own.
Components: `ContextBar` (Dioxus) · `ContextBarEl` (GPUI), models in `oxyn_core::parity::ctxvisfid`.

## 1. Purpose
Say in one glance how full the window is and which kind fills it, with one action that gives room back.

## 2. Parts
| Part | Look |
|---|---|
| Card | 320 px, 18 px radius, hairline, soft shadow; neutral greys (`--oxth-*`) |
| Head | "Context" 14/500 and a Compact pill (ink) that turns into a grey Undo pill |
| Figure | used tokens 34/400 tabular (`fmt_tokens`: 166.4K) and "of 200K" 13 muted; rolls in when it changes |
| Bar | 40 px, 4 px gaps, 5 px radius segments, widths by share (`ctx_shares`), free space grey |
| Field | `look="field"`: 20 × 10 dots, one per half percent, coloured by `field_cells` in load order |
| Legend | pills 26 px with a 9 px kind swatch; kind colours from `kind_hex`, greys from `kind_mono` |

## 3. Behaviour
Pointing at (or focusing) a kind dims the others to 32 % and puts that kind's own count and name in the figure. Compact replaces the chat with a summary of about 18 % of it (`ctx_compact`), segments glide to their new widths with a little overshoot and the figure rolls; Undo restores. `on_compact(bool)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
bar (default) · field · mono · big window (1M, compacted).

## 5. Keyboard and accessibility
The bar or field is `role="img"` labelled by `ctx_said` ("Context: 166.4K of 200K tokens, 83 percent"). Legend chips are buttons named with kind and tokens. Compact is a toggle (`aria-pressed`); the result is said in a polite status line.

## 6. Responsive
Card shrinks with its container; the field keeps 20 columns; the legend wraps.

## 7. Known gaps
No sound on compaction. GPUI: static snapshot (host routes `compact` / `undo` / `kind-<i>`).
