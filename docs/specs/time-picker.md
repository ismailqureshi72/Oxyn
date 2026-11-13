**Status:** v1 — implemented (plan #25).

# Spec: time picker

> Implemented in `oxyn-core::primitives::date` (`TimeOfDay`, `HourCycle`, `time_options`,
> `nearest_option`, `Travel`), `oxyn-dioxus::TimePicker`, `oxyn-gpui::TimePickerEl`. Source: Arc UI
> Time picker (behaviour only, no source used).

## 1. Purpose
Pick a time of day: when a recurring agent runs, a reminder, a meeting. The list is short enough to
scan and fixed to sensible steps.

## 2. State machine
```
Closed(value) ─click / Enter / Space / ↓ / ↑─▶ Open(active = nearest option to value, scrolled into view)
Open ─↑ / ↓ (wraps)─▶ Open(active')
Open ─Enter / Space / click─▶ Closed(value'), label rises (later) or drops (earlier)
Open ─Esc / blur─▶ Closed(value)
```

## 3. States table
| State | Trigger | List | ARIA |
|---|---|---|---|
| placeholder | clock, "Select a time" | — | `aria-haspopup=listbox` |
| value | "9:30 AM" (+ zone tag) | — | hidden value in the name |
| open | chevron turned | times at the step, centred on the value | `listbox`, `aria-activedescendant` |
| active | soft highlight | — | — |
| selected | check | — | `aria-selected=true` |
| disabled | 50 % | — | `disabled` |

## 4. Micro-interactions
The shown time rises in from below when the new time is later and drops from above when it is
earlier. The list opens scrolled to the selected time and scrolls itself, never the page.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| List open | 180 ms | `ease.out` | 80 ms fade |
| Value rise / drop | 180 ms, blur 3 → 0 | `ease.out` | crossfade |
| Highlight | 214 ms `spring.snap` | spring | jump |

## 6. Variants
Free: step 1 / 5 / 10 / 15 / 30 minutes, 12- or 24-hour, description, disabled.

**Oxyn plus:** a window (`08:00-18:00`) that limits the list to working hours; a time-zone tag;
typed entry parsing in the core (`TimeOfDay::parse`: `9`, `9:30`, `930`, `9:30 pm`, `21:30`) —
the typed field on the trigger is an open gap.

## 7. Edge cases
A value off the step selects the nearest option for the highlight but keeps the exact value. 1-
minute steps make 1,440 rows: the list stays virtual-friendly by capping height and scrolling. 12 AM
/ 12 PM read correctly. Narrow screens: the list spans the field.

## 8. Accessibility
Same pattern as `select.md`: a trigger button with `aria-haspopup=listbox`, `aria-expanded`,
`aria-controls` and `aria-activedescendant`; options carry `aria-selected`. Labels read the
formatted time ("9:30 AM").

## 9. Props sketch
```rust
TimePicker { label, value: Option<String /* HH:mm */>, description, placeholder, minute_step,
             format: "12h" | "24h", window: Option<String>, zone: Option<String>, open, disabled,
             on_change: EventHandler<String> }
```

## 10. Related components and references
`select.md` (list pattern), `date-reel.md` (wheel alternative), `time-dial.md`. Arc
`/components/time-picker` — behaviour only.
