**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Task rows (`task-rows`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Beautiful UI · Task Rows.

## 1. Purpose
Live agent task status as compact rows, capsules or a hairline list: a ring with the current step for running rows, a Completed pill (green) or Failed pill with Retry, an amount on the right, and details per row that open and close. Rows enter 80 ms apart.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| pending | Hollow mark |  |
| running | Ring 0→100 %, Step n of m |  |
| done | Check, Completed |  |
| failed | Failed + Retry | retry → running |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| capsules | Pill rows (default) |
| list | Hairline list |
| closed | All rows closed |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space | Open / close a row |

## 6. Accessibility
- Rows are buttons with aria-expanded.

## 7. Responsive
- Amount truncates first.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| rows | Vec<TaskRow> | — |  |
| variant | capsules · list | capsules |  |
| on_toggle | EventHandler<usize> | — |  |
| on_retry | EventHandler<usize> | — |  |

## 9. Edge cases
- A row with no steps shows an empty ring.

## 10. Implementation
- Core: `TaskRow` (`oxyn_core::parity`), with tests.
- Dioxus: `TaskRows` (parity/run.rs, run.css).
- GPUI: `TaskRowsEl`.

## 11. References
- Beautiful UI Task Rows (https://www.beautifului.dev/#task-rows), MIT, © 2026 Shane Levine — behaviour rebuilt in Rust; no Central Icons used.
