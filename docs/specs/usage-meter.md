**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Usage meter (`usage-meter`)

Family: Meters ([composer-context-meter.md](composer-context-meter.md)). Closes the parity row(s): Arc UI · Usage meter. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
What fills an allowance. Segments grow in left to right on mount, then re-flow on one spring each when values change. Past the limit the bar rescales to the total used, a hatched overage fades in right of a limit marker and the legend shows the overage. The badge springs between "62% used", "Almost full" (from `warn_at`, 0.9) and "Over limit by 2.0 GB". The legend is a roving tab stop: arrows move, Home / End jump, Enter or Space pins a category (it stays lit and the rest dim), Escape clears.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| ok | Segments, free space | — |
| almost | Amber badge | — |
| over | Hatched overage, limit marker, red badge | — |
| pinned | One category lit | Enter / Esc |

## 3. Timing and motion
- Grow-in 600 ms with 70 ms stagger; re-flow 500 ms spring; badge 220 ms.
- Reduced motion jumps values and swaps text.

## 4. Variants
| Variant | What differs |
|---|---|
| ok | Under the line |
| almost | Past 90 % |
| over | Past the limit |

## 5. Keyboard
| Key | Does |
|---|---|
| ← → ↑ ↓ | Move between legend items |
| Home / End | First / last |
| Enter / Space | Pin / unpin |
| Esc | Clear the pin |

## 6. Accessibility
- The bar is role=img with totals and badge; legend items are toggle buttons with aria-pressed.

## 7. Responsive
- The legend wraps.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| title | String | — |  |
| shares | Vec<Share> | — | id, label, value |
| limit | f64 | — |  |
| unit / decimals | String / usize | — / 1 |  |
| free_label / over_label | String | Free / Over limit |  |
| warn_at | f64 | 0.9 |  |

## 9. Gaps against the source
- Segments use a monochrome grey ramp (Oxyn's colour rule) rather than series hues; Studio presets can colour them.

## 10. GPUI
UsageMeterEl draws the bar, overage and badge from `parity::meters::usage`.
