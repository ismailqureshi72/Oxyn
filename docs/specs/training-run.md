**Status:** built v0.1 (2026-11-21, C1e: AI special components II). Pro.

# Spec: Training run (`training-run`)

Family: AI Components (page `/components/ai/training-run/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TrainingRun` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Training` (GPUI), rules in `oxyn_core::aikit::training`.

## 1. Purpose
Follow a fine-tuning job from the queue to a deployed endpoint, with the loss curve as the main signal.

## 2. States
Queued · Starting · Training · Paused · Confirm cancel (3 s) · Cancelled · Failed (out of memory, Retry from checkpoint) · Completed (results, Deploy) · Deploying · Deployed (endpoint, copy).

## 3. Micro-interactions
The smoothed train loss draws over its faint raw samples with validation dashed; epoch bands under the plot fill as progress. Events slide in newest first. Arrow keys inspect a sample with a guide line.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Queue | 1400 ms | — |
| Start-up | 900 ms | — |
| Step | 18 ms per step at 1× | linear |
| Deploy | 1400 ms | — |
| Cancel confirm window | 3000 ms | — |

Reduced motion: loops and travel stop; each state shows its end pose and its words.

## 5. Variants
| Variant | What it shows |
|---|---|
| `training` | A LoRA run from the queue to Deploy. |
| `oom` | Fails at step 940; Retry from checkpoint 800. |
| `quiet` | No event log, quarter speed (Oxyn extra). |

## 6. Responsive and edge cases
The x axis spans the whole run and the y range is fixed, so nothing rescales. Under 440 px (container query) stats go two by two and the legend shortens to Val; the footer wraps under the dataset. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand and model marks.

## 7. Accessibility
Status changes are announced, never every step; the chart is a labelled group whose name carries the inspected step; stats are a description list; Cancel needs a second press. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Training, ..)`; presses by id (tr-pause, tr-resume, tr-cancel, tr-retry, tr-restart, tr-deploy, tr-copy), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
run-meter, eval-board.
