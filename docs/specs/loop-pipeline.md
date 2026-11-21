**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Loop pipeline (`loop-pipeline`)

Page `/components/agents/loop-pipeline/` (family plans). Reference: an X post the founders saved (R23 (agent-run to-dos with a loop pipeline) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `LoopPipeline` (Dioxus, `oxyn_dioxus::xpost`, `xpost/run.rs`) · `XPostView` with `XPostVariant::LoopPipeline` (GPUI), rules in `oxyn_core::xpost::loops`.

## 1. Purpose
Show an agent that works in rounds (work, check, improve) until a check passes, with every round and its score kept in view.

## 2. States
Running (marker on Work, Check or Improve) · Paused · Passed (exits at the target) · Out of rounds (best score, One more round) · Stopped by hand.

## 3. Micro-interactions
A marker travels round a dashed loop; the active station outlines; the round number rolls; each finished check drops a chip with its score (green at or above target). On success the loop turns solid green.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Station | 700 ms (STATION_MS) | linear |
| Round | 2100 ms | — |
| Chip in | 180 ms | ease-out |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `passes` | Exits on round 4 at 88%. |
| `runs-out` | Target out of reach: stops after 3 rounds and offers one more. |
| `first-try` | Passes on the first round. |

## 6. Responsive and edge cases
The ring and the score panel sit side by side and stack under 520 px. The final round ends at Check (no Improve after the last score). Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The status line names the round, station and score; the ring is decorative; round chips are buttons in a list; Space pauses, Escape stops, arrows move between chips. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::LoopPipeline, ..)`; presses by id (lp-pause, lp-stop, lp-more, lp-restart, lp-round-<n>), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
plan-todo, run-meter, eval-suite.
