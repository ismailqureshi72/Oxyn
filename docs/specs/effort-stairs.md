**Status:** built v0.1 (2026-11-16, variants 1: Thinking Selector family).

# Spec: Effort stairs (`effort-stairs`)

Family: Thinking Selector. Reference: none (an Oxyn original) (layout idea only; names, wording and colours are our own).
Components: `EffortStairs` (Dioxus) · `EffortStairsEl` (GPUI), models in `oxyn_core::kit::{stairsfid, thinkfid}`.

## 1. Purpose
Show what each effort level costs before picking it: rising bars, one per level, and a rough budget line for the level under the pointer.

## 2. Parts
| Part | Look |
|---|---|
| Card | 300 px, 16 px radius, hairline, soft shadow |
| Heading | level 16/600 (red at Max); budget line 12.5 muted, tabular (`stairs_line`: "≈ 4k thinking tokens · ~8 s", or "No cap · as long as it needs") |
| Bars | one per level, heights 28 %→100 % (`stair_height`), 6 px gaps, 4 px radius; lit bars ink, the top lit bar red at Max; unlit grey with a hairline |

## 3. Behaviour
Click a bar to pick its level; hovering previews its name and budget (the bar lifts 6 %). Keys step levels.
Budgets double per step from 1k tokens / 2 s and are illustrative (`stairs_budget`). `on_change`.
prefers-reduced-motion: no transitions or pop-in.

## 4. Variants
high (default) · max · plain (no budget line).

## 5. Keyboard and accessibility
The bar row is one `role="slider"`; bars are click targets. The status line says each change.

## 6. Responsive
300 px card; bars share the width.

## 7. Known gaps
Budgets are fixed placeholders; a host should pass its own estimate (not yet a prop).
