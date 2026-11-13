**Status:** v1 — implemented (plan #25).

# Spec: time dial

> Implemented in `oxyn-core::primitives::dial` (`TimeDial`, `City`, `spring_step`, `rubber`),
> `oxyn-dioxus::TimeDial`, `oxyn-gpui::TimeDialEl`. Source: Arc UI Time dial (behaviour only, no
> source used).

## 1. Purpose
Pick a meeting or hand-off time across time zones — when a human reviewer in London and an agent
schedule in Karachi overlap. The user spins until enough cities are at work.

## 2. State machine
```
Idle(value UTC minutes) ─pointer down─▶ Dragging(raw)
Dragging ─move─▶ value = raw with soft detents (snaps within 3 min of :00 / :15 / :30 / :45);
                 past min / max the dial rubber-bands
Dragging ─release(velocity)─▶ Coasting(target = nearest quarter hour of value + v·0.3 s, clamped)
Coasting ─critically damped spring─▶ Idle(target) ; on_change(target) fires at release
Idle ─→ / ↑ ─▶ +15 min ; ← / ↓ ─▶ −15 ; PageUp / PageDown ─▶ ±60 ; Home / End ─▶ home day start / 23:45
city row click ─▶ home city changes (face re-centres, knob moves to that city)
Best overlap ─▶ value = earliest quarter hour with the most cities at work      Oxyn plus
```

## 3. States table
| State | Face | Readout | ARIA |
|---|---|---|---|
| idle | home time at the top, working band, city dots | odometer digits, sun / moon, weekday · city | `slider`, `aria-valuetext` |
| dragging | grabbing cursor, no transition | digits roll | — |
| coasting | face settles on a quarter hour | — | announced once at rest |
| bound | rubber-band, then back to the bound | — | — |
| city at work | dot filled | row "At work" (green dot) | — |
| quiet hours (Oxyn plus) | dashed inner band | row "Quiet" | — |
| day change | — | row "+1 day" / "−1 day" | — |

## 4. Micro-interactions
Every city rides the dial at its own local time; the home city is the knob with its code. Digits
roll like an odometer; the sun turns into the moon at 18:00. A flick coasts and lands exactly on a
quarter hour. Dragging past a bound stretches and springs back.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Coast / settle | spring ω = 14 (≈ 420 ms) | critically damped | lands at once |
| Face rotate (web) | 420 ms | `cubic-bezier(.16,1,.3,1)` | 1 ms |
| Digit roll | 180 ms | `ease.out` | swap |
| Sun / moon | 300 ms fade | `ease.out` | swap |
| Detent window | ±3 min | — | — |

## 6. Variants
Free: 2–5 cities, home city, 12- or 24-hour readout, working hours, min / max bounds.

**Oxyn plus:** quiet-hours band; "Best overlap" jump; weekday per city with ±1 day notes.

## 7. Edge cases
Half- and quarter-hour offsets (India +5:30, Nepal +5:45) work because offsets are minutes. Values
past midnight keep counting (day notes stay right). Two cities with the same offset overlap; the
list disambiguates. Narrow containers (< 600 px): dial over list. Dense panels (< 360 px): tighter
rows. Touch: `touch-action: none` on the face.

## 8. Accessibility
The face is a focusable `role=slider` with `aria-valuenow` in home-city minutes and an
`aria-valuetext` such as "9:30 AM Tuesday in London, 3 of 4 cities at work"; the SVG and the
readout are `aria-hidden`. City rows are buttons with `aria-pressed` for the home city. Status is
never colour only: each row says At work / Off / Quiet.

## 9. Props sketch
```rust
TimeDial { cities: Vec<City { id, name, code, offset /* minutes */ }>, value: i32 /* UTC minutes */,
           home: Option<String>, min, max, weekday: u8, hour_cycle: "12" | "24",
           working_hours: (i32, i32), quiet_hours: Option<(i32, i32)>, overlap, label,
           on_change: EventHandler<i32>, on_home_change: EventHandler<String> }
```

## 10. Related components and references
`time-picker.md`, `date-reel.md`, `slider.md` (keyboard pattern). Arc `/components/time-dial` —
behaviour only.
