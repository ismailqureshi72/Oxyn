**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent fleet (`agent-fleet`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Agent Fleet.

## 1. Purpose
Several main agents side by side: each card sums up its sub-agents (running, need you, done, failed) and its branch cost; a click opens that agent's tree to the right while the previous one folds back into its card; a second click folds it. The header shows the whole fleet's cost.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| folded | Cards only |  |
| open | Card pressed + tree | click again → folded |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| open | Lead open |
| support | Another agent open |
| folded | All folded |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter / Space | Open / fold |

## 6. Accessibility
- Cards are toggle buttons with aria-pressed.

## 7. Responsive
- Cards stack above the tree below 560 px (container query).

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| agents | Vec<CrewAgent> | — |  |
| now | ClockMs | — |  |
| open | Option<String> | None |  |
| on_open | EventHandler<Option<String>> | — |  |
| on_resolve | EventHandler<(String, Resolve)> | — |  |

## 9. Edge cases
- Opening an unknown id shows no tree.

## 10. Implementation
- Core: `CrewAgent::tally`, `branch_cost` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentFleet` (parity/run.rs, run.css).
- GPUI: `FleetEl` (summary cards; the tree is `AgentTreeEl`).

## 11. References
- affectUI Agent Fleet (https://www.affectui.com/components/agent-fleet) — behaviour-only rebuild, no source copied.
