**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Polling cost (`polling-cost`)

Family: Data display. Closes the parity row(s): X #24 @TekonBurns (text only: background subagent polling). Behaviour-only rebuild; nothing is copied.

## 1. Purpose
What background subagents spend checking on each other (`poll_cost`). A full poll re-reads the whole transcript every interval, and the transcript grows; a status ping costs a few hundred tokens. Two bars compare the window, the saving rolls as a percent, and a line says how many polls ran and how much of the usage limit that buys back for prompts.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Two bars and the saving | inputs change |

## 3. Timing and motion
- Bars grow 800 ms and re-flow 500 ms; numbers roll.
- Reduced motion: no grow or roll.

## 4. Variants
| Variant | What differs |
|---|---|
| hour | Three subagents for an hour |
| long | Six subagents for five hours |

## 5. Keyboard
None (display only).

## 6. Accessibility
- Numbers sit next to each bar as text.

## 7. Responsive
- Up to 440 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| agents | u32 | 3 |  |
| every | u32 | 30 | seconds |
| minutes | u32 | 60 |  |
| transcript / growth / ping | f64 | 40 000 / 600 / 300 | tokens |
| limit | f64 | 3e8 | tokens |

## 9. Gaps against the source
- The post is text only; this is a behaviour built from its claim, not a rebuild of a UI.

## 10. GPUI
PollingCostEl draws the two bars and the saving.
