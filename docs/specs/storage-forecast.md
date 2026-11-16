**Status:** built v0.1 (2026-11-16, variants 2: Storage family).

# Spec: Storage forecast (`storage-forecast`)

Family: Storage. Reference: none (an Oxyn original).
Components: `StorageForecast` (Dioxus) · `StorageForecastEl` (GPUI), models in `oxyn_core::parity::storagefid`.

## 1. Purpose
Turn a fortnight of growth into one answer: when the disk is full at this pace.

## 2. Parts
| Part | Look |
|---|---|
| Head | "Disk forecast" and "Full in about 78 days at 2.9 GB a day" (`store_pace`, `days_until_full`); a reading in the corner |
| Chart | 14 dot columns lit to each day's use (`col_dots`), a gap, then ghost columns a week apart at the pace; dots in the top tenth turn red |
| Axis | 2 weeks ago · Today · +8 weeks |

## 3. Behaviour
Pointing at a column reads its day ("3 days ago", "Today", "in 2 weeks") and use. `pace` overrides the history's own pace.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · fast (6 GB a day) · slow (1 GB a day, finer columns).

## 5. Keyboard and accessibility
The chart is an image labelled with used space and the summary; the corner reading is a live region.

## 6. Responsive
460 px card; columns share the width, dots stay round.

## 7. Known gaps
Linear pace only. GPUI: history columns only.
