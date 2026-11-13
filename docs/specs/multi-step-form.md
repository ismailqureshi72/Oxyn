**Status:** v1 — implemented (plan #25).

# Spec: multi-step form

> Implemented by the stepper model in `oxyn-core::primitives::stepper` with
> `StepperStyle::Minimal`, `oxyn-dioxus::StepperForm { style: "minimal" }`, `oxyn-gpui::StepperEl`.
> Source: Arc UI Multi-step form (behaviour only, no source used).

## 1. Purpose
One decision at a time inside a card — a short setup, a request form an agent hands the user.
Less chrome than the stepper: a step count and a bar instead of a step list.

## 2. State machine
```
Step(i of n) ─Next (valid)─▶ Step(i+1) (count rolls up, bar grows, step slides in)
Step(i) ─Next (invalid)─▶ Step(i) + errors, first invalid field shakes
Step(i) ─Back─▶ Step(i−1) (count rolls down); values kept
Step(n) ─Submit (complete_label)─▶ Submitting ─▶ Success (role=status)
Save draft / Resume (Oxyn plus)
```

## 3. States table
| State | Header | Primary | Keyboard |
|---|---|---|---|
| first | "Step 1 of 4", bar 25 % | Next | Enter |
| middle | count rolls, bar grows | Next; Back shows | Enter, Tab |
| last | "Step 4 of 4" | complete label (e.g. "Create workspace"), width follows the label | Enter submits |
| invalid | messages | Next dimmed but operable | — |
| submitting | — | spinner + "Creating…" | locked |
| success | check, heading | Start over | — |

## 4. Micro-interactions
The step number rolls like an odometer; the progress bar eases to the new width; the step slides
in the direction of travel; the submit button's width animates to fit its label.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Count roll | 200 ms | `ease.out` | swap |
| Bar | 360 ms | `ease.out` | 1 ms |
| Step slide | 260 ms | `ease.out` | 1 ms |
| Button width / label | `ox-snap` | spring | instant |

## 6. Variants
Free: steps, complete label, start at a step.

**Oxyn plus:** save draft and resume; conditional steps; the same field kinds as the stepper (chips,
choice cards, prefixed text).

## 7. Edge cases
Below 420 px the header stacks and padding tightens. One-step forms show no count. Long labels on
the submit button wrap to two lines rather than overflow. Touch: 40 px buttons.

## 8. Accessibility
Each step is a `fieldset` with a `legend`; the step count is announced politely ("Step 2 of 4:
Team") from a visually hidden status line. Success is `role=status`.

## 9. Props sketch
```rust
StepperForm { steps, style: "minimal", submit_label /* completeLabel */, initial_step, drafts,
              on_step_change, on_complete }
```

## 10. Related components and references
`stepper-form.md` (numbered variant, same model), `genui-forms.md`. Arc
`/components/multi-step-form` — behaviour only.
