**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Token usage chart (`token-usage-chart`)

Family: Data display. Closes the parity row(s): X #21 @pitus_ 100% AI-generated designs (thread): token usage card. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Daily tokens as an area with the cached share as a dashed line (`spark_path`). Hovering a day or arrowing through the plot draws a rule and a tooltip with the date, Tokens and Cached. Range buttons (30d, 90d, All) re-window the plot, the date line and the totals: Total usage, Avg. daily usage and Est. cost (`usage_totals`: uncached tokens at `per_m_cents` a million, cached at a tenth), which roll to their new values.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Area, line, totals | hover / keys / range |
| day | Rule and tooltip | leave / Esc |
| range change | Plot fades in, totals roll | — |

## 3. Timing and motion
- Plot fade 350 ms; totals roll 260 ms; tooltip 120 ms.
- Reduced motion: no fade or roll.

## 4. Variants
| Variant | What differs |
|---|---|
| quarter | 93 days, three ranges |
| short | 14 days |

## 5. Keyboard
| Key | Does |
|---|---|
| ← / → | Previous / next day |
| Esc | Hide the tooltip |

## 6. Accessibility
- The plot is one focusable image with totals in its label; the tooltip is role=status; range buttons are aria-pressed.

## 7. Responsive
- Up to 420 px; the area stretches and strokes stay 1.5 px (non-scaling).

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| days | Vec<UsageDay> | — | tokens and cached |
| start | (i32, u32) | (2026, 177) | year and day of the first day |
| per_m_cents | f64 | 300 |  |
| title | String | Token usage |  |
| ranges | Vec<usize> | [30, 90, 0] | 0 = all |

## 9. Gaps against the source
- The reference's companion cards (thinking timeline with file chips, voice waveform composer) are covered by the existing thinking-traces and voice-recorder components.

## 10. GPUI
TokenUsageChartEl shows the days as bars with the same totals; `hover` marks a day.
