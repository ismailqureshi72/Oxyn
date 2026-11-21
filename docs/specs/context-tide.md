**Status:** built v0.1 (2026-11-16, variants 2: Context Visualizer family).

# Spec: Context tide (`context-tide`)

Family: Context Visualizer. Reference: none (an Oxyn original).
Components: `ContextTide` (Dioxus) · `ContextTideEl` (GPUI), models in `oxyn_core::kit::ctxvisfid`.

## 1. Purpose
A calm, glanceable fill gauge for a sidebar: how high the water is and how many turns are left before it compacts.

## 2. Parts
| Part | Look |
|---|---|
| Glass | 92 × 220 px, rounded more at the bottom, inset grey; marks at 75 and at `auto` (red) |
| Water | height = percent, a slow moving wave on top; grey, amber from 75 %, red at the line (`tint`) |
| Side | "Context", percent 40/300 tabular, "166.4K of 200K tokens", "≈ N turns before it compacts" |
| Actions | Send a turn · Compact · Reset |

## 3. Behaviour
Send a turn raises the water by `per_turn` tokens; reaching the line compacts on its own (the water drains). Compact drains by hand. `on_change(used)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · quiet (no tint) · big turns.

## 5. Keyboard and accessibility
The glass is `role="meter"` with `aria-valuenow` (percent) and value text (tokens). Turns left are written out; changes are said in a status line.

## 6. Responsive
340 px card; glass and text side by side.

## 7. Known gaps
GPUI: static snapshot without the wave (`turn`, `compact`).
