**Status:** built v0.1 (2026-11-16, variants 2: Context Visualizer family).

# Spec: Context stack (`context-stack`)

Family: Context Visualizer. Reference: none (an Oxyn original).
Components: `ContextStack` (Dioxus) · `ContextStackEl` (GPUI), models in `oxyn_core::kit::ctxvisfid`.

## 1. Purpose
Make compaction physical: the window is a pile, the chat is the top slab, and there is a line it may not cross.

## 2. Parts
| Part | Look |
|---|---|
| Column | 112 px, dashed outline over a faint hatch (free space), slabs bottom-up in load order (`ctx_load_order`), heights by share (`slab_heights`, at least 12 px) |
| Line | red dashed line at `auto` % with "Compacts at 92%" |
| Legend | kinds top-down with swatch, name and tokens; pointing at a slab or a row lights the pair (the slab slides out 6 px) |
| Actions | Add a turn · Compact · Reset; a note line under them |
| Summary | after a compaction the chat slab becomes a striped summary slab |

## 3. Behaviour
Add a turn adds `per_turn` tokens to the chat (`ctx_add_turn`); when that reaches `auto` %, it compacts on its own and says so. Compact does it by hand (then disabled); Reset restores the start. Slabs glide to their new height. `on_compact(auto)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · early line (85 %) · mono (big turns).

## 5. Keyboard and accessibility
The column is `role="img"` labelled by `ctx_said`; the legend repeats every kind as text; each action reports in a status line.

## 6. Responsive
340 px card; the legend takes what the column leaves.

## 7. Known gaps
Slabs are not draggable. GPUI: static snapshot (`turn`, `compact`).
