**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Action states (`action-swap`)

Family: Loaders & text ([loaders.md](loaders.md), [text-reveal.md](text-reveal.md)). Closes the parity row(s): Arc UI · Action swap.

## 1. Purpose
A button whose label swaps to its next state while keeping its place (Follow → Following). Each state has a label, tone (neutral, accent, success, warning, quiet), icon, an optional pending label and an optional hold that advances on its own. A press either advances or starts a working phase: the next state's pending label, or the pill folds into a round spinner for at least 450 ms; a failed action shows the failure label (“Try again”) for 1.8 s and stays. Labels morph letter by letter; a success icon wipes in.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Label, tone, icon | press → working / next |
| working | Pending label or round spinner | settle ok → next; fail → failed |
| failed | Try again (warning) | 1.8 s → idle |
| held | Advances after hold_ms |  |

## 3. Timing and motion
- Reduced motion: no rise, blur or stagger; labels and values swap instantly (slot reels land at once, the shimmer band stops and the text stays muted).

## 4. Variants
| Variant | What differs |
|---|---|
| follow | Round spinner |
| copy | Hold 1.5 s |
| deploy | Pending labels |
| fails | Failure |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space | Advance |

## 6. Accessibility
- Native button; aria-busy while working; polite label changes.

## 7. Responsive
- Width follows the label.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | ActionSwap | — | states, current, phase, since, failure_label |
| size | sm · md · lg | md |  |
| on_press | EventHandler<()> | — | call press(async, now); later settle(ok, now) |

## 9. Edge cases
- Presses while working are ignored.

## 10. Implementation
- Core: `ActionSwap`, `SwapState`, `SwapPhase` (`oxyn_core::kit::text`), with tests.
- Dioxus: `ActionSwapButton` (kit/text.rs, text.css).
- GPUI: not yet (plan #28 follow-up; the core models are shared).

## 11. References
- Arc UI Action swap (https://uiarc.dev/components/action-swap) — behaviour-only rebuild, no source copied.
