**Status:** v1 — implemented (plan #25).

# Spec: date reel

> Implemented in `oxyn-core::primitives::dial` (`DateReel`, `Reel`, `ReelKind`, `ReelKey`,
> `ReelMode`, `ReelPreset`, `ConfirmState`, `spring_step`, `rubber`), `oxyn-dioxus::DateReel`,
> `oxyn-gpui::DateReelEl`. Source: Arc UI Date reel (behaviour only, no source used).

## 1. Purpose
"Send later" and "remind me" moments: pick a day and time with wheels, the way a phone does. Fast
with a thumb, exact with a keyboard.

## 2. State machine
```
per reel: Rest(i) ─drag─▶ Dragging(pos; past the ends it stretches)
Dragging ─release(v)─▶ Gliding(target = round(pos + v·τ), τ = 0.325 s / speed) ─spring─▶ Rest(target)
Rest ─↑/↓─▶ target ∓ 1 (hours, minutes, AM/PM wrap; days, years stop at the ends)
Rest ─PageUp/PageDown─▶ ± large step (7 days, 3 or 6 hours, 15 min, 3 months, 10 years)
Rest ─Home/End─▶ first / last
Rest ─type─▶ jump (4 → 4 o'clock, 45 → :45, p → PM, ma → March; letters within 1 s build a prefix)
quick pick ─▶ every reel spins to the preset
all reels at rest on a new value ─▶ on_change once
date mode: Feb 30 dimmed; the day reel turns back to Feb 28/29 by itself
Confirm: Idle ─click─▶ Pending ("Scheduling…") ─host─▶ Success ("Scheduled") ─▶ Idle
```

## 3. States table
| State | Shows | ARIA |
|---|---|---|
| rest | entries on a cylinder, current one bold under the lens | `spinbutton` with `aria-valuetext` |
| dragging | no transition, grabbing | — |
| gliding | entries spin, then land | announced at rest |
| invalid day | dimmed + struck | — |
| quick pick active | filled chip | `aria-pressed=true` |
| confirm pending / success | label morphs in place | `aria-busy` |
| words (Oxyn plus) | "Or type: tomorrow 6 pm" | labelled input |

## 4. Micro-interactions
Each reel is projected onto a cylinder (20° per entry, fading toward the back). Flicks keep their
momentum and land on an entry through a critically damped spring, so the stop never overshoots.
Keyboard steps add to the motion rather than restarting it. The summary ("Tomorrow at 9:30 AM") and
the relative phrase update when the reels rest.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Glide (web) | 220 ms + 45 ms per row, ≤ 900 ms, ÷ speed | `cubic-bezier(.2,.9,.25,1)` | lands at once |
| Spring (core) | ω = 16 × speed | critically damped | lands at once |
| Rubber band | 0.55 coefficient, 1.5 rows max | — | — |
| Type-ahead window | 1,000 ms | — | — |
| Confirm success | 900 ms pending (demo), 1.6 s shown | — | same |

## 6. Variants
Free: date-time (day, hour, minute, AM/PM), date (month, day, year), time; minute step; 12 / 24
hour; title; quick picks (This evening, Tomorrow, Monday); confirm button; speed.

**Oxyn plus:** natural-language field ("in 12 days 6:15 pm"); a weekday-routine pick ("Weekdays
9:00"); relative phrase ("In 12 days").

## 7. Edge cases
Days default to 30 days back and a year ahead (date mode: 100 years back, 10 on). Minute values off
the step land on the step below. Leap years in date mode. 320 px phones fit; reels size to their
widest entry. Below 480 px the confirm button is full width and 44 px tall. Wheel / trackpad
scroll turns one entry per notch.

## 8. Accessibility
A named group; each reel is a focusable `spinbutton` (Day, Hour, Minute, AM or PM / Month, Date,
Year) with `aria-valuenow` and a spoken `aria-valuetext` ("Fri, Sep 25"). The full moment is
announced politely once the reels rest. Quick picks use `aria-pressed`. Invalid days are dimmed and
never selected.

## 9. Props sketch
```rust
DateReel { mode: "datetime" | "date" | "time", date, time, today, minute_step, hour_cycle, title,
           presets, routine, confirm: Option<String>, speed: f32, natural, label,
           on_change: EventHandler<(String, String)>, on_confirm: EventHandler<(String, String)> }
```

## 10. Related components and references
`date-picker.md`, `time-picker.md`, `time-dial.md`. Arc `/components/date-reel` — behaviour only.
