**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Test ticks (`test-ticks`)

Page `/components/agents/test-ticks/` (family run-trace). Reference: an X post the founders saved (R37 (tests visualised) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TestTicks` (Dioxus, `oxyn_dioxus::xpost`, `xpost/run.rs`) · `XPostView` with `XPostVariant::TestTicks` (GPUI), rules in `oxyn_core::xpost::ticks`.

## 1. Purpose
Show a test run as it happens, one tick per test, so failures are found at a glance and fixed without leaving the view.

## 2. States
Running (ticks fill in report order, timer and counters roll) · Stopped (keeps what ran) · Passed (all green) · Failed (passes dim, failures stay lit, fix card opens over the failing file) · Re-running the failed tests after Apply fix · Failed only filter.

## 3. Micro-interactions
Queued ticks are grey, the running tick is outlined and pulses, passes fill green, failures fill red and grow slightly taller. The fix card pops over the grid with a two-line diff; applying it flips only the failed ticks back to running and then to green.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Test | 90–240 ms each at 1× (dur) | stepped |
| Tick fill | 180 ms | ease |
| Fix card | 220 ms pop | overshoot |
| Re-run | 450 ms per failed test | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `failing` | Two failures, then the fix card over the failing file. |
| `all-pass` | A clean run that ends green. |
| `compact` | Grid only, no file names, fast (Oxyn extra). |

## 6. Responsive and edge cases
Under 520 px the file name sits above its ticks; ticks wrap; the fix card narrows to the grid. Counters keep tabular digits. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The grid is one tab stop with role grid; arrows move the selection and each tick speaks its test, result and time; results always appear as numbers and text, not colour alone; the fix card is a labelled dialog. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::TestTicks, ..)`; presses by id (tt-run, tt-stop, tt-filter, tt-fix, tt-fix-close, tt-tick-<n>), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
terminal-output, eval-suite, pr-card.
