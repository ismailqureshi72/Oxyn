**Status:** v1 — implemented (plan #25).

# Spec: stepper form

> Implemented in `oxyn-core::primitives::stepper` (`Stepper`, `Step`, `StepField`, `FieldKind`,
> `FieldValue`, `StepperPhase`, `StepperStyle`), `oxyn-dioxus::StepperForm`, `oxyn-gpui::StepperEl`.
> Source: affectUI Stepper Form (behaviour only, no source used); the same model drives
> `multi-step-form.md`. Builds on the form state machine in `genui-forms.md`.

## 1. Purpose
Onboarding and setup: name a workspace, invite the team, pick a plan, review, create. The user
wants to know where they are, fix mistakes in place and go back without losing anything.

## 2. State machine
```
Editing(step i)
Continue ─invalid─▶ Editing(i) + errors; first invalid field shakes and takes focus
Continue ─valid─▶ done[i] = true; Editing(next visible step) (slides in from the right)
Back ─▶ Editing(previous visible step) (slides in from the left); values kept
step list / review Edit ─▶ Editing(finished step j)
Continue on the last step ─▶ Submitting ("Creating…", nav locked) ─ok─▶ Success ─Start over─▶ Editing(0)
Submitting ─error─▶ Failed(message next to the button) ─Continue─▶ Submitting
Conditional step (Oxyn plus): only_if(field = value) hides / shows a step
Save draft / Resume (Oxyn plus)
```

## 3. States table
| State | Shows | Keyboard |
|---|---|---|
| step list | numbered steps; current underlined, done with a tick | Tab to finished steps |
| editing | heading, description, fields | Enter continues; Tab moves through fields |
| invalid | messages under fields, first field shakes, Continue dimmed | Enter re-checks |
| chip field | invite emails as chips | `,` / Enter add, ⌫ removes |
| plan cards | radio cards with a gliding ring | ← → select, Enter picks and continues |
| review | label / value rows with Edit links | Tab to Edit |
| submitting | spinner + "Creating…", Back disabled | — |
| failed | message beside the button | Enter retries |
| success | check pops in, heading, Start over | — |

## 4. Micro-interactions
Steps fill and check off along the top. The card slides in from the side you're heading to and
its height animates. The primary button's label crossfades and its width follows (Continue →
Finish → Creating…). The first invalid field shakes once.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Step slide | 260 ms, 20 px | `ease.out` | 1 ms |
| Card height | 280 ms | `ease.out` | instant |
| Step fill / tick | 260 ms draw | `ease.out` | drawn |
| Field shake | 360 ms | damped | none |
| Label swap | `ox-base` | `ease.out` | swap |
| Success check | 320 ms pop | `ease.out` | instant |

## 6. Variants
Free: text, prefixed text (`acme.app/`), email, chip (tags) and choice-card fields; review step;
success; start at a step; submit failure.

**Oxyn plus:** vertical step rail; save draft and resume; conditional steps (`only_if`); minimal
header (`multi-step-form.md`).

## 7. Edge cases
Below 420 px the step list collapses to "Step 2 of 4 · Team" (kept for screen readers), plan cards
stack and review rows wrap. Optional fields skip the required check. Values survive Back and Edit.
Very long values wrap in review. Touch: 16 px fields and 40 px buttons.

## 8. Accessibility
The steps are an ordered list with `aria-current=step`; finished steps are buttons. Each step is a
`fieldset` with a `legend`. Errors are linked to fields and announced ("2 fields need attention");
Continue stays operable while dimmed so the reason is always heard. Submitting and success go to a
status region.

## 9. Props sketch
```rust
StepperForm { steps: Vec<Step { id, title, heading, description, fields: Vec<StepField>, review,
              only_if }>, style: "stepper" | "minimal" | "vertical", initial_step, submit_label,
              submitting_label, success_title, latency_ms, fail, drafts,
              initial_values: Vec<(String, String)>, on_step_change: EventHandler<usize>,
              on_complete: EventHandler<Vec<(String, String)>> }
```

## 10. Related components and references
`multi-step-form.md`, `genui-forms.md`, `tag-input.md`, `toggles.md` (radio cards). affectUI
`/components/stepper-form` — behaviour only.
