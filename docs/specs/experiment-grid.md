**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Experiment grid (`experiment-grid`)

Family: Chat ([ai-chat-screens.md](ai-chat-screens.md)). Closes the parity row(s): X #7 @naklecha Cloudy (experiments panel and steer-in-place). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A budgeted research run's experiments as shape marks in bands of hours (`experiment_grid`): kept (filled square), no gain (open square), crash fixed (triangle, danger tone), summary (diamond) and you (circle). The header shows the budget used (`budget_line`: "2h 12m of 24h") with a pulsing hairline while running; the counts line reads "27 runs · 5 kept · 2 crashes fixed" (`experiment_counts`). Marks pop in as the run clock reaches them; legend buttons count each outcome and pin one (others dim); hovering or focusing a mark writes its id and note under the grid.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| running | Pulsing hairline, marks popping in | budget spent / pause |
| paused | Still hairline | resume |
| pinned | Other outcomes dimmed | press again |

## 3. Timing and motion
- Marks pop 300 ms with overshoot; hairline pulses 1.8 s; notes swap 220 ms.
- Reduced motion: no pop or pulse.

## 4. Variants
| Variant | What differs |
|---|---|
| live | Marks arrive with the clock |
| finished | Whole budget |
| short | 4 hours in hour rows |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Legend and marks |
| Enter / Space | Pin an outcome |

## 6. Accessibility
- Marks are buttons labelled id, outcome and note; the note line is role=status; budget use is a progressbar.

## 7. Responsive
- Up to 440 px; marks wrap within a band.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| experiments | Vec<Experiment> | — | id, at, outcome, note |
| budget | u32 | 1440 | minutes |
| elapsed | u64 | — | seconds |
| band | u32 | 4 | hours a row |
| paused | bool | false |  |

## 9. Gaps against the source
- The decode-speed line chart and the report pane are not built; the steer-in-place composer is the existing agent-steering pattern with Pause / Stop (shown in the run-mode-menu demo).

## 10. GPUI
ExperimentGridEl draws the band rows and shapes with the budget line.
