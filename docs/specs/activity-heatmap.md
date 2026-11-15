**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Activity heat map (`activity-heatmap`)

Family: Data display. Closes the parity row(s): X #53 Threadbear activity; X #44 @flohoeller overview (Activity card). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A year of activity as week columns, Sunday rows first (`heat_year`, with `jan1_weekday` and `month_cols` for the labels). Five levels split at the quartiles of the non-zero days (`heat_levels`), so one busy day never washes the year out. Hovering a day, or arrowing through the grid, shows a tooltip (`heat_tip`: "5 chats · Mon, Oct 5"). With more than one year the header steps with ‹ › and the total rolls; the cells fade in column by column. `weeks` gives the compact strip of the last weeks used in overview cards. A footnote and a Less → More legend sit underneath.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Grid, legend | hover / keys / year |
| day | Outlined cell and tooltip | leave / Esc |
| year change | Cells fade in by column, total rolls | — |

## 3. Timing and motion
- Cell fade 350 ms, delayed 9 ms a column.
- Tooltip fades in over 120 ms.
- Reduced motion: no fade or roll.

## 4. Variants
| Variant | What differs |
|---|---|
| year | Full year with navigation |
| compact | Last 24 weeks, no labels |

## 5. Keyboard
| Key | Does |
|---|---|
| ← / → | Week back / forward |
| ↑ / ↓ | Day back / forward |
| Home / End | First / last day |
| Esc | Hide the tooltip |

## 6. Accessibility
- The grid is one focusable element with a summary label; the tooltip is role=status.
- Year buttons are labelled and disabled at the ends.

## 7. Responsive
- Scrolls sideways in its card below the year's width; the compact strip scales to the card.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| years | Vec<(i32, Vec<u32>)> | — | oldest first |
| title / noun / verb | String | — / chat / started |  |
| footnote | Option<String> | None |  |
| weeks | Option<u32> | None | compact strip |
| cell | f32 | 11 | px |

## 9. Gaps against the source
- Two heat maps side by side (Threadbear's GitHub row) are two instances; no data fetching.

## 10. GPUI
ActivityHeatmapEl draws the same cells as GPUI divs (year or `weeks`).
