**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Gauge (`gauge`)

Family: Meters ([composer-context-meter.md](composer-context-meter.md)). Closes the parity row(s): Arc UI · Gauge. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A value against a known range as a 270° ring. The ring fills once on mount without overshoot, then springs to new values while the number rolls digit by digit. Threshold bands (`Band { from, tone, label }`) set the tone and caption: the highest band reached wins, exactly as the value crosses it. Without bands the `tone` applies. `raw` shows the value with a unit instead of the percent of the range; `detail` adds a second caption.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Ring, number, caption | value change |
| band change | Tone and caption swap | — |

## 3. Timing and motion
- First fill 1 s ease-out; changes 700 ms spring; digits roll 260 ms.
- Reduced motion lands on the value at once.

## 4. Variants
| Variant | What differs |
|---|---|
| bands | Healthy / Busy / Critical |
| critical | Past 90 |
| plain | One tone |

## 5. Keyboard
None (display only).

## 6. Accessibility
- role=meter with min, max, now and a valuetext including the band.

## 7. Responsive
- `size` sets the diameter (140).

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| value / min / max | f64 | — / 0 / 100 |  |
| label / detail | String / Option<String> | — |  |
| tone | accent · success · warning · danger | accent | no band |
| bands | Vec<Band> | — |  |
| size | f32 | 140 |  |
| raw / unit | bool / String | false / — |  |

## 9. Gaps against the source
- The first fill starts on mount, not when half in view (no IntersectionObserver without JS).

## 10. GPUI
GaugeEl draws the arc with the same `gauge()` view (tone and caption) using GPUI canvas paths.
