**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Loading state (`loading-state`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): Beautiful UI · Loading State.

## 1. Purpose
A 4×4 pixel-grid loader (drive: a lit column sweeping with a trail; dots; orbit: a dot with a tail around the edge; surfer: a rolling wave), a shimmering verb (“Churning”) and the elapsed time with one decimal.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| running | Frames every 110 ms, timer counting |  |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| drive | Default |
| orbit | Orbit |
| surfer | Surfer |
| dots | No timer |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- role=status named by the verb; grid and timer aria-hidden.

## 7. Responsive
- Timer keeps 4ch.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| label | String | — |  |
| variant | drive · dots · orbit · surfer | drive |  |
| started / now | ClockMs | — |  |
| elapsed | bool | true |  |
| reduced | bool | false | one still frame |

## 9. Edge cases
- Cells use four brightness levels of the text colour (monochrome).

## 10. Implementation
- Core: `pixel_frame`, `elapsed_label` (`oxyn_core::parity::text`), with tests.
- Dioxus: `LoadingState` (parity/text.rs, text.css).
- GPUI: `LoadingStateEl` (4×4 pixel frames, shimmer verb, elapsed seconds; `reduced(true)` holds frame 0).

## 11. References
- Beautiful UI Loading State (https://www.beautifului.dev/#loading-state), MIT, © 2026 Shane Levine — behaviour rebuilt in Rust; no source or icons copied.
