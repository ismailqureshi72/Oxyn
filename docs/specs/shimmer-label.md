**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Text shimmer (`shimmer-label`)

Family: Loaders & text ([text-shimmer.md](text-shimmer.md)). Closes the parity row(s): Arc UI · Text shimmer.

## 1. Purpose
Ongoing work as a calm band in the current text colour sweeping over a muted base while `active`, one pass per `duration_ms` with a pause between passes (the CSS animation pauses and resumes in place); a changed label rises in with a soft blur. Complements the engine-driven `TextShimmer` (agent states, stall).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| active | Band sweeping |  |
| inactive | Muted, no sweep |  |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| sweep | Default |
| labels | Label changes |
| paused | Inactive |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- Plain text.

## 7. Responsive
- Inline.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| text | String | — |  |
| active | bool | true |  |
| duration_ms | u32 | 1800 |  |

## 9. Edge cases
- The browser pauses CSS animations in background tabs.

## 10. Implementation
- Core: — (`oxyn_core::kit::text`), with tests.
- Dioxus: `SweepText` (kit/text.rs, text.css).
- GPUI: `ShimmerEl` (per-letter band colour; `active(false)` / `reduced(true)` rest it). The rise-in on a label change is Dioxus only.

## 11. References
- Arc UI Text shimmer (https://uiarc.dev/components/text-shimmer) — behaviour-only rebuild, no source copied.
