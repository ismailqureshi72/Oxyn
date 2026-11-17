**Status:** built v0.1 (2026-11-17, variants 3: Theme Selector family).

# Spec: Theme schedule (`theme-schedule`)

Family: Theme Selector. Reference: none (an Oxyn original).
Components: `ThemeSchedule` (Dioxus) · `ThemeScheduleEl` (GPUI), models in `oxyn_core::parity::themefid`.

## 1. Purpose
Let the app go light and dark by the clock – always, with the sun, or on your own hours – and show what any time of day will look like.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Light and dark" and a segmented Light / Dark / Sun / Custom |
| Line | "Your hours: light 07:00–19:00. Turns dark in 1 h." (`fmt_hm`, `next_switch`, `fmt_in`) |
| Strip | 34 px 24-hour strip: the light window as a soft band (split in two when it wraps past midnight), hour ticks with majors every six hours, a "now" rule with its time |
| Sample | 180 px window in Light or Dark that cross-fades as the previewed time crosses a switch |
| Hours | two sliders, Light from and Dark from, 15-minute steps, disabled outside Custom |

## 3. Behaviour
Dragging along the strip (a transparent native range, 15-minute steps) previews that time; the sample turns light or dark by `sched_light`. Sun uses the demo place's sunrise and sunset (`SUN`). Custom hours move the band at once. `on_change((mode, from, to))`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default (Custom, previewed at 18:00) · follows the sun · night owl (window wraps past midnight) · always dark.

## 5. Keyboard and accessibility
Modes are a radio group; the summary is a polite live region. The strip is a range named Preview time with the time and theme as value text. Hour sliders carry `aria-valuetext` and are disabled outside Custom.

## 6. Responsive
The sample stacks over the sliders under 480 px.

## 7. Known gaps
Sunrise and sunset are fixed for the demo place (no location lookup). GPUI: static snapshot (host routes `mode-<i>`).
