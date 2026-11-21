**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Reasoning panel (`reasoning-panel`)

Family: Thinking Selector. Reference: user-supplied screenshots of a model-details side panel with a reasoning slider,
Fast mode and Favorite (layout idea only; the model, names and wording are our own; the model is fictional).
Components: `ReasoningPanel` (Dioxus) · `ReasoningPanelEl` (GPUI), models in `oxyn_core::kit::{thinkfid, reasonfid}`.

## 1. Purpose
The details of one model next to a model list: its context length, how hard it reasons, whether Fast mode is on, and a
way to keep it in favorites. Every control has a single-letter key so a keyboard user never leaves the list.

## 2. Parts
| Part | Look |
|---|---|
| Root | 340 px panel, 14 px radius, hairline, soft shadow, neutral greys (`--oxth-*`) |
| Head | globe-like model mark + model name 15.5/600 |
| Context row | "Context length" muted · value (`context_label`: 1.1M, 512K) |
| Reasoning row | "Reasoning" + R key cap · level name (red at the top level) |
| Track | 22 px inset bar, 7 px radius: grey fill up to the 18 px rounded-square white thumb; notches mark the levels still ahead |
| Max | the fill turns into a 56 × 4 grid of red dots (`pixel_fill`, deterministic), wiping in left to right |
| Fast mode | "Fast mode" + F key cap · 36 × 21 switch (on = ink) |
| Favorite | full-width 36 px button with a star; "Favorited" with a filled star when on |

## 3. Behaviour
- R anywhere in the panel steps to the next level and wraps after the top (`effort_cycle`); F flips Fast mode
  (`panel_key`, ignored when Ctrl/Cmd/Alt is held). The slider also takes press, drag (snapping) and arrows.
- Controlled or uncontrolled for each of level, fast and favorite; `on_change`, `on_fast_change`, `on_favorite_change`.
- A model without Fast mode (`has_fast: false`) hides the row and the F key.
- prefers-reduced-motion: no thumb glide, no dot wipe.

## 4. Variants
high (default) · max (red dot fill) · fast (Fast mode on, favorited) · low.

## 5. Keyboard and accessibility
The bar is `role="slider"` named Reasoning with `aria-valuetext` = level name and `aria-keyshortcuts="R"`. Fast mode is
`role="switch"` with `aria-keyshortcuts="F"`; Favorite is a toggle button (`aria-pressed`). A polite status line says
each change.

## 6. Responsive
340 px, shrinking with its container; rows keep label left, value right.

## 7. Known gaps
- The model list beside the panel (number keys per model) is not part of this component; pair it with Model Picker.
- GPUI: static snapshot; the level is chosen by pressing its stretch of the bar.
