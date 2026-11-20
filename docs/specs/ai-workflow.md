**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: AI workflow (`ai-workflow`)

Family: AI Components (page `/components/ai/ai-workflow/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `AiWorkflow` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Workflow` (GPUI), rules in `oxyn_core::aikit::workflow`.

## 1. Purpose
Build and test a small automation: a trigger, an AI step, a branch and actions on one hairline rail, then watch a test run go through it.

## 2. States
Empty (pick a trigger) · Editing · Running (a dot glides down each connector, each step runs and opens its output inline) · Passed (untaken path skipped) · Failed (the step shows its error and Retry, which resumes from it) · Waiting for approval (gate before the first action).

## 3. Micro-interactions
The connector fills top to bottom as the dot travels; a finished step settles with a check and its output streams word by word. Add step opens a command menu with app marks; remove is a small button on hover/focus. Skipped steps fade to 50 %.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Connector travel | 240 ms per wire (TRAVEL_MS) | ease-in-out |
| Output stream | 34 ms per word tick | linear |
| Step settle | 160 ms | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `passes` | A test run that passes; the untaken path is skipped. |
| `fails` | The Linear step fails; Retry resumes from it. |
| `approval-gate` | The run waits for your approval before the first action (Oxyn extra). |
| `empty` | No steps yet: pick a trigger. |

## 6. Responsive and edge cases
Step titles truncate; outputs wrap and scroll inside the step when long. The rail stays one column at every width. Reduced motion shows filled connectors immediately. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Steps are a list; each step announces its status. Run, Retry and Approve are buttons; the menu is a menu with menuitems and type-ahead. Progress lines go to the polite live region. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Workflow, ..)`; presses by id (wf-run, wf-retry, wf-approve, wf-add, wf-pick-<n>, wf-rm-<id>, wf-path-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
workflow, plan-todo, tool-call.
