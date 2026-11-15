**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent status (`agent-status`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Agent status.

## 1. Purpose
Who is working and how: the agent's avatar with a state dot (idle, thinking, working, needs you, error, done), its name and state, what it does right now (shimmering while busy) and a live timer. Pill, card or inline.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Grey dot |  |
| thinking / acting | Info dot pulsing, shimmer |  |
| waiting | Warning dot |  |
| error | Danger dot |  |
| done | Success dot |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| acting | Working |
| waiting | Needs you |
| card | Card |
| error | Error |

## 5. Keyboard
| Key | Does |
|---|---|
| — | Not interactive |

## 6. Accessibility
- role=status, polite.

## 7. Responsive
- Activity truncates first.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| name | String | — |  |
| state | String | — |  |
| activity | Option<String> | None |  |
| started | Option<ClockMs> | None |  |
| now | ClockMs | — |  |
| variant | pill · card · inline | pill |  |

## 9. Edge cases
- Unknown states read Idle.

## 10. Implementation
- Core: uses `short_duration` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentStatus` (parity/run.rs, run.css).
- GPUI: `AgentStatusEl`.

## 11. References
- Arc UI Agent status (https://uiarc.dev/components/agent-status) — behaviour-only rebuild, no source copied.
