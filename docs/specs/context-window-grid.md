**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Context window (grid) (`context-window-grid`)

Family: Meters ([composer-context-meter.md](composer-context-meter.md)). Closes the parity row(s): Arc UI · Context window. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
An AI context window as 200 cells in 20 columns. Items (`GridItem { cat, label, tokens }`) are grouped by category in fill order: system prompt, tools, files, summary, messages; the reply reserve fills the tail as dashed cells. A dashed threshold row marks where compaction starts (`threshold`, 0.8). Send message and Attach file add the next turn or file; crossing the threshold compacts on its own: messages fold into a summary (about 6 %, 4K–24K tokens, `summarize`) placed after the files, and the status line reports folded count, summary size and before → after. Compact now does the same by hand. New tokens fill in a wave from where the change happened (6 ms a cell, capped at 360 ms). Switching models (a Morph select) re-scales every cell from the first. Pointing at a legend row lights its cells.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Grid, totals, legend | actions |
| auto-compacted | Summary lands, freed cells clear, status | — |

## 3. Timing and motion
- Wave 6 ms per cell from the origin, max 360 ms; numbers roll 260 ms; status crossfades 200 ms.
- Reduced motion changes cells and numbers at once.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Send until it compacts |
| near | Close to the threshold |
| early | Low threshold |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Model menu, Send message, Attach file, Compact now |
| Enter / Space | Open the menu or run an action |
| ↑ / ↓, Enter | Choose a model |
| Esc | Close the menu |

## 6. Accessibility
- The grid is role=img with totals and the threshold; the status line is role=status.

## 7. Responsive
- Twenty columns scale with the card.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| models | Vec<CtxModel> | — | id, name, window, reserve |
| items | Vec<GridItem> | — |  |
| threshold | f64 | 0.8 |  |
| turns / files | Vec<(String, u64)> | — | what the actions add |
| cells | usize | 200 |  |
| now | ClockMs | — | menu type-ahead |
| on_compact | EventHandler<(usize, u64, u64, u64, bool)> | — | folded, summary, before, after, automatic |

## 9. Gaps against the source
- A custom `summarize` function is not a prop; the 6 % rule is fixed.

## 10. GPUI
ContextGridEl draws the grid and threshold row from `ContextGrid::cells`.
