**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent trail (`agent-trail`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Agent Trail.

## 1. Purpose
The steps an agent works through as one quiet trail: a header with the agent and the task and how many steps are done (3/5); done steps with a tinted check and their time; the running step shimmering what it does now with a live timer and its sub-steps; gated steps with a dashed ring and an Approval tag until reached, then Needs you; failed and declined steps with their tag; a tool mark before each title, faded while the step is still to come.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| pending | Hollow ring, faded | → running |
| running | Spinning ring, status shimmer, timer, sub-steps | → done / failed / waiting |
| waiting | Orange disc with pause, Needs you | → running / declined |
| done | Check + time |  |
| failed / declined | Tag |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| running | Default |
| needs-you | Gate reached |
| failed | Failure |
| dense | Dense, no sub-steps |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Read-only |

## 6. Accessibility
- Ordered list; state in badge text.

## 7. Responsive
- Times and tool marks keep their width.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| trail | Trail | — | title, agent, steps |
| now | ClockMs | — |  |
| substeps | bool | true |  |
| dense | bool | false |  |

## 9. Edge cases
- Durations format as 12s, 1m 4s, 1h 2m, 3d 4h.

## 10. Implementation
- Core: `Trail`, `TrailStep` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentTrail` (parity/run.rs, run.css).
- GPUI: `AgentTrailEl`.

## 11. References
- affectUI Agent Trail (https://www.affectui.com/components/agent-trail) — behaviour-only rebuild, no source copied.
