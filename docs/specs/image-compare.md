**Status:** built v0.1 (2026-11-15, plan #30).

# Spec: Image compare (`image-compare`)

Family: Diff & review. Closes Arc UI **Image compare** (free). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Before and after visuals: a retouch, a redesign, a screenshot an agent changed. Two stacked images, the before clipped at the divider (`clip-path: inset`). Drag anywhere on the frame; keys move the divider (`compare_key`); captions fade as the divider reaches them (`caption_opacity`). Oxyn plus: side by side, onion skin with an opacity slider and a difference blend.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Divider at position, round knob | drag / keys |
| dragging | Knob stretches into a capsule | release |
| edge bump | Knob squeezes when a key hits 0 or 100 | — |
| side / onion / difference | Other compare modes | mode |

## 3. Timing and motion
- The divider follows the pointer directly; the knob stretches to 18×40 px over 180 ms while dragging.
- An edge key press gives a 200 ms bump; captions fade over 150 ms.
- Reduced motion: no stretch or bump.

## 4. Variants
| Variant | What differs |
|---|---|
| slider | Horizontal divider |
| vertical | Top and bottom |
| side-by-side | Both at once |
| onion | Opacity slider |
| difference | Changed areas light up |

## 5. Keyboard
| Key | Does |
|---|---|
| Arrow keys | 1 % (10 % with Shift) |
| Page Up / Page Down | 10 % |
| Home / End | All before / all after |
| Double-click | Recentre |

## 6. Accessibility
- role=slider with aria-orientation, aria-valuenow and aria-valuetext "40% after"; captions aria-hidden; images need their own alt.

## 7. Responsive
- Full width at a fixed aspect ratio; touch-action pan-y keeps vertical page scroll on touch (pan-x when vertical).

## 8. Edge cases
- Positions clamp to 0–100.
- Images of different sizes are cover-fitted; give both the same crop.

## 9. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| before | Element | — |  |
| after | Element | — |  |
| position | Option<f32> | None | controlled |
| default_position | f32 | 50 |  |
| orientation | String | "horizontal" |  |
| labels | Option<(String, String)> | Before / After |  |
| label | String | "Before and after" |  |
| aspect_ratio | String | "3 / 2" |  |
| mode | CompareMode | Slider | Oxyn plus |
| opacity | f32 | 0.5 | onion |
| on_position_change | EventHandler<f32> | — | on settle |

## 10. Gaps against the source
- No release-velocity settle spring; the divider stops where you let go.
- Touch waits on the browser's pan rules rather than a custom sideways-drag threshold.

## 11. GPUI
`ImageCompareEl::new(before, after, w, h).position(p)` clips the before side with `overflow_hidden`.
