**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Rubber slider (`elastic-slider`)

Family: Composer & inputs ([slider.md](slider.md)). Closes the parity row(s): Arc UI · Elastic slider. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A slider that stretches like rubber past its ends: the track widens (and shifts for the start end) by `rubber(over, 22)` and the start / end icons are pushed outward by 60 % of that, then everything springs back on release. The thumb swells to 1.28× while dragged; a value tag rides over it and tilts with its speed (`tag_tilt`, ±14°). Detents (`marks`) catch a dragged thumb within 7 px and hold it there (`detent`); a detent's tick pops when the thumb clicks in. Labelled marks print under the track and are clickable. Home / End at a limit stretch the track toward the press until the key is released.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Track, thumb | drag / keys |
| dragging | Swollen thumb, tag on, stretch past ends | release |
| detent | Thumb held, tick pops | leave 7 px |
| key push | Track stretched at a limit | key up |

## 3. Timing and motion
- Spring back 450 ms (cubic-bezier .3,1.5,.5,1); thumb swell 180 ms; tag sway 350 ms.
- Reduced motion removes the stretch, sway and tick pop; the thumb jumps and the tag fades.

## 4. Variants
| Variant | What differs |
|---|---|
| detents | Off / Half / Max with unlabelled quarter ticks |
| plain | No marks |
| bare | No icons or readout |

## 5. Keyboard
| Key | Does |
|---|---|
| ← → ↑ ↓ | One step |
| Shift + arrows, Page Up / Down | A tenth of the range |
| Home / End | Min / max; stretch at a limit |

## 6. Accessibility
- The thumb is role=slider with aria-valuetext; disabled removes it from the tab order.

## 7. Responsive
- The track fills its row between the icons.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — |  |
| value / min / max / step | f64 | 0 / 0 / 100 / 1 |  |
| marks | Vec<(f64, String)> | — | detents |
| suffix | String | — |  |
| show_value | bool | true |  |
| start_icon / end_icon | Option<String> | — | icon names |
| on_change / on_commit | EventHandler<f64> | — |  |

## 9. Gaps against the source
- Touch vibration on detents is not offered (no Vibration API without JS).

## 10. GPUI
ElasticSliderEl renders the track, thumb, marks and tag from the same detent and stretch helpers; the drag velocity and spring are host-driven.
