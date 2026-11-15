**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent plan (`agent-plan`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Agent Plan; Arc UI · Agent plan.

## 1. Purpose
An agent's step-by-step plan to review before it runs. In draft every title is editable, steps drag to reorder (or ⌥↑/⌥↓), can be removed (at least one stays) or added; an estimate shows in the header. Run plan starts the first step; each step then shows a live timer, done steps their time, a failed step its error with Retry from here; the tray offers Run again once done.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| draft | Editable titles, grips, remove, add, Cancel, Run plan | run → running |
| running | Marks, live timer on the current step, n/m | step fails → failed; last done → done |
| failed | Error under the step + Retry from here | retry → running from that step |
| done | All checks, Run again | run → running |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| review | Draft |
| running | Live timers |
| failed | Retry from here |
| done | Run again |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Titles, remove, add, Cancel, Run |
| ⌥↑ / ⌥↓ | Move the focused step |
| Enter | Commit rename / add |

## 6. Accessibility
- Ordered list labelled by the goal; inputs labelled “Step n”.
- Remove buttons named; errors are text.

## 7. Responsive
- Fills its container.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| plan | PlanDraft | — | title, estimate, steps, phase |
| now | ClockMs | — |  |
| cancellable | bool | true |  |
| on_rename/on_move/on_remove/on_add | EventHandler | — |  |
| on_run/on_cancel | EventHandler<()> | — |  |
| on_retry | EventHandler<String> | — | step id |

## 9. Edge cases
- Renaming to blank is ignored.
- Editing is refused outside draft.

## 10. Implementation
- Core: `PlanDraft`, `PlanStep`, `PlanPhase`, `StepState`, `short_duration` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentPlan` (parity/run.rs, run.css).
- GPUI: `AgentPlanEl` (`run`, `cancel`, `retry`); title editing in GPUI waits on the IME text input (plan #28 chunk 4).

## 11. References
- affectUI Agent Plan (https://www.affectui.com/components/agent-plan) — behaviour-only rebuild, no source copied. Also Arc UI Agent plan (https://uiarc.dev/components/agent-plan).
