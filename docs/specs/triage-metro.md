**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Triage metro (`triage-metro`)

Page `/components/agents/triage-metro/` (family run-trace). Reference: an X post the founders saved (R38 (security agent site) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `TriageMetro` (Dioxus, `oxyn_dioxus::xpost`, `xpost/run.rs`) · `XPostView` with `XPostVariant::TriageMetro` (GPUI), rules in `oxyn_core::xpost::metro`.

## 1. Purpose
Show a security agent sorting findings as a metro map, so the outcome of every finding is visible and the ones that need a person stand out.

## 2. States
Riding (dots travel Scan → Triage, pause, branch) · Landed (terminal counters roll) · Open finding (card) · Needs you (Approve patch / Not a risk reroutes the dot) · Paused · Looping (restarts until someone interacts).

## 3. Micro-interactions
Dots enter every 900 ms, dwell 500 ms at Triage and ride to their terminal. A rerouted dot rides from its terminal along a short connector to the new one. Counters roll when a dot lands.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Entry gap | 900 ms (GAP) | — |
| Ride | 3200 ms incl. dwell (RIDE) | linear |
| Dwell at Triage | 500 ms (DWELL) | — |
| Reroute | 1200 ms | linear |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `map` | The full map with labels and counters. |
| `bare` | Lines and dots only, for a dense dashboard tile (Oxyn extra). |
| `once` | Runs once and stays, so you can work the Needs you stop. |

## 6. Responsive and edge cases
The SVG map scales with the card; dot hit targets are 22 px buttons laid over it. At 390 px terminal names shrink and sit above their stop. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
Every dot is a named button (finding, file, severity, outcome); the map is one focusable group with arrow keys; terminals are named in text; Space pauses all motion. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::TriageMetro, ..)`; presses by id (tm-pause, tm-dot-<n>, tm-close, tm-patch, tm-dismiss, tm-restart), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
agent-findings, root-cause-tree, review-center.
