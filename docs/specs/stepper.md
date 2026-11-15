**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Stepper (`stepper`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Stepper.

## 1. Purpose
Where a person is in a multi-step flow and what is done. Horizontal or vertical; numbers morph into checks or an alert; connectors fill one after another; descriptions for all steps or only the current one (errors always show); optional steps; compact markers with a caption (“Step 3 of 4 · Billing”), automatic below 30 rem; completed steps are selectable with arrow keys when `on_select` is set.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| upcoming | Number in an outline |  |
| current | Number, ring | → complete |
| complete | Filled check | select → current |
| error | Alert |  |
| all complete | Complete label announced |  |

## 3. Timing and motion
- Connectors fill over 350 ms; the current marker gets a 4 px ring.
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| horizontal | Default |
| vertical | Descriptions for current |
| compact | Markers + caption |
| error | Failed step |
| complete | All done |

## 5. Keyboard
| Key | Does |
|---|---|
| ← → ↑ ↓ | Move among reachable steps |
| Home / End | First / current |
| Enter / Space | Select a completed step |

## 6. Accessibility
- nav with an accessible name; aria-current=step on the current item.
- Completion announced in a status region.

## 7. Responsive
- Container query: text hides below 30 rem in horizontal mode.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| stepper | Stepper | — | steps, current |
| orientation | horizontal · vertical | horizontal |  |
| details | all · current | all |  |
| compact | bool | false |  |
| label | String | Progress |  |
| complete_label | String | All steps complete |  |
| on_select | Option<EventHandler<usize>> | None | read only without |

## 9. Edge cases
- Focus moves are clamped to completed and current steps.

## 10. Implementation
- Core: `Stepper`, `FlowStep` (`oxyn_core::parity`), with tests.
- Dioxus: `FlowStepper` (parity/run.rs, run.css).
- GPUI: `FlowStepperEl`.

## 11. References
- Arc UI Stepper (https://uiarc.dev/components/stepper) — behaviour-only rebuild, no source copied.
