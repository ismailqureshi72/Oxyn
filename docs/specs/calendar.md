**Status:** v1 — implemented (plan #25).

# Spec: calendar

> Implemented in `oxyn-core::primitives::date` (`Date`, `Calendar`, `CalKey`, `DayCell`,
> `Blocked`, `Travel`), `oxyn-dioxus::Calendar`, `oxyn-gpui::CalendarEl`. Source: Arc UI Calendar
> (behaviour only, no source used). The date arithmetic is Oxyn's own (proleptic Gregorian, no
> time zone).

## 1. Purpose
A compact month view to pick one date: a due date, a run schedule, a checkpoint day. Also the grid
inside the date picker and the range picker.

## 2. State machine
```
Month(m), focus(f), value(v)
click enabled day ─▶ value = day (selection moves); day in another month ─▶ month follows
←/→ ─▶ focus ±1 day, skipping blocked days, crossing months (month slides)
↑/↓ ─▶ focus ±7 days (skips blocked weeks by repeating the step)
Home / End ─▶ start / end of the week (respecting week_starts_on)
PageUp / PageDown ─▶ ±1 month (Shift: ±1 year), clamped to the month's last day
Enter / Space ─▶ select focus
Today ─▶ month = today's, select today (when enabled)
prev / next ─▶ month ±1 unless it leaves [min, max] (aria-disabled)
```

## 3. States table
| Day state | Shows | ARIA |
|---|---|---|
| default | number | `gridcell` button named "Wednesday, November 18, 2026" |
| hover | soft surface | — |
| focus (keyboard) | 2 px focus ring | `aria-activedescendant` on the grid |
| today | dot under the number | `aria-current=date` |
| selected | dark fill, light number | `aria-selected=true` |
| outside month | 55 % | — |
| blocked / out of range | strike-through, `fg.disabled` | `aria-disabled=true` |
| heat (Oxyn plus) | tint 0–4 under the number | — |

## 4. Micro-interactions
Months slide in from the side you're travelling to (later → from the right). The title rolls. The
selection moves between days on a snap. The Today button jumps and slides like a page.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Month slide | 240 ms, 20 px | `ease.out` | 1 ms |
| Title roll | 220 ms | `ease.out` | swap |
| Selection | 214 ms `spring.snap` colour | spring | instant |
| Hover | 120 ms | `ease.out` | 80 ms |

## 6. Variants
Free: value, min / max, blocked weekends, week starts Sunday or Monday, Today button.

**Oxyn plus:** ISO week numbers column; activity heat overlay (runs per day) with a legend;
`on_month_change`.

## 7. Edge cases
Leap years and month ends (31 Jan + 1 month = 28/29 Feb). A value outside min / max shows but
can't be re-picked. Six-row months keep the grid height steady. Narrow containers (< 260 px): the
title takes its own row. Tiny panels: cells stay square and scale. RTL: the grid mirrors, ← moves
to the next day visually.

## 8. Accessibility
One focusable `role=grid` with `aria-activedescendant` (a single tab stop); weekday headers are
`columnheader`s with full names. The month title is a polite live region. Pagers are labelled and
`aria-disabled` at bounds. Keys follow the APG date-grid pattern.

## 9. Props sketch
```rust
Calendar { value: Option<String /* ISO */>, today, min, max, blocked: "none" | "weekends",
           week_starts_on: u8, show_today, week_numbers, activity: Vec<(String, u8)>, label,
           on_change: EventHandler<String>, on_month_change: EventHandler<String> }
```

## 10. Related components and references
`date-picker.md`, `date-range-picker.md`, `date-reel.md`. Arc `/components/calendar` — behaviour
only. WAI-ARIA APG date picker dialog.
