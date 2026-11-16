**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 6).

# Spec: Context Map (`context-map`)

Family: Agents. Closes the parity row: affectUI · Context Map (Pro). Behaviour-only rebuild; nothing is copied (the
session, agents, files and numbers in the demo are our own). Related Oxyn component: composer-context-meter (just the
fill level).
Components: `ContextMap` (Dioxus) · `ContextMapEl` (GPUI), model in `oxyn_core::parity::ctxmapfid`.

## 1. Purpose
A treemap of everything in an agent's context: each prompt, tool list, memory, file, chat turn and tool result is a
tile sized by tokens; tiles slide as the window fills, unused room stays hatched, and compaction folds old turns into one
summary tile.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 680 px (max 100 %), container-sized, no frame of its own |
| Bar | blue treemap glyph, "Context" 13/600, the agent button (maker mark, name, a "200k" chip, chevron → a menu of agents with model and window), a status pill ("Compacting"), Before \| After after a compaction, "30.4k / 200k" (the used count with a running shine) and a percent pill (amber at 90 %+) |
| Map | 282 px, hairline box; an ordered squarified treemap – tiles in the blocks' order, 4 px apart, 6 px radius, a tint and a tinted edge per kind (system grey, tools teal, memory and chat violet, files blue, results amber, summary green); label 12.5/500 and "14.8k · Tools"; small tiles keep only their colour |
| Free | white, dashed, faint 45° hatch, "Free" and the tokens left |
| Card | pointed tile: 250 px card – icon and name, "Tools · 14.8k tokens · 7.4%", and the items it holds in two columns |
| Line | "No compaction so far – …", or "194k → 110k" and kept / summarized into / dropped |

## 3. Behaviour
- `ctx_layout` keeps the order (stable parts first), so tiles glide to their new place and size when anything
  changes (CSS transitions on the absolute rects); new parts fade in.
- Pointing at a tile shows its card; a click (or Enter / Space) pins it, a second click or Escape lets go
  (`on_select`). The agent menu switches the window size (`on_agent_change`). Before \| After shows the blocks before
  the compaction with summarized and dropped parts dashed and tagged (`on_view_change`).
- `live` (demo): the session grows for 9 s, shows Compacting, then settles compacted.
- prefers-reduced-motion: tiles jump to their place.

## 4. Variants
live (default) · fresh · nearly-full · after-compaction · before-compaction · open-a-part (card) · agent-menu · narrow
(380 px) · classic (a grey band, saturated tiles with a solid edge, a white bordered percent).

## 5. Keyboard and accessibility
Tab reaches the agent button, Before \| After and every tile; Enter / Space pins a tile's card; Escape lets go and closes
the menu. The map is a list named by its title, every tile an item with name, kind and tokens; the fill level is text.

## 6. Responsive
680 px up to its container; the map keeps its height and the layout stretches. Under 480 px the map is 220 px and the
numbers take a line of their own.

## 7. Known gaps
- The layout is computed for the 680 × 282 aspect; narrower maps stretch it rather than re-squarify.
- The live demo plays once (affectUI loops grow → compact → reset); Before \| After switches without the grow-back tween.
- GPUI: a static snapshot – no card, menu, glide or shine.
