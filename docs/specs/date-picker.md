**Status:** v1 — implemented (plan #25).

# Spec: date picker

> Implemented in `oxyn-core::primitives::date` (`DatePicker`, `DateFormat`, `parse_natural`,
> `Travel`) over `Calendar`, `oxyn-dioxus::DatePicker`, `oxyn-gpui::DatePickerEl`. Source: Arc UI
> Date picker (behaviour only, no source used).

## 1. Purpose
A form field for one date — a due date, when a scheduled agent should start. Closed it reads like
a text field; open it shows the calendar.

## 2. State machine
```
Closed(value) ─click / Enter / Space / ↓─▶ Open(focus = value, else today)
Open ─calendar keys─▶ Open(focus')
Open ─pick─▶ Closing(value', highlight lands) ─240 ms─▶ Closed(value'), parts roll
Open ─Esc / outside─▶ Closed (value unchanged), focus back to the trigger
Open ─type words + Enter─▶ value = parse_natural(...)         Oxyn plus
Open ─preset chip─▶ value = Today | Tomorrow | Next week       Oxyn plus
Closed(value) ─Clear─▶ Closed(None)
```

## 3. States table
| State | Trigger | Popover | ARIA |
|---|---|---|---|
| placeholder | calendar icon, "Select a date" | — | `aria-haspopup=dialog`, `aria-expanded=false` |
| value | formatted date, Clear × | — | hidden span carries the value |
| open | edge darkens | calendar dialog | `dialog` named "Due date calendar" |
| changed | only the changed parts roll (up for later) | — | parts `aria-hidden` |
| words (Oxyn plus) | input + live parse hint | — | labelled "Type a date" |
| disabled | 50 % | — | `disabled` |

## 4. Micro-interactions
The popover grows from the trigger's corner. After a pick it waits 240 ms so the selection can land
before closing. In the field, only the parts that changed (month, day, year) roll — up when the new
date is later, down when earlier.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Popover in | 200 ms, scale .97 → 1 | `ease.out` | 80 ms fade |
| Close after pick | 240 ms delay | — | 0 ms |
| Part roll | 220 ms, 0.6 em, blur 2 → 0 | `ease.out` | swap |
| Calendar | see `calendar.md` | | |

## 6. Variants
Free: placeholder, description, min / max, blocked weekends, format (medium, long, weekday,
day-first, month-first, ISO), Today button, disabled.

**Oxyn plus:** natural-language entry (`today`, `tomorrow`, `next tue`, `in 2 weeks`, `+3d`,
`dec 24`, ISO); relative preset chips; review-sheet `open`.

## 7. Edge cases
Words that don't parse show "Not a date yet" and change nothing. Parsed dates outside min / max are
refused. Narrow screens (< 360 px): the popover stretches to both field edges. Long formats
ellipsize in narrow fields. RTL: the popover anchors to the inline start.

## 8. Accessibility
The trigger is a button labelled by the label and a hidden value; the rolling parts are
`aria-hidden`. The popover is a `dialog`; focus moves into the grid and returns to the trigger on
close. Esc closes from anywhere inside.

## 9. Props sketch
```rust
DatePicker { label, value: Option<String>, description, placeholder, today, min, max, blocked,
             format, show_today, natural, presets, open, disabled,
             on_change: EventHandler<Option<String>> }
```

## 10. Related components and references
`calendar.md`, `date-range-picker.md`, `popover` (`tooltip-popover.md`), `date-reel.md`. Arc
`/components/date-picker` — behaviour only.
