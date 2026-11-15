**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Task breakdown (`task-breakdown`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Task breakdown (block).

## 1. Purpose
One goal broken into a plan: tasks stream in with estimates (working days, steps of 0.5), owners and dependencies; a schedule strip per task respects dependencies and one task per owner at a time; a verdict says whether the deadline holds and by how much. Move tasks up or down; Regenerate.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| streaming | Tasks appear 70 ms apart |  |
| fits | Green verdict |  |
| late | Red verdict with the overrun |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| fits | 8-day deadline |
| late | 5-day deadline |
| stream | Streaming |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Regenerate, move buttons |

## 6. Accessibility
- Verdict is a status region.

## 7. Responsive
- Gantt strip scales to the row.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | Breakdown | — | goal, deadline_days, tasks |
| shown | usize | all | streamed count |
| on_shift | EventHandler<(usize, bool)> | — |  |
| on_regenerate | EventHandler<()> | — |  |

## 9. Edge cases
- Dependencies on later tasks are ignored until reordered.

## 10. Implementation
- Core: `Breakdown`, `BTask` (schedule, span, verdict, shift) (`oxyn_core::parity`), with tests.
- Dioxus: `TaskBreakdown` (parity/run.rs, run.css).
- GPUI: `BreakdownEl`.

## 11. References
- Arc UI Task breakdown (https://uiarc.dev/components/blocks/task-breakdown) — behaviour-only rebuild, no source copied.
