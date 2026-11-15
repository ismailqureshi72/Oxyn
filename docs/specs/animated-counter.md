**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Animated counter (`animated-counter`)

Family: Data display ([display-atoms.md](display-atoms.md)). Closes the parity row(s): Arc UI · Animated counter.

## 1. Purpose
A changing total with a clear sense of movement: a small label, prefix and suffix, fixed decimals and thousands separators; each digit column rolls in the direction the whole number moved with a slight stagger, columns that do not change stay still.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| static | Formatted value |  |
| rolling | Per-column roll |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| money | $1,284.50 |
| percent | No decimals, % |
| tokens | Suffix |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- The full value is the accessible name; digits are aria-hidden.

## 7. Responsive
- Size is a prop.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| value | f64 | — |  |
| from | Option<f64> | None |  |
| t | f32 | 1.0 | roll progress |
| label | Option<String> | None |  |
| prefix / suffix | String |  |  |
| decimals | usize | 0 |  |
| size | f32 | 32 |  |

## 9. Edge cases
- Negative values use a real minus sign.

## 10. Implementation
- Core: `digits_roll`, `group` (`oxyn_core::parity`), with tests.
- Dioxus: `AnimatedCounter` (parity/run.rs, run.css).
- GPUI: `CounterEl`.

## 11. References
- Arc UI Animated counter (https://uiarc.dev/components/animated-counter) — behaviour-only rebuild, no source copied.
