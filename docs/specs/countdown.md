**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Countdown (`countdown`)

Family: Data display ([display-atoms.md](display-atoms.md)). Closes the parity row(s): Arc UI · Countdown.

## 1. Purpose
A launch countdown: large digits with unit labels (each value rises in when it changes) or one compact line for banners and table cells; units can drop days so hours absorb them (hours pass 24); at zero it morphs into a live state with a pulsing dot.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| counting | Digits |  |
| live | Dot + Live now |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| large | Days to seconds |
| compact | One line |
| live | At zero |
| hours | No days |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- role=timer with an accessible name.

## 7. Responsive
- Large wraps by unit.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | Countdown | — | target |
| now | ClockMs | — |  |
| variant | large · compact | large |  |
| days | bool | true |  |
| complete_label | String | Live now |  |
| label | String | Time remaining |  |

## 9. Edge cases
- Seconds round up so the display never shows 00 while time is left.

## 10. Implementation
- Core: `Countdown` (`oxyn_core::parity`), with tests.
- Dioxus: `Countdown` (parity/run.rs, run.css).
- GPUI: `CountdownEl`.

## 11. References
- Arc UI Countdown (https://uiarc.dev/components/countdown) — behaviour-only rebuild, no source copied.
