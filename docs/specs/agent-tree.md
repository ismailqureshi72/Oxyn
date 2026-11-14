**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Agent tree (`agent-tree`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): affectUI · Agent Tree; Arc UI · Tree table (hierarchy part).

## 1. Purpose
Who started whom: an orchestrator and its sub-agents as a live tree. Every card shows status, model, task, cost and time; a card with children shows its branch cost; branches fold to one line (“3 sub-agents folded”); the header counts running, need-you, done and failed and the total cost. An agent that needs you opens a window with its question and Approve / Reject; a failed one shows the error with Retry / Dismiss.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| queued / stopped | Muted |  |
| running | Spinning mark, live time |  |
| waiting | Needs you + window | approve → running; reject → declined |
| failed | Failed + window | retry → running; dismiss → stopped |
| done | Check, final time |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| needs-you | Reviewer waits |
| failed | Reviewer failed |
| folded | Folded branch |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Fold toggles, window buttons |

## 6. Accessibility
- role=tree, treeitem with aria-level / aria-expanded; windows are dialogs.

## 7. Responsive
- 22 px indent per level.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| root | CrewAgent | — | id, name, task, model, state, cost, started, ended, ask, error, tools, children |
| now | ClockMs | — |  |
| title | String | Agents |  |
| folded | Vec<String> | [] |  |
| on_fold | EventHandler<String> | — |  |
| on_resolve | EventHandler<(String, Resolve)> | — |  |

## 9. Edge cases
- Costs format as $0.42.

## 10. Implementation
- Core: `CrewAgent`, `Resolve`, `dollars` (`oxyn_core::parity`), with tests.
- Dioxus: `AgentTree` (parity/run.rs, run.css).
- GPUI: `AgentTreeEl` (answers for the first agent that needs one).

## 11. References
- affectUI Agent Tree (https://www.affectui.com/components/agent-tree) — behaviour-only rebuild, no source copied.
