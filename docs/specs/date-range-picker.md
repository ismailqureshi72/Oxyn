**Status:** v1 — implemented (plan #25).

# Spec: date range picker

> Implemented in `oxyn-core::primitives::date` (`RangePicker`, `DateRange`, `Compare`, `Preset`,
> `Band`, `default_presets`) over `Calendar`, `oxyn-dioxus::DateRangePicker`,
> `oxyn-gpui::RangePickerEl`. One model for affectUI Date Range Picker and Arc UI Date range picker
> (behaviour only, no source used).

## 1. Purpose
Choose a span for usage, runs, cost or logs, often compared with the span before it. The user
starts from a preset most of the time and fine-tunes with two clicks.

## 2. State machine
```
Closed(value) ─click─▶ Open(draft = value, compare = default)
Open ─preset─▶ draft = preset range (months jump to show it)
Open ─click day (no anchor)─▶ Anchored(start) ─hover─▶ preview band follows the pointer / focus
Anchored ─click day─▶ draft = [min(a, b), max(a, b)]
Open ─compare: Off | Previous period | Previous year─▶ comparison band updates
Open ─Apply─▶ Closed(value = draft) + on_change(range, compare range)
Open ─Cancel / Esc / outside─▶ Closed(value unchanged)
Apply is disabled while only a start is picked.
```

## 3. States table
| State | Shows | ARIA |
|---|---|---|
| trigger | icon, "Oct 15 – Nov 13, 2026", "vs Sep 15 – Oct 14" | dialog trigger named with the range |
| presets | rail (Today, Yesterday, Last 7 / 30 / 90 days, This / Last month, Last 12 months) | listbox, `aria-selected` |
| range | dark ends, grey band between | `aria-selected` inside |
| preview | lighter band after the first click | — |
| compare | hatched band, outlined ends | — |
| today | dot | `aria-current=date` |
| footer | "Oct 15 – Nov 13 · 30 days · vs …", Cancel / Apply | status announces changes |

## 4. Micro-interactions
The panel grows from the trigger. Months slide when paging. The band grows to the hovered day.
The trigger label rolls on Apply. The preset rail highlight moves to the matching preset (or none
for a custom range).

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Panel in | 220 ms | `ease.out` | 80 ms fade |
| Band / preview | 120 ms colour | `ease.out` | instant |
| Month slide | 240 ms | `ease.out` | 1 ms |
| Trigger roll | 220 ms | `ease.out` | swap |

## 6. Variants
Free: one or two months, compare on / off with a default, week starts Sunday / Monday, min / max
(max defaults to today), default open.

**Oxyn plus:** agent-run presets (Last 90 days, Last 12 months); footer with day count and the
comparison spelled out; keyboard preview with arrows after the first pick.

## 7. Edge cases
Reverse picks (end before start) are ordered. Ranges across a year show both years. Comparison for
leap days clamps. Very narrow (< 600 px): presets become a scrolling chip row, one month shows, the
footer stacks. Max today: future days are disabled.

## 8. Accessibility
The trigger's name includes the current range. Each month is a `grid` with full-date cells,
`aria-selected` for every day inside the range and `aria-current` for today. A polite region
announces "Start Oct 15, pick an end" and then the range with its length. Compare is a
`radiogroup`.

## 9. Props sketch
```rust
DateRangePicker { label, start: Option<String>, end: Option<String>, placeholder, today, compare,
                  default_compare: "off" | "previous" | "year", week_starts_on, months: u8, min,
                  max, default_open, on_change: EventHandler<RangeValue> }
```

## 10. Related components and references
`calendar.md`, `date-picker.md`, `run-meter.md` (usage charts). affectUI
`/components/date-range-picker`; Arc `/components/date-range-picker` — behaviour only.
